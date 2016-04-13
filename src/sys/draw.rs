use std::sync::mpsc;
use gfx;
use world as w;
use ColorFormat;

gfx_pipeline!(pipe {
    output: gfx::RenderTarget<gfx::format::Srgba8> = "Target0",
});

pub struct EncoderChannel<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    pub receiver: mpsc::Receiver<gfx::Encoder<R, C>>,
    pub sender: mpsc::Sender<gfx::Encoder<R, C>>,
}

/*fn game_loop<
    R: gfx::Resources + Send + 'static,
    C: gfx::CommandBuffer<R> + Send,
>(  mut game: game::Game,
    ren_recv: mpsc::Receiver<gfx::Encoder<R, C>>,
    ren_end: mpsc::Sender<gfx::Encoder<R, C>>)
{
    while game.is_alive() {
        let mut encoder = match ren_recv.recv() {
            Ok(r) => r,
            Err(_) => break,
        };
        game.render(&mut encoder);
        match ren_end.send(encoder) {
            Ok(_) => (),
            Err(_) => break,
        }
    }
}*/


pub struct System<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    extents: [f32; 2],
    channel: EncoderChannel<R, C>,
    out_color: gfx::handle::RenderTargetView<R, ColorFormat>,
    pso: Vec<gfx::PipelineState<R, pipe::Meta>>,
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
            pso: Vec::new(),
        }
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
        pl.run(move |arg| {
            arg.fetch(|_| {});
            encoder.clear(&out, [0.2, 0.3, 0.4, 1.0]);
            //game.render(&mut encoder);
            match sender.send(encoder) {
                Ok(_) => (),
                Err(_) => return,
            }
        });

        /*let clear_data = gfx::ClearData {
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
        }*/
    }
}
