use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::{
    capture::CaptureCommandHandler, config::Config, restore::RestoreCommandHandler,
    stage::commands::StageListCommandHandler,
};

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

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub json: bool,
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

impl From<ListArgs> for StageListCommandHandler {
    fn from(value: ListArgs) -> Self {
        StageListCommandHandler {
            json_output: value.json,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    Capture(CaptureArgs),
    Restore(RestoreArgs),
    List(ListArgs),
}

impl Commands {
    fn create_handler(self) -> Box<dyn CommandHandler> {
        match self {
            Self::Capture(capture_args) => Box::new(CaptureCommandHandler::from(capture_args)),
            Self::Restore(restore_args) => Box::new(RestoreCommandHandler::from(restore_args)),
            Self::List(list_args) => Box::new(StageListCommandHandler::from(list_args)),
        }
    }
}

pub fn execute_command(command: Commands, config: &Config) -> Result<()> {
    command.create_handler().run_command(config)
}
