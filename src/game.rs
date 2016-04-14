use std::sync::mpsc;
use time;
use cgmath::{Rad, Point2, Vector2};
use specs;
use gfx;
use gfx::traits::FactoryExt;
use sys;
use sys::draw::{Vertex, ColorFormat, VisualType};
use world;


const SCREEN_EXTENTS: [f32; 2] = [10.0, 10.0];

pub struct Game {
    planner: specs::Planner,
    systems: Vec<Box<sys::System>>,
    last_time: u64,
}

fn create_ship(visual: VisualType, world: &specs::World) -> specs::Entity {
    world.create_now()
         .with(visual)
         .build()
    /*world::Entity {
        draw: Some(data.draw.add(batch)),
        space: Some(data.space.add(world::Spatial {
            pos: Point2::new(0.0, 0.0),
            orient: Rad{ s: 0.0 },
            scale: 1.0,
        })),
        inertia: Some(data.inertia.add(world::Inertial {
            velocity: Vector2::new(0.0, 0.0),
            angular_velocity: Rad{ s:0.0 },
        })),
        control: Some(data.control.add(world::Control {
            thrust_speed: 4.0,
            turn_speed: -90.0,
        })),
        bullet: None,
        aster: None,
        collision: Some(data.collision.add(world::Collision {
            radius: 0.2,
            health: 3,
            damage: 2,
        })),
    }*/
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
        w.register::<VisualType>();
        // prepare systems
        let mut draw_system = sys::draw::System::new(SCREEN_EXTENTS, encoder_chan, main_color);
        // prepare entities
        let _ship = {
            let rast = gfx::state::Rasterizer::new_fill(gfx::state::CullFace::Nothing);
            let visual = draw_system.add_visual(factory,
                gfx::Primitive::TriangleList, rast, &[
                Vertex::new(-0.3, -0.5, 0x20C02000),
                Vertex::new(0.3, -0.5,  0x20C02000),
                Vertex::new(0.0, 0.5,   0xC0404000),
            ]);
            create_ship(visual, &w)
        };
        /*let program = create_program(factory);
        let bullet_draw_id = {
            let mesh = factory.create_mesh(&[
                Vertex::new(0.0, 0.0, 0xFF808000),
            ]);
            let slice = mesh.to_slice(gfx::PrimitiveType::Point);
            let mut state = gfx::DrawState::new();
            state.primitive.method = gfx::state::RasterMethod::Point;
            let batch = draw_system.context.make_batch(&program, world::ShaderParam::new(), &mesh, slice, &state).unwrap();
            w.data.draw.add(batch)
        };
        let aster_draw_id = {
            let mesh = factory.create_mesh(&[
                Vertex::new(-0.5, -0.5, 0xFFFFFF00),
                Vertex::new(0.5, -0.5,  0xFFFFFF00),
                Vertex::new(-0.5, 0.5,  0xFFFFFF00),
                Vertex::new(0.5, 0.5,   0xFFFFFF00),
            ]);
            let slice = mesh.to_slice(gfx::PrimitiveType::TriangleStrip);
            let mut state = gfx::DrawState::new();
            state.primitive.method = gfx::state::RasterMethod::Fill(gfx::state::CullFace::Nothing);
            let batch = draw_system.context.make_batch(&program, world::ShaderParam::new(), &mesh, slice, &state).unwrap();
            w.data.draw.add(batch)
        };
        let ship = create_ship(factory, &mut w.data, &mut draw_system, program);
        let (space_id, inertia_id) = (ship.space.unwrap(), ship.inertia.unwrap());
        // populate world and return
        w.entities.push(ship);
        let systems = vec![
            Box::new(draw_system) as Box<worldsystem<R, C, O>>,
            Box::new(sys::inertia::System),
            Box::new(sys::control::System::new(ev_control)),
            Box::new(sys::bullet::System::new(ev_bullet,
                space_id, inertia_id, bullet_draw_id)),
            Box::new(sys::aster::System::new(SCREEN_EXTENTS, aster_draw_id)),
            Box::new(sys::physics::System::new()),
        ];*/
        let systems = vec![
            Box::new(draw_system) as Box<sys::System>,
        ];
        Game {
            planner: specs::Planner::new(w, 4),
            systems: systems,
            last_time: time::precise_time_ns(),
        }
    }

    pub fn frame(&mut self) -> bool {
        let new_time = time::precise_time_ns();
        let delta = (new_time - self.last_time) as f32 / 1e9;
        self.last_time = new_time;
        for sys in self.systems.iter_mut() {
            sys.process(&mut self.planner, delta);
        }
        /*
        self.world.entities.iter().find(|e| {
            match (e.control, e.collision) {
                (Some(_), Some(o_id)) =>
                    self.world.data.collision.get(o_id).health != 0,
                _ => false,
            }
        }).is_some()*/
        true
    }
}
