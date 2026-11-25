use std::{
    fs::{self, File, OpenOptions, TryLockError},
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::stage::repository::{StageRepositoryError, normalize_stage_filepath};

#[derive(Error, Debug)]
pub enum StageLockError {
    #[error("stage '{}' is locked by a running `aerostage watch`", path.display())]
    Busy { path: PathBuf },
    #[error("failed to lock stage '{}'", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    InvalidPath(#[from] StageRepositoryError),
}

/// Held while a process may write a stage file. The OS releases it when the
/// process exits, so a crashed watcher never leaves a stale lock.
#[derive(Debug)]
pub struct StageLock {
    _file: File,
}

impl StageLock {
    pub fn try_acquire(stage_path: &Path) -> Result<Self, StageLockError> {
        let stage_path = normalize_stage_filepath(stage_path)?;
        let io_error = |source| StageLockError::Io {
            path: stage_path.clone(),
            source,
        };

        // Saving renames a new file over the stage file, so locking the stage
        // file itself would lock an inode that the next save replaces.
        let lock_path = stage_path.with_extension("toml.lock");
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&lock_path)
            .map_err(io_error)?;

        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(StageLockError::Busy { path: stage_path }),
            Err(TryLockError::Error(source)) => Err(io_error(source)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("aerostage-lock-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_held_lock_blocks_a_second_acquire() {
        let dir = temp_dir("held");

        let _lock = StageLock::try_acquire(&dir.join("work")).unwrap();
        let error = StageLock::try_acquire(&dir.join("work.toml")).unwrap_err();

        assert!(matches!(error, StageLockError::Busy { .. }));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dropping_the_lock_releases_it() {
        let dir = temp_dir("dropped");

        drop(StageLock::try_acquire(&dir.join("work")).unwrap());

        assert!(StageLock::try_acquire(&dir.join("work")).is_ok());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn stages_are_locked_independently() {
        let dir = temp_dir("independent");

        let _work = StageLock::try_acquire(&dir.join("work")).unwrap();

        assert!(StageLock::try_acquire(&dir.join("home")).is_ok());
        let _ = fs::remove_dir_all(dir);
    }
}
