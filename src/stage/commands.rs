use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    cli::CommandHandler, common::format::format_table, config::Config,
    stage::repository::StageRepository,
};

#[derive(Serialize)]
struct StageListOutput<'a> {
    stage_name: Option<&'a str>,
    file_path: Option<&'a str>,
    file_name: Option<&'a str>,
}

pub struct StageListCommandHandler {
    pub json_output: bool,
}

impl CommandHandler for StageListCommandHandler {
    fn run_command(&self, config: &Config) -> Result<()> {
        let stage_files = StageRepository::load_from_config(config)
            .with_context(|| "Failed to load stages from config.")?;

        let stages_output: Vec<StageListOutput> = stage_files
            .iter()
            .map(|s| StageListOutput {
                stage_name: s.stage.name.as_deref(),
                file_name: s.path.file_name().and_then(|f| f.to_str()),
                file_path: s.path.to_str(),
            })
            .collect();

        if self.json_output {
            let json_output = serde_json::to_string_pretty(&stages_output)?;

            println!("{}", json_output);
        } else {
            let rows: Vec<_> = stages_output
                .iter()
                .map(|s| {
                    vec![
                        s.stage_name.unwrap_or(""),
                        s.file_name.unwrap_or(""),
                        s.file_path.unwrap_or(""),
                    ]
                })
                .collect();
            if !rows.is_empty() {
                let refs: Vec<_> = rows.iter().map(|r| r.as_slice()).collect();
                print!("{}", format_table(&refs)?);
            }
        }

        Ok(())
    }
}
