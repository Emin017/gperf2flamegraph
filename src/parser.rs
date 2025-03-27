use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub struct Stacktrace {
    pub sample_count: u64,
    pub pcs: Vec<u64>,
    pub symbols: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ProfilerResult {
    pub sampling_period_in_us: u64,
    pub proc_mapped_objects: String,
    pub stacktraces: Vec<Stacktrace>,
}

pub fn parse_profiler_result(filepath: &Path) -> Result<ProfilerResult> {
    let mut file = File::open(filepath).context(format!(
        "Failed to open profiler result file: {:?}",
        filepath
    ))?;

    // Read header
    let header_count = file.read_u64::<LittleEndian>()?;
    let header_slots = file.read_u64::<LittleEndian>()?;
    let version = file.read_u64::<LittleEndian>()?;
    let sampling_period_in_us = file.read_u64::<LittleEndian>()?;
    let padding = file.read_u64::<LittleEndian>()?;

    // Verify header validity
    if header_count != 0 || header_slots != 3 || version != 0 || padding != 0 {
        anyhow::bail!("Invalid header, this profiler result is not valid");
    }

    let mut result = ProfilerResult {
        sampling_period_in_us,
        proc_mapped_objects: String::new(),
        stacktraces: Vec::new(),
    };

    // Read the analysis records
    loop {
        let sample_count = file.read_u64::<LittleEndian>()?;
        let num_pcs = file.read_u64::<LittleEndian>()?;

        if sample_count == 0 {
            if num_pcs != 1 {
                anyhow::bail!("Invalid trailer");
            }
            break;
        }

        let mut pcs = Vec::with_capacity(num_pcs as usize);
        for _ in 0..num_pcs {
            pcs.push(file.read_u64::<LittleEndian>()?);
        }

        result.stacktraces.push(Stacktrace {
            sample_count,
            pcs,
            symbols: None,
        });
    }

    // Read remaining data as mapped objects information
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    result.proc_mapped_objects = String::from_utf8_lossy(&buf).to_string();

    Ok(result)
}
