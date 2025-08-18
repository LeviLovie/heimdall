use std::path::Path;

use anyhow::{Context, Result};

fn try_main() -> Result<()> {
    let schemas: &[&Path] = &[Path::new("schemas/log.fbs")];
    let out_dir: &str = "target/schemas/";

    for schema in schemas {
        println!("cargo:rerun-if-changed={}", schema.display());
    }

    flatc_rust::run(flatc_rust::Args {
        inputs: schemas,
        out_dir: Path::new(out_dir),
        ..Default::default()
    })
    .context("Failed to run flatc")?;

    Ok(())
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}
