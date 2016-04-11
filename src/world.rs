use std::cmp;
use std::marker::PhantomData;
use cgmath::{Rad, Basis2, Rotation, Rotation2, Point2, Vector2};
use gfx;


gfx_constant_struct!(ShaderParam {
    transform: [f32; 4] = "u_Transform",
    screen_scale: [f32; 4] = "u_ScreenScale",
});

impl ShaderParam {
    pub fn new() -> ShaderParam {
        ShaderParam {
            transform: [0.0; 4],
            screen_scale: [1.0; 4],
        }
    }
}

/// --- Components ---

pub struct Drawable<R: gfx::Resources> {
    cbuf: gfx::handle::Buffer<R, ShaderParam>,
}

pub struct Spatial {
    pub pos: Point2<f32>,
    pub orient: Rad<f32>,
    pub scale: f32,
}

impl Spatial {
    pub fn get_direction(&self) -> Vector2<f32> {
        let rot: Basis2<f32> = Rotation2::from_angle(self.orient);
        rot.rotate_vector(Vector2::unit_y())
    }
}

pub struct Inertial {
    pub velocity: Vector2<f32>,
    pub angular_velocity: Rad<f32>,
}

pub struct Control {
    pub thrust_speed: f32,
    pub turn_speed: f32,
}

pub struct Bullet {
    pub life_time: Option<f32>,
}

pub struct Asteroid {
    pub kind: u8,
}

pub struct Collision {
    pub radius: f32,
    pub health: u16,
    pub damage: u16,
}

impl Collision {
    pub fn hit(&mut self, d: u16) {
        self.health = cmp::max(self.health, d) - d;
    }
}
