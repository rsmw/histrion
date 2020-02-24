use std::fmt::{self, Display};

use crate::action::*;

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Halt => write!(f, "halt"),

            Action::Trace { comment } => {
                write!(f, "trace {:?}", comment)
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

            Action::WriteLocal { name, .. } => {
                write!(f, "{} = ...", name)
            },

            //_ => write!(f, "UNIMPLEMENTED"),
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

