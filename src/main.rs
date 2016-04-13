extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate specs;
extern crate env_logger;
extern crate rand;
extern crate time;

use std::sync::{Arc, Mutex, mpsc};

mod event;
mod game;
mod world;
mod sys;

type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::Depth;

fn game_loop<
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
}

static USAGE: &'static str = "
Controls:
    A - thrust
    S - shoot
    Left/Right - turn
";

pub fn main() {
    env_logger::init().unwrap();
    println!("{}", USAGE);

    let title = "Asteroids example for #scene-rs";
    let (ev_send, ev_recv) = event::SenderHub::new();
    let (game_send, dev_recv) = mpsc::channel();
    let (dev_send, game_recv) = mpsc::channel();

    let builder = glutin::WindowBuilder::new()
        .with_title(title.to_string())
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)));
    let (window, mut device, mut factory, main_color, _main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    let game = game::Game::new(&mut factory, ev_recv);

    let enc: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    game_send.send(enc.clone_empty()).unwrap(); // double-buffering renderers
    game_send.send(enc).unwrap();

    std::thread::spawn(move || game_loop(game, game_recv, game_send));

    'main: loop {
        use gfx::Device;
        let mut encoder = match dev_recv.recv() {
            Ok(r) => r,
            Err(_) => break 'main,
        };
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => break 'main,
                glutin::Event::Closed => break 'main,
                _ => ev_send.process_glutin(event),
            }
        }
        // draw a frame
        encoder.flush(&mut device);
        dev_send.send(encoder).unwrap();
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
