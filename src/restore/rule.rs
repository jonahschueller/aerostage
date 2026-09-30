use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId},
    restore::types::{ResolveTarget, ResolvedWindowMatch},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub window_id: AerospaceWindowId,
    pub score: f64,
}

pub trait WindowResolverRule {
    fn propose(&self, windows: &[AerospaceWindow], target: &ResolveTarget<'_>) -> Vec<Candidate>;

    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget<'_>,
    ) -> Option<ResolvedWindowMatch> {
        let mut hits = self.propose(windows, target);
        let only = hits.pop().filter(|_| hits.is_empty())?;
        Some(ResolvedWindowMatch {
            target_workspace: target.target_workspace.name.clone(),
            window_id: only.window_id,
        })
    }
}
