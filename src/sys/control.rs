use std::sync::mpsc;
use cgmath::{Rad};
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

impl super::System for System {
    fn process(&mut self, plan: &mut super::Planner, time: super::Delta) {
        self.check_input();
        let (thrust, turn) = (self.thrust, self.turn);
        plan.run1w2r(move |inertia: &mut w::Inertial, space: &w::Spatial, control: &w::Control| {
            let rotate = time * control.turn_speed * turn;
            inertia.angular_velocity = Rad{ s: rotate };
            let dir = space.get_direction();
            let velocity = time * control.thrust_speed * thrust;
            inertia.velocity = inertia.velocity + dir * velocity;
        });
    }
}
