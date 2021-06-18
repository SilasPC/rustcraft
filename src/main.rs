
mod util;
mod chunk;
mod engine;
mod rustcraft;
use engine::program::*;
use rustcraft::component::*;
use cgmath::*;
use std::time::Instant;
use crate::rustcraft::input::Input;
use crate::engine::texture::*;
use engine::*;
use rustcraft::*;

pub struct Settings {
    pub fov: Deg<f32>,
}

pub struct Data {
    pub paused: bool,
    pub settings: Settings,
    pub fov: PerspectiveFov<f32>,
    pub cam: hecs::Entity,
    pub input: Input,
    pub frame_time: Instant,
    pub delta: f32,
    pub world: world::WorldData,
    pub ecs: hecs::World,
    pub block_map: Vec<block::Block>,
    pub atlas: TextureAtlas,
}

impl Data {
    pub fn new(settings: Settings) -> Self {
        let fov = PerspectiveFov {
            near: 0.1,
            far: 1000.,
            fovy: Rad::from(settings.fov),
            aspect: 900./700.
        };
        let mut ecs = hecs::World::new();
        let cam = ecs.spawn((Position::from(Vector3 {x:2., y: 12., z: 7.12}), Physics::new(Vector3 {
            x: 0.8,
            y: 1.9,
            z: 0.8,
        }), View::from(Vector3 {
            x: 0.5,
            y: 1.8,
            z: 0.5,
        })));
        let atlas = TextureAtlas::new(
            texture::Texture::from_path("assets/atlas.png"),
            4,
        );
        let block_map = vec![
            Block { id: 0, transparent: true, solid: false, no_render: true, texture: (0,0,0) }, // air
            Block { id: 1, transparent: false, solid: true, no_render: false, texture: (0,0,0) }, // stone
            Block { id: 2, transparent: false, solid: true, no_render: false, texture: (1,1,1) }, // dirt
            Block { id: 3, transparent: false, solid: true, no_render: false, texture: (3,2,1) }, // grass
            Block { id: 4, transparent: false, solid: true, no_render: false, texture: (5,5,5) }, // sand
        ];
        use block::Block;
        Data {
            paused: false,
            settings,
            fov,
            input: Input::default(),
            cam,
            frame_time: Instant::now(),
            world: world::WorldData::new(&block_map, &atlas),
            atlas,
            delta: 0.,
            ecs,
            block_map
        }
    }
}

fn main() {
    
    let mut display = display::GLDisplay::new((900,700));

    unsafe {
        gl::Viewport(0, 0, 900, 700);
        gl::ClearColor(
            110./256.,
            160./256.,
            240./256.,
            1.
        );
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    let settings = Settings {
        fov: Deg(90.)
    };
    
    let mut data = Data::new(settings);

    data.ecs.spawn((Position::from(Vector3 {x:2., y: 12., z: 3.1}), Physics::new(Vector3 {
        x: 0.8,
        y: 1.9,
        z: 0.8,
    }), WanderingAI::new()));

    game_loop::game_loop(&mut display, &mut data);
    
}
