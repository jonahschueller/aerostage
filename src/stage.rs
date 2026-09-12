use anyhow::{Context, Result, ensure};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::{fs, io::Write};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stage {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "workspace", default)]
    pub workspaces: Vec<StageWorkspace>,
    pub default_workspace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StageWorkspace {
    pub name: String,
    #[serde(rename = "window", default)]
    pub windows: Vec<StageWindow>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct StageWindow {
    pub app: Option<String>,
    pub title: Option<String>,
    pub bundle_id: Option<String>,
}

impl Stage {
    pub fn write(&self, writer: Box<dyn Write>) -> Result<()> {
        let toml_stage = toml::to_string_pretty(&self)
            .with_context(|| "Failed to convert stage to toml format.")?;

        let mut buffered = BufWriter::new(writer);

        writeln!(buffered, "{}", toml_stage)
            .with_context(|| "Failed to write context to output.")?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.write(Box::new(File::create(&path)?))
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Stage> {
        let path = path.as_ref();

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file '{}'.", path.display()))?;

        let stage = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from '{}'.", path.display()))?;

        Ok(stage)
    }

    #[allow(dead_code)]
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
                let stage = Stage::load_from_file(&path).with_context(|| {
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

    #[allow(dead_code)]
    pub fn load_from_config() -> Result<Vec<Stage>> {
        let config = crate::config::Config::load(None)
            .with_context(|| "Failed to load aerostage config.")?;

        Stage::load_from_dir(&config.stage_directory)
            .with_context(|| "Failed to load stages from stage directory.")
    }
}

impl StageWindow {
    #[cfg(test)]
    pub fn dummy() -> Self {
        StageWindow {
            app: Some("Test App".into()),
            title: Some("Test Title".into()),
            bundle_id: Some("com.example.test".into()),
        }
    }

    #[cfg(test)]
    pub fn with_bundle_id(mut self, bundle_id: &str) -> Self {
        self.bundle_id = Some(bundle_id.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_app(mut self, app: &str) -> Self {
        self.app = Some(app.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_title(mut self, title: Option<&str>) -> Self {
        self.title = title.map(|t| t.to_string());
        self
    }
}

impl StageWorkspace {
    #[cfg(test)]
    pub fn dummy() -> Self {
        StageWorkspace {
            name: "1".into(),
            windows: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

#[cfg(test)]
mod tests {
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
"#,
        )
        .unwrap();

        assert_eq!(stage.workspaces.len(), 1);
        assert!(stage.workspaces[0].windows.is_empty());
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

        let stages = Stage::load_from_dir(&dir).unwrap();
        assert_eq!(stages.len(), 1);
        assert_eq!(stages[0].name.as_deref(), Some("work"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_from_dir_rejects_files() {
        let dir = temp_dir("not-a-dir");
        let file = dir.join("stage.toml");
        fs::write(&file, "name = \"x\"\n").unwrap();

        let error = Stage::load_from_dir(&file).unwrap_err();
        assert!(error.to_string().contains("is not a directory"));

        let _ = fs::remove_dir_all(dir);
    }
}
