use std::{fmt::Display, fs::File, io::Write};

use anyhow::{Context, Ok, Result};
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
    Capture { output: Option<String> },
    Restore { stage: String },
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capture => f.write_str("capture"),
            Restore => f.write_str("restore"),
        }
    }
}

fn execute_capture(output: Option<&str>, config: &Config) -> Result<()> {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let stage_filepath = output.map(|out| config.stage_directory.join(&out));

    let stage = capture_stage(&aerospace, output).expect("Failed to capture stage");

    let writer: Box<dyn Write> = match &stage_filepath {
        Some(file_path) => Box::new(File::create(file_path)?),
        None => Box::new(std::io::stdout().lock()),
    };

    stage
        .write(writer)
        .with_context(|| "Failed to capture stage.")?;

    Ok(())
}

fn execute_restore(stage_name: String, config: &Config) -> Result<()> {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let stage_path = config.stage_directory.join(&stage_name);

    let stage =
        Stage::load_from_file(&stage_path).with_context(|| "Failed to load stage from file.")?;

    restore_stage(&aerospace, &stage)
        .with_context(|| format!("Failed to restore stage '{}'", stage_name))?;

    Ok(())
}

pub fn execute_command(command: Commands, config: &Config) {
    let result = match &command {
        Capture { output } => execute_capture(output.as_deref(), &config),
        Restore { stage } => execute_restore(stage.clone(), &config),
    };

    if let Err(err) = result {
        eprintln!("Failed to execute command: '{}': {err}", command);
    }
}
