use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

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

#[derive(Debug)]
pub(crate) struct CaptureTarget {
    pub path: Option<PathBuf>,
    pub name: Option<String>,
}

pub(crate) fn resolve_capture_target(
    stage_directory: &Path,
    default_stage: &str,
    output: Option<&str>,
    name: Option<&str>,
    stdout_output: bool,
) -> Result<CaptureTarget> {
    ensure!(
        !stdout_output || output.is_none(),
        "--stdout cannot be combined with a stage filename"
    );

    let path = if let Some(output) = output {
        Some(stage_directory.join(output))
    } else if !stdout_output {
        Some(stage_directory.join(default_stage))
    } else {
        None
    };

    let filename = path
        .as_ref()
        .and_then(|file_path| file_path.file_name())
        .and_then(|file_name| file_name.to_str());

    Ok(CaptureTarget {
        name: resolve_captured_stage_name(name, filename).map(str::to_owned),
        path,
    })
}

impl CommandHandler for CaptureCommandHandler {
    fn run_command(&self, config: &crate::config::Config) -> Result<()> {
        let target = resolve_capture_target(
            &config.stage_directory,
            &config.default_stage,
            self.output.as_deref(),
            self.name.as_deref(),
            self.stdout_output,
        )?;

        let aerospace = Aerospace::connect()?;

        let capture_workspaces = self.workspaces.as_deref().map(parse_workspace_list);

        let capturer = StageCapturer::new(&aerospace);

        let stage = capturer
            .capture(
                target.name.as_deref(),
                capture_workspaces.as_deref(),
                self.default_workspace.as_deref(),
            )
            .context("Failed to capture stage")?;

        let writer: Box<dyn Write> = match &target.path {
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

    #[test]
    fn resolve_capture_target_writes_default_stage_when_output_is_missing() {
        let target =
            resolve_capture_target(Path::new("/stages"), "default.toml", None, None, false)
                .unwrap();

        assert_eq!(target.path.unwrap(), Path::new("/stages/default.toml"));
        assert_eq!(target.name.as_deref(), Some("default.toml"));
    }

    #[test]
    fn resolve_capture_target_prefers_explicit_name() {
        let target = resolve_capture_target(
            Path::new("/stages"),
            "default.toml",
            Some("work"),
            Some("focus"),
            false,
        )
        .unwrap();

        assert_eq!(target.path.unwrap(), Path::new("/stages/work"));
        assert_eq!(target.name.as_deref(), Some("focus"));
    }

    #[test]
    fn resolve_capture_target_stdout_has_no_path() {
        let target =
            resolve_capture_target(Path::new("/stages"), "default.toml", None, None, true).unwrap();

        assert!(target.path.is_none());
        assert!(target.name.is_none());
    }

    #[test]
    fn resolve_capture_target_stdout_keeps_explicit_name() {
        let target = resolve_capture_target(
            Path::new("/stages"),
            "default.toml",
            None,
            Some("focus"),
            true,
        )
        .unwrap();

        assert!(target.path.is_none());
        assert_eq!(target.name.as_deref(), Some("focus"));
    }

    #[test]
    fn resolve_capture_target_rejects_stdout_with_filename() {
        let error = resolve_capture_target(
            Path::new("/stages"),
            "default.toml",
            Some("work"),
            None,
            true,
        )
        .unwrap_err();

        assert!(error.to_string().contains("cannot be combined"));
    }
}
