use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{
    aerospace::Aerospace,
    arrangement::Arrangement,
    capture::capture_arrangement,
    cli::Commands::{Capture, Restore},
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

fn execute_capture(output: String) -> Result<()> {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let arrangement =
        capture_arrangement(&aerospace, &output).expect("Failed to capture arrangement");

    arrangement.save_to_file(output.clone())
}

fn execute_restore(arrangement_name: String) {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let arrangement = Arrangement::load_from_file(&arrangement_name)
        .expect(&format!("Failed to load arrangement: {}", arrangement_name));

    restore_arrangement(&aerospace, &arrangement).expect(&format!(
        "Failed to restore arrangement: {}",
        arrangement_name
    ))
}

pub fn execute_command(command: Commands) {
    match command {
        Capture { output } => {
            execute_capture(output);
        }
        Restore { arrangement } => {
            execute_restore(arrangement);
        }
    }
}
