use std::collections::BTreeMap;
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

    Die,

    WriteLocal {
        name: Arc<str>,
        value: Arc<Expr>,
    },

    DefGlobalMethod {
        name: Arc<str>,
        body: Arc<Method>,
    },

    Call {
        name: Arc<str>,
        args: Arc<[Expr]>,
    },

    Return,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Myself,

    Field {
        subject: Arc<Expr>,
        field_name: Arc<str>,
    },

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
    pub body: Arc<[Value]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Value {
    ActorId(specs::Entity),
    Num(NotNan<f64>),
    Struct(BTreeMap<Arc<str>, Value>),
}

#[derive(Clone, Debug)]
pub struct Method {
    pub(crate) params: Arc<[Arc<str>]>,
    pub(crate) script: Arc<[Action]>,
}
