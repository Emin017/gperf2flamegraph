mod flamegraph;
mod parser;
mod symbols;

use anyhow::Result;
use clap::{ArgAction, Parser};
use log::info;
use std::path::PathBuf;

use crate::flamegraph::Gperf2Flamegraph;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Convert gperf profiler result to flamegraph", long_about = None)]
struct Args {
    /// Executable files path
    #[clap(name = "exe")]
    exe: PathBuf,

    /// Profiler result file path
    #[clap(name = "prof")]
    prof: PathBuf,

    /// SVG output path
    #[clap(long)]
    svg_output: Option<PathBuf>,

    /// text output path
    #[clap(long)]
    text_output: Option<PathBuf>,

    /// simplify symbol name
    #[clap(long, action = ArgAction::SetTrue)]
    simplify_symbol: bool,

    /// only show executable symbols
    #[clap(long, action = ArgAction::SetTrue)]
    executable_only: bool,

    /// add annotation to the library name
    #[clap(long, action = ArgAction::SetTrue)]
    annotate_libname: bool,

    /// use microsecond as time unit
    #[clap(long, action = ArgAction::SetTrue)]
    to_microsecond: bool,
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();

    info!("parse params: {:?}", args);
    info!("reading results: {:?}", args.prof);

    let mut processor = Gperf2Flamegraph::new(&args.exe, &args.prof, args.executable_only);

    info!("Processing profiler result...");
    let flamegraph_data = processor.process(
        args.simplify_symbol,
        args.annotate_libname,
        args.to_microsecond,
    )?;

    if let Some(text_output) = args.text_output.as_ref() {
        info!("Writing text output: {:?}", text_output);
        flamegraph_data.write_text_output(text_output)?;
    }

    if let Some(svg_output) = args.svg_output.as_ref() {
        info!("Generating SVG output: {:?}", svg_output);
        flamegraph_data.write_svg_output(svg_output, &[])?;
    }

    info!("Finished processing profiler result");
    Ok(())
}
