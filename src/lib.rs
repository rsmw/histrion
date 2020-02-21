pub mod action;
pub mod time;
pub mod task;

use std::cmp::Reverse;
use std::collections::{HashMap, BinaryHeap};
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
    flag_map: HashMap<Flag, Vec<Waiting>>,
    task_queue: BinaryHeap<Reverse<QueuedTask>>,
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

impl Workspace {
    pub fn new() -> Self {
        let mut world = World::new();

        world.register::<Position>();
        world.register::<Trajectory>();
        world.register::<CreationDate>();
        world.register::<Name>();

        Workspace {
            now: Instant::default(),
            world,
            globals: HashMap::new(),
            has_halted: false,
            task_queue: BinaryHeap::new(),
            task_counter: 0,
            flag_map: HashMap::new(),
        }
    }

    pub fn simulate(&mut self) {
        while !self.has_halted {
            self.update();
        }
    }

    pub fn has_halted(&self) -> bool {
        self.has_halted
    }

    pub fn perform(&mut self, action: &Action) {
        eprintln!("{:<8.0}: {}", f64::from(self.now), action.kind());

        match action {
            Action::Halt => self.has_halted = true,

            Action::Block { body } => {
                for action in body.iter() {
                    self.perform(action);
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

            Action::SetTrajectory { name, value } => {
                let &id = self.globals.get(name.as_ref())
                    .unwrap_or_else(|| panic!("No such name {}", name));

                let pos_fn = match value.as_ref() {
                    &TrajectoryExpr::Fixed { value } => {
                        let value = Position(value);
                        Trajectory::Fixed { value }
                    },

                    &TrajectoryExpr::Linear { velocity } => {
                        let start_time = self.now;
                        let &start_place = self.world.read_component::<Position>()
                            .get(id).unwrap_or_else(|| panic!("No position for {}", name));
                        Trajectory::Linear { velocity, start_place, start_time }
                    },
                };

                self.world.write_component::<Trajectory>()
                    .insert(id, pos_fn)
                    .unwrap_or_else(|_err| panic!("Can't write pos fn for {}", name));
            },

            Action::CreateTask { wait_for, and_then } => {
                let counter = self.task_counter;
                self.task_counter += 1;
                let action = and_then.as_ref().clone();

                match wait_for.as_ref().clone() {
                    WaitExpr::Delay { interval } => {
                        self.task_queue.push(Reverse(QueuedTask {
                            action,
                            counter,
                            eta: self.now + interval,
                        }));
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
                let ready = match self.flag_map.get_mut(&flag) {
                    Some(ready) => ready,
                    _ => return,
                };

                for Waiting { action, counter } in ready.drain(..) {
                    self.task_queue.push(Reverse(QueuedTask {
                        eta: self.now,
                        action,
                        counter,
                    }));
                }
            },

            //_ => eprintln!("Not yet implemented: {:?}", action),
        }
    }

    pub fn update(&mut self) {
        let (time, action) = self.task_queue.pop()
            .map(|Reverse(task)| task.into())
            .unwrap_or_else(|| (self.now + Interval::one(), Action::Halt));

        assert!(time >= self.now, "Time went backwards");
        self.now = time;
        self.update_positions();
        self.perform(&action);
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
