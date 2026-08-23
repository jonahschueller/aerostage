use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{
    aerospace::Aerospace,
    capture::capture_stage,
    cli::Commands::{Capture, Restore},
    config::Config,
    restore::restore_stage,
    stage::Stage,
};

#[derive(Parser)]
#[command(name = "aerostage")]
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
        stage: String,
    },
}

fn execute_capture(output: String) {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let stage = capture_stage(&aerospace, &output).expect("Failed to capture stage");

    stage
        .save_to_file(output.clone())
        .expect("Failed to capture stage.")
}

fn execute_restore(stage_name: String, config: &Config) {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let stage_path = config.stage_directory.join(&stage_name);

    let stage = match Stage::load_from_file(&stage_path) {
        Ok(a) => a,
        Err(err) => {
            eprintln!("Error loading stage '{stage_name}': {err}");
            return;
        }
    };

    if let Err(err) = restore_stage(&aerospace, &stage) {
        eprintln!("Failed to restore stage '{stage_name}': {err}");
    }
}

pub fn execute_command(command: Commands, config: &Config) {
    match command {
        Capture { output } => {
            execute_capture(output);
        }
        Restore { stage } => {
            execute_restore(stage, config);
        }
    }
}
