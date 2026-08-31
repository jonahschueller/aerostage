use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use anyhow::{Context, Ok, Result};
use clap::{Args, Parser, Subcommand};

use crate::{
    aerospace::Aerospace,
    capture::StageCapturer,
    cli::Commands::{Capture, Restore},
    config::Config,
    restore::restore_stage,
    stage::Stage,
};

#[derive(Parser)]
#[command(name = "aerostage")]
#[command(about = "Captures and restores aerospace workspace states", long_about = None)]
pub struct Cli {
    #[arg(long)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args, Debug)]
pub struct CaptureArgs {
    output: Option<String>,

    #[arg(long)]
    workspaces: Option<String>,
    #[arg(long)]
    default_workspace: Option<String>,
}

#[derive(Args, Debug)]
pub struct RestoreArgs {
    stage: String,
}

#[derive(Subcommand)]
pub enum Commands {
    Capture(CaptureArgs),
    Restore(RestoreArgs),
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Capture { .. } => f.write_str("capture"),
            Self::Restore { .. } => f.write_str("restore"),
        }
    }
}

fn execute_capture(capture_args: &CaptureArgs, config: &Config) -> Result<()> {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let stage_filepath = capture_args
        .output
        .as_deref()
        .map(|out| config.stage_directory.join(&out));

    let capture_workspaces = capture_args
        .workspaces
        .as_ref()
        .map(|ws| ws.split(",").collect::<Vec<_>>());

    let capturer = StageCapturer::new(&aerospace);

    let stage = capturer
        .capture(
            capture_args.output.as_deref(),
            capture_workspaces.as_deref(),
            capture_args.default_workspace.as_deref(),
        )
        .context("Failed to capture stage")?;

    let writer: Box<dyn Write> = match &stage_filepath {
        Some(file_path) => {
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            Box::new(File::create(file_path)?)
        }
        None => Box::new(std::io::stdout().lock()),
    };

    stage.write(writer).context("Failed to capture stage.")?;

    Ok(())
}

fn execute_restore(restore: &RestoreArgs, config: &Config) -> Result<()> {
    Aerospace::ensure_aerospace_installed();
    let aerospace = Aerospace::default();

    let stage_path = config.stage_directory.join(&restore.stage);

    let stage =
        Stage::load_from_file(&stage_path).with_context(|| "Failed to load stage from file.")?;

    restore_stage(&aerospace, &stage)
        .with_context(|| format!("Failed to restore stage '{}'", restore.stage))?;

    Ok(())
}

pub fn execute_command(command: Commands, config: &Config) {
    let result = match &command {
        Capture(args) => execute_capture(args, &config),
        Restore(args) => execute_restore(args, &config),
    };

    if let Err(err) = result {
        eprintln!("Failed to execute command: '{}': {err}", command);
    }
}
