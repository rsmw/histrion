use std::sync::Arc;
use std::slice;

use vek::Vec3;

use crate::action::{Action, ArgExpr};
use crate::time::Interval;

#[derive(Clone, Debug)]
pub struct Script {
    pub(crate) body: Arc<[Stmt]>,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Halt,

    Trace {
        comment: Arc<str>,
    },

    CreateActor {
        name: Arc<str>,
    },

    Wait {
        interval: TimeExpr,
    },

    ListenFor {
        name: Arc<str>,
        args: Arc<[ArgExpr]>,
    },

    AsActor {
        name: Arc<str>,
        body: Arc<[Stmt]>,
    },

    MoveTo {
        target: Vec3<f64>,
        speed: f64,
    },

    StopMoving,

    Transmit {
        name: Arc<str>,
        args: Arc<[ArgExpr]>,
    },
}

#[derive(Clone, Debug)]
pub enum TimeExpr {
    Constant {
        number: f64,
        unit: TimeUnit,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum TimeUnit {
    Sec,
    Min,
    Hour,
    Day,
    Week,
    Year,
}

#[derive(Copy, Clone, Debug)]
pub enum LenUnit {
    LightSec,
}

#[derive(Copy, Clone, Debug)]
pub enum VelocityUnit {
    Cee,
}

#[derive(Copy, Clone, Debug)]
pub enum AccelUnit {
    CeePerSec,
    Gee,
}

#[derive(Clone, Debug)]
pub enum AtomicExpr {
    Var {
        name: Arc<str>,
    },

    Field {
        object: Arc<AtomicExpr>,
        name: Arc<str>,
    },

    Constant {
        number: f64,
    },
}

impl Script {
    pub fn new(body: Arc<[Stmt]>) -> Self {
        Script { body }
    }

    pub fn to_action(&self) -> Action {
        compile_block(&mut self.body.iter())
    }
}

fn compile_block(src: &mut slice::Iter<Stmt>) -> Action {
    let mut actions = vec![];

    while let Some(stmt) = src.next() {
        actions.push(match stmt.clone() {
            Stmt::Halt => {
                Action::Halt
            },

            Stmt::Trace { comment } => {
                Action::Trace { comment }
            },

            Stmt::CreateActor { name } => {
                Action::CreateActor { name }
            },

            Stmt::Wait { interval } => {
                use crate::action::WaitExpr;

                let interval = compile_interval(&interval);

                Action::CreateTask {
                    wait_for: WaitExpr::Delay { interval }.into(),
                    and_then: compile_block(src).into(),
                }
            },

            Stmt::ListenFor { name, args } => {
                use crate::action::WaitExpr;

                Action::CreateTask {
                    wait_for: WaitExpr::Signal { head: name, args }.into(),
                    and_then: compile_block(src).into(),
                }
            },

            Stmt::AsActor { name, body } => {
                Action::AsActor {
                    name,
                    action: compile_block(&mut body.iter()).into(),
                }
            },

            Stmt::MoveTo { .. } => {
                unimplemented!()
            },

            Stmt::StopMoving => {
                use crate::action::TrajectoryExpr;

                Action::SetTrajectory {
                    value: TrajectoryExpr::Linear {
                        velocity: (0.0, 0.0, 0.0).into(),
                    }.into(),
                }
            },

            Stmt::Transmit { name, args } => {
                use crate::action::Signal;
                Action::Transmit {
                    signal: Signal {
                        head: name,
                        body: args.into_iter().map(|arg| match arg {
                            _ => unimplemented!(),
                        }).collect(),
                    },
                }
            },
        });
    }

    Action::Block { body: actions.into() }
}

fn compile_interval(src: &TimeExpr) -> Interval {
    match *src {
        TimeExpr::Constant { number, unit } => {
            Interval::from_f64(number * f64::from(unit))
        },
    }
}

impl From<TimeUnit> for f64 {
    fn from(unit: TimeUnit) -> Self {
        use TimeUnit::*;

        match unit {
            Sec => 1.0,
            Min => 60.0,
            Hour => f64::from(Min) * 60.0,
            Day => f64::from(Hour) * 24.0,
            Week => f64::from(Day) * 7.0,
            Year => f64::from(Day) * 365.2425,
        }
    }
}

impl From<AccelUnit> for f64 {
    fn from(unit: AccelUnit) -> Self {
        use AccelUnit::*;

        match unit {
            CeePerSec => 1.0,
            Gee => 9.81 / 299_792_458.0,
        }
    }
}
