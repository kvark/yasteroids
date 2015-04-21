use cgmath::{Angle, Point, Vector};
use gfx;
use id::Storage;
use world as w;

pub struct System;

impl<R: gfx::Resources, C: gfx::CommandBuffer<R>, O> w::System<R, C, O> for System {
    fn process(&mut self, time: w::Delta, _: &mut gfx::Renderer<R, C>, _: &O,
               data: &mut w::Components<R>, entities: &mut Vec<w::Entity<R>>) {
        for ent in entities.iter() {
            ent.space.map(|s_id| {
                let s = data.space.get_mut(s_id);
                match ent.inertia {
                    Some(i_id) => {
                        let i = data.inertia.get(i_id);
                        let moved = i.velocity.mul_s(time);
                        s.pos.add_self_v(&moved);
                        s.orient.add_self_a(i.angular_velocity.mul_s(time));
                    },
                    None => (),
                }
            });
        }
    }
}
