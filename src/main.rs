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
            report_error("Failed to load aerostage config", &err);
            exit(1);
        }
    };

    if let Err(err) = cli::execute_command(cli.command, &config) {
        report_error("Failed to execute command", &err);
        exit(1);
    }
}

fn report_error(prefix: &str, err: &anyhow::Error) {
    eprintln!("{prefix}: {err}");
    for cause in err.chain().skip(1) {
        eprintln!("  caused by: {cause}");
    }
}
