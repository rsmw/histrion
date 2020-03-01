use std::fmt::{self, Display};

use crate::Position;
use crate::action::*;
use crate::script::Script;

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

            Action::ListenFor { head, args } => {
                write!(f, "listen #{}({})", head, args.iter().map(|arg| {
                    format!("{}", arg)
                }).collect::<Vec<_>>().join(", "))
            },

            Action::AsActor { name, .. } => {
                write!(f, "as {} do ...", name)
            },

            Action::SetAccel { value } => {
                write!(f, "self.accel = {}", Value::from(Position(*value)))
            },

            Action::Transmit { head, args } => {
                write!(f, "transmit #{}({})", head, args.iter().map(|arg| {
                    format!("{}", arg)
                }).collect::<Vec<_>>().join(", "))
            },

            Action::Die => {
                write!(f, "die")
            },

            Action::WriteLocal { name, value } => {
                write!(f, "{} = {}", name, value)
            },

            Action::DefGlobalMethod { name, .. } => {
                write!(f, "def {} do ...", name)
            },

            Action::Call { name, args } => {
                write!(f, "call {}({})", name, args.iter().map(|arg| {
                    format!("{}", arg)
                }).collect::<Vec<_>>().join(", "))
            },

            Action::Return => {
                write!(f, "return")
            }

            //_ => write!(f, "UNIMPLEMENTED"),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Myself => write!(f, "self"),
            Expr::Field { subject, field_name } => write!(f, "{}.{}", subject, field_name),
            Expr::NumConst { value } => write!(f, "{}", value),
            Expr::Var { name } => write!(f, "{}", name),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Num(value) => write!(f, "{}", value),
            Value::ActorId(id) => write!(f, "{:?}", id),
            Value::Struct(fields) => {
                write!(f, "{{ {} }}", fields.iter().map(|(name, value)| {
                    format!("{} = {};", name, value)
                }).collect::<Vec<String>>().join(" "))
            },
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

impl Script {
    pub fn pretty_print(&self) -> String {
        let mut printer = Printer::default();
        for action in self.body.iter() {
            printer.print_action(action);
        }
        printer.buffer
    }
}

#[derive(Default)]
struct Printer {
    indent: usize,
    buffer: String,
}

impl Printer {
    fn write_indent(&mut self) {
        const INDENT: &str = "    ";
        for _ in 0 .. self.indent {
            self.buffer.push_str(INDENT);
        }
    }

    fn print_action(&mut self, action: &Action) {
        match action {
            Action::AsActor { name, script } => {
                self.write_indent();
                self.buffer.push_str(&format!("as {} do\n", fmt_actor_name(name)));
                self.indent += 1;

                for action in script.iter() {
                    self.print_action(action);
                }

                self.indent -= 1;
                self.write_indent();
                self.buffer.push_str("done\n");
            },

            Action::DefGlobalMethod { name, body } => {
                let params = body.params.iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<String>>().join(", ");

                self.write_indent();
                self.buffer.push_str(&format!("def {}({}) do", name, params));
                self.indent += 1;

                for action in body.script.iter() {
                    self.print_action(action);
                }

                self.indent -= 1;
                self.write_indent();
                self.buffer.push_str("done\n");
            },

            _ => {
                self.write_indent();
                self.buffer.push_str(&format!("{}\n", action));
            },
        }

        if self.indent == 0 {
            self.buffer.push('\n');
        }
    }
}
