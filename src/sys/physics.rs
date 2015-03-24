use cgmath::{Point, EuclideanVector};
use world as w;

pub struct System;

impl System {
	pub fn new() -> System {
		System
	}
}

impl w::System for System {
	fn process(&mut self, _: w::Params, data: &mut w::Components, entities: &mut Vec<w::Entity>) {
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
						let ac = *data.collision.get(ac_id);
						let bc = *data.collision.get(bc_id);
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
