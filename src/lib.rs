pub mod action;
pub mod time;
pub mod task;

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
    implicit_self: Entity,
    flag_map: HashMap<Flag, Vec<Waiting>>,
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

#[derive(Component)]
#[storage(VecStorage)]
pub struct Agenda(Option<QueuedTask>);

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

        let implicit_self = world.create_entity()
            .with(Name(init_name.clone()))
            .with(Agenda(None))
            .build();

        let mut globals = HashMap::new();
        globals.insert(init_name, implicit_self);

        Workspace {
            now: Instant::default(),
            world,
            globals,
            implicit_self,
            has_halted: false,
            task_counter: 0,
            flag_map: HashMap::new(),
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

    pub fn perform(&mut self, action: &Action) -> Result<()> {
        eprintln!("{:<8.0}: {}", f64::from(self.now), action.kind());

        match action {
            Action::Halt => self.has_halted = true,

            Action::Block { body } => {
                for action in body.iter() {
                    self.perform(action)?;
                }
            },

            Action::CreateActor { name } => {
                let id = self.world.create_entity()
                    .with(Name(name.as_ref().into()))
                    .with(CreationDate(self.now))
                    .with(Position::default())
                    .with(Trajectory::default())
                    .build();

                self.globals.insert(name.clone(), id);
            },

            Action::AsActor { name, action } => {
                let previous_self = self.implicit_self;
                self.implicit_self = *self.globals.get(name.as_ref())
                    .ok_or_else(|| Error::NoSuchGlobal { name: name.clone() })?;
                self.perform(action.as_ref())?;
                self.implicit_self = previous_self;
            },

            Action::SetTrajectory { value } => {
                let name = self.world.read_component::<Name>()
                    .get(self.implicit_self)
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
                            .get(self.implicit_self)
                            .ok_or(Error::MissingPosition { name: name.clone() })?;
                        Trajectory::Linear { velocity, start_place, start_time }
                    },
                };

                self.world.write_component::<Trajectory>()
                    .insert(self.implicit_self, pos_fn)
                    .map_err(|_err| Error::CouldNotWrite { name: name.clone() })?;
            },

            Action::CreateTask { wait_for, and_then } => {
                let counter = self.task_counter;
                self.task_counter += 1;
                let action = and_then.as_ref().clone();

                match wait_for.as_ref().clone() {
                    WaitExpr::Delay { interval } => {
                        self.world.write_component::<Agenda>()
                            .insert(self.implicit_self, Agenda(Some(QueuedTask {
                                action,
                                counter,
                                eta: self.now + interval,
                            }))).unwrap();
                    },

                    WaitExpr::Flag { head, args } => {
                        let flag = action::Flag {
                            head,
                            body: args.into_iter().map(|arg| match arg {
                                ArgExpr::NumConst { value } => {
                                    Scalar::Num((*value).into())
                                },

                                ArgExpr::ActorName { name } => {
                                    let &id = self.globals.get(name.as_ref()).unwrap();
                                    Scalar::ActorId(id)
                                },
                            }).collect::<Vec<Scalar>>().into(),
                        };

                        self.flag_map.entry(flag)
                            .or_default()
                            .push(Waiting {
                                counter,
                                action
                            });
                    },
                }
            },

            Action::Fulfill { flag } => {
                eprintln!("TODO: Fulfill {:?}", flag);
                /*
                let ready = match self.flag_map.get_mut(&flag) {
                    Some(ready) => ready,
                    _ => return Ok(()),
                };

                for Waiting { action, counter } in ready.drain(..) {
                    self.task_queue.push(Reverse(QueuedTask {
                        eta: self.now,
                        action,
                        counter,
                    }));
                }
                */
            },

            //_ => eprintln!("Not yet implemented: {:?}", action),
        }

        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let (time, action) = self.find_next_task();

        assert!(time >= self.now, "Time went backwards");
        self.now = time;
        self.update_positions();
        self.perform(&action)
    }

    // When tasks were queued globally, we just called pop() on a BinaryHeap.
    // In order to implement a cancellation policy, tasks are now queued on the
    // actors that will perform them. Finding the task in question thus becomes
    // a little bit more complicated: We have to inspect every actor.
    fn find_next_task(&mut self) -> (Instant, Action) {
        let entities = self.world.entities();
        let mut agenda = self.world.write_component::<Agenda>();

        // Expanded from a .filter_map().min_by_key() iterator
        // because the borrow checker got a little confused
        let mut next: Option<(Entity, QueuedTask)> = None;
        for (id, agenda) in (&entities, &agenda).join() {
            let task = match agenda.0.clone() {
                Some(task) => task,
                None => continue,
            };

            if let Some((_, prev_task)) = next.as_ref() {
                if prev_task < &task {
                    continue;
                }
            }

            next = Some((id, task));
        }

        if let Some((id, task)) = next {
            agenda.remove(id);
            self.implicit_self = id;
            return task.into();
        } else {
            (self.now + Interval::one(), Action::Halt)
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
