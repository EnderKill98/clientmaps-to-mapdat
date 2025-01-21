use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use anyhow::{Context, Result};
use clap::Parser;
use flate2::Compression;
use flate2::write::GzEncoder;
use rayon::prelude::*;

#[derive(clap::Parser)]
struct Opts {
    input_folder: PathBuf,
    output_folder: PathBuf,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    if !opts.input_folder.is_dir() {
        eprintln!("Input folder does not exist or is not a directory!");
        exit(1);
    }

    if !opts.output_folder.is_dir() {
        eprintln!("Output directory does not exist or is not a directory!");
        exit(1);
    }

    opts.input_folder.read_dir()?.into_iter().par_bridge().for_each(|entry| {
        let entry = entry.expect("DirEntry should be valid");
        let name = entry.file_name().as_os_str().to_str().unwrap().to_owned();
        if name.parse::<u32>().is_ok() {
            let input = entry.path();
            let output =  opts.output_folder.join(format!("map_{}.dat", &name));

            if input.is_dir() {
                return; // Skip folders
            }

            println!("{:?} -> {:?}", input, output);
            clientmap_to_mapdat(&input, &output).expect("Converting map");
        }
    });

    Ok(())
}

type Integer = i32;
type Long = i64;
type Byte = i8;
#[derive(serde::Serialize)]
struct Map {
    #[serde(rename = "DataVersion")]
    data_version: Integer,
    data: MapData,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MapData {
    #[serde(rename = "UUIDLeast")]
    uuid_least: Long,
    #[serde(rename = "UUIDMost")]
    uuid_most: Long,

    /// Empty TagList
    banners: fastnbt::Value,
    // ByteArray of map id's file content (128*128 => 16384 bytes: each a color/pixel)
    colors: fastnbt::ByteArray,
    dimension: String,
    /// Empty TagList
    frames: fastnbt::Value,
    locked: Byte,
    scale: Byte,
    tracking_position: Byte,
    unlimited_tracking: Byte,
    x_center: Integer,
    z_center: Integer,
}

fn clientmap_to_mapdat(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<()> {
    let colors = std::fs::read(input).context("Reading ClientMaps input file")?;

    let new_uuid = uuid::Uuid::new_v4();
    let map = Map {
        data_version: 3465,
        data: MapData {
            uuid_least: new_uuid.as_u64_pair().1 as Long,
            uuid_most: new_uuid.as_u64_pair().0 as Long,

            banners: fastnbt::Value::List(Vec::<fastnbt::Value>::new()),
            colors: fastnbt::ByteArray::new(colors.iter().map(|b| *b as Byte).collect::<Vec<i8>>()),
            dimension: String::from("minecraft:overworld"),
            frames: fastnbt::Value::List(Vec::<fastnbt::Value>::new()),
            locked: 1,
            scale: 0,
            tracking_position: 0,
            unlimited_tracking: 0,
            x_center: 64_000_000,
            z_center: 64_000_000,
        },
    };

    let output_file = std::fs::File::create(output).context("Creating output file")?;
    let new_bytes = fastnbt::to_bytes(&map).context("Encode nbt file")?;
    let mut encoder = GzEncoder::new(output_file, Compression::fast());
    encoder.write_all(&new_bytes).context("Write gzip compressed nbt file")?;
    Ok(())
}
