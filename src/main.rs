use anyhow::Result;
use clap::Parser;

use cli::{ CliArgs, run };

mod core;
mod cli;

fn main() -> Result<()> {
    let args = CliArgs::parse();
    run(args)?;

    Ok(())
}

