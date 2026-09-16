use anyhow::{Context, Result};

use crate::{cli::CommandHandler, config::Config, stage::repository::StageRepository};

pub struct StageListCommandHandler {}

impl CommandHandler for StageListCommandHandler {
    fn run_command(&self, config: &Config) -> Result<()> {
        let stages = StageRepository::load_from_config(config)
            .with_context(|| "Failed to load stages from config.")?;

        let json_output = serde_json::to_string_pretty(&stages)?;

        println!("{}", json_output);

        Ok(())
    }
}
