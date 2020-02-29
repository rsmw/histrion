use std::collections::HashMap;
use std::sync::Arc;

use specs::Entity;

use crate::action::{Action, Value};
use crate::time::Instant;

#[derive(Clone)]
pub struct Fiber {
    pub(crate) me: Entity,
    pub(crate) stack: Vec<StackFrame>,
}

#[derive(Clone)]
pub struct StackFrame {
    pub(crate) pc: usize,
    pub(crate) script: Arc<[Action]>,
    pub(crate) locals: HashMap<Arc<str>, Value>,
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
    pub(crate) fn new(me: Entity, script: Arc<[Action]>) -> Self {
        Fiber {
            me,
            stack: vec![
                StackFrame {
                    pc: 0,
                    script,
                    locals: HashMap::new(),
                },
            ],
        }
    }

    pub(crate) fn frame(&self) -> Option<&StackFrame> {
        self.stack.last()
    }

    pub(crate) fn frame_mut(&mut self) -> Option<&mut StackFrame> {
        self.stack.last_mut()
    }

    pub(crate) fn fetch(&mut self) -> Option<Action> {
        let frame = self.stack.last_mut()?;
        let action = frame.script.get(frame.pc)?.clone();
        frame.pc += 1;
        Some(action)
    }
}
