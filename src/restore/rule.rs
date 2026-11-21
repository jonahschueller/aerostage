use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId},
    restore::types::ResolveTarget,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub window_id: AerospaceWindowId,
    pub score: f64,
}

pub trait WindowResolverRule {
    fn propose(&self, windows: &[AerospaceWindow], target: &ResolveTarget<'_>) -> Vec<Candidate>;

    #[cfg(test)]
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget<'_>,
    ) -> Option<AerospaceWindowId> {
        let mut hits = self.propose(windows, target);
        let only = hits.pop().filter(|_| hits.is_empty())?;
        Some(only.window_id)
    }
}
