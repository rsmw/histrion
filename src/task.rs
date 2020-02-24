use std::collections::HashMap;
use std::sync::Arc;

use specs::Entity;

use crate::action::{Action, Scalar};
use crate::time::Instant;

#[derive(Clone)]
pub struct Fiber {
    pub(crate) me: Entity,
    pub(crate) pc: usize,
    pub(crate) script: Arc<[Action]>,
    pub(crate) locals: HashMap<Arc<str>, Scalar>,
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SortToken {
    pub(crate) eta: Instant,
    pub(crate) guid: u64,
}

#[derive(Clone)]
pub struct QueuedTask {
    pub(crate) token: SortToken,
    pub(crate) fiber: Box<Fiber>,
}

#[derive(Clone)]
pub struct Waiting {
    pub(crate) guid: u64,
    pub(crate) fiber: Box<Fiber>,
}

impl QueuedTask {
    pub fn new(token: SortToken, fiber: Box<Fiber>) -> Self {
        QueuedTask {
            token,
            fiber,
        }
    }
}

impl From<QueuedTask> for (Instant, Box<Fiber>) {
    fn from(task: QueuedTask) -> Self {
        (task.token.eta, task.fiber)
    }
}

impl Fiber {
    pub(crate) fn fetch(&mut self) -> Option<Action> {
        let action = self.script.get(self.pc)?.clone();
        self.pc += 1;
        Some(action)
    }
}
