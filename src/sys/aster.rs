use rand::{Rng, StdRng};
use cgmath::{Angle, Deg, Rad, Point2};
use specs;
use world as w;

//const KINDS: usize = 2;

pub struct System {
    screen_ext: [f32; 2],
    spawn_radius: f32,
    rate: f32,
    time_left: super::Delta,
    visual: w::VisualType,
    rng: StdRng,
}

impl System {
    pub fn new(extents: [f32; 2], visual: w::VisualType) -> System {
        let radius = extents[0] + extents[1];
        System {
            screen_ext: extents,
            spawn_radius: radius,
            rate: 1.0,
            time_left: 3.0,
            visual: visual,
            rng: StdRng::new().unwrap(),
        }
    }

    fn spawn(&mut self, w: &specs::World) -> specs::Entity {
        let origin_angle: Rad<_> = Deg{ s: self.rng.gen_range(0f32, 360f32) }.into();
        let origin_pos = Point2::new(
            self.spawn_radius * f32::cos(origin_angle.s),
            self.spawn_radius * f32::sin(origin_angle.s),
        );
        let target = Point2::new(
            self.rng.gen_range(-self.screen_ext[0], self.screen_ext[0]),
            self.rng.gen_range(-self.screen_ext[1], self.screen_ext[1]),
        );
        w.create_now()
            .with(w::Spatial {
                pos: origin_pos,
                orient: Rad{ s: 0.0 },
                scale: 1.0,
            })
            .with(self.visual.clone())
            .with(w::Inertial {
                velocity: (target - origin_pos) * 0.1,
                angular_velocity: Rad{ s: self.rng.gen_range(-2.0, 2.0) },
            })
            .with(w::Asteroid {
                kind: 0,
            })
            .with(w::Collision {
                radius: 0.5,
                health: 1,
                damage: 2,
            })
            .build()
    }
}

impl specs::System<super::Delta> for System {
    fn run(&mut self, arg: specs::RunArg, time: super::Delta) {
        use specs::Join;
        self.time_left += time;
        let (aster, space, inertia, entities) = arg.fetch(|w| {
            while self.time_left >= self.rate {
                self.time_left -= self.rate;
                self.spawn(w);
            }
            (w.read::<w::Asteroid>(), w.read::<w::Spatial>(), w.read::<w::Inertial>(), w.entities())
        });
        for (_, s, i, e) in (&aster, &space, &inertia, &entities).iter() {
            if  (s.pos.x.abs() > self.screen_ext[0] && s.pos.x * i.velocity.x >= 0.0) ||
                (s.pos.y.abs() > self.screen_ext[1] && s.pos.y * i.velocity.y >= 0.0) {
                arg.delete(e);
            }
        }
    }
}
