use std::cmp;
use std::marker::PhantomData;
use cgmath::{Rad, Basis2, Rotation, Rotation2, Point2, Vector2};
use gfx;
use gfx_device_gl;
use id;

#[shader_param]
pub struct ShaderParam<R: gfx::Resources> {
    //TODO: hide these
    pub transform: [f32; 4],
    pub screen_scale: [f32; 4],
    _dummy: PhantomData<R>,
}

impl<R: gfx::Resources> ShaderParam<R> {
    pub fn new() -> ShaderParam<R> {
        ShaderParam {
            transform: [0.0; 4],
            screen_scale: [1.0; 4],
            _dummy: PhantomData,
        }
    }
}

/// --- Components ---

pub type Drawable = gfx::batch::RefBatch<ShaderParam<gfx_device_gl::GlResources>>;

#[derive(Clone)]
pub struct Spatial {
    pub pos: Point2<f32>,
    pub orient: Rad<f32>,
    pub scale: f32,
}

impl Spatial {
    pub fn get_direction(&self) -> Vector2<f32> {
        let rot: Basis2<f32> = Rotation2::from_angle(self.orient);
        rot.rotate_vector(&Vector2::unit_y())
    }
}

#[derive(Clone)]
pub struct Inertial {
    pub velocity: Vector2<f32>,
    pub angular_velocity: Rad<f32>,
}

#[derive(Clone)]
pub struct Control {
    pub thrust_speed: f32,
    pub turn_speed: f32,
}

#[derive(Clone)]
pub struct Bullet {
    pub life_time: Option<f32>,
}

#[derive(Clone)]
pub struct Asteroid {
    pub kind: u8,
}

#[derive(Clone)]
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

#[secs(id)]
pub struct Prototype {
    draw: Drawable,
    space: Spatial,
    inertia: Inertial,
    control: Control,
    bullet: Bullet,
    aster: Asteroid,
    collision: Collision,
}

pub type Delta = f32;

pub trait System: Send {
    fn process(&mut self, Delta, &mut ::Renderer, &mut Components, &mut Vec<Entity>);
}
