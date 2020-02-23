use std::sync::Arc;

use ordered_float::NotNan;
use vek::*;

use crate::time::Interval;

#[derive(Clone, Debug)]
pub enum Action {
    Halt,

    CreateActor {
        name: Arc<str>,
    },

    CreateTask {
        wait_for: Arc<WaitExpr>,
        and_then: Arc<Action>,
    },

    Block {
        body: Arc<[Action]>,
    },

    AsActor {
        name: Arc<str>,
        action: Arc<Action>,
    },

    SetTrajectory {
        value: Arc<TrajectoryExpr>,
    },

    Fulfill {
        flag: Flag,
    },
}

#[derive(Clone, Debug)]
pub enum Expr {
    Var {
        name: Arc<str>,
    },

    Field {
        object: Arc<Expr>,
        name: Arc<str>,
    },

    Constant {
        number: f64,
        unit: Unit,
    },
}

#[derive(Clone, Debug)]
pub enum TrajectoryExpr {
    Fixed {
        value: Vec3<f64>,
    },

    Linear {
        velocity: Vec3<f64>,
    },
}

#[derive(Clone, Debug)]
pub enum WaitExpr {
    Delay {
        interval: Interval,
    },

    Flag {
        head: Arc<str>,
        args: Arc<[ArgExpr]>,
    },
}

#[derive(Clone, Debug)]
pub enum ArgExpr {
    NumConst {
        value: f64,
    },

    ActorName {
        name: Arc<str>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Flag {
    pub head: Arc<str>,
    pub body: Arc<[Scalar]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Scalar {
    ActorId(specs::Entity),
    Num(NotNan<f64>),
}

#[derive(Copy, Clone, Debug)]
pub enum Unit {
    Cee,
    Gee,
    LightSec,
}

impl WaitExpr {
    pub fn and_then(self, and_then: impl Into<Arc<Action>>) -> Action {
        let wait_for = self.into();
        let and_then = and_then.into();
        Action::CreateTask {
            wait_for,
            and_then,
        }
    }
}

impl Action {
    pub fn kind(&self) -> &'static str {
        match self {
            Action::Halt => "halt",
            Action::Block { .. } => "block",
            Action::CreateTask { .. } => "create_task",
            Action::CreateActor { .. } => "create_actor",
            Action::AsActor { .. } => "as_actor",
            Action::SetTrajectory { .. } => "set_trajectory",
            Action::Fulfill { .. } => "fulfill",
        }
    }
}
