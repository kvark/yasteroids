use cgmath::{Rad, Point2, Vector2};
use specs;
use gfx;
use pegasus;

use event::ReceiverHub;
use sys;
use sys::draw::{Painter, Vertex};
use world;


const SCREEN_EXTENTS: [f32; 2] = [10.0, 10.0];

pub struct Init {
    hub: ReceiverHub,
    vis_ship: world::Drawable,
    vis_bullet: world::Drawable,
    vis_aster: world::Drawable,
}

impl Init {
    pub fn new<R, F>(factory: &mut F, painter: &mut Painter<R>, hub: ReceiverHub)
               -> Init where
    R: 'static + gfx::Resources,
    F: gfx::Factory<R>,
    {
        Init {
            hub: hub,
            vis_ship: {
                let rast = gfx::state::Rasterizer::new_fill();
                painter.add_visual(factory,
                    gfx::Primitive::TriangleList, rast, &[
                    Vertex::new(-0.3, -0.5, 0x20C02000),
                    Vertex::new(0.3, -0.5,  0x20C02000),
                    Vertex::new(0.0, 0.5,   0xC0404000),
                ])
            },
            vis_bullet: {
                let mut rast = gfx::state::Rasterizer::new_fill();
                rast.method = gfx::state::RasterMethod::Point;
                painter.add_visual(factory,
                    gfx::Primitive::PointList, rast, &[
                    Vertex::new(0.0, 0.0, 0xFF808000),
                ])
            },
            vis_aster: {
                let rast = gfx::state::Rasterizer::new_fill();
                painter.add_visual(factory,
                    gfx::Primitive::TriangleStrip, rast, &[
                    Vertex::new(-0.5, -0.5, 0xFFFFFF00),
                    Vertex::new(0.5, -0.5,  0xFFFFFF00),
                    Vertex::new(-0.5, 0.5,  0xFFFFFF00),
                    Vertex::new(0.5, 0.5,   0xFFFFFF00),
                ])
            },
        }
    }
}


fn create_ship(drawable: world::Drawable, world: &mut specs::World) -> specs::Entity {
    world.create_now()
         .with(drawable)
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

pub struct Game {
    player: specs::Entity,
}

impl pegasus::Init for Init {
    type Shell = Game;

    fn start(self, plan: &mut pegasus::Planner) -> Game {
        let player = {
            let w = plan.mut_world();
            w.register::<world::Spatial>();
            w.register::<world::Inertial>();
            w.register::<world::Control>();
            w.register::<world::Bullet>();
            w.register::<world::Asteroid>();
            w.register::<world::Collision>();
            create_ship(self.vis_ship, w)
        };

        plan.add_system(sys::control::System::new(self.hub.control), "control", 30);
        plan.add_system(sys::draw::System::new(SCREEN_EXTENTS), "pre-draw", pegasus::DRAW_PRIORITY + 5);
        plan.add_system(sys::inertia::System, "inertia", 15);
        plan.add_system(sys::bullet::System::new(self.hub.bullet, player, self.vis_bullet), "bullet", 25);
        plan.add_system(sys::aster::System::new(SCREEN_EXTENTS, self.vis_aster), "aster", 24);
        plan.add_system(sys::physics::System::new(), "physics", 5);

        Game {
            player: player,
        }
    }

    fn proceed(game: &mut Game, world: &specs::World) -> bool {
        world.is_alive(game.player)
    }
}
