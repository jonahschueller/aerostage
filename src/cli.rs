use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{
    aerospace::Aerospace,
    arrangement::Arrangement,
    capture::capture_arrangement,
    cli::Commands::{Capture, Restore},
    config::Config,
    restore::restore_arrangement,
};

#[derive(Parser)]
#[command(name = "aerospace-arrangement")]
#[command(about = "Captures and restores aerospace workspace states", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Capture {
        #[arg(short, long)]
        output: String,
    },
    Restore {
        #[arg(short, long)]
        arrangement: String,
    },
}

fn execute_capture(output: String) {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let arrangement =
        capture_arrangement(&aerospace, &output).expect("Failed to capture arrangement");

    arrangement
        .save_to_file(output.clone())
        .expect("Failed to capture arrangement.")
}

fn execute_restore(arrangement_name: String, config: &Config) {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let arrangement_path = config.arrangement_directory.join(&arrangement_name);

    let arrangement = match Arrangement::load_from_file(&arrangement_path) {
        Ok(a) => a,
        Err(err) => {
            eprintln!("Error loading arrangement '{arrangement_name}': {err}");
            return;
        }
    };

    if let Err(err) = restore_arrangement(&aerospace, &arrangement) {
        eprintln!("Failed to restore arrangement '{arrangement_name}': {err}");
    }
}

pub fn execute_command(command: Commands, config: &Config) {
    match command {
        Capture { output } => {
            execute_capture(output);
        }
        Restore { arrangement } => {
            execute_restore(arrangement, config);
        }
    }
}
