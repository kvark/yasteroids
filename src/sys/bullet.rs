use std::sync::mpsc;
use cgmath::{Rad};
use specs;
use world as w;


const COOL_TIME: f32 = 0.1;

pub enum Event {
    EvShoot(bool),
}

pub struct System {
    input: mpsc::Receiver<Event>,
    shoot: bool,
    ship_entity: specs::Entity,
    drawable: w::Drawable,
    cool_time: f32,
}

impl System {
    pub fn new(chan: mpsc::Receiver<Event>, ship: specs::Entity, drawable: w::Drawable)
               -> System
    {
        System {
            input: chan,
            shoot: false,
            ship_entity: ship,
            drawable: drawable,
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
        let velocity = 5.0f32;
        let s0 = {
            let s = w.read::<w::Spatial>();
            s.get(self.ship_entity).unwrap().clone()
        };
        let i0 = {
            let s = w.read::<w::Inertial>();
            s.get(self.ship_entity).unwrap().clone()
        };
        w.create_later_build()
            .with(w::Bullet {
                life_time: Some(1.0),
            })
            .with(self.drawable.clone())
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

impl specs::System<super::Delta> for System {
    fn run(&mut self, arg: specs::RunArg, time: super::Delta) {
        use specs::Join;
        self.check_input();
        let (mut bullet, entities) = arg.fetch(|w| {
            if self.shoot && self.cool_time == 0.0 {
                self.spawn(w);
                self.cool_time += COOL_TIME;
            }
            self.cool_time = (self.cool_time - time).max(0.0);
            (w.write::<w::Bullet>(), w.entities())
        });
        for (b, e) in (&mut bullet, &entities).iter() {
            match b.life_time {
                Some(ref mut t) if *t>time => {
                    *t -= time;
                },
                Some(_) => {
                    b.life_time = None;
                    arg.delete(e);
                },
                None => (),
            }
        }
    }
}
