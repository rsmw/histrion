use std::fmt::{self, Display};

use crate::action::*;

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Halt => write!(f, "halt"),

            Action::Trace { expr } => {
                write!(f, "trace {}", expr)
            },

            Action::Spawn { name } => {
                write!(f, "spawn {}", fmt_actor_name(name))
            },

            Action::Wait { interval } => {
                write!(f, "wait {}sec", f64::from(*interval))
            },

            Action::ListenFor { head, .. } => {
                write!(f, "listen #{}(...)", head)
            },

            Action::AsActor { name, .. } => {
                write!(f, "as {} do ...", name)
            },

            Action::SetTrajectory { .. } => {
                write!(f, "self.trajectory = ...")
            },

            Action::Transmit { head, .. } => {
                write!(f, "transmit #{}(...)", head)
            },

            Action::WriteLocal { name, value } => {
                write!(f, "{} = {}", name, value)
            },

            //_ => write!(f, "UNIMPLEMENTED"),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::NumConst { value } => write!(f, "{}", value),
            Expr::Var { name } => write!(f, "{}", name),
        }
    }
}

impl Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Scalar::Num(value) => write!(f, "{}", value),
            Scalar::ActorId(id) => write!(f, "{:?}", id),
        }
    }
}

fn fmt_actor_name(name: &str) -> String {
    if name.contains(' ') {
        format!("[{}]", name)
    } else {
        name.to_owned()
    }
}

