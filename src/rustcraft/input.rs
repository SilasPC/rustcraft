
use cgmath::*;

#[derive(Default, Debug)]
pub struct Input {
    forward: i32,
    rightward: i32,
    jump: u32,
    sneak: u32,
    primary: u32,
    secondary: u32,
    scroll: i32,
    mouse_rel: (i32,i32),
    mouse: (i32,i32),
}

impl Input {

    pub fn clicked_primary(&self) -> bool {self.primary == 1}
    pub fn clicked_secondary(&self) -> bool {self.secondary == 1}
    pub fn holding_primary(&self) -> bool {self.primary >= 1}
    pub fn holding_sneak(&self) -> bool {self.sneak >= 1}
    pub fn holding_secondary(&self) -> bool {self.secondary >= 1}
    pub fn clicked_jump(&self) -> bool {self.jump == 1}
    pub fn holding_jump(&self) -> bool {self.jump >= 1}
    pub fn scroll(&self) -> i32 {self.scroll}
    pub fn mouse_x(&self) -> i32 {self.mouse_rel.0}
    pub fn mouse_y(&self) -> i32 {self.mouse_rel.1}

    pub fn start_new_frame(&mut self) {
        self.mouse_rel = (0,0);
        self.scroll = 0;
        if self.primary > 0 {self.primary += 1}
        if self.secondary > 0 {self.secondary += 1}
        if self.jump > 0 {self.jump += 1}
    }
    pub fn reset(&mut self) {*self = Self::default();}

    pub fn update_scancodes(&mut self, state: sdl2::keyboard::KeyboardState) {
        use sdl2::keyboard::Scancode::*;
        if state.is_scancode_pressed(LShift) {
            self.sneak += 1;
        } else {
            self.sneak = 0;
        }
    }
    pub fn update(&mut self, event: &sdl2::event::Event) {
        use sdl2::event::Event;
        use sdl2::mouse::MouseButton::{Right, Left};
        use sdl2::keyboard::Keycode::*;

        match event {
            Event::KeyDown {keycode: Some(W), ..} => self.forward += 1,
            Event::KeyDown {keycode: Some(S), ..} => self.forward -= 1,
            Event::KeyDown {keycode: Some(A), ..} => self.rightward -= 1,
            Event::KeyDown {keycode: Some(D), ..} => self.rightward += 1,
            Event::KeyDown {keycode: Some(Space), ..} => self.jump += 1,
            Event::KeyUp {keycode: Some(W), ..} => self.forward += -1,
            Event::KeyUp {keycode: Some(S), ..} => self.forward -= -1,
            Event::KeyUp {keycode: Some(A), ..} => self.rightward -= -1,
            Event::KeyUp {keycode: Some(D), ..} => self.rightward += -1,
            Event::KeyUp {keycode: Some(Space), ..} => self.jump = 0,
            Event::MouseButtonDown { mouse_btn: Left, .. } => self.primary += 1,
            Event::MouseButtonDown { mouse_btn: Right, .. } => self.secondary += 1,
            Event::MouseButtonUp { mouse_btn: Left, .. } => self.primary = 0,
            Event::MouseButtonUp { mouse_btn: Right, .. } => self.secondary = 0,
            Event::MouseMotion { x, y, xrel, yrel, .. } => {
                self.mouse.0 = *x;
                self.mouse.1 = *y;
                self.mouse_rel.0 += xrel;
                self.mouse_rel.1 += yrel;
            }
            Event::MouseWheel { y, .. } => self.scroll += y,
            _ => {}
        }

        self.forward = self.forward.min(1).max(-1);
        self.rightward = self.rightward.min(1).max(-1);

    }

    pub fn mouse_pos(&self) -> (i32,i32) {self.mouse}

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