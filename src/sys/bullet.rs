use std::sync::mpsc;
use cgmath::{Rad};
use specs;
use world as w;

pub enum Event {
    EvShoot(bool),
}

pub struct System {
    input: mpsc::Receiver<Event>,
    shoot: bool,
    ship_entity: specs::Entity,
    visual: w::VisualType,
    cool_time: f32,
}

impl System {
    pub fn new(chan: mpsc::Receiver<Event>, ship: specs::Entity, visual: w::VisualType)
               -> System
    {
        System {
            input: chan,
            shoot: false,
            ship_entity: ship,
            visual: visual,
            cool_time: 1.0,
        }
    }

    fn check_input(&mut self) {
        loop {
            match self.input.try_recv() {
                Ok(Event::EvShoot(value)) => self.shoot = value,
                Err(_) => return,
            }
        }
    }

    fn spawn(&self, w: &specs::World) -> specs::Entity {
        use specs::Storage;
        let velocity = 5.0f32;
        let s0 = {
            let s = w.read::<w::Spatial>();
            s.get(self.ship_entity).unwrap().clone()
        };
        let i0 = {
            let s = w.read::<w::Inertial>();
            s.get(self.ship_entity).unwrap().clone()
        };
        w.create_now()
            .with(w::Bullet {
                life_time: Some(1.0),
            })
            .with(self.visual.clone())
            .with(w::Spatial {
                pos: s0.pos + s0.get_direction() * 0.5,
                orient: Rad{ s: 0.0 },
                scale: 0.1,
            })
            .with(w::Inertial {
                velocity: i0.velocity + s0.get_direction() * velocity,
                angular_velocity: Rad{ s: 0.0 },
            })
            .with(w::Collision {
                radius: 0.01,
                health: 1,
                damage: 1,
            })
            .build()
    }
}

impl super::System for System {
    fn process(&mut self, plan: &mut super::Planner, time: super::Delta) {
        self.check_input();
        self.cool_time = if self.cool_time > time {self.cool_time - time} else {0.0};
        if self.shoot {
            self.spawn(&plan.world);
        }
        plan.run(move |arg| {
            let (mut bullet, entities) = arg.fetch(|w|
                (w.write::<w::Bullet>(), w.entities())
            );
            for e in entities {
                use specs::Storage;
                match bullet.get_mut(e) {
                    Some(bt) => match bt.life_time {
                        Some(ref mut t) if *t>time => {
                            *t -= time;
                        },
                        Some(_) => {
                            bt.life_time = None;
                            arg.delete(e);
                        },
                        None => (),
                    },
                    None => (),
                }
            }
        });
    }
}
