use anyhow::{Context, Result};

use crate::{
    aerospace::Aerospace, cli::CommandHandler, config::Config, stage::repository::StageRepository,
    watch::watched_stage::WatchedStage,
};

pub struct WatchCommandHandler {
    pub stage: Option<String>,
}

impl CommandHandler for WatchCommandHandler {
    fn run_command(&self, config: &Config) -> Result<()> {
        let stage_name = config.resolve_stage_name(self.stage.as_deref());
        let stage_file = StageRepository::load_from_relative_path(config, stage_name)
            .context("Failed to load stage from file.")?;
        let aerospace = Aerospace::connect()?;

        StageWatcher::start(aerospace, WatchedStage::from(stage_file.stage))
            .with_context(|| format!("Failed to restore stage '{stage_name}'"))?;

        Ok(())
    }
}

struct StageWatcher {
    aerospace: Aerospace,
    stage: WatchedStage,
}

impl StageWatcher {
    /// Restores the stage once and binds the entries whose window was matched.
    fn start(aerospace: Aerospace, mut stage: WatchedStage) -> Result<Self> {
        let focused = focused_workspace(&aerospace, &stage)?;
        stage.restore(&aerospace, focused.as_deref())?;

        Ok(Self { aerospace, stage })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{aerospace::AerospaceWindow, stage::UnresolvedStage};

    fn watched(toml: &str) -> WatchedStage {
        WatchedStage::from(toml::from_str::<UnresolvedStage>(toml).unwrap())
    }

    fn slack() -> AerospaceWindow {
        AerospaceWindow::dummy()
            .with_window_id(12)
            .with_app_name("Slack")
            .with_window_title("General")
            .with_workspace("4")
    }

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

        let watcher =
            StageWatcher::start(Aerospace::mock_with_windows(vec![slack()]), stage).unwrap();

        let binding = watcher.stage.entries()[0].binding.as_ref().unwrap();
        assert_eq!(binding.window_id, 12);
        assert_eq!(binding.workspace, "A");
    }
}
