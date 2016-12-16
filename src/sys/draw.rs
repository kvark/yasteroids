use std::sync::Arc;

use pegasus;
use specs;
use gfx;

use world;


pub type ColorFormat = gfx::format::Srgba8;

gfx_vertex_struct!( Vertex {
    pos: [f32; 2] = "a_Pos",
    color: [gfx::format::U8Norm; 4] = "a_Color",
});

impl Vertex {
    pub fn new(x: f32, y: f32, col: u32) -> Vertex {
        let c4 = [(col>>24) as u8, (col>>16) as u8, (col>>8) as u8, col as u8];
        Vertex {
            pos: [x, y],
            color: gfx::format::U8Norm::cast4(c4),
        }
    }
}

gfx_constant_struct!(ShaderParam {
    transform: [f32; 4] = "u_Transform",
    screen_scale: [f32; 4] = "u_ScreenScale",
});

gfx_pipeline!(pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    param: gfx::ConstantBuffer<ShaderParam> = "c_Parameters",
    output: gfx::RenderTarget<gfx::format::Srgba8> = "Target0",
});


const SHADER_VERT: &'static [u8] = b"
    #version 150 core
    in vec2 a_Pos;
    in vec4 a_Color;
    uniform c_Parameters {
        vec4 u_Transform;
        vec4 u_ScreenScale;
    };
    out vec4 v_Color;
    void main() {
        v_Color = a_Color;
        vec2 sc = vec2(sin(u_Transform.z), cos(u_Transform.z));
        vec2 p = vec2(a_Pos.x*sc.y - a_Pos.y*sc.x, a_Pos.x*sc.x + a_Pos.y*sc.y);
        p = (p * u_Transform.w + u_Transform.xy) * u_ScreenScale.xy;
        gl_Position = vec4(p, 0.0, 1.0);
    }
";
const SHADER_FRAG: &'static [u8] = b"
    #version 150 core
    in vec4 v_Color;
    out vec4 Target0;
    void main() {
        Target0 = v_Color;
    }
";


#[derive(Clone)]
pub struct Drawable(usize, ShaderParam);

impl specs::Component for Drawable {
    type Storage = specs::VecStorage<Drawable>;
}

pub struct Painter<R: gfx::Resources> {
    out_color: gfx::handle::RenderTargetView<R, ColorFormat>,
    bundles: Arc<Vec<gfx::Bundle<R, pipe::Data<R>>>>,
}

impl<R: gfx::Resources> Painter<R> {
    pub fn new(target: gfx::handle::RenderTargetView<R, ColorFormat>) -> Painter<R> {
        Painter {
            out_color: target,
            bundles: Arc::new(Vec::new()),
        }
    }

    pub fn add_visual<F: gfx::Factory<R>>(&mut self, factory: &mut F, primitive: gfx::Primitive,
                      rast: gfx::state::Rasterizer, vertices: &[Vertex]) -> Drawable {
        use gfx::traits::FactoryExt;
        let program = factory.link_program(SHADER_VERT, SHADER_FRAG).unwrap();
        let pso = factory.create_pipeline_from_program(&program, primitive, rast, pipe::new()).unwrap();
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(vertices, ());
        let data = pipe::Data {
            vbuf: vbuf,
            param: factory.create_constant_buffer(1),
            output: self.out_color.clone(),
        };
        let id = self.bundles.len();
        let mut bundles = Arc::get_mut(&mut self.bundles).unwrap();
        bundles.push(gfx::Bundle::new(slice, pso, data));
        Drawable(id, ShaderParam {
            transform: [0.0; 4],
            screen_scale: [0.0; 4],
        })
    }
}

impl<R: gfx::Resources> pegasus::Painter<R> for Painter<R> {
    type Visual = Drawable;
    fn draw<'a, I, C>(&mut self, iter: I, encoder: &mut gfx::Encoder<R, C>) where
        I: Iterator<Item = &'a Self::Visual>,
        C: gfx::CommandBuffer<R>
    {
        encoder.clear(&self.out_color, [0.0, 0.0, 0.0, 1.0]);
        for &Drawable(vi, ref param) in iter {
            let b = &self.bundles[vi];
            encoder.update_constant_buffer(&b.data.param, param);
            b.encode(encoder);
        }
    }
}

// the pre-draw system updates the Drawables with the fresh info
pub struct System {
    extents: [f32; 2],
}

impl System {
    pub fn new(extents: [f32; 2]) -> System {
        System {
            extents: extents,
        }
    }
}

impl specs::System<pegasus::Delta> for System {
    fn run(&mut self, arg: specs::RunArg, _: pegasus::Delta) {
        use specs::Join;
        let (mut draw, space) = arg.fetch(|w| {
            (w.write::<Drawable>(), w.read::<world::Spatial>())
        });
        let scale = [1.0 / self.extents[0], 1.0 / self.extents[1], 0.0, 0.0];
        for (d, s) in (&mut draw, &space).iter() {
            d.1 = ShaderParam {
                transform: [s.pos.x, s.pos.y, s.orient.s, s.scale],
                screen_scale: scale,
            };
        }
    }
}
