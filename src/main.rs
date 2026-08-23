mod aerospace;
mod capture;
mod cli;
mod config;
mod restore;
mod stage;

use clap::Parser;

use crate::config::Config;

fn main() {
    let cli = cli::Cli::parse();

    let config = Config::new();
    config.make_stage_dir_if_not_exists();

    cli::execute_command(cli.command, &config);
}
