use std::rc::Rc;

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
