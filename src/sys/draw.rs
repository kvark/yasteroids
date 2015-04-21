use gfx;
use id::Storage;
use world as w;

pub struct System<R: gfx::Resources> {
    extents: [f32; 2],
    pub context: gfx::batch::Context<R>,
}

impl<R: gfx::Resources> System<R> {
    pub fn new(extents: [f32; 2]) -> System<R> {
        System {
            extents: extents,
            context: gfx::batch::Context::new(),
        }
    }
}

impl<R: gfx::Resources, C: gfx::CommandBuffer<R>, O: gfx::Output<R>> w::System<R, C, O> for System<R> {
    fn process(&mut self, _: w::Delta, renderer: &mut gfx::Renderer<R, C>, output: &O,
               data: &mut w::Components<R>, entities: &mut Vec<w::Entity<R>>) {
        let clear_data = gfx::ClearData {
            color: [0.0, 0.0, 0.1, 0.0],
            depth: 1.0,
            stencil: 0,
        };
        renderer.clear(clear_data, gfx::COLOR, output);
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
                renderer.draw(&(&*drawable, &self.context), output).unwrap();
            });
        }
    }
}
