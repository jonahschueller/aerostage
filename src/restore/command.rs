use anyhow::{Context, Result};

use crate::{
    aerospace::Aerospace, cli::CommandHandler, config::Config, restore::restore::StageRestorer,
    stage::repository::StageRepository,
};

pub struct RestoreCommandHandler {
    pub stage: Option<String>,
}

impl CommandHandler for RestoreCommandHandler {
    fn run_command(&self, config: &Config) -> Result<()> {
        let aerospace = Aerospace::connect()?;

        let stage_name = config.resolve_stage_name(self.stage.as_deref());
        let stage_path = config.stage_directory.join(&stage_name);

        let stage_file = StageRepository::load_from_file(&stage_path)
            .with_context(|| "Failed to load stage from file.")?;

        let stage_restorer = StageRestorer::new(&aerospace, &stage_file.stage);

        stage_restorer
            .restore_stage()
            .with_context(|| format!("Failed to restore stage '{}'", &stage_name))?;

        Ok(())
    }
}
