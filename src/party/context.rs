use crate::{
    clock::Clock, git::Push, history::PushHistory, scoring::PointsBreakdown, state::State,
};

/// Constructed per-push and passed to each party that is
/// to be rendered
pub struct RenderContext<'a> {
    pub push: &'a Push,
    pub history: &'a PushHistory,
    pub breakdown: &'a PointsBreakdown,
    pub state: &'a State,
    pub clock: &'a Clock,
    pub pack_thresholds: Vec<u64>,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        push: &'a Push,
        history: &'a PushHistory,
        breakdown: &'a PointsBreakdown,
        state: &'a State,
        clock: &'a Clock,
        pack_thresholds: Vec<u64>,
    ) -> Self {
        Self {
            push,
            history,
            breakdown,
            state,
            clock,
            pack_thresholds,
        }
    }
}
