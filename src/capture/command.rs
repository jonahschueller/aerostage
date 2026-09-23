use std::{fs::File, io::Write};

use anyhow::{Context, Result, ensure};

use crate::{
    aerospace::Aerospace, capture::capture::StageCapturer, cli::CommandHandler,
    stage::repository::normalize_stage_filepath,
};

pub struct CaptureCommandHandler {
    pub output: Option<String>,
    pub name: Option<String>,
    pub workspaces: Option<String>,
    pub default_workspace: Option<String>,
    pub stdout_output: bool,
}

pub(crate) fn parse_workspace_list(workspaces: &str) -> Vec<&str> {
    workspaces
        .split(',')
        .map(str::trim)
        .filter(|workspace| !workspace.is_empty())
        .collect()
}

pub(crate) fn resolve_captured_stage_name<'a>(
    name: Option<&'a str>,
    output: Option<&'a str>,
) -> Option<&'a str> {
    name.or(output)
}

impl CommandHandler for CaptureCommandHandler {
    fn run_command(&self, config: &crate::config::Config) -> Result<()> {
        ensure!(
            !self.stdout_output || self.output.is_none(),
            "--stdout output cannot be used when output is specified"
        );

        let aerospace = Aerospace::connect()?;

        let stage_filepath = if let Some(output) = self.output.as_deref() {
            Some(config.stage_directory.join(output))
        } else if !self.stdout_output {
            Some(config.stage_directory.join(&config.default_stage))
        } else {
            None
        };

        let capture_workspaces = self.workspaces.as_deref().map(parse_workspace_list);

        let capturer = StageCapturer::new(&aerospace);

        let stage = capturer
            .capture(
                resolve_captured_stage_name(self.name.as_deref(), self.output.as_deref()),
                capture_workspaces.as_deref(),
                self.default_workspace.as_deref(),
            )
            .context("Failed to capture stage")?;

        let writer: Box<dyn Write> = match &stage_filepath {
            Some(file_path) => {
                let normalized_path = normalize_stage_filepath(file_path)?;
                if let Some(parent) = normalized_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                Box::new(File::create(normalized_path)?)
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

    #[test]
    fn resolve_captured_stage_name_prefers_explicit_name() {
        assert_eq!(
            resolve_captured_stage_name(Some("focus"), Some("work")),
            Some("focus")
        );
    }

    #[test]
    fn resolve_captured_stage_name_falls_back_to_output() {
        assert_eq!(
            resolve_captured_stage_name(None, Some("work")),
            Some("work")
        );
    }

    #[test]
    fn resolve_captured_stage_name_is_none_when_both_missing() {
        assert_eq!(resolve_captured_stage_name(None, None), None);
    }
}
