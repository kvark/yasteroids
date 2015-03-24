use gfx;
use world as w;

pub struct System {
    extents: [f32, ..2],
    pub frame: gfx::Frame,
    pub context: gfx::batch::Context,
}

impl System {
    pub fn new(extents: [f32, ..2], frame: gfx::Frame) -> System {
        System {
            extents: extents,
            frame: frame,
            context: gfx::batch::Context::new(),
        }
    }
}

impl w::System for System {
    fn process(&mut self, &(_, ref mut renderer): w::Params, data: &mut w::Components,
               entities: &mut Vec<w::Entity>) {
        let clear_data = gfx::ClearData {
            color: [0.0, 0.0, 0.1, 0.0],
            depth: 1.0,
            stencil: 0,
        };
        renderer.clear(clear_data, gfx::Color, &self.frame);
        let mut param = w::ShaderParam {
            transform: [0.0, 0.0, 0.0, 1.0],
            screen_scale: [1.0 / self.extents[0], 1.0 / self.extents[1], 0.0, 0.0],
        };
        for ent in entities.iter() {
            ent.draw.map(|d_id| {
                let drawable = data.draw.get(d_id);
                match ent.space {
                    Some(s_id) => {
                        let s = data.space.get(s_id);
                        param.transform = [s.pos.x, s.pos.y, s.orient.s, s.scale];
                    }
                    None => ()
                }
                renderer.draw(&(drawable, &param, &self.context), &self.frame);
            });
        }
    }
}
