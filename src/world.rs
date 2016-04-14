use std::cmp;
use std::marker::PhantomData;
use cgmath::{Rad, Basis2, Rotation, Rotation2, Point2, Vector2};
use specs;
//use sys::draw::VisualType;


/// --- Components ---

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

impl specs::Component for Spatial {
    type Storage = specs::VecStorage<Spatial>;
}

pub struct Inertial {
    pub velocity: Vector2<f32>,
    pub angular_velocity: Rad<f32>,
}

impl specs::Component for Inertial {
    type Storage = specs::VecStorage<Inertial>;
}

pub struct Control {
    pub thrust_speed: f32,
    pub turn_speed: f32,
}

impl specs::Component for Control {
    type Storage = specs::VecStorage<Control>;
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
