use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{config::Config, stage::Stage};

#[derive(Error, Debug)]
pub enum StageRepositoryError {
    #[error("failed to create stage file '{}'", path.display())]
    CreateFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to save stage to '{}'", path.display())]
    Save {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },
    #[error("failed to read stage file '{}'", path.display())]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse stage file '{}'", path.display())]
    Deserialize {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("'{}' is not a directory", path.display())]
    NotADirectory { path: PathBuf },
    #[error("invalid stage file path: {}", path.display())]
    InvalidStageFilePath { path: PathBuf },
    #[error("failed to read directory '{}'", path.display())]
    ReadDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read directory entry in '{}'", path.display())]
    ReadDirEntry {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

type Result<T> = std::result::Result<T, StageRepositoryError>;

pub fn normalize_stage_filepath(path: &Path) -> Result<PathBuf> {
    match path.extension() {
        None => Ok(path.with_extension("toml")),
        Some(ext) if ext == "toml" => Ok(path.to_path_buf()),
        Some(_) => Err(StageRepositoryError::InvalidStageFilePath {
            path: path.to_path_buf(),
        }),
    }
}

#[derive(Debug)]
pub struct StageFile {
    pub path: PathBuf,
    pub stage: Stage,
}

pub struct StageRepository {}

impl StageRepository {
    #[allow(dead_code)]
    pub fn save_to_file<P: AsRef<Path>>(stage: Stage, path: P) -> Result<()> {
        let path = path.as_ref();
        let file = File::create(path).map_err(|source| StageRepositoryError::CreateFile {
            path: path.to_path_buf(),
            source,
        })?;

        stage
            .write(Box::new(file))
            .map_err(|source| StageRepositoryError::Save {
                path: path.to_path_buf(),
                source,
            })
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<StageFile> {
        let path = normalize_stage_filepath(path.as_ref())?;

        let content =
            fs::read_to_string(&path).map_err(|source| StageRepositoryError::ReadFile {
                path: path.clone(),
                source,
            })?;

        let stage: Stage =
            toml::from_str(&content).map_err(|source| StageRepositoryError::Deserialize {
                path: path.clone(),
                source,
            })?;

        Ok(StageFile { path, stage })
    }

    pub fn load_from_relative_path(config: &Config, stage_name: &str) -> Result<StageFile> {
        let full_path = config.stage_directory.join(stage_name);

        StageRepository::load_from_file(full_path)
    }

    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<StageFile>> {
        let dir = dir.as_ref();

        if !dir.exists() {
            return Ok(Vec::new());
        }

        if !dir.is_dir() {
            return Err(StageRepositoryError::NotADirectory {
                path: dir.to_path_buf(),
            });
        }

        let entries = fs::read_dir(dir).map_err(|source| StageRepositoryError::ReadDir {
            path: dir.to_path_buf(),
            source,
        })?;

        let mut stages = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|source| StageRepositoryError::ReadDirEntry {
                path: dir.to_path_buf(),
                source,
            })?;

            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }

            match StageRepository::load_from_file(&path) {
                Ok(stage) => stages.push(stage),
                Err(err) => eprintln!("{err}"),
            }
        }

        stages.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));

        Ok(stages)
    }

    pub fn load_from_config(config: &Config) -> Result<Vec<StageFile>> {
        StageRepository::load_from_dir(&config.stage_directory)
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
        assert_eq!(stages[0].stage.name.as_deref(), Some("work"));
        assert_eq!(stages[0].path, dir.join("work.toml"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_from_dir_rejects_files() {
        let dir = temp_dir("not-a-dir");
        let file = dir.join("stage.toml");
        fs::write(&file, "name = \"x\"\n").unwrap();

        let error = StageRepository::load_from_dir(&file).unwrap_err();
        assert!(matches!(error, StageRepositoryError::NotADirectory { .. }));
        assert!(error.to_string().contains("is not a directory"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_from_dir_returns_empty_for_empty_directories() {
        let dir = temp_dir("empty-dir");

        let stages = StageRepository::load_from_dir(&dir).unwrap();
        assert!(stages.is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_from_dir_returns_empty_when_directory_is_missing() {
        let dir = temp_dir("missing-dir");
        fs::remove_dir_all(&dir).unwrap();

        let stages = StageRepository::load_from_dir(&dir).unwrap();
        assert!(stages.is_empty());
    }

    #[test]
    fn load_from_dir_skips_unreadable_toml_files() {
        let dir = temp_dir("skip-bad-toml");
        fs::write(
            dir.join("work.toml"),
            r#"
name = "work"

[[workspace]]
name = "1"
"#,
        )
        .unwrap();
        fs::write(dir.join("bad.toml"), "this is not toml [[[").unwrap();

        let stages = StageRepository::load_from_dir(&dir).unwrap();
        assert_eq!(stages.len(), 1);
        assert_eq!(stages[0].path, dir.join("work.toml"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_from_file_reports_parse_errors() {
        let dir = temp_dir("bad-toml");
        let file = dir.join("bad.toml");
        fs::write(&file, "this is not toml [[[").unwrap();

        let error = StageRepository::load_from_file(&file).unwrap_err();
        assert!(matches!(error, StageRepositoryError::Deserialize { .. }));

        let _ = fs::remove_dir_all(dir);
    }
}
