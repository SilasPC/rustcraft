
use cgmath::*;

#[derive(Default, Debug)]
pub struct Input {
    forward: i32,
    rightward: i32,
    jump: bool,
    primary: bool,
    secondary: bool,
    scroll: i32,
    mouse: (i32,i32)
}

impl Input {

    pub fn primary(&self) -> bool {self.primary}
    pub fn secondary(&self) -> bool {self.secondary}
    pub fn jump(&self) -> bool {self.jump}
    pub fn scroll(&self) -> i32 {self.scroll()}
    pub fn mouse_x(&self) -> i32 {self.mouse.0}
    pub fn mouse_y(&self) -> i32 {self.mouse.1}

    pub fn start_new_frame(&mut self) {
        self.mouse = (0,0);
        self.scroll = 0;
    }
    pub fn reset(&mut self) {*self = Self::default();}

    pub fn update(&mut self, event: &sdl2::event::Event) {
        use sdl2::event::Event;
        use sdl2::mouse::MouseButton::{Right, Left};
        use sdl2::keyboard::Keycode::*;

        match event {
            Event::KeyDown {keycode: Some(W), ..} => self.forward += 1,
            Event::KeyDown {keycode: Some(S), ..} => self.forward -= 1,
            Event::KeyDown {keycode: Some(A), ..} => self.rightward -= 1,
            Event::KeyDown {keycode: Some(D), ..} => self.rightward += 1,
            Event::KeyDown {keycode: Some(Space), ..} => self.jump = true,
            Event::KeyUp {keycode: Some(W), ..} => self.forward += -1,
            Event::KeyUp {keycode: Some(S), ..} => self.forward -= -1,
            Event::KeyUp {keycode: Some(A), ..} => self.rightward -= -1,
            Event::KeyUp {keycode: Some(D), ..} => self.rightward += -1,
            Event::KeyUp {keycode: Some(Space), ..} => self.jump = false,
            Event::MouseButtonDown { mouse_btn: Left, .. } => self.primary = true,
            Event::MouseButtonDown { mouse_btn: Right, .. } => self.secondary = true,
            Event::MouseButtonUp { mouse_btn: Left, .. } => self.primary = false,
            Event::MouseButtonUp { mouse_btn: Right, .. } => self.secondary = false,
            Event::MouseMotion { xrel, yrel, .. } => {
                self.mouse.0 += xrel;
                self.mouse.1 += yrel;
            }
            Event::MouseWheel { y, .. } => self.scroll += y,
            _ => {}
        }

        self.forward = self.forward.min(1).max(-1);
        self.rightward = self.rightward.min(1).max(-1);

    }

    pub fn compute_movement_vector(&self, yaw: Deg<f32>) -> Vector3<f32> {
        let rad = Rad::from(yaw);

        let mut dir = Vector3 {x:0.,y:0.,z:0.};

        if self.forward > 0 {
            dir.x += rad.sin();
            dir.z += -rad.cos();
        } else if self.forward < 0 {
            dir.x = -rad.sin();
            dir.z = rad.cos();
        }

        if self.rightward > 0 {
            dir.x += rad.cos();
            dir.z += rad.sin();
        } else if self.rightward < 0 {
            dir.x += -rad.cos();
            dir.z += -rad.sin();
        }

        let mag = dir.magnitude();
        if mag > 0. {
            dir / mag
        } else {
            dir
        }
        
    }

}