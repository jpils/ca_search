use clap::Parser;

use cli::{ CliArgs, run };

mod core;
mod cli;

fn main() {
    // if log level is not set use info as default
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    let args = CliArgs::parse();
    if let Err(e) = run(args) {
         log::error!("Application error: {e}");
         std::process::exit(1);
    };
}
