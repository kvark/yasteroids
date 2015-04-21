use std::sync::mpsc;
use cgmath::{Angle, Rad, Point, Vector};
use gfx;
use id::{Id, Storage};
use world as w;

pub enum Event {
    EvShoot(bool),
}

pub struct System<R: gfx::Resources> {
    input: mpsc::Receiver<Event>,
    shoot: bool,
    ship_space_id: Id<w::Spatial>,
    ship_inertia_id: Id<w::Inertial>,
    draw_id: Id<w::Drawable<R>>,
    cool_time: f32,
    pool: Vec<w::Entity<R>>,
}

impl<R: gfx::Resources> System<R> {
    pub fn new(chan: mpsc::Receiver<Event>, space_id: Id<w::Spatial>,
               inertia_id: Id<w::Inertial>, draw_id: Id<w::Drawable<R>>)
               -> System<R> {
        System {
            input: chan,
            shoot: false,
            ship_space_id: space_id,
            ship_inertia_id: inertia_id,
            draw_id: draw_id,
            cool_time: 1.0,
            pool: Vec::new(),
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
}

impl<R: gfx::Resources + Send, C: gfx::CommandBuffer<R>, O> w::System<R, C, O> for System<R> {
    fn process(&mut self, time: w::Delta, _: &mut gfx::Renderer<R, C>, _: &O,
               data: &mut w::Components<R>, entities: &mut Vec<w::Entity<R>>) {
        self.check_input();
        self.cool_time = if self.cool_time > time {self.cool_time - time} else {0.0};
        if self.shoot && self.cool_time <= 0.0 {
            self.cool_time = 0.2;
            let velocity = 5.0f32;
            let bullet = w::Bullet {
                life_time: Some(1.0f32),
            };
            let (space, inertia) = {
                let e_space = data.space.get(self.ship_space_id);
                let e_inertia = data.inertia.get(self.ship_inertia_id);
                let offset = e_space.get_direction().mul_s(0.5);
                (w::Spatial {
                    pos: e_space.pos.add_v(&offset),
                    orient: Rad{ s: 0.0 },
                    scale: 0.1,
                }, w::Inertial {
                    velocity: e_inertia.velocity + e_space.get_direction().mul_s(velocity),
                    angular_velocity: Rad{ s: 0.0 },
                })
            };
            let collide = w::Collision {
                radius: 0.01,
                health: 1,
                damage: 1,
            };
            let ent = match self.pool.pop() {
                Some(ent) => {
                    *data.bullet.get_mut(ent.bullet.unwrap()) = bullet;
                    *data.space.get_mut(ent.space.unwrap()) = space;
                    *data.inertia.get_mut(ent.inertia.unwrap()) = inertia;
                    *data.collision.get_mut(ent.collision.unwrap()) = collide;
                    ent
                },
                None => w::Entity {
                    draw: Some(self.draw_id),
                    space: Some(data.space.add(space)),
                    inertia: Some(data.inertia.add(inertia)),
                    control: None,
                    bullet: Some(data.bullet.add(bullet)),
                    aster: None,
                    collision: Some(data.collision.add(collide)),
                },
            };
            entities.push(ent);
        }
        let (new_entities, reserve): (Vec<_>, _) = entities.drain().partition(|ent| {
            match (ent.bullet, ent.collision) {
                (Some(b_id), Some(c_id)) => {
                    let is_destroyed = data.collision.get(c_id).health == 0;
                    let bullet = data.bullet.get_mut(b_id);
                    let is_in_time = match bullet.life_time {
                        Some(ref mut t) if *t>time => {
                            *t -= time;
                            true
                        },
                        Some(_) => {
                            bullet.life_time = None;
                            false
                        },
                        None => true,
                    };
                    !is_destroyed && is_in_time
                },
                _ => true,
            }
        });
        *entities = new_entities;
        self.pool.extend(reserve);
    }
}
