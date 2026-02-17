use crate::git::Push;
use crate::history::PushHistory;

use super::Clock;

/// context for evaluating bonuses on a push
pub struct PushContext<'a> {
    pub push: &'a Push,
    pub history: &'a PushHistory,
    pub clock: &'a Clock,
}
