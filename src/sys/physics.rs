use cgmath::{Point, EuclideanVector};
use gfx;
use id::Storage;
use world as w;

pub struct System;

impl System {
	pub fn new() -> System {
		System
	}
}

impl<R: gfx::Resources, C: gfx::CommandBuffer<R>> w::System<R, C> for System {
	fn process(&mut self, _: w::Delta, _: &mut gfx::Renderer<R, C>,
			   data: &mut w::Components<R>, entities: &mut Vec<w::Entity<R>>) {
		let mut ia = entities.iter();
		loop {
			let a = match ia.next() {
				Some(e) => e,
				None => break,
			};
			for b in ia.clone() {
				match (a.collision, a.space, b.collision, b.space) {
					(Some(ac_id), Some(as_id), Some(bc_id), Some(bs_id)) => {
						let dist = data.space.get(as_id).pos
							.sub_p(&data.space.get(bs_id).pos)
							.length2();
						let ac = data.collision.get(ac_id).clone();
						let bc = data.collision.get(bc_id).clone();
						let diameter = ac.radius + bc.radius;
						if dist < diameter && ac.health > 0 && bc.health > 0 {
							//println!("Hit radius {} with radius {}",
							//	ac.radius, bc.radius);
							data.collision.get_mut(ac_id).hit(bc.damage);
							data.collision.get_mut(bc_id).hit(ac.damage);
						}
					},
					_ => (),
				}
			}
		}
	}
}
