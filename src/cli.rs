use clap::Parser;
use std::env;
use std::path::PathBuf;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::core::*;

#[derive(Parser, Debug)]
#[command(
    name = "ca-search",
    version,
    about = "Finding optimal c/a ratio for given volume",
    long_about = None
)]
pub struct CliArgs {
    #[arg(long)]
    pub cwd: Option<PathBuf>,

    #[arg(long)]
    pub init: Option<PathBuf>,

    #[arg(long)]
    pub poscar: Option<PathBuf>,

    #[arg(long, conflicts_with = "volume")]
    pub outcar: Option<PathBuf>,

    #[arg(long, conflicts_with = "outcar")] 
    pub volume: Option<f64>,

    #[arg(long, num_args = 1.., required_unless_present = "ratio_count", conflicts_with = "ratio_count")]
    pub ratios: Option<Vec<f64>>,

    #[arg(long, required_unless_present = "ratios", conflicts_with = "ratios", requires = "step")]
    pub ratio_count: Option<u32>,

    #[arg(long, conflicts_with = "ratios")]
    pub step: Option<f64>,

    #[arg(long)]
    pub run: bool,
}

pub fn run(args: CliArgs)-> Result<()> {
    let cwd = args.cwd.unwrap_or(env::current_dir()?);
    let init_path = args.init.unwrap_or(cwd.join("init"));
    let poscar_path = args.poscar.unwrap_or(cwd.join("POSCAR"));

    let volume = match args.volume {
        Some(volume) => volume,
        None => {
            let outcar_path = args.outcar.unwrap_or(cwd.join("OUTCAR"));
            let outcar_file = File::open(&outcar_path)
                .context(format!("Failed to open OUTCAR at {outcar_path:?}"))?;
            let outcar_reader = BufReader::new(outcar_file);

            let volume = get_volume(&mut outcar_reader.lines())
                .context("Could not find volume")?;
            volume
        }
    };

    let poscar = File::open(&poscar_path)
        .context(format!("Failed to open POSCAR at {poscar_path:#?}"))?;
    let poscar_reader = BufReader::new(poscar);
    let mut poscar_lines = poscar_reader.lines();

    let current_matrix = get_cell_matrix(&mut poscar_lines).
        context("Failed to read cell matrix")?;

    let ratios = match args.ratios {
        Some(r) => r,
        None => {
            let step = args.step
                .expect("Should be required for ratio_count");
            let reference = current_matrix[2][2]/current_matrix[0][0];
            let ratio_count = args.ratio_count
                .expect("Should be required if ratios are not set");
            get_ratios(reference, ratio_count, step)?
        }
    };

    let lattice_params = ratios
        .iter()
        .map(|&ratio| get_lattice_params(volume, ratio))
        .collect::<Vec<_>>();

    let cell_matrices = lattice_params
        .iter()
        .map(|&params| get_new_matrix(params, &current_matrix))
        .collect::<Vec<_>>();

    let positions = get_positions(&mut poscar_lines)
        .context("Failed to read positions")?;

    save_poscars(cwd, init_path, ratios, cell_matrices, &positions)?;

    if args.run {
        todo!()
    }

    Ok(())
}
