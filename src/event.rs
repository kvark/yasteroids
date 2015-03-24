use glutin;
use glfw;
use sys;

pub type ReceiverHub = (
    Receiver<sys::control::Event>,
    Receiver<sys::bullet::Event>
);

pub struct SenderHub {
    control: Sender<sys::control::Event>,
    bullet: Sender<sys::bullet::Event>,
}

impl SenderHub {
    pub fn new() -> (SenderHub, ReceiverHub) {
        let (sc, rc) = channel();
        let (sb, rb) = channel();
        (SenderHub {
            control: sc,
            bullet: sb,
        }, (rc, rb))
    }

    pub fn process_glutin(&self, event: glutin::Event) {
        use sys::control::{EvThrust, EvTurn};
        use sys::bullet::{EvShoot};
        match event {
            glutin::KeyboardInput(state, _, Some(glutin::A), _) =>
                self.control.send(EvThrust(match state {
                    glutin::Pressed => 1.0,
                    glutin::Released => 0.0,
                })),
            glutin::KeyboardInput(glutin::Pressed, _, Some(glutin::Left), _) =>
                self.control.send(EvTurn(-1.0)),
            glutin::KeyboardInput(glutin::Pressed, _, Some(glutin::Right), _) =>
                self.control.send(EvTurn(1.0)),
            glutin::KeyboardInput(glutin::Released, _, Some(k), _)
                if k == glutin::Left || k == glutin::Right =>
                self.control.send(EvTurn(0.0)),
            glutin::KeyboardInput(state, _, Some(glutin::S), _) =>
                self.bullet.send(EvShoot(match state {
                    glutin::Pressed => true,
                    glutin::Released => false,
                })),
            _ => (),
        }
    }

    pub fn process_glfw(&self, event: glfw::WindowEvent) {
        use sys::control::{EvThrust, EvTurn};
        use sys::bullet::{EvShoot};
        match event {
            glfw::KeyEvent(glfw::KeyA, _, state, _) =>
                self.control.send(EvThrust(match state {
                    glfw::Press | glfw::Repeat => 1.0,
                    glfw::Release => 0.0,
                })),
            glfw::KeyEvent(glfw::KeyLeft, _, glfw::Press, _) =>
                self.control.send(EvTurn(-1.0)),
            glfw::KeyEvent(glfw::KeyRight, _, glfw::Press, _) =>
                self.control.send(EvTurn(1.0)),
            glfw::KeyEvent(k, _, glfw::Release, _)
                if k == glfw::KeyLeft || k == glfw::KeyRight =>
                self.control.send(EvTurn(0.0)),
            glfw::KeyEvent(glfw::KeyS, _, state, _) =>
                self.bullet.send(EvShoot(match state {
                    glfw::Press | glfw::Repeat => true,
                    glfw::Release => false,
                })),
            _ => (),
        }
    }
}
