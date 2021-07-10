#![allow(dead_code)]
#![allow(unused)]
#![feature(box_patterns)]

mod util;
mod engine;
mod coords;
mod rustcraft;
mod perlin;
use crate::lines::box_vao;
use crate::chunk::meshing::cube_mesh;
use crate::crafting::CraftingRegistry;
use crate::text::font::Font;
use crate::vao::VAO;
use crate::display::GLDisplay;
use std::sync::Arc;
use crate::registry::Registry;
use crate::util::BVH;
use crate::content::*;
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
    pub mouse_sensitivity: f32,
}

pub struct RenderData {
    pub bbox: Arc<Texture>,
    pub cube: VAO,
    pub line_box: VAO,
    pub view_mat: Matrix4<f32>,
    pub font: Arc<Font>,
}

impl RenderData {
    pub fn new(data: &mut Data) -> Self {
        let bbox = data.loader.load_texture("assets/bbox.png");
        let font = data.loader.load_font("assets/font.png", "assets/font.fnt");
        let cube = cube_mesh();
        let view_mat = Matrix4::one();
        let line_box = box_vao();
        Self {
            bbox,
            cube,
            font,
            view_mat,
            line_box
        }
    }
}

pub struct Data {
    pub crafting: CraftingRegistry,
    pub loader: crate::engine::loader::Loader,
    pub paused: bool,
    pub settings: Settings,
    pub fov: PerspectiveFov<f32>,
    pub cam: hecs::Entity,
    pub input: Input,
    pub frame_time: Instant,
    pub delta: f32,
    pub world: world::WorldData,
    pub ecs: hecs::World,
    pub registry: Arc<Registry>,
    pub atlas: Arc<TextureAtlas>,
    pub ent_tree: BVH<hecs::Entity, ()>,
}

impl Data {
    pub fn new(settings: Settings) -> Self {
        let mut loader = crate::engine::loader::Loader::new();
        let fov = PerspectiveFov {
            near: 0.1,
            far: 1000.,
            fovy: Rad::from(settings.fov),
            aspect: 900./700.
        };
        let mut ecs = hecs::World::new();
        
        let mut ent_tree = BVH::new();
        let cam = {
            let (cam, aabb) = make_player();
            let cam = ecs.spawn(cam);
            ent_tree.insert(cam, (), &aabb);
            cam
        };
        let atlas = loader.load_texture_atlas("assets/atlas.png", 4);
        let registry = make_registry(atlas.clone());
        let crafting = make_crafting_registry(&registry);
        Data {
            crafting,
            loader,
            paused: false,
            settings,
            fov,
            input: Input::default(),
            cam,
            frame_time: Instant::now(),
            world: world::WorldData::new("seed!", registry[0].clone()),
            registry,
            atlas,
            delta: 0.,
            ecs,
            ent_tree
        }
    }
}

fn main() {

    let mut display = display::GLDisplay::new("Rustcraft", (900,700));

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
        fov: Deg(90.),
        mouse_sensitivity: 0.5,
    };
    
    let mut data = Data::new(settings);
    let mut rdata = RenderData::new(&mut data);

    game_loop::game_loop(&mut display, &mut data, &mut rdata);
    
}
