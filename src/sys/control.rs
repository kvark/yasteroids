use std::sync::mpsc;
use cgmath::{Angle, Rad, Point, Vector};
use gfx;
use world as w;

pub enum Event {
    EvThrust(f32),
    EvTurn(f32),
}

pub struct System {
    input: mpsc::Receiver<Event>,
    thrust: f32,
    turn: f32,
}

impl System {
    pub fn new(chan: mpsc::Receiver<Event>) -> System {
        System {
            input: chan,
            thrust: 0.0,
            turn: 0.0,
        }
    }

    fn check_input(&mut self) {
        loop {
            match self.input.try_recv() {
                Ok(Event::EvThrust(v)) => self.thrust = v,
                Ok(Event::EvTurn(v)) => self.turn = v,
                Err(_) => return,
            }
        }
    }
}

impl<R: gfx::Resources, C: gfx::CommandBuffer<R>> w::System<R, C> for System {
    fn process(&mut self, time: w::Delta, _: &mut gfx::Renderer<R, C>,
               data: &mut w::Components<R>, entities: &mut Vec<w::Entity<R>>) {
        self.check_input();
        for ent in entities.iter() {
            match (ent.control, ent.inertia) {
                (Some(c_id), Some(i_id)) => {
                    let c = data.control.get(c_id);
                    let i = data.inertia.get_mut(i_id);
                    let rotate = time * c.turn_speed * self.turn;
                    i.angular_velocity = Rad{ s: rotate };
                    match ent.space {
                        Some(s_id) => {
                            let s = data.space.get_mut(s_id);
                            let dir = s.get_direction();
                            let thrust = time * c.thrust_speed * self.thrust;
                            i.velocity.add_self_v(&dir.mul_s(thrust));
                        },
                        None => (),
                    }
                },
                (_, _) => (),
            }
        }
    }
}
