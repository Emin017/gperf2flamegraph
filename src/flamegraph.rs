use ahash::{AHashMap, AHashSet};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::parser::parse_profiler_result;
use crate::symbols::SymbolResolver;

const UNKNOWN_SYMBOL: &str = "???";

#[derive(Debug)]
pub struct FlamegraphData {
    data: String,
    default_flamegraph_args: Vec<String>,
}

impl FlamegraphData {
    pub fn new(stacks: &AHashMap<String, u64>, to_microsecond: bool) -> Self {
        let default_flamegraph_args = if to_microsecond {
            vec!["--countname".to_string(), "us".to_string()]
        } else {
            Vec::new()
        };

        let data = stacks
            .iter()
            .map(|(stack, count)| format!("{} {}", stack, count))
            .collect::<Vec<String>>()
            .join("\n")
            + "\n";

        FlamegraphData {
            data,
            default_flamegraph_args,
        }
    }

    pub fn write_text_output(&self, filepath: &Path) -> Result<()> {
        let mut file = File::create(filepath)?;
        file.write_all(self.data.as_bytes())?;
        Ok(())
    }

    pub fn write_svg_output(&self, filepath: &Path, flamegraph_args: &[String]) -> Result<()> {
        let mut args =
            Vec::with_capacity(self.default_flamegraph_args.len() + flamegraph_args.len());
        args.extend(self.default_flamegraph_args.iter().cloned());
        args.extend(flamegraph_args.iter().cloned());

        let mut child = Command::new("flamegraph.pl")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to start flamegraph.pl")?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(self.data.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        let mut file = File::create(filepath)?;
        file.write_all(&output.stdout)?;

        Ok(())
    }
}

pub struct Gperf2Flamegraph {
    symbol_resolver: Option<SymbolResolver>,
    executable_path: PathBuf,
    profile_result_path: PathBuf,
    executable_only: bool,
}

impl Gperf2Flamegraph {
    pub fn new(executable_path: &Path, profile_result_path: &Path, executable_only: bool) -> Self {
        Gperf2Flamegraph {
            symbol_resolver: None,
            executable_path: executable_path.to_path_buf(),
            profile_result_path: profile_result_path.to_path_buf(),
            executable_only,
        }
    }

    pub fn process(
        &mut self,
        simplify_symbol: bool,
        annotate_libname: bool,
        to_microsecond: bool,
    ) -> Result<FlamegraphData> {
        let profiler_result = parse_profiler_result(&self.profile_result_path)?;

        if self.symbol_resolver.is_none() {
            self.symbol_resolver = Some(SymbolResolver::new(
                &self.executable_path,
                &profiler_result.proc_mapped_objects,
                self.executable_only,
            )?);
        }

        // Collect all program counter addresses
        let mut all_pcs = AHashSet::new();
        for stacktrace in &profiler_result.stacktraces {
            for pc in &stacktrace.pcs {
                all_pcs.insert(*pc);
            }
        }

        // Batch resolve symbols
        let pcs_to_symbols = self
            .symbol_resolver
            .as_mut()
            .unwrap()
            .resolve_symbols_batch(&all_pcs, simplify_symbol, annotate_libname);

        // For each stacktrace, we need to fill in the symbols for each program counter
        let mut stacktraces = profiler_result.stacktraces;
        for stacktrace in &mut stacktraces {
            let mut symbols = Vec::with_capacity(stacktrace.pcs.len());
            for pc in &stacktrace.pcs {
                symbols.push(
                    pcs_to_symbols
                        .get(pc)
                        .cloned()
                        .unwrap_or_else(|| UNKNOWN_SYMBOL.to_string()),
                );
            }
            stacktrace.symbols = Some(symbols);
        }

        // Collect stacks
        let mut stacks = AHashMap::new();
        for stacktrace in stacktraces {
            if let Some(symbols) = stacktrace.symbols {
                if symbols.is_empty() {
                    continue;
                }

                // Reverse the stacktrace to have the outermost function first
                let mut sym_vec = symbols;
                sym_vec.reverse();

                // Remove trailing unknown symbols
                while sym_vec.len() > 1 && sym_vec.last() == Some(&UNKNOWN_SYMBOL.to_string()) {
                    sym_vec.pop();
                }

                let stack_key = sym_vec.join(";");
                let count = if to_microsecond {
                    stacktrace.sample_count * profiler_result.sampling_period_in_us
                } else {
                    stacktrace.sample_count
                };

                *stacks.entry(stack_key).or_insert(0) += count;
            }
        }

        Ok(FlamegraphData::new(&stacks, to_microsecond))
    }
}
