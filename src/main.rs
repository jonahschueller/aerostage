mod aerospace;
mod capture;
mod cli;
mod config;
mod restore;
mod stage;

use std::process::exit;

use clap::Parser;

use crate::config::Config;

fn main() {
    let cli = cli::Cli::parse();

    let config = match Config::load(cli.config) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load aerostage config: {}", err);
            exit(1);
        }
    };

    cli::execute_command(cli.command, &config);
}
