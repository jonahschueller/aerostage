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
    pub output: Option<String>,

    #[arg(long)]
    pub workspaces: Option<String>,
    #[arg(long)]
    pub default_workspace: Option<String>,
}

#[derive(Args, Debug)]
pub struct RestoreArgs {
    pub stage: String,
}

impl From<RestoreArgs> for RestoreCommandHandler {
    fn from(args: RestoreArgs) -> Self {
        RestoreCommandHandler { stage: args.stage }
    }
}

impl From<CaptureArgs> for CaptureCommandHandler {
    fn from(args: CaptureArgs) -> Self {
        CaptureCommandHandler {
            output: args.output,
            workspaces: args.workspaces,
            default_workspace: args.default_workspace,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    Capture(CaptureArgs),
    Restore(RestoreArgs),
}

impl Commands {
    fn create_handler(self) -> Box<dyn CommandHandler> {
        match self {
            Self::Capture(capture_args) => Box::new(CaptureCommandHandler::from(capture_args)),
            Self::Restore(restore_args) => Box::new(RestoreCommandHandler::from(restore_args)),
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
