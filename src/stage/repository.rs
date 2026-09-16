use std::{
    fs::{self, File},
    path::Path,
};

use anyhow::{Context, Result, ensure};

use crate::{config::Config, stage::Stage};

pub struct StageRepository {}

impl StageRepository {
    #[allow(dead_code)]
    pub fn save_to_file<P: AsRef<Path>>(stage: Stage, path: P) -> Result<()> {
        stage.write(Box::new(File::create(&path)?))
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Stage> {
        let path = path.as_ref();

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file '{}'.", path.display()))?;

        let stage = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from '{}'.", path.display()))?;

        Ok(stage)
    }

    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<Stage>> {
        let dir = dir.as_ref();

        ensure!(dir.is_dir(), "'{}' is not a directory", dir.display());

        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory '{}'", dir.display()))?;

        let mut stages = Vec::new();

        for entry in entries {
            let entry = entry.with_context(|| "Failed to read directory entry.")?;

            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                println!("Loading from file: {}", path.to_str().unwrap());
                let stage = StageRepository::load_from_file(&path).with_context(|| {
                    format!("Failed to load stage from file '{}'.", path.display())
                })?;

                stages.push(stage);
            }
        }

        ensure!(
            !stages.is_empty(),
            "No stages found in directory '{}'",
            dir.display()
        );

        Ok(stages)
    }

    pub fn load_from_config(config: &Config) -> Result<Vec<Stage>> {
        StageRepository::load_from_dir(&config.stage_directory)
            .with_context(|| "Failed to load stages from stage directory.")
    }
}

#[cfg(test)]
mod tests {
    use crate::stage::StageWorkspaceLayout;

    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aerostage-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parses_workspace_without_windows() {
        let stage: Stage = toml::from_str(
            r#"
name = "empty"

[[workspace]]
name = "1"
layout = "h_tiles"
"#,
        )
        .unwrap();

        assert_eq!(stage.workspaces.len(), 1);
        let workspace = stage.workspaces.first().unwrap();
        assert!(workspace.windows.is_empty());
        assert!(workspace.layout.is_some());
        assert_eq!(workspace.layout, Some(StageWorkspaceLayout::HTiles));
    }

    #[test]
    fn load_from_dir_reads_toml_stages() {
        let dir = temp_dir("load-from-dir");
        fs::write(
            dir.join("work.toml"),
            r#"
name = "work"

[[workspace]]
name = "1"
"#,
        )
        .unwrap();

        let stages = StageRepository::load_from_dir(&dir).unwrap();
        assert_eq!(stages.len(), 1);
        assert_eq!(stages[0].name.as_deref(), Some("work"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_from_dir_rejects_files() {
        let dir = temp_dir("not-a-dir");
        let file = dir.join("stage.toml");
        fs::write(&file, "name = \"x\"\n").unwrap();

        let error = StageRepository::load_from_dir(&file).unwrap_err();
        assert!(error.to_string().contains("is not a directory"));

        let _ = fs::remove_dir_all(dir);
    }
}
