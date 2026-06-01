#![allow(unused)]

use std::fs::{File, rename};
use std::process::{Command, Output};
use std::path::Path;
use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
use anyhow::{Context, Result, anyhow};

fn main() -> Result<()> {
    let original_path_poscar = "SrTiO3_tetragonal.poscar";
    let outcar_path = "OUTCAR";

    let input_file = File::open(original_path_poscar)
        .context("failed to open poscar")?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();

    let outcar_file = File::open(outcar_path)
        .context("failed to open outcar")?;
    let outcar_reader = BufReader::new(outcar_file);

    let volume = get_volume(&mut outcar_reader.lines())?;

    let current_matrix = get_cell_matrix(&mut lines).
        context("Failed to read cell matrix")?;
    let ratios = [1.05, 1.10, 1.15, 1.20];
    let lattice_params = ratios
        .iter()
        .map(|&ratio| get_lattice_params(volume, ratio))
        .collect::<Vec<_>>();

    let cell_matrices = lattice_params
        .iter()
        .map(|&params| get_new_matrix(params, &current_matrix))
        .collect::<Vec<_>>();

    let positions = get_positions(&mut lines)
        .context("Failed to read positions")?;

    write_poscars(ratios, cell_matrices, &positions)?;

    Ok(())
}

fn get_volume(lines: &mut Lines<BufReader<File>>) -> Result<f64> {
    lines
        .by_ref()
        .filter_map(|res| {
            let line = res.ok()?;

            if line.contains("volume of cell") {
                line.split_whitespace().last()?.parse::<f64>().ok()
            } else {
                None
            }
        })
        .last()
        .context("Failed to find volume")
}

fn get_cell_matrix(lines: &mut Lines<BufReader<File>>) -> Result<[[f64; 3]; 3]> {
    lines.next();
    lines.next();

    let mut matrix = [[0.0; 3]; 3];

    for i in 0..matrix.len() {
         matrix[i] = lines
            .next()
            .transpose()?
            .context("No cell matrix")?
            .split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|v| anyhow!("Didn't collect 3 items"))?;
    }

    Ok(matrix)
}

fn get_new_matrix(
    lattice_params: (f64, f64), 
    current_matrix: &[[f64; 3]; 3]
) -> [[f64; 3]; 3] {

    let mut new_matrix = *current_matrix;
    new_matrix[0][0] = lattice_params.0;
    new_matrix[1][1] = lattice_params.0;
    new_matrix[2][2] = lattice_params.1;

    new_matrix
}

fn get_lattice_params(volume: f64, ratio: f64) -> (f64, f64) {
    let a = (volume / ratio).powf(1.0/3.0);
    let c = ratio * a;

    (a, c)
}

fn get_positions(lines: &mut Lines<BufReader<File>>) -> Result<Vec<[f64; 3]>> {
    lines.next();
    lines.next();
    lines.next();
    
    let mut positions = Vec::new();
    for line in lines.by_ref() {
        let vec = line?
            .split_whitespace()
            .map(|s| s.parse::<f64>().unwrap())
            .collect::<Vec<_>>();

        positions.push(
            vec
                .try_into()
                .map_err(|_| anyhow!("Didn't collect 3 items in read_positions"))?
        );
    }

    Ok(positions)
}

fn write_poscars(ratios: [f64; 4], cell_matrices: Vec<[[f64; 3]; 3]>, positions: &[[f64; 3]]) -> Result<()> {
    for (ratio, matrix) in ratios.iter().zip(cell_matrices) {
        let cmd = Command::new("mkdir")
            .arg(format!("k_{ratio}"))
            .output();

        let temp_path = format!("k_{ratio}/POSCAR");
        let output_file = File::create_new(&temp_path)
            .context(format!("File exists: {temp_path}"))?;
        let mut writer = BufWriter::new(output_file);

        writeln!(writer, "STO c/a optimization")?;
        writeln!(writer, "1.0")?;

        writeln!(writer, "   {:.16}    {:.16}    {:.16}", matrix[0][0], matrix[0][1], matrix[0][2])?;
        writeln!(writer, "   {:.16}    {:.16}    {:.16}", matrix[1][0], matrix[1][1], matrix[1][2])?;
        writeln!(writer, "   {:.16}    {:.16}    {:.16}", matrix[2][0], matrix[2][1], matrix[2][2])?;

        writeln!(writer, "Sr Ti O")?;
        writeln!(writer, "4 4 12")?;
        writeln!(writer, "direct")?;

        for position in positions {
            writeln!(writer, "   {:.16}    {:.16}    {:.16}", position[0], position[1], position[2])?;
        }

        writer.flush()?;
    }
    Ok(())
}
