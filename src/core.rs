use std::fs::{File};
use std::process::{Command};
use std::io::{BufReader, BufWriter, Lines, Write};
use anyhow::{Context, Ok, Result, anyhow, ensure};
use std::path::PathBuf;

pub fn get_volume(lines: &mut Lines<BufReader<File>>) -> Option<f64> {
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
}

pub fn get_cell_matrix(lines: &mut Lines<BufReader<File>>) -> Result<[[f64; 3]; 3]> {
    lines.next();
    lines.next();

    let mut matrix = [[0.0; 3]; 3];

    for item in &mut matrix {
        *item = lines
            .next()
            .transpose()?
            .context("No cell matrix")?
            .split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| anyhow!("Didn't collect 3 items"))?;
    }

    Ok(matrix)
}

pub fn get_new_matrix(
    lattice_params: (f64, f64), 
    current_matrix: &[[f64; 3]; 3]
) -> [[f64; 3]; 3] {

    let mut new_matrix = *current_matrix;
    new_matrix[0][0] = lattice_params.0;
    new_matrix[1][1] = lattice_params.0;
    new_matrix[2][2] = lattice_params.1;

    new_matrix
}

pub fn get_ratios(reference: f64, n: u32, step: f64) -> Result<Vec<f64> > {
    if !n.is_multiple_of(2) {
        return Err(anyhow!("ratios_count must be divisible by 2"));
    }
    let lower = (1..=n/2)
        .map(|x| reference - x as f64 *step)
        .collect::<Vec<_>>();

    let upper = (0..n/2)
        .map(|x| reference + x as f64 * step)
        .collect::<Vec<_>>();

    let mut lower = lower;
    lower.extend(upper);

    Ok(lower)
}

pub fn get_lattice_params(volume: f64, ratio: f64) -> (f64, f64) {
    let a = (volume / ratio).powf(1.0/3.0);
    let c = ratio * a;

    (a, c)
}

pub fn get_positions(lines: &mut Lines<BufReader<File>>) -> Result<Vec<[f64; 3]>> {
    lines.next();
    lines.next();
    lines.next();
    
    let mut positions = Vec::new();
    for line in lines.by_ref() {
        let line = line?;
        
        if line.trim().is_empty() {
            break;
        }

        let vec = line
            .split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect::<Vec<_>>();

        positions.push(
            vec
                .try_into()
                .map_err(|_| anyhow!("Didn't collect 3 items in read_positions"))?
        );
    }

    Ok(positions)
}

pub fn save_poscars(
    cwd: PathBuf,
    init: PathBuf, 
    ratios: Vec<f64>, 
    cell_matrices: Vec<[[f64; 3]; 3]>, 
    positions: &[[f64; 3]]
) -> Result<()> {

    let Some(init_path_str) = init.to_str() else {
        return Err(anyhow!("init path not valid"))
    };

    for (ratio, matrix) in ratios.iter().zip(cell_matrices) {
        let dir = create_dir(ratio, init_path_str, cwd.clone())?;
        write_poscars(dir.join("POSCAR"), matrix, positions)?;
    }
    Ok(())
}

fn create_dir(ratio: &f64, init: &str, cwd: PathBuf) -> Result<PathBuf> {
    let Some(cwd_str) = cwd.to_str() else {
        return Err(anyhow!(format!("cwd path not valid at: {cwd:#?}")))
    };

    let path = format!("{cwd_str}/k_{ratio:.4}");

    let status = Command::new("cp")
        .arg("-r")
        .arg(format!("{}", init))
        .arg(&path)
        .status()?;
        
    ensure!(status.success(), "Failed to copy {:?} to {:?}", init, path);

    Ok(PathBuf::from(path))
}

fn write_poscars(dir: PathBuf, matrix: [[f64; 3]; 3], positions: &[[f64; 3]]) -> Result<()> {
    let output_file = File::create_new(&dir)
        .context(format!("File exists: {dir:?}"))?;
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufRead;

    use super::*;

    #[test]
    fn test_positions() {
        let file = File::open("/home/jay/remote1/p_0/r2scan/CONTCAR").unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        for _ in 0..8 {
            lines.next().unwrap();
        }

        let positions = get_positions(&mut lines).unwrap();
        println!("{positions:#?}");
        todo!()
    }
}
