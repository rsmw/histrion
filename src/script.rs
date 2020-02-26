use std::sync::Arc;

use crate::action::Action;
use crate::time::Interval;

#[derive(Clone, Debug)]
pub struct Script {
    pub(crate) body: Arc<[Action]>,
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

impl Script {
    pub fn new(body: Arc<[Action]>) -> Self {
        Script { body }
    }

    pub fn into_inner(&self) -> Arc<[Action]> {
        self.body.clone()
    }
}

impl From<TimeExpr> for Interval {
    fn from(src: TimeExpr) -> Self {
        match src {
            TimeExpr::Constant { number, unit } => {
                Interval::from_f64(number * f64::from(unit))
            }
        }
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

impl Default for Script {
    fn default() -> Self {
        use crate::action::Expr;

        let body =  vec![
                Action::Spawn {
                    name: "Mars".into(),
                },

                Action::WriteLocal {
                    name: "foo".into(),
                    value: Expr::NumConst {
                        value: 2.0,
                    }.into(),
                },

                Action::AsActor {
                    name: "Mars".into(),
                    script: vec![
                        Action::SetAccel { value: (1e-5, 0.0, 0.0).into(), },

                        Action::Wait {
                            interval: TimeExpr::Constant {
                                number: 0.5,
                                unit: TimeUnit::Hour,
                            }.into(),
                        },

                        Action::Trace {
                            expr: Expr::Var { name: "foo".into(), }.into(),
                        },

                        Action::SetAccel { value: (-1e-5, 0.0, 0.0).into(), },

                        Action::Wait {
                            interval: TimeExpr::Constant {
                                number: 0.5,
                                unit: TimeUnit::Hour,
                            }.into(),
                        },

                        Action::Trace {
                            expr: Expr::Field {
                                subject: Expr::Myself.into(),
                                field_name: "x".into(),
                            }.into(),
                        },

                        Action::Transmit {
                            head: "arrived".into(),
                            args: vec![
                                Expr::Var { name: "Mars".into() },
                            ].into(),
                        },
                    ].into()
                },

                Action::ListenFor {
                    head: "arrived".into(),
                    args: vec![
                        Expr::Var { name: "Mars".into() },
                    ].into(),
                },

                Action::Halt,
        ].into();

        Script { body }
    }
}
