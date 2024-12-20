#![allow(unused, dead_code)]
use std::fs::File;
use std::io::Read;

use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use vortex::aliases::hash_set::HashSet;
use vortex::array::PrimitiveArray;
use vortex::dtype::{DType, Nullability, PType};
use vortex::nbytes::ArrayNBytes;
use vortex::sampling_compressor::{CompressConfig, SamplingCompressor, ALL_COMPRESSORS};
use vortex::{ArrayDType, IntoArrayData};

pub fn load_file(path: &str) -> anyhow::Result<PrimitiveArray> {
    let mut bin_file = File::open(path)?;
    let mut floats: Vec<f32> = Vec::new();

    let mut buf: [u8; 4] = [0; 4];
    while bin_file.read_exact(&mut buf[0..4]).is_ok() {
        floats.push(f32::from_be_bytes(buf));
    }

    Ok(PrimitiveArray::from(floats))
}

pub fn main() -> anyhow::Result<()> {
    // Set the log level for everything
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Always,
    )?;

    let prims = load_file("/Users/aduffy/Downloads/sample_floats.bin")?;

    let compressor = SamplingCompressor::new_with_options(
        HashSet::from(ALL_COMPRESSORS),
        CompressConfig {
            sample_size: 64,
            sample_count: 1024,
            ..Default::default()
        },
    );

    let array = prims.into_array();
    assert_eq!(
        array.dtype(),
        &DType::Primitive(PType::F32, Nullability::NonNullable)
    );

    let compressed = compressor.compress(&array, None)?.into_array();

    println!("sampling compressor chose {}", compressed.tree_display());
    println!(
        "compression ratio: {}",
        (array.nbytes() as f64) / (compressed.nbytes() as f64)
    );

    Ok(())
}
