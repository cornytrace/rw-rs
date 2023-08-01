use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::{bail, Result};
use clap::Parser;

use rw_rs::img::*;

#[derive(Parser)]
struct Args {
    input: PathBuf,
    name: String,
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut img = Img::new(&args.input)?;
    if let Some(file) = img.get_file(&args.name) {
        fs::write(args.output.unwrap_or(args.name.into()), file)?;
        Ok(())
    } else {
        bail!("File not found in img");
    }
}
