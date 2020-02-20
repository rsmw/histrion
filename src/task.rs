use crate::action::Action;
use crate::time::Instant;

pub struct QueuedTask {
    pub(crate) eta: Instant,
    pub(crate) counter: u64,
    pub(crate) action: Action,
}

impl QueuedTask {
    pub fn new(eta: Instant, action: Action, counter: u64) -> Self {
        QueuedTask {
            eta,
            counter,
            action,
        }
    }

    pub fn eta(&self) -> Instant {
        self.eta
    }

    pub fn action(&self) -> Action {
        self.action.clone()
    }
}

impl Eq for QueuedTask {

}

impl Ord for QueuedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.eta.cmp(&other.eta).then(self.counter.cmp(&other.counter))
    }
}

impl PartialEq for QueuedTask {
    fn eq(&self, other: &Self) -> bool {
        self.eta == other.eta && self.counter == other.counter
    }
}

impl PartialOrd for QueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<QueuedTask> for (Instant, Action) {
    fn from(QueuedTask { eta, action, .. }: QueuedTask) -> Self {
        (eta, action)
    }
}
