mod assign;
mod command;
mod resolution;
#[cfg(test)]
mod resolution_scenario;
mod restore;
mod rule;
mod rules;
mod title;
mod types;

pub use command::RestoreCommandHandler;
pub use resolution::{WindowResolution, WindowResolver};
pub use restore::StageRestorer;
pub use types::{MatchOrigin, ResolvedWindowMatch};
