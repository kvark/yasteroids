use time;
use id::Storage;
use cgmath::{Rad, Point2, Vector2};
use gfx;
use gfx::traits::*;
use world;

const SCREEN_EXTENTS: [f32; 2] = [10.0, 10.0];

#[derive(Copy)]
#[vertex_format]
struct Vertex {
    pos: [f32; 2],
    #[normalized]
    color: [u8; 4],
}

impl Vertex {
    fn new(x: f32, y: f32, col: u32) -> Vertex {
        Vertex {
            pos: [x, y],
            color: [(col>>24) as u8, (col>>16) as u8, (col>>8) as u8, col as u8],
        }
    }
}

pub struct Game<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    world: world::World<R>,
    systems: Vec<Box<world::System<R, C>>>,
    last_time: u64,
}

fn create_program<R: gfx::Resources, F: gfx::Factory<R>>(
                  factory: &mut F) -> gfx::ProgramHandle<R>
{
    factory.link_program(
        b"
            #version 150 core
            in vec2 pos;
            in vec4 color;
            uniform vec4 transform, screen_scale;
            out vec4 v_color;
            void main() {
                v_color = color;
                vec2 sc = vec2(sin(transform.z), cos(transform.z));
                vec2 p = vec2(pos.x*sc.y - pos.y*sc.x, pos.x*sc.x + pos.y*sc.y);
                p = (p * transform.w + transform.xy) * screen_scale.xy;
                gl_Position = vec4(p, 0.0, 1.0);
            }
        ",
        b"
            #version 150 core
            in vec4 v_color;
            out vec4 color;
            void main() {
                color = v_color;
            }
        "
    ).unwrap()
}

fn create_ship<R: gfx::Resources, F: gfx::Factory<R>>(factory: &mut F,
               data: &mut world::Components<R>, draw: &mut ::sys::draw::System<R>,
               program: gfx::ProgramHandle<R>) -> world::Entity<R>

{
    let mesh = factory.create_mesh(&[
        Vertex::new(-0.3, -0.5, 0x20C02000),
        Vertex::new(0.3, -0.5,  0x20C02000),
        Vertex::new(0.0, 0.5,   0xC0404000),
    ]);
    let slice = mesh.to_slice(gfx::PrimitiveType::TriangleList);
    let mut state = gfx::DrawState::new();
    state.primitive.method = gfx::state::RasterMethod::Fill(gfx::state::CullFace::Nothing);
    let batch = draw.context.make_batch(&program, world::ShaderParam::new(), &mesh, slice, &state).unwrap();
    world::Entity {
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
    }
}

impl<R: gfx::Resources + Send + 'static, C: gfx::CommandBuffer<R>> Game<R, C> {
    pub fn new<F: gfx::Factory<R>>(factory: &mut F,
               (ev_control, ev_bullet): ::event::ReceiverHub,
               frame: gfx::Frame<R>) -> Game<R, C> where
        R::Buffer: 'static,
        R::ArrayBuffer: 'static,
        R::Shader: 'static,
        R::Program: 'static,
        R::FrameBuffer: 'static,
        R::Surface: 'static,
        R::Texture: 'static,
        R::Sampler: 'static,
    {
        let mut w = world::World::new();
        // prepare systems
        let program = create_program(factory);
        let mut draw_system = ::sys::draw::System::new(SCREEN_EXTENTS, frame);
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
            Box::new(draw_system) as Box<world::System<R, C>>,
            Box::new(::sys::inertia::System),
            Box::new(::sys::control::System::new(ev_control)),
            Box::new(::sys::bullet::System::new(ev_bullet,
                space_id, inertia_id, bullet_draw_id)),
            Box::new(::sys::aster::System::new(SCREEN_EXTENTS, aster_draw_id)),
            Box::new(::sys::physics::System::new()),
        ];
        Game {
            world: w,
            systems: systems,
            last_time: time::precise_time_ns(),
        }
    }

    pub fn render(&mut self, renderer: &mut gfx::Renderer<R, C>) {
        let new_time = time::precise_time_ns();
        let delta = (new_time - self.last_time) as f32 / 1e9;
        self.last_time = new_time;
        for sys in self.systems.iter_mut() {
            sys.process(delta, renderer, &mut self.world.data, &mut self.world.entities);
        }
    }

    pub fn is_alive(&self) -> bool {
        self.world.entities.iter().find(|e| {
            match (e.control, e.collision) {
                (Some(_), Some(o_id)) =>
                    self.world.data.collision.get(o_id).health != 0,
                _ => false,
            }
        }).is_some()
    }
}
