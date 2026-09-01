use std::{fs::File, io::Write};

use anyhow::{Context, Result};

use crate::{aerospace::Aerospace, capture::capture::StageCapturer, cli::CommandHandler};

pub struct CaptureCommandHandler {
    output: Option<String>,
    workspaces: Option<String>,
    default_workspace: Option<String>,
}

impl CaptureCommandHandler {
    pub fn new(
        output: Option<String>,
        workspaces: Option<String>,
        default_workspace: Option<String>,
    ) -> Self {
        CaptureCommandHandler {
            output,
            workspaces,
            default_workspace,
        }
    }
}

impl CommandHandler for CaptureCommandHandler {
    fn run_command(&self, config: &crate::config::Config) -> Result<()> {
        Aerospace::ensure_aerospace_installed();
        let aerospace = Aerospace::default();

        let stage_filepath = self
            .output
            .as_deref()
            .map(|out| config.stage_directory.join(&out));

        let capture_workspaces = self
            .workspaces
            .as_ref()
            .map(|ws| ws.split(",").collect::<Vec<_>>());

        let capturer = StageCapturer::new(&aerospace);

        let stage = capturer
            .capture(
                self.output.as_deref(),
                capture_workspaces.as_deref(),
                self.default_workspace.as_deref(),
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
}
