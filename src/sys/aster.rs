use std::rand::{Rng, StdRng};
use cgmath::{Angle, Deg, Rad, ToRad, Point, Point2, Vector, sin, cos};
use ecs;
use world as w;

static KINDS: uint = 2;

pub struct System {
    screen_ext: [f32, ..2],
    spawn_radius: f32,
    rate: f32,
    time_left: w::Delta,
    draw_id: ecs::Id<w::Drawable>,
    pools: [Vec<w::Entity>, ..KINDS],
    rng: StdRng,
}

impl System {
    pub fn new(extents: [f32, ..2], draw_id: ecs::Id<w::Drawable>) -> System {
        let radius = extents[0] + extents[1];
        System {
            screen_ext: extents,
            spawn_radius: radius,
            rate: 1.0,
            time_left: 3.0,
            draw_id: draw_id,
            pools: [Vec::new(), Vec::new()],
            rng: StdRng::new().unwrap(),
        }
    }

    fn spawn(&mut self, data: &mut w::Components) -> w::Entity {
        let origin_angle = Deg{ s: self.rng.gen_range(0f32, 360f32) }.to_rad();
        let origin_pos = Point2::new(
            self.spawn_radius * cos(origin_angle),
            self.spawn_radius * sin(origin_angle),
        );
        let target = Point2::new(
            self.rng.gen_range(-self.screen_ext[0], self.screen_ext[0]),
            self.rng.gen_range(-self.screen_ext[1], self.screen_ext[1]),
        );
        let space = w::Spatial {
            pos: origin_pos,
            orient: Rad{ s: 0.0 },
            scale: 1.0,
        };
        let inertia = w::Inertial {
            velocity: target.sub_p(&origin_pos).mul_s(0.1),
            angular_velocity: Rad{ s: self.rng.gen_range(-2.0, 2.0) },
        };
        let aster = w::Asteroid {
            kind: 0,
        };
        let collide = w::Collision {
            radius: 0.5,
            health: 1,
            damage: 2,
        };
        match self.pools[0].pop() {
            Some(ent) => {
                *data.space.get_mut(ent.space.unwrap()) = space;
                *data.inertia.get_mut(ent.inertia.unwrap()) = inertia;
                *data.aster.get_mut(ent.aster.unwrap()) = aster;
                *data.collision.get_mut(ent.collision.unwrap()) = collide;
                ent
            },
            None => {
                let mut ent = data.add()
                    .space(space)
                    .inertia(inertia)
                    .aster(aster)
                    .collision(collide)
                    .entity;
                ent.draw = Some(self.draw_id);
                ent
            },
        }
    }
}

impl w::System for System {
    fn process(&mut self, &(time, _): w::Params, data: &mut w::Components, entities: &mut Vec<w::Entity>) {
        // cleanup
        let (new_entities, reserve) = entities.partitioned(|e| {
            match (e.aster, e.space, e.collision) {
                (Some(_), Some(s_id), Some(c_id)) => {
                    let is_destroyed = data.collision.get(c_id).health == 0;
                    let s = data.space.get(s_id);
                    let is_in =
                        s.pos.x.abs() <= self.screen_ext[0] &&
                        s.pos.y.abs() <= self.screen_ext[1];
                    let is_heading_in = match e.inertia {
                        Some(i_id) => {
                            let i = data.inertia.get(i_id);
                            s.pos.sub_p(&Point::origin()).dot(&i.velocity) < 0.0
                        },
                        None => false,
                    };
                    !is_destroyed && (is_in || is_heading_in)
                },
                _ => true,
            }
        });
        *entities = new_entities;
        self.pools[0].push_all_move(reserve);
        // spawn
        self.time_left += time;
        while self.time_left >= self.rate {
            self.time_left -= self.rate;
            entities.push(self.spawn(data));
        }
    }
}
