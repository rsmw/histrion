use std::sync::Arc;

use ordered_float::NotNan;
use vek::*;

use crate::time::Interval;

#[derive(Clone, Debug)]
pub enum Action {
    Halt,

    Trace {
        expr: Arc<Expr>,
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

    SetAccel {
        value: Vec3<f64>,
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
