
use sdl2::*;
use sdl2::video::*;

pub struct GLDisplay {
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    pub _gl_ctx: GLContext,
    pub event_pump: EventPump,
}

impl GLDisplay {
    pub fn new(size: (u32, u32)) -> Self {
            
        let sdl = sdl2::init().unwrap();

        let video = sdl.video().unwrap();
    
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 5);
    
        let window = video.window("RustCraft", size.0, size.1)
            .resizable()
            .opengl()
            .build()
            .unwrap();
        let _gl_ctx = window.gl_create_context().unwrap();
        
        gl::load_with(|s| video.gl_get_proc_address(s) as *const std::os::raw::c_void);
        
        let event_pump = sdl.event_pump().unwrap();

        Self {
            sdl,
            video,
            window,
            _gl_ctx,
            event_pump,
        }
        
    }

    pub fn refresh(&mut self) {
        let size = self.size();
        unsafe {
            gl::Viewport(0, 0, size.0 as i32, size.1 as i32);
        }
    }

    pub fn size(&self) -> (u32,u32) {self.window.size()}

    pub fn aspect_ratio(&self) -> f32 {
        let (x,y) = self.window.size();
        x as f32 / y as f32
    }

    pub fn toggle_fullscren(&mut self) -> bool {
        let fs = self.window.fullscreen_state();
        use sdl2::video::FullscreenType::*;
        self.window.set_fullscreen(
            match fs {
                Off => True,
                _ => Off
            }
        ).is_ok()
    }

}