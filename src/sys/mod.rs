//pub mod aster;
pub mod bullet;
pub mod control;
//pub mod draw;
//pub mod inertia;
//pub mod physics;

use parsec::Scheduler;

pub type Delta = f32;

pub trait System: Send {
    fn process(&mut self, &Scheduler, Delta);
}
