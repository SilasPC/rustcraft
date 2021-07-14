
use sdl2::*;
use sdl2::video::*;

#[derive(Debug)]
pub struct DisplayState {
    pub vsync: bool,
    pub fullscreen: bool,
}

pub struct GLDisplay {
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    pub _gl_ctx: GLContext,
    pub state: DisplayState,
}

impl GLDisplay {
    pub fn new(title: &str, size: (u32, u32)) -> Self {
            
        let sdl = sdl2::init().unwrap();
        let mut video = sdl.video().unwrap();
        video.gl_set_swap_interval(sdl2::video::SwapInterval::Immediate);
    
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 5);
    
        let window = video.window(title, size.0, size.1)
            .resizable()
            .opengl()
            .build()
            .unwrap();
        let _gl_ctx = window.gl_create_context().unwrap();
        
        gl::load_with(|s| video.gl_get_proc_address(s) as *const std::os::raw::c_void);

        Self {
            sdl,
            video,
            window,
            _gl_ctx,
            state: DisplayState {
                vsync: false,
                fullscreen: false
            }
        }
        
    }

    pub fn event_pump(&self) -> EventPump {
        self.sdl.event_pump().unwrap()
    }

    pub fn set_title(&mut self, title: &str) {
        let _ = self.window.set_title(title);
    }

    pub fn refresh(&mut self) {
        let size = self.size();
        unsafe {
            gl::Viewport(0, 0, size.0 as i32, size.1 as i32);
        }
    }

    pub fn set_vsync(&mut self, on: bool) {
        let mode = sdl2::video::SwapInterval::from(on as i32);
        if self.video.gl_set_swap_interval(mode).is_ok() {
            self.state.fullscreen = on;
        }
    }

    pub fn set_mouse_capture(&mut self, capture: bool) {
        let mouse = self.sdl.mouse();
        mouse.show_cursor(!capture);
        mouse.capture(capture);
        mouse.set_relative_mouse_mode(capture);
    }

    pub fn size(&self) -> (u32,u32) {self.window.size()}
    pub fn size_i32(&self) -> (i32, i32) {
        let (x,y) = self.window.size();
        (x as i32, y as i32)
    }

    pub fn aspect_ratio(&self) -> f32 {
        let (x,y) = self.window.size();
        x as f32 / y as f32
    }

    pub fn set_fullscreen(&mut self, on: bool) {
        use sdl2::video::FullscreenType;;
        let state = match on {
            true => FullscreenType::True,
            false => FullscreenType::Off
        };
        if self.window.set_fullscreen(state).is_ok() {
            self.state.fullscreen = on;
        }
    }

}