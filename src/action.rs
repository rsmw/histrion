use std::sync::Arc;

use ordered_float::NotNan;
use vek::*;

use crate::time::Interval;

#[derive(Clone, Debug)]
pub enum Action {
    Halt,

    Trace {
        comment: Arc<str>,
    },

    Spawn {
        name: Arc<str>,
    },

    Wait {
        interval: Interval,
    },

    ListenFor {
        head: Arc<str>,
        args: Arc<[Expr]>,
    },

    AsActor {
        name: Arc<str>,
        script: Arc<[Action]>,
    },

    SetTrajectory {
        value: Arc<TrajectoryExpr>,
    },

    Transmit {
        head: Arc<str>,
        args: Arc<[Expr]>,
    },

    WriteLocal {
        name: Arc<str>,
        value: Arc<Expr>,
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
pub enum Expr {
    NumConst {
        value: f64,
    },

    Var {
        name: Arc<str>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Signal {
    pub head: Arc<str>,
    pub body: Arc<[Scalar]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Scalar {
    ActorId(specs::Entity),
    Num(NotNan<f64>),
}
