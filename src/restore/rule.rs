use crate::{
    aerospace::AerospaceWindow,
    restore::types::{ResolveTarget, ResolvedWindowMatch},
};

pub trait WindowResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch>;
}
