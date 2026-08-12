mod aerospace;
mod arrangement;
mod capture;
mod cli;
mod restore;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    cli::execute_command(cli.command);
}
