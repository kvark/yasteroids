use specs;
use world as w;

pub struct System;

impl specs::System<super::Delta> for System {
	fn run(&mut self, arg: specs::RunArg, time: super::Delta) {
		use specs::Join;
		let (mut space, inertia) = arg.fetch(|w|
			(w.write::<w::Spatial>(), w.read::<w::Inertial>())
		);
		for (s, i) in (&mut space, &inertia).iter() {
			s.pos = s.pos + i.velocity * time;
            s.orient = s.orient + i.angular_velocity * time;
		}
	}
}
