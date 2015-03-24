#![feature(plugin)]
#![plugin(gfx_macros)]

extern crate cgmath;
extern crate gfx;
extern crate gfx_device_gl;
extern crate glutin;
extern crate glfw;
#[macro_use]
extern crate id;
extern crate ecs;

use glfw::Context;
use gfx::{Device, DeviceHelper};

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

fn game_loop<R: gfx::Resources, C: gfx::GlCommandBuffer<R>>(
             mut game: game::Game, ren_recv: Receiver<gfx::Renderer<R, C>>,
             ren_end: Sender<Renderer<R, C>>) {
    while game.is_alive() {
        let mut renderer = match ren_recv.recv_opt() {
            Ok(r) => r,
            Err(_) => break,
        };
        renderer.reset();
        game.render(&mut renderer);
        match ren_end.send_opt(renderer) {
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

fn main() {
    let use_glfw = true;
    let title = "Asteroids example for #scene-rs";
    let (ev_send, ev_recv) = event::SenderHub::new();
    let (game_send, dev_recv) = channel();
    let (dev_send, game_recv) = channel();

    println!("{}", USAGE);

    if use_glfw {
        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::ContextVersion(3, 2));
        glfw.window_hint(glfw::OpenglForwardCompat(true));
        glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));
        glfw.set_error_callback(glfw::FAIL_ON_ERRORS);

        let (window, events) = glfw
            .create_window(640, 480, title, glfw::Windowed)
            .expect("Failed to create GLFW window.");

        window.make_current();
        window.set_key_polling(true); // so we can quit when Esc is pressed
        let mut device = gfx::GlDevice::new(|s| glfw.get_proc_address(s));

        let (w, h) = window.get_framebuffer_size();
        let frame = gfx::Frame::new(w as u16, h as u16);
        let game = game::Game::new(frame, ev_recv, &mut device);

        let renderer = device.create_renderer();
        game_send.send(renderer.clone_empty()); // double-buffering renderers
        game_send.send(renderer);

        spawn(proc() game_loop(game, game_recv, game_send));

        while !window.should_close() {
            let renderer = match dev_recv.recv_opt() {
                Ok(r) => r,
                Err(_) => break,
            };
            glfw.poll_events();
            // quit when Esc is pressed.
            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) =>
                        window.set_should_close(true),
                    _ => ev_send.process_glfw(event),
                }
            }
            device.submit(renderer.as_buffer());
            match dev_send.send_opt(renderer) {
                Ok(_) => (),
                Err(_) => break,
            }
            window.swap_buffers();
        }
    }else {
        let window = glutin::WindowBuilder::new()
            .with_title(title.to_string())
            .with_gl_version((3,2))
            .build().unwrap();

        unsafe { window.make_current() };
        let mut device = gfx::GlDevice::new(|s| window.get_proc_address(s));

        let (w, h) = window.get_inner_size().unwrap();
        let frame = gfx::Frame::new(w as u16, h as u16);
        let game = game::Game::new(frame, ev_recv, &mut device);

        let renderer = device.create_renderer();
        game_send.send(renderer.clone_empty()); // double-buffering renderers
        game_send.send(renderer);

        spawn(proc() game_loop(game, game_recv, game_send));

        'main: loop {
            let renderer = dev_recv.recv();
            // quit when Esc is pressed.
            for event in window.poll_events() {
                match event {
                    glutin::KeyboardInput(_, _, Some(glutin::Escape), _) => break 'main,
                    glutin::Closed => break 'main,
                    _ => ev_send.process_glutin(event),
                }
            }
            device.submit(renderer.as_buffer());
            dev_send.send(renderer);
            window.swap_buffers();
        }
    };
}
