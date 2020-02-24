use crate::action::Action;
use crate::time::Instant;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SortToken {
    pub(crate) eta: Instant,
    pub(crate) guid: u64,
}

#[derive(Clone)]
pub struct QueuedTask {
    pub(crate) token: SortToken,
    pub(crate) action: Action,
}

#[derive(Clone)]
pub struct Waiting {
    pub(crate) guid: u64,
    pub(crate) action: Action,
}

impl QueuedTask {
    pub fn new(token: SortToken, action: Action) -> Self {
        QueuedTask {
            token,
            action,
        }
    }

    pub fn eta(&self) -> Instant {
        self.token.eta
    }

    pub fn action(&self) -> Action {
        self.action.clone()
    }
}

impl From<QueuedTask> for (Instant, Action) {
    fn from(task: QueuedTask) -> Self {
        (task.token.eta, task.action)
    }
}
