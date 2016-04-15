pub mod aster;
pub mod bullet;
pub mod control;
pub mod draw;
pub mod inertia;
//pub mod physics;

pub use specs::Planner;

pub type Delta = f32;

pub trait System: Send {
    fn process(&mut self, &mut Planner, Delta);
}
