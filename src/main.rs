extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate pegasus;
extern crate specs;
extern crate rand;

mod event;
mod game;
mod world;
mod sys;

type DepthFormat = gfx::format::Depth;

static USAGE: &'static str = "
Controls:
    A - thrust
    S - shoot
    Left/Right - turn
";

pub fn main() {
    println!("{}", USAGE);

    let title = "Asteroids demo for gfx-rs, specs, and pegasus";
    let (ev_send, ev_recv) = event::SenderHub::new();

    let builder = glutin::WindowBuilder::new()
        .with_title(title.to_string())
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)));
    let (window, device, mut factory, main_color, _main_depth) =
        gfx_window_glutin::init::<sys::draw::ColorFormat, DepthFormat>(builder);

    let mut painter = sys::draw::Painter::new(main_color);
    let init = game::Init::new(&mut factory, &mut painter, ev_recv);

    let mut pegasus = pegasus::Pegasus::new(init, device, painter, ||
        factory.create_command_buffer());
    
    'main: while let Some(_swing) = pegasus.swing() {
        window.swap_buffers().unwrap();
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => ev_send.process_glutin(event),
            }
        }
    }
}
