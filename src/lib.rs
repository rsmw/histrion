pub mod action;
pub mod time;
pub mod task;
pub mod script;
pub mod pretty_print;

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use specs::{prelude::*, Component, VecStorage};

use action::*;
use time::*;
use task::*;

use vek::Vec3;

pub struct Workspace {
    has_halted: bool,
    world: World,
    now: Instant,
    globals: HashMap<Arc<str>, Entity>,
    methods: HashMap<Arc<str>, Arc<Method>>,
    supervisor: Entity,
    task_counter: u64,
}

/// Trajectory in space, as a function from time to position
#[derive(Copy, Clone, Component)]
#[storage(VecStorage)]
pub enum Trajectory {
    Fixed {
        value: Position,
    },

    Linear {
        start_place: Position,
        start_time: Instant,
        start_velocity: Vec3<f64>,
        accel: Vec3<f64>,
    },

    // TODO: Orbit, once parent/child entities are implemented

    // TODO: ThrustBrake, with accel in light-seconds per second per second
    // Total travel time is 2.0 * (distance / accel).sqrt(), for const accel

    // TODO: ThrustCoast, maybe
}

#[derive(Default, Component)]
#[storage(VecStorage)]
pub struct Agenda {
    next: Option<QueuedTask>,
    listening: HashMap<Signal, Waiting>,
}

/// Current position in space, measured in light-seconds
#[derive(Copy, Clone, Component)]
#[storage(VecStorage)]
pub struct Position(vek::Vec3<f64>);

#[derive(Copy, Clone, Component)]
#[storage(VecStorage)]
pub struct CreationDate(Instant);

#[derive(Clone, Component)]
#[storage(VecStorage)]
pub struct Name(Arc<str>);

#[derive(Copy, Clone, Component)]
#[storage(VecStorage)]
pub enum Liveness {
    Alive,
    Dead,
}

#[derive(Clone, Debug)]
pub enum Error {
    NoSuchGlobal { name: Arc<str>, },
    MissingPosition { name: Arc<str>, },
    CouldNotWrite { component: &'static str, },
    NoSuchField { name: Arc<str>, on_value: Value },
    NoSuchMethod { name: Arc<str>, },
    ArgListMismatch { name: Arc<str>, wanted: usize, got: usize, },
}

pub type Result<T, E=Error> = std::result::Result<T, E>;

impl Workspace {
    pub fn new() -> Self {
        let mut world = World::new();

        world.register::<Position>();
        world.register::<Trajectory>();
        world.register::<Agenda>();
        world.register::<CreationDate>();
        world.register::<Liveness>();
        world.register::<Name>();

        let init_name: Arc<str> = "Everything".into();

        let supervisor = world.create_entity()
            .with(Name(init_name.clone()))
            .with(Agenda::default())
            .build();

        let mut globals = HashMap::new();
        globals.insert(init_name, supervisor);

        let methods = HashMap::new();

        Workspace {
            now: Instant::default(),
            world,
            globals,
            methods,
            supervisor,
            has_halted: false,
            task_counter: 0,
        }
    }

    pub fn simulate(&mut self) -> Result<()> {
        while !self.has_halted {
            self.update()?;
        }

        Ok(())
    }

    pub fn has_halted(&self) -> bool {
        self.has_halted
    }

    pub fn perform(&mut self, script: Arc<[Action]>) -> Result<()> {
        self.run(Fiber::new(self.supervisor, script).into())
    }

    fn run(&mut self, mut fiber: Box<Fiber>) -> Result<()> {
        while let Some(action) = fiber.fetch() {
            eprintln!("{:<8.0}: {}", f64::from(self.now), action);

            match action {
                Action::Halt => self.has_halted = true,

                Action::Trace { expr } => {
                    eprintln!("\t> {} = {}", &expr, self.eval_expr(&fiber, &expr)?);
                },

                Action::Spawn { name } => {
                    let position = self.get_position(fiber.me)?;

                    let id = self.world.create_entity()
                        .with(Name(name.as_ref().into()))
                        .with(CreationDate(self.now))
                        .with(Agenda::default())
                        .with(Trajectory::Fixed { value: position })
                        .build();

                    self.globals.insert(name.clone(), id);
                },

                Action::AsActor { name, script } => {
                    let me = *self.globals.get(name.as_ref())
                        .ok_or_else(|| Error::NoSuchGlobal { name: name.clone() })?;

                    let locals = fiber.frame().unwrap().locals.clone();

                    let mut fiber = Box::new(Fiber::new(me, script));
                    fiber.frame_mut().unwrap().locals = locals;
                    self.run(fiber)?;
                    // Execution resumes where it left off
                },

                Action::SetAccel { value } => {
                    let start_time = self.now;
                    let start_place = self.get_position(fiber.me)?;

                    let mut storage = self.world.write_component::<Trajectory>();

                    let trajectory = storage.get_mut(fiber.me)
                        .ok_or_else(|| Error::CouldNotWrite { component: "Trajectory" })?;

                    let start_velocity = trajectory.velocity_at(self.now);

                    if start_velocity.magnitude_squared() == 0.0 && value.magnitude_squared() == 0.0 {
                        *trajectory = Trajectory::Fixed { value: start_place };
                    } else {
                        *trajectory = Trajectory::Linear {
                            start_place,
                            start_time,
                            start_velocity,
                            accel: value,
                        }
                    };
                },

                Action::Wait { interval } => {
                    let guid = self.make_guid();
                    let token = SortToken { guid, eta: self.now + interval };

                    let me = fiber.me;
                    self.world.write_component::<Agenda>()
                        .get_mut(me)
                        .expect("Agenda missing")
                        .next = Some(QueuedTask { fiber, token });

                    break;
                },

                Action::ListenFor { head, args } => {
                    let guid = self.make_guid();

                    let body = args.iter().map(|arg| {
                        self.eval_expr(&fiber, arg)
                    }).collect::<Result<Arc<[Value]>>>()?;

                    let signal = Signal { head, body };

                    self.world.write_component::<Agenda>().get_mut(fiber.me).unwrap()
                        .listening.insert(signal, Waiting { guid, fiber });

                    break;
                },

                Action::Transmit { head, args } => {
                    let body = args.iter().map(|arg| {
                        self.eval_expr(&fiber, arg)
                    }).collect::<Result<Arc<[Value]>>>()?;

                    let signal = Signal { head, body };

                    // TODO: Light cone signal delay?
                    let mut agenda = self.world.write_component::<Agenda>();

                    for agenda in (&mut agenda).join() {
                        if let Some(Waiting { guid, fiber }) = agenda.listening.remove(&signal) {
                            let eta = self.now;
                            let token = SortToken { eta, guid };
                            agenda.next = Some(QueuedTask { token, fiber });
                        }
                    }
                },

                Action::Die => {
                    self.world.write_component::<Liveness>().insert(fiber.me, Liveness::Dead)
                    .map_err(|_err| Error::CouldNotWrite { component: "Liveness" })?;
                },

                Action::WriteLocal { name, value } => {
                    let value = self.eval_expr(&fiber, &value)?;
                    fiber.frame_mut().unwrap().locals.insert(name, value);
                },

                Action::DefGlobalMethod { name, body } => {
                    self.methods.insert(name, body);
                },

                Action::Call { name, args } => {
                    let method = self.methods.get(&name).cloned().ok_or_else(|| {
                        Error::NoSuchMethod { name: name.clone() }
                    })?;

                    if method.params.len() != args.len() {
                        return Err(Error::ArgListMismatch {
                            name: name.clone(),
                            wanted: method.params.len(),
                            got: args.len(),
                        });
                    }

                    let mut locals = HashMap::new();

                    for (param, arg) in method.params.iter().zip(args.iter()) {
                        locals.insert(param.clone(), self.eval_expr(&fiber, arg)?);
                    }

                    fiber.stack.push(StackFrame {
                        pc: 0,
                        locals,
                        script: method.script.clone(),
                    });
                },

                Action::Return => {
                    fiber.stack.pop();
                },

                //_ => eprintln!("Not yet implemented: {:?}", action),
            }

        }

        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let (time, fiber) = self.find_next_task();

        assert!(time >= self.now, "Time went backwards");
        self.now = time;
        self.world.write_component::<Position>().clear();
        self.run(fiber)
    }

    fn make_guid(&mut self) -> u64 {
        let guid = self.task_counter;
        self.task_counter += 1;
        guid
    }

    fn eval_expr(&mut self, fiber: &Fiber, expr: &Expr) -> Result<Value> {
        Ok(match expr {
            Expr::Myself => Value::ActorId(fiber.me),

            Expr::Field { subject, field_name } => {
                match self.eval_expr(fiber, &subject)? {
                    Value::ActorId(id) => match field_name.as_ref() {
                        "position" => self.get_position(id)?.into(),

                        _ => Err(Error::NoSuchField {
                            name: field_name.clone(),
                            on_value: Value::ActorId(id),
                        })?,
                    },

                    Value::Struct(dict) => {
                        dict.get(field_name.as_ref()).ok_or_else(|| {
                            Error::NoSuchField {
                                name: field_name.clone(),
                                on_value: Value::Struct(dict.clone()),
                            }
                        })?.clone()
                    },

                    other => Err(Error::NoSuchField {
                        name: field_name.clone(),
                        on_value: other,
                    })?,
                }
            },

            Expr::Var { name } => {
                fiber.frame().unwrap().locals.get(name).map(Clone::clone).or_else(|| {
                    self.globals.get(name).map(|&id| Value::ActorId(id))
                }).ok_or(Error::NoSuchGlobal { name: name.clone() })?
            },

            Expr::NumConst { value } => {
                Value::Num((*value).into())
            },
        })
    }

    fn get_position(&self, id: Entity) -> Result<Position> {
        let mut positions = self.world.write_component::<Position>();
        let trajectories = self.world.read_component::<Trajectory>();

        if let Some(position) = positions.get(id).cloned() {
            return Ok(position);
        }

        let position = trajectories.get(id).cloned()
            .unwrap_or_default().sample_at(self.now);

        positions.insert(id, position).unwrap();
        Ok(position)
    }

    // When tasks were queued globally, we just called pop() on a BinaryHeap.
    // In order to implement a cancellation policy, tasks are now queued on the
    // actors that will perform them. Finding the task in question thus becomes
    // a little bit more complicated: We have to inspect every actor.
    fn find_next_task(&mut self) -> (Instant, Box<Fiber>) {
        let entities = self.world.entities();
        let mut agenda = self.world.write_component::<Agenda>();

        // Expanded from a .filter_map().min_by_key() iterator
        // because the borrow checker got a little confused
        let mut next: Option<(Entity, SortToken)> = None;
        for (id, agenda) in (&entities, &agenda).join() {
            let token = match agenda.next.clone() {
                Some(task) => task.token,
                None => continue,
            };

            if let Some((_, prev_token)) = next.as_ref() {
                if prev_token < &token {
                    continue;
                }
            }

            next = Some((id, token));
        }

        if let Some((id, _token)) = next {
            let task = agenda.get_mut(id).unwrap().next.take().unwrap();
            return task.into();
        } else {
            let eta = self.now + Interval::one();
            let script = vec![Action::Halt].into();
            let fiber = Fiber::new(self.supervisor, script).into();
            (eta, fiber)
        }
    }
}

impl Trajectory {
    pub fn velocity_at(&self, time: Instant) -> Vec3<f64> {
        match *self {
            Trajectory::Fixed { .. } => Vec3::zero(),

            Trajectory::Linear {
                start_time,
                start_velocity,
                accel,
                ..
            } => {
                let dt = start_time.delta(time);
                let dv = accel * f64::from(dt).powi(2) * 0.5;
                start_velocity + dv
            },
        }
    }

    pub fn sample_at(&self, time: Instant) -> Position {
        match *self {
            Trajectory::Fixed { value } => value,

            Trajectory::Linear {
                start_place,
                start_time,
                ..
            } => {
                let dt = start_time.delta(time);
                let velocity = self.velocity_at(time);
                start_place.offset(velocity * f64::from(dt))
            },
        }
    }
}

impl Default for Trajectory {
    fn default() -> Self {
        Trajectory::Fixed {
            value: Position::default(),
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Position(Vec3::default())
    }
}

impl Position {
    fn offset(self, delta: Vec3<f64>) -> Self {
        Position(self.0 + delta)
    }
}

impl From<Position> for Value {
    fn from(Position(p): Position) -> Self {
        Value::Struct({
            let mut dict: BTreeMap<Arc<str>, Value> = Default::default();
            dict.insert("x".into(), Value::Num(p.x.into()));
            dict.insert("y".into(), Value::Num(p.y.into()));
            dict.insert("z".into(), Value::Num(p.z.into()));
            dict
        })
    }
}

impl Default for Liveness {
    fn default() -> Self {
        Liveness::Alive
    }
}
