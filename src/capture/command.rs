use std::{fs::File, io::Write};

use anyhow::{Context, Result};

use crate::{aerospace::Aerospace, capture::capture::StageCapturer, cli::CommandHandler};

pub struct CaptureCommandHandler {
    pub output: Option<String>,
    pub workspaces: Option<String>,
    pub default_workspace: Option<String>,
}

pub(crate) fn parse_workspace_list(workspaces: &str) -> Vec<&str> {
    workspaces
        .split(',')
        .map(str::trim)
        .filter(|workspace| !workspace.is_empty())
        .collect()
}

impl CommandHandler for CaptureCommandHandler {
    fn run_command(&self, config: &crate::config::Config) -> Result<()> {
        Aerospace::ensure_aerospace_installed()?;
        let aerospace = Aerospace::default();

        let stage_filepath = self
            .output
            .as_deref()
            .map(|out| config.stage_directory.join(out));

        let capture_workspaces = self.workspaces.as_deref().map(parse_workspace_list);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_list_trims_and_drops_empty_entries() {
        assert_eq!(parse_workspace_list("1, 2, 3"), vec!["1", "2", "3"]);
        assert_eq!(parse_workspace_list("1,,2,"), vec!["1", "2"]);
    }
}
