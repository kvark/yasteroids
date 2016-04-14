use std::sync::{mpsc, Arc};
use specs;
use gfx;
use world as w;


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


pub struct EncoderChannel<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    pub receiver: mpsc::Receiver<gfx::Encoder<R, C>>,
    pub sender: mpsc::Sender<gfx::Encoder<R, C>>,
}

pub struct VisualType(usize);

impl specs::Component for VisualType {
    type Storage = specs::VecStorage<VisualType>;
}


pub struct System<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    extents: [f32; 2],
    channel: EncoderChannel<R, C>,
    out_color: gfx::handle::RenderTargetView<R, ColorFormat>,
    bundles: Arc<Vec<gfx::Bundle<R, pipe::Data<R>>>>,
}

impl<R: gfx::Resources, C: gfx::CommandBuffer<R>> System<R, C> {
    pub fn new(extents: [f32; 2], chan: EncoderChannel<R, C>,
               out: gfx::handle::RenderTargetView<R, ColorFormat>)
               -> System<R, C>
    {
        System {
            extents: extents,
            channel: chan,
            out_color: out,
            bundles: Arc::new(Vec::new()),
        }
    }

    pub fn add_visual<F: gfx::Factory<R>>(&mut self, factory: &mut F, primitive: gfx::Primitive,
                      rast: gfx::state::Rasterizer, vertices: &[Vertex]) -> VisualType {
        use gfx::traits::FactoryExt;
        let program = factory.link_program(SHADER_VERT, SHADER_FRAG).unwrap();
        let pso = factory.create_pipeline_from_program(&program, primitive, rast, pipe::new()).unwrap();
        let (vbuf, slice) = factory.create_vertex_buffer(vertices);
        let data = pipe::Data {
            vbuf: vbuf,
            param: factory.create_constant_buffer(1),
            output: self.out_color.clone(),
        };
        let id = self.bundles.len();
        let mut bundles = Arc::get_mut(&mut self.bundles).unwrap();
        bundles.push(gfx::Bundle::new(slice, pso, data));
        VisualType(id)
    }
}

impl<R, C> super::System for System<R, C> where
R: 'static + gfx::Resources,
C: 'static + gfx::CommandBuffer<R> + Send,
{
    fn process(&mut self, pl: &mut super::Planner, _: super::Delta) {
        let mut encoder = match self.channel.receiver.recv() {
            Ok(r) => r,
            Err(_) => return,
        };
        let sender = self.channel.sender.clone();
        let out = self.out_color.clone();
        let scale = [1.0 / self.extents[0], 1.0 / self.extents[1], 0.0, 0.0];
        let bundles = self.bundles.clone();
        pl.run(move |arg| {
            let (draw, space, entities) = arg.fetch(|fa| {
                (fa.read::<VisualType>(), fa.read::<w::Spatial>(), fa.entities())
            });
            encoder.clear(&out, [0.2, 0.3, 0.4, 1.0]);
            // render entities
            for e in entities {
                use specs::Storage;
                if let Some(d) = draw.get(e) {
                    let trans = match space.get(e) {
                        Some(s) => [s.pos.x, s.pos.y, s.orient.s, s.scale],
                        None => [0.0, 0.0, 0.0, 0.0],
                    };
                    let param = ShaderParam {
                        transform: trans,
                        screen_scale: scale,
                    };
                    let b = &bundles[d.0];
                    encoder.update_constant_buffer(&b.data.param, &param);
                    b.encode(&mut encoder);
                }
            }
            // done
            match sender.send(encoder) {
                Ok(_) => (),
                Err(_) => return,
            }
        });
    }
}
