use anyhow::{Context, Result};

use crate::{
    aerospace::Aerospace, cli::CommandHandler, restore::restore::restore_stage, stage::Stage,
};

pub struct RestoreCommandHandler {
    stage: String,
}

impl RestoreCommandHandler {
    pub fn new(stage: String) -> Self {
        RestoreCommandHandler { stage }
    }
}

impl CommandHandler for RestoreCommandHandler {
    fn run_command(&self, config: &crate::config::Config) -> Result<()> {
        Aerospace::ensure_aerospace_installed();
        let aerospace = Aerospace::default();

        let stage_path = config.stage_directory.join(&self.stage);

        let stage = Stage::load_from_file(&stage_path)
            .with_context(|| "Failed to load stage from file.")?;

        restore_stage(&aerospace, &stage)
            .with_context(|| format!("Failed to restore stage '{}'", &self.stage))?;

        Ok(())
    }
}
