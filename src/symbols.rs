use ahash::{AHashMap, AHashSet};
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;

lazy_static! {
    static ref READELF_REGEX: Regex =
        Regex::new(r"\.text\s+PROGBITS\s+([0-9a-f]+)\s+([0-9a-f]+)").unwrap();
}

#[derive(Debug)]
pub struct Symbol {
    pub address: u64,
    pub symbol: String,
    cleaned_symbol: Option<String>,
}

impl Symbol {
    pub fn new(address: u64, symbol: String) -> Self {
        Symbol {
            address,
            symbol,
            cleaned_symbol: None,
        }
    }

    pub fn simplified_symbol(&mut self) -> &str {
        if self.cleaned_symbol.is_none() {
            self.cleaned_symbol = Some(cleanup_symbol(&self.symbol));
        }
        self.cleaned_symbol.as_ref().unwrap()
    }
}

#[derive(Debug)]
struct MappedObject {
    start_address: u64,
    end_address: u64,
    offset: u64,
    obj_path: PathBuf,
    is_executable: bool,
    all_symbols_sorted: Vec<Symbol>,
    all_addrs_sorted: Vec<u64>,
    obj_start_vma: u64,
}

#[derive(Debug)]
pub struct SymbolResolver {
    objects: Vec<MappedObject>,
}

impl SymbolResolver {
    pub fn new(
        executable: &Path,
        proc_mapped_objects: &str,
        executable_only: bool,
    ) -> Result<Self> {
        let mut objects = Vec::new();

        // Parse the mapped objects list
        for line in proc_mapped_objects.lines() {
            if line.starts_with("build=") {
                continue;
            }

            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() != 6 || !fields[1].contains('x') {
                continue;
            }

            let addr_fields: Vec<&str> = fields[0].split('-').collect();
            if addr_fields.len() != 2 {
                continue;
            }

            let obj_path_orig = PathBuf::from(fields[5]);
            let is_executable = obj_path_orig.file_name() == executable.file_name();

            let obj_path = if is_executable {
                executable.to_path_buf()
            } else {
                obj_path_orig
            };

            // Only parse executable files, not related dynamic libraries
            if executable_only && !is_executable {
                continue;
            }

            if !obj_path.exists() {
                continue;
            }

            // Get symbol information
            let all_symbols_sorted = find_object_all_symbols_sorted(&obj_path)?;

            let all_addrs_sorted: Vec<u64> = all_symbols_sorted
                .iter()
                .map(|symbol| symbol.address)
                .collect();

            let obj_start_vma = find_object_start_vma_before_linked(&obj_path)?;

            objects.push(MappedObject {
                start_address: u64::from_str_radix(addr_fields[0], 16)?,
                end_address: u64::from_str_radix(addr_fields[1], 16)?,
                offset: u64::from_str_radix(fields[2], 16)?,
                obj_path,
                is_executable,
                all_symbols_sorted,
                all_addrs_sorted,
                obj_start_vma,
            });
        }

        Ok(SymbolResolver { objects })
    }

    pub fn resolve_symbols_batch(
        &mut self,
        pcs: &AHashSet<u64>,
        simplify_symbol: bool,
        annotate_libname: bool,
    ) -> AHashMap<u64, String> {
        let mut result = AHashMap::new();

        for obj in &mut self.objects {
            for &pc in pcs {
                if pc < obj.start_address || pc >= obj.end_address {
                    continue;
                }

                let addr_before_linked = pc - obj.start_address + obj.offset + obj.obj_start_vma;

                // binary search to find the closest symbol
                let idx = match obj.all_addrs_sorted.binary_search(&addr_before_linked) {
                    Ok(i) => i,
                    Err(i) if i > 0 => i - 1,
                    _ => continue,
                };

                if idx < obj.all_symbols_sorted.len() {
                    let sym = &mut obj.all_symbols_sorted[idx];
                    let mut sym_str = if simplify_symbol {
                        sym.simplified_symbol().to_string()
                    } else {
                        sym.symbol.clone()
                    };

                    if annotate_libname && !obj.is_executable {
                        if let Some(name) = obj.obj_path.file_name() {
                            sym_str.push_str(&format!(" [{}]", name.to_string_lossy()));
                        }
                    }

                    result.insert(pc, sym_str);
                }
            }
        }

        result
    }
}

fn find_object_start_vma_before_linked(filepath: &Path) -> Result<u64> {
    let output = Command::new("readelf")
        .args(&["-W", "-S", filepath.to_str().unwrap()])
        .output()
        .context("Failed to execute readelf command")?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    if let Some(captures) = READELF_REGEX.captures(&output_str) {
        let vma = u64::from_str_radix(&captures[1], 16)?;
        let offset = u64::from_str_radix(&captures[2], 16)?;
        Ok(vma - offset)
    } else {
        Ok(0)
    }
}

fn find_object_all_symbols_sorted(filepath: &Path) -> Result<Vec<Symbol>> {
    let mut symbols = Vec::new();
    let mut success = false;

    // Try to extract symbols using nm command
    for extra_args in [&[][..], &["-D"][..]] {
        let output = Command::new("nm")
            .args(&["-C", "-n", "--defined-only", "--no-recurse-limit"])
            .args(extra_args)
            .arg(filepath.to_str().unwrap())
            .output()
            .context("Failed to execute nm command")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        if !output_str.trim().is_empty() {
            for line in output_str.lines() {
                let fields: Vec<&str> = line.splitn(3, ' ').collect();
                if fields.len() == 3 {
                    let addr = u64::from_str_radix(fields[0], 16)?;
                    let symbol = fields[2].to_string();
                    symbols.push(Symbol::new(addr, symbol));
                }
            }
            success = true;
            break;
        }
    }

    if !success {
        anyhow::bail!("Failed to extract symbols from {:?}", filepath);
    }

    symbols.sort_by_key(|symbol| symbol.address);
    Ok(symbols)
}

fn cleanup_symbol(s: &str) -> String {
    // Remove brackets and their contents
    let s = remove_matching_brackets(s, '(', ')');
    let s = remove_matching_brackets(&s, '[', ']');
    let s = remove_matching_brackets(&s, '<', '>');
    s.trim_end_matches(':').to_string()
}

fn remove_matching_brackets(s: &str, _begin: char, _end: char) -> String {
    let mut result = String::with_capacity(s.len());
    let mut depth = 0;

    for c in s.chars() {
        if c == _begin {
            depth += 1;
            continue;
        }
        if c == _end && depth > 0 {
            depth -= 1;
            continue;
        }
        if depth == 0 {
            result.push(c);
        }
    }

    result
}
