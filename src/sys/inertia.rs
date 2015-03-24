use cgmath::{Angle, Point, Vector};
use world as w;

pub struct System;

impl w::System for System {
    fn process(&mut self, &(time, _): w::Params, data: &mut w::Components, entities: &mut Vec<w::Entity>) {
        for ent in entities.iter() {
            ent.space.map(|s_id| {
                let s = data.space.get_mut(s_id);
                match ent.inertia {
                    Some(i_id) => {
                        let i = data.inertia.get(i_id);
                        let move = i.velocity.mul_s(time);
                        s.pos.add_self_v(&move);
                        s.orient.add_self_a(i.angular_velocity.mul_s(time));
                    },
                    None => (),
                }
            });
        }
    }
}
