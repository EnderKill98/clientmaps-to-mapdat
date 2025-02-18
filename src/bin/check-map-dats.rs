use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::exit;
use anyhow::{Context, ensure, Result};
use clap::Parser;
use flate2::read::GzDecoder;

#[derive(clap::Parser)]
struct Opts {
    folder: PathBuf,
    #[clap(short, long)]
    verbose: bool
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    if !opts.folder.is_dir() {
        eprintln!("Specified folder does not exist or is not a directory!");
        exit(1);
    }

    let mut invalid_filenames = vec![];

    opts.folder.read_dir()?.into_iter().for_each(|entry| {
        let entry = entry.expect("DirEntry should be valid");
        let name = entry.file_name().as_os_str().to_str().unwrap().to_owned();
        if name.starts_with("map_") && name.ends_with(".dat") {
            let path = entry.path();

            if path.is_dir() {
                return; // Skip folders
            }

            if let Err(err) = check(&path) {
                invalid_filenames.push(name.to_owned());
                if opts.verbose {
                    eprintln!("{name}: {err:?}");
                }
            }
        }
    });

    println!("Invalid files ({}): {}", invalid_filenames.len(), invalid_filenames.join(" "));
    Ok(())
}

type Integer = i32;
type Byte = i8;
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct Map {
    #[serde(rename = "DataVersion")]
    data_version: Integer,
    data: MapData,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MapData {
    // ByteArray of map id's file content (128*128 => 16384 bytes: each a color/pixel)
    colors: fastnbt::ByteArray,
    dimension: String,
    locked: Byte,
    tracking_position: Byte,
    x_center: Integer,
    z_center: Integer,
}

fn check(path: impl AsRef<Path>) -> Result<()> {
    let file = std::fs::File::open(path).context("Open file")?;
    let mut decoder = GzDecoder::new(file);
    let mut bytes = vec![];
    decoder.read_to_end(&mut bytes).context("Uncompress file")?;

    let map: Map = fastnbt::from_bytes(&bytes)?;

    ensure!(map.data.colors.len() == 128*128, "Map color array is of unexpected size");
    Ok(())
}
