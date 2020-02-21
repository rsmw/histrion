use std::rc::Rc;

use ordered_float::NotNan;
use vek::*;

use crate::time::Interval;

#[derive(Clone, Debug)]
pub enum Action {
    Halt,

    CreateActor {
        name: Rc<str>,
    },

    CreateTask {
        wait_for: Rc<WaitExpr>,
        and_then: Rc<Action>,
    },

    Block {
        body: Rc<[Action]>,
    },

    SetTrajectory {
        name: Rc<str>,
        value: Rc<TrajectoryExpr>,
    },

    Fulfill {
        flag: Flag,
    },
}

#[derive(Clone, Debug)]
pub enum Expr {
    Var {
        name: Rc<str>,
    },

    Field {
        object: Rc<Expr>,
        name: Rc<str>,
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
        head: Rc<str>,
        args: Rc<[ArgExpr]>,
    },
}

#[derive(Clone, Debug)]
pub enum ArgExpr {
    NumConst {
        value: f64,
    },

    ActorName {
        name: Rc<str>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Flag {
    pub head: Rc<str>,
    pub body: Rc<[Scalar]>,
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
    pub fn and_then(self, and_then: impl Into<Rc<Action>>) -> Action {
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
            Action::SetTrajectory { .. } => "set_trajectory",
            Action::Fulfill { .. } => "fulfill",
        }
    }
}
