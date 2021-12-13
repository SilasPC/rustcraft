
use crate::prelude::*;

pub struct IData {
    pub content: Arc<Content>,
    pub item_cubes: Arc<VAO>,
    pub cube: Arc<VAO>,
    pub line_box: Arc<VAO>,
    pub font: Arc<Font>,
    pub break_atlas: Arc<TextureAtlas>,
    pub atlas: Arc<TextureAtlas>,
    pub vign: Arc<Texture>,
    pub clouds: Arc<Texture>,
}

impl IData {
    pub fn air(&self) -> &BlockData {
        self.content.blocks.get("air").unwrap()
    }
}

pub struct RData {
    pub frame_time: Instant,
    pub delta: f32,
    pub fov: PerspectiveFov<f32>,
    pub view_mat: Matrix4<f32>,
    pub proj_mat: Matrix4<f32>,
}

pub struct Data {
    pub display: GLDisplay,
    pub audio: AudioSys,
    pub event_pump: sdl2::EventPump,
    pub input: Input,
    pub settings: Settings,
    pub paused: bool,
}