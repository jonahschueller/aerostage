pub mod app_identity;
pub mod title_match;
pub mod workspace;

pub use app_identity::{UniqueAppNameResolverRule, UniqueBundleIdResolverRule};
pub use title_match::{TitleMatchResolverRule, TitleSimilarityResolverRule};
pub use workspace::TargetWorkspaceResolverRule;
