#![allow(unused)]

use crate::prelude::*;
use std::fs::{File, rename};
use std::process::{Command, Output};
use std::path::Path;
use std::io::{BufReader, BufWriter, Lines, Write};

mod error;
mod prelude;

fn change_line(line_number: usize, lines: &mut Lines<BufReader<File>>) -> Result<()> {
    for _ in 0..line_number {
        lines
            .next()
            .transpose()?
            .ok_or_else(|| Error::Generic("Read error in fn change_line".into()))?;
    }
    todo!()
}

fn main() -> Result<()> {
    let original_path = "SrTiO3_tetragonal.poscar";
    let temp_path = "SrTiO3_tetragonal.tmp";

    let input_file = File::open(original_path)?;
    let reader = BufReader::new(input_file);

    let output_file = File::create_new(temp_path)?;
    let mut writer = BufWriter::new(output_file);

    // do something

    writer.flush()?;
    rename(temp_path, original_path)?;

    Ok(())
}
