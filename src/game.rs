use time;
use cgmath::{Rad, Point2, Vector2};
use specs;
use gfx;
use sys;
use sys::draw::{Vertex, ColorFormat};
use world;


const SCREEN_EXTENTS: [f32; 2] = [10.0, 10.0];

pub struct Game {
    planner: specs::Planner<sys::Delta>,
    last_time: u64,
    player: specs::Entity,
}

fn create_ship(visual: world::VisualType, world: &specs::World) -> specs::Entity {
    world.create_now()
         .with(visual)
         .with(world::Spatial {
             pos: Point2::new(0.0, 0.0),
             orient: Rad{ s: 0.0 },
             scale: 1.0,
         })
         .with(world::Inertial {
             velocity: Vector2::new(0.0, 0.0),
             angular_velocity: Rad{ s:0.0 },
         })
         .with(world::Control {
             thrust_speed: 4.0,
             turn_speed: -90.0,
         })
         .with(world::Collision {
            radius: 0.2,
            health: 3,
            damage: 2,
         })
         .build()
}

impl Game {
    pub fn new<R, F, C>(factory: &mut F,
               (ev_control, ev_bullet): ::event::ReceiverHub,
               encoder_chan: sys::draw::EncoderChannel<R, C>,
               main_color: gfx::handle::RenderTargetView<R, ColorFormat>)
               -> Game where
    R: 'static + gfx::Resources,
    F: gfx::Factory<R>,
    C: 'static + gfx::CommandBuffer<R> + Send,
    {
        let mut w = specs::World::new();
        w.register::<world::Spatial>();
        w.register::<world::Inertial>();
        w.register::<world::Control>();
        w.register::<world::VisualType>();
        w.register::<world::Bullet>();
        w.register::<world::Asteroid>();
        w.register::<world::Collision>();
        // prepare systems
        let mut draw_system = sys::draw::System::new(SCREEN_EXTENTS, encoder_chan, main_color);
        // prepare entities
        let ship = {
            let rast = gfx::state::Rasterizer::new_fill();
            let visual = draw_system.add_visual(factory,
                gfx::Primitive::TriangleList, rast, &[
                Vertex::new(-0.3, -0.5, 0x20C02000),
                Vertex::new(0.3, -0.5,  0x20C02000),
                Vertex::new(0.0, 0.5,   0xC0404000),
            ]);
            create_ship(visual, &w)
        };
        let bullet_visual = {
            let mut rast = gfx::state::Rasterizer::new_fill();
            rast.method = gfx::state::RasterMethod::Point;
            draw_system.add_visual(factory,
                gfx::Primitive::PointList, rast, &[
                Vertex::new(0.0, 0.0, 0xFF808000),
            ])
        };
        let aster_visual = {
            let rast = gfx::state::Rasterizer::new_fill();
            draw_system.add_visual(factory,
                gfx::Primitive::TriangleStrip, rast, &[
                Vertex::new(-0.5, -0.5, 0xFFFFFF00),
                Vertex::new(0.5, -0.5,  0xFFFFFF00),
                Vertex::new(-0.5, 0.5,  0xFFFFFF00),
                Vertex::new(0.5, 0.5,   0xFFFFFF00),
            ])
        };
        let mut plan = specs::Planner::new(w, 2);
        plan.add_system(draw_system, "draw", 10);
        plan.add_system(sys::control::System::new(ev_control), "control", 30);
        plan.add_system(sys::inertia::System, "inertia", 15);
        plan.add_system(sys::bullet::System::new(ev_bullet, ship, bullet_visual), "bullet", 25);
        plan.add_system(sys::aster::System::new(SCREEN_EXTENTS, aster_visual), "aster", 24);
        plan.add_system(sys::physics::System::new(), "physics", 5);
        Game {
            planner: plan,
            last_time: time::precise_time_ns(),
            player: ship,
        }
    }

    pub fn frame(&mut self) -> bool {
        let new_time = time::precise_time_ns();
        let delta = (new_time - self.last_time) as f32 / 1e9;
        self.last_time = new_time;
        self.planner.dispatch(delta);
        self.planner.world.is_alive(self.player)
    }
}
