use world as w;

pub struct System;

impl super::System for System {
    fn process(&mut self, plan: &mut super::Planner, time: super::Delta) {
        plan.run1w1r(move |space: &mut w::Spatial, inertia: &w::Inertial| {
            space.pos = space.pos + inertia.velocity * time;
            space.orient = space.orient + inertia.angular_velocity * time;
        });
    }
}
