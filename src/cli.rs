use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::{capture::CaptureCommandHandler, config::Config, restore::RestoreCommandHandler};

pub trait CommandHandler {
    fn run_command(&self, config: &Config) -> Result<()>;
}

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

impl Commands {
    fn create_handler(self) -> Box<dyn CommandHandler> {
        match self {
            Self::Capture(capture_args) => Box::new(CaptureCommandHandler::new(
                capture_args.output,
                capture_args.workspaces,
                capture_args.default_workspace,
            )),
            Self::Restore(restore_args) => Box::new(RestoreCommandHandler::new(restore_args.stage)),
        }
    }
}

pub fn execute_command(command: Commands, config: &Config) {
    let command_handler = command.create_handler();

    let result = command_handler.run_command(config);

    if let Err(err) = result {
        eprintln!("Failed to execute command: {}", err);
    }
}
