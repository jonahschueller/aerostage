use std::{
    path::PathBuf,
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};

use crate::{
    aerospace::{Aerospace, AerospaceEventStream},
    cli::CommandHandler,
    config::Config,
    stage::{lock::StageLock, repository::StageRepository},
    watch::watched_stage::{WatchedStage, WindowMove},
};

const QUIET_PERIOD: Duration = Duration::from_millis(300);

pub struct WatchCommandHandler {
    pub stage: Option<String>,
}

impl CommandHandler for WatchCommandHandler {
    fn run_command(&self, config: &Config) -> Result<()> {
        let stage_name = config.resolve_stage_name(self.stage.as_deref());
        let _lock = StageLock::try_acquire(&config.stage_directory.join(stage_name))?;
        let stage_file = StageRepository::load_from_relative_path(config, stage_name)
            .context("Failed to load stage from file.")?;
        let aerospace = Aerospace::connect()?;

        // Subscribe first, so changes made during the restore are not missed.
        let events = aerospace
            .subscribe()
            .context("Failed to subscribe to AeroSpace events.")?;

        let mut watcher = StageWatcher::start(
            aerospace,
            WatchedStage::from(stage_file.stage),
            stage_file.path,
        )
        .with_context(|| format!("Failed to restore stage '{stage_name}'"))?;

        for burst in bursts(events, QUIET_PERIOD) {
            burst.context("Failed to read AeroSpace events.")?;
            if let Err(err) = watcher.sync() {
                eprintln!("Failed to sync stage '{stage_name}': {err:#}");
            }
        }

        Ok(())
    }
}

struct StageWatcher {
    aerospace: Aerospace,
    stage: WatchedStage,
    path: PathBuf,
}

impl StageWatcher {
    /// Restores the stage once and binds the entries whose window was matched.
    fn start(aerospace: Aerospace, mut stage: WatchedStage, path: PathBuf) -> Result<Self> {
        let focused = focused_workspace(&aerospace, &stage)?;
        stage.restore(&aerospace, focused.as_deref())?;

        Ok(Self {
            aerospace,
            stage,
            path,
        })
    }

    fn sync(&mut self) -> Result<()> {
        let windows = self.aerospace.list_windows()?;
        let workspaces = self.aerospace.list_workspaces()?;
        let changed = self.stage.reconcile(&windows, &workspaces);

        let focused = focused_workspace(&self.aerospace, &self.stage)?;
        let moves = self.stage.bind_new_windows(&windows, focused.as_deref())?;
        self.apply(&moves)?;

        if changed {
            StageRepository::save_to_file(&self.stage.to_stage(), &self.path)?;
        }
        Ok(())
    }

    /// A window that has focus keeps it while it moves.
    fn apply(&self, moves: &[WindowMove]) -> Result<()> {
        if moves.is_empty() {
            return Ok(());
        }

        let focused_window = self
            .aerospace
            .list_window_focused()
            .ok()
            .map(|window| window.window_id);
        for window_move in moves {
            if focused_window == Some(window_move.window_id) {
                self.aerospace.move_node_to_workspace_and_focus(
                    &window_move.workspace,
                    window_move.window_id,
                )?;
            } else {
                self.aerospace
                    .move_node_to_workspace(&window_move.workspace, window_move.window_id)?;
            }
        }
        Ok(())
    }
}

fn focused_workspace(aerospace: &Aerospace, stage: &WatchedStage) -> Result<Option<String>> {
    if !stage.to_stage().references_focused() {
        return Ok(None);
    }

    let focused = aerospace
        .list_workspace_focused()
        .context("Failed to query the focused workspace.")?;
    Ok(Some(focused.workspace))
}

/// Yields once per burst of events, after no event arrived for `quiet`. Ends
/// when the event stream ends.
fn bursts(events: AerospaceEventStream, quiet: Duration) -> impl Iterator<Item = Result<()>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for event in events {
            if sender.send(event).is_err() {
                break;
            }
        }
    });

    std::iter::from_fn(move || {
        if let Err(err) = receiver.recv().ok()? {
            return Some(Err(err));
        }
        loop {
            match receiver.recv_timeout(quiet) {
                Ok(Ok(_)) => continue,
                Ok(Err(err)) => return Some(Err(err)),
                Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
                    return Some(Ok(()));
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::{
        aerospace::{AerospaceEvent, AerospaceWindow},
        stage::UnresolvedStage,
    };

    fn watched(toml: &str) -> WatchedStage {
        WatchedStage::from(toml::from_str::<UnresolvedStage>(toml).unwrap())
    }

    fn window(id: u32, app: &str, title: &str, workspace: &str) -> AerospaceWindow {
        AerospaceWindow::dummy()
            .with_window_id(id)
            .with_app_name(app)
            .with_window_title(title)
            .with_workspace(workspace)
    }

    fn temp_stage_file(name: &str, content: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("aerostage-watch-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("stage.toml");
        fs::write(&path, content).unwrap();
        path
    }

    const SAFARI_ON_1: &str = r#"# kept unless watch saves
[[workspace]]
name = "1"

[[workspace.window]]
app = "Safari"
title = "Docs"
"#;

    #[test]
    fn start_binds_entries_on_the_focused_workspace() {
        let stage = watched(
            r#"
[[workspace]]
name = "@focused"

[[workspace.window]]
app = "Slack"
"#,
        );
        let aerospace = Aerospace::mock_with_windows(vec![window(12, "Slack", "General", "4")]);

        let watcher = StageWatcher::start(aerospace, stage, PathBuf::new()).unwrap();

        let binding = watcher.stage.entries()[0].binding.as_ref().unwrap();
        assert_eq!(binding.window_id, 12);
        assert_eq!(binding.workspace, "A");
    }

    #[test]
    fn sync_saves_the_stage_when_a_bound_window_moved() {
        let path = temp_stage_file("moved", SAFARI_ON_1);
        // The mock ignores moves, so the window stays on "3" after the restore.
        let aerospace = Aerospace::mock_with_windows(vec![window(10, "Safari", "Docs", "3")]);
        let mut watcher =
            StageWatcher::start(aerospace, watched(SAFARI_ON_1), path.clone()).unwrap();

        watcher.sync().unwrap();

        let saved = StageRepository::load_from_file(&path).unwrap().stage;
        let with_windows: Vec<_> = saved
            .workspaces
            .iter()
            .filter(|workspace| !workspace.windows.is_empty())
            .map(|workspace| workspace.name.to_string())
            .collect();
        assert_eq!(with_windows, ["3"]);
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn sync_leaves_the_stage_file_alone_when_nothing_changed() {
        let path = temp_stage_file("unchanged", SAFARI_ON_1);
        let aerospace = Aerospace::mock_with_windows(vec![window(10, "Safari", "Docs", "1")]);
        let mut watcher =
            StageWatcher::start(aerospace, watched(SAFARI_ON_1), path.clone()).unwrap();

        watcher.sync().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), SAFARI_ON_1);
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    struct Paced(Vec<(Duration, AerospaceEvent)>);

    impl Iterator for Paced {
        type Item = Result<AerospaceEvent>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.0.is_empty() {
                return None;
            }
            let (delay, event) = self.0.remove(0);
            thread::sleep(delay);
            Some(Ok(event))
        }
    }

    #[test]
    fn bursts_group_events_that_arrive_close_together() {
        let short = Duration::from_millis(1);
        let long = Duration::from_millis(200);
        let events = Paced(vec![
            (short, AerospaceEvent::FocusChanged),
            (short, AerospaceEvent::WindowDetected),
            (long, AerospaceEvent::FocusChanged),
        ]);

        let bursts: Vec<_> = bursts(Box::new(events), Duration::from_millis(50)).collect();

        assert_eq!(bursts.len(), 2);
        assert!(bursts.iter().all(Result::is_ok));
    }
}
