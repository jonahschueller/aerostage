use std::collections::{HashMap, HashSet};

use crate::aerospace::AerospaceWindowId;

/// Position of a staged window in the restore target list.
type TargetIndex = usize;

/// Similarity of one staged window to one live window.
type MatchScore = f64;

/// A staged target paired with one live window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TargetWindow {
    target_index: TargetIndex,
    window_id: AerospaceWindowId,
}

impl From<&ScoredPair> for TargetWindow {
    fn from(pair: &ScoredPair) -> Self {
        Self {
            target_index: pair.target_index,
            window_id: pair.window_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoredPair {
    pub target_index: TargetIndex,
    pub window_id: AerospaceWindowId,
    pub score: MatchScore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub target_index: TargetIndex,
    pub window_id: AerospaceWindowId,
}

/// Greedy unique matching: highest score first, then stage order, then window id.
/// Equal scores still assign (stable), so duplicate titles bind in order rather than
/// remaining unmatched.
pub fn assign_unique_best(pairs: impl IntoIterator<Item = ScoredPair>) -> Vec<Assignment> {
    let mut best: HashMap<TargetWindow, MatchScore> = HashMap::new();
    for pair in pairs {
        best.entry(TargetWindow::from(&pair))
            .and_modify(|score| *score = score.max(pair.score))
            .or_insert(pair.score);
    }

    let mut ranked: Vec<ScoredPair> = best
        .into_iter()
        .map(|(target_window, score)| ScoredPair {
            target_index: target_window.target_index,
            window_id: target_window.window_id,
            score,
        })
        .collect();

    ranked.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then(left.target_index.cmp(&right.target_index))
            .then(left.window_id.cmp(&right.window_id))
    });

    let mut used_targets = HashSet::new();
    let mut used_windows = HashSet::new();
    let mut assignments = Vec::new();

    for pair in ranked {
        if used_targets.contains(&pair.target_index) || used_windows.contains(&pair.window_id) {
            continue;
        }
        used_targets.insert(pair.target_index);
        used_windows.insert(pair.window_id);
        assignments.push(Assignment {
            target_index: pair.target_index,
            window_id: pair.window_id,
        });
    }

    assignments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn higher_score_wins_the_shared_window() {
        let assigned = assign_unique_best([
            ScoredPair {
                target_index: 0,
                window_id: 1,
                score: 0.8,
            },
            ScoredPair {
                target_index: 1,
                window_id: 1,
                score: 1.0,
            },
        ]);

        assert_eq!(
            assigned,
            vec![Assignment {
                target_index: 1,
                window_id: 1
            }]
        );
    }

    #[test]
    fn distinct_best_partners_both_bind() {
        let assigned = assign_unique_best([
            ScoredPair {
                target_index: 0,
                window_id: 10,
                score: 0.9,
            },
            ScoredPair {
                target_index: 0,
                window_id: 20,
                score: 0.75,
            },
            ScoredPair {
                target_index: 1,
                window_id: 10,
                score: 0.75,
            },
            ScoredPair {
                target_index: 1,
                window_id: 20,
                score: 0.9,
            },
        ]);

        assert_eq!(
            assigned,
            vec![
                Assignment {
                    target_index: 0,
                    window_id: 10
                },
                Assignment {
                    target_index: 1,
                    window_id: 20
                },
            ]
        );
    }

    #[test]
    fn keeps_the_higher_score_when_the_same_pair_is_proposed_twice() {
        let assigned = assign_unique_best([
            ScoredPair {
                target_index: 0,
                window_id: 1,
                score: 0.8,
            },
            ScoredPair {
                target_index: 0,
                window_id: 1,
                score: 1.0,
            },
        ]);

        assert_eq!(
            assigned,
            vec![Assignment {
                target_index: 0,
                window_id: 1
            }]
        );
    }

    #[test]
    fn equal_scores_assign_in_stage_then_window_order() {
        let assigned = assign_unique_best([
            ScoredPair {
                target_index: 1,
                window_id: 2,
                score: 1.0,
            },
            ScoredPair {
                target_index: 0,
                window_id: 2,
                score: 1.0,
            },
            ScoredPair {
                target_index: 0,
                window_id: 1,
                score: 1.0,
            },
            ScoredPair {
                target_index: 1,
                window_id: 1,
                score: 1.0,
            },
        ]);

        assert_eq!(
            assigned,
            vec![
                Assignment {
                    target_index: 0,
                    window_id: 1
                },
                Assignment {
                    target_index: 1,
                    window_id: 2
                },
            ]
        );
    }
}
