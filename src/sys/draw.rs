use gfx;
use gfx_device_gl::GlResources;
use world as w;

pub struct System {
    extents: [f32; 2],
    pub frame: gfx::Frame<GlResources>,
    pub context: gfx::batch::Context<GlResources>,
}

impl System {
    pub fn new(extents: [f32; 2], frame: gfx::Frame<GlResources>) -> System {
        System {
            extents: extents,
            frame: frame,
            context: gfx::batch::Context::new(),
        }
    }
}

impl w::System for System {
    fn process(&mut self, _: w::Delta, renderer: &mut ::Renderer,
               data: &mut w::Components, entities: &mut Vec<w::Entity>) {
        let clear_data = gfx::ClearData {
            color: [0.0, 0.0, 0.1, 0.0],
            depth: 1.0,
            stencil: 0,
        };
        renderer.clear(clear_data, gfx::COLOR, &self.frame);
        for ent in entities.iter() {
            ent.draw.map(|d_id| {
                let drawable = data.draw.get_mut(d_id);
                drawable.params.screen_scale = [1.0 / self.extents[0], 1.0 / self.extents[1], 0.0, 0.0];
                match ent.space {
                    Some(s_id) => {
                        let s = data.space.get(s_id);
                        drawable.params.transform = [s.pos.x, s.pos.y, s.orient.s, s.scale];
                    }
                    None => ()
                }
                renderer.draw(&(&*drawable, &self.context), &self.frame).unwrap();
            });
        }
    }
}
