pub mod action;
pub mod time;
pub mod task;
pub mod script;

use std::collections::HashMap;
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
    supervisor: Entity,
    task_counter: u64,
}

/// Trajectory in space, as a function from time to position
#[derive(Component)]
#[storage(VecStorage)]
pub enum Trajectory {
    Fixed {
        value: Position,
    },

    Linear {
        start_place: Position,
        start_time: Instant,
        /// Measured in light-seconds per second
        velocity: Vec3<f64>,
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

#[derive(Clone, Debug)]
pub enum Error {
    NoSuchGlobal { name: Arc<str>, },
    MissingPosition { name: Arc<str>, },
    CouldNotWrite { name: Arc<str>, },
}

pub type Result<T, E=Error> = std::result::Result<T, E>;

impl Workspace {
    pub fn new() -> Self {
        let mut world = World::new();

        world.register::<Position>();
        world.register::<Trajectory>();
        world.register::<Agenda>();
        world.register::<CreationDate>();
        world.register::<Name>();

        let init_name: Arc<str> = "Everything".into();

        let supervisor = world.create_entity()
            .with(Name(init_name.clone()))
            .with(Agenda::default())
            .build();

        let mut globals = HashMap::new();
        globals.insert(init_name, supervisor);

        Workspace {
            now: Instant::default(),
            world,
            globals,
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
        self.run(Fiber {
            me: self.supervisor,
            pc: 0,
            locals: HashMap::new(),
            script,
        }.into())
    }

    fn run(&mut self, mut fiber: Box<Fiber>) -> Result<()> {
        while let Some(action) = fiber.fetch() {
            eprintln!("{:<8.0}: {}", f64::from(self.now), action.kind());

            match action {
                Action::Halt => self.has_halted = true,

                Action::Trace { comment } => {
                    eprintln!("\t> {}", comment);
                },

                Action::Spawn { name } => {
                    let id = self.world.create_entity()
                        .with(Name(name.as_ref().into()))
                        .with(CreationDate(self.now))
                        .with(Agenda::default())
                        .with(Position::default())
                        .with(Trajectory::default())
                        .build();

                    self.globals.insert(name.clone(), id);
                },

                Action::AsActor { name, script } => {
                    let me = *self.globals.get(name.as_ref())
                        .ok_or_else(|| Error::NoSuchGlobal { name: name.clone() })?;

                    let locals = fiber.locals.clone();

                    let fiber = Box::new(Fiber {
                        me,
                        pc: 0,
                        locals,
                        script,
                    });

                    self.run(fiber)?;
                    // Execution resumes where it left off
                },

                Action::SetTrajectory { value } => {
                    let name = self.world.read_component::<Name>()
                        .get(fiber.me)
                        .expect("Nameless entity")
                        .0.clone();

                    let pos_fn = match value.as_ref() {
                        &TrajectoryExpr::Fixed { value } => {
                            let value = Position(value);
                            Trajectory::Fixed { value }
                        },

                        &TrajectoryExpr::Linear { velocity } => {
                            let start_time = self.now;
                            let &start_place = self.world.read_component::<Position>()
                                .get(fiber.me)
                                .ok_or(Error::MissingPosition { name: name.clone() })?;
                            Trajectory::Linear { velocity, start_place, start_time }
                        },
                    };

                    self.world.write_component::<Trajectory>()
                        .insert(fiber.me, pos_fn)
                        .map_err(|_err| Error::CouldNotWrite { name: name.clone() })?;
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

                    let signal = self.eval_signal(head, args)?;

                    self.world.write_component::<Agenda>().get_mut(fiber.me).unwrap()
                        .listening.insert(signal, Waiting { guid, fiber });

                    break;
                },

                Action::Transmit { head, args } => {
                    let signal = self.eval_signal(head, args)?;

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

                //_ => eprintln!("Not yet implemented: {:?}", action),
            }

        }

        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let (time, fiber) = self.find_next_task();

        assert!(time >= self.now, "Time went backwards");
        self.now = time;
        self.update_positions();
        self.run(fiber)
    }

    fn make_guid(&mut self) -> u64 {
        let guid = self.task_counter;
        self.task_counter += 1;
        guid
    }

    fn eval_signal(&self, head: Arc<str>, args: Arc<[ArgExpr]>) -> Result<Signal> {
        let body = args.into_iter().map(|arg| match arg {
            ArgExpr::NumConst { value } => {
                Scalar::Num((*value).into())
            },

            ArgExpr::ActorName { name } => {
                let &id = self.globals.get(name.as_ref()).unwrap();
                Scalar::ActorId(id)
            },
        }).collect::<Vec<Scalar>>().into();

        Ok(Signal { head, body })
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
            (self.now + Interval::one(), Fiber {
                me: self.supervisor,
                pc: 0,
                locals: HashMap::new(),
                script: vec![Action::Halt].into(),
            }.into())
        }
    }

    fn update_positions(&mut self) {
        let time = self.now;

        self.world.exec(|(func, mut pos): (ReadStorage<'_, Trajectory>, WriteStorage<'_, Position>)| {
            for (func, pos) in (&func, &mut pos).join() {
                *pos = func.sample_at(time);
            }
        });
    }
}

impl Trajectory {
    pub fn sample_at(&self, time: Instant) -> Position {
        match *self {
            Trajectory::Fixed { value } => value,

            Trajectory::Linear {
                start_place,
                start_time,
                velocity,
            } => {
                let dt = start_time.delta(time);
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
