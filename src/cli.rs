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
    Capture { output: String },
    Restore { stage: String },
}

fn execute_capture(output: String, config: &Config) {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let stage_filepath = config.stage_directory.join(&output);

    let stage_name: String = stage_filepath
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| stage_filepath.display().to_string());

    let stage = capture_stage(&aerospace, &stage_name).expect("Failed to capture stage");

    stage
        .save_to_file(&stage_filepath)
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
            execute_capture(output, &config);
        }
        Restore { stage } => {
            execute_restore(stage, &config);
        }
    }
}
