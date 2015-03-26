#![feature(collections, core, custom_attribute, plugin)]
#![plugin(gfx_macros)]

extern crate cgmath;
extern crate gfx;
extern crate gfx_device_gl;
extern crate glutin;
#[cfg(feature = "glfw")]
extern crate glfw;
extern crate id;
#[macro_use]
extern crate ecs;
extern crate env_logger;
extern crate rand;
extern crate time;

use std::sync::mpsc;
use gfx::traits::*;

mod event;
mod game;
mod world;
mod sys {
    pub mod aster;
    pub mod bullet;
    pub mod control;
    pub mod draw;
    pub mod inertia;
    pub mod physics;
}

pub type Renderer = gfx::Renderer<gfx_device_gl::GlResources, gfx_device_gl::CommandBuffer>;

fn game_loop(mut game: game::Game, ren_recv: mpsc::Receiver<Renderer>,
             ren_end: mpsc::Sender<Renderer>) {
    while game.is_alive() {
        let mut renderer = match ren_recv.recv() {
            Ok(r) => r,
            Err(_) => break,
        };
        renderer.reset();
        game.render(&mut renderer);
        match ren_end.send(renderer) {
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

#[cfg(feature = "glfw")]
fn main() {
    use glfw::Context;
    env_logger::init().unwrap();
    println!("{}", USAGE);

    let title = "Asteroids example for #scene-rs";
    let (ev_send, ev_recv) = event::SenderHub::new();
    let (game_send, dev_recv) = mpsc::channel();
    let (dev_send, game_recv) = mpsc::channel();

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenglProfile(glfw::OpenGlProfileHint::Core));
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);

    let (mut window, events) = glfw
        .create_window(640, 480, title, glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_key_polling(true); // so we can quit when Esc is pressed
    let mut device = gfx_device_gl::GlDevice::new(|s| glfw.get_proc_address(s));

    let (w, h) = window.get_framebuffer_size();
    let frame = gfx::Frame::new(w as u16, h as u16);
    let game = game::Game::new(frame, ev_recv, &mut device);

    let renderer = device.create_renderer();
    game_send.send(renderer.clone_empty()).unwrap(); // double-buffering renderers
    game_send.send(renderer).unwrap();

    std::thread::spawn(|| game_loop(game, game_recv, game_send));

    while !window.should_close() {
        let renderer = match dev_recv.recv() {
            Ok(r) => r,
            Err(_) => break,
        };
        glfw.poll_events();
        // quit when Esc is pressed.
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) =>
                    window.set_should_close(true),
                _ => ev_send.process_glfw(event),
            }
        }
        device.submit(renderer.as_buffer());
        match dev_send.send(renderer) {
            Ok(_) => (),
            Err(_) => break,
        }
        window.swap_buffers();
        device.after_frame();
    }
}

#[cfg(not(feature = "glfw"))]
fn main() {
    env_logger::init().unwrap();
    println!("{}", USAGE);

    let title = "Asteroids example for #scene-rs";
    let (ev_send, ev_recv) = event::SenderHub::new();
    let (game_send, dev_recv) = mpsc::channel();
    let (dev_send, game_recv) = mpsc::channel();

    let window = glutin::WindowBuilder::new()
        .with_title(title.to_string())
        .with_gl_version((3,2))
        .build().unwrap();

    unsafe { window.make_current() };
    let mut device = gfx_device_gl::GlDevice::new(|s| window.get_proc_address(s));

    let (w, h) = window.get_inner_size().unwrap();
    let frame = gfx::Frame::new(w as u16, h as u16);
    let game = game::Game::new(frame, ev_recv, &mut device);

    let renderer = device.create_renderer();
    game_send.send(renderer.clone_empty()).unwrap(); // double-buffering renderers
    game_send.send(renderer).unwrap();

    std::thread::spawn(|| game_loop(game, game_recv, game_send));

    'main: loop {
        let renderer = match dev_recv.recv() {
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
        device.submit(renderer.as_buffer());
        dev_send.send(renderer).unwrap();
        window.swap_buffers();
        device.after_frame();
    }
}
