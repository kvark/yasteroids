use std::sync::mpsc;
use cgmath::{Rad};
use specs;
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

impl specs::System<super::Delta> for System {
    fn run(&mut self, arg: specs::RunArg, time: super::Delta) {
        use specs::Join;
        self.check_input();
        let (mut inertia, space, control) = arg.fetch(|w|
            (w.write::<w::Inertial>(), w.read::<w::Spatial>(), w.read::<w::Control>())
        );
        for (i, s, c) in (&mut inertia, &space, &control).iter() {
            let rotate = c.turn_speed * self.turn;
            i.angular_velocity = Rad{ s: rotate };
            let dir = s.get_direction();
            let velocity = time * c.thrust_speed * self.thrust;
            i.velocity = i.velocity + dir * velocity;
        }
    }
}
