#![allow(dead_code)]
#![allow(unused)]
#![feature(box_patterns)]

mod util;
mod chunk;
mod engine;
mod rustcraft;
mod perlin;
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

pub struct Data {
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
    pub block_map: Vec<std::sync::Arc<block::Block>>,
    pub atlas: std::rc::Rc<TextureAtlas>,
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
        let block_map = make_blocks();
        Data {
            loader,
            paused: false,
            settings,
            fov,
            input: Input::default(),
            cam,
            frame_time: Instant::now(),
            world: world::WorldData::new("seed!"),
            atlas,
            delta: 0.,
            ecs,
            block_map,
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

    game_loop::game_loop(&mut display, &mut data);
    
}

pub fn gen_item_vao(b: &Vec<std::sync::Arc<crate::rustcraft::block::Block>>, a: &TextureAtlas) -> crate::engine::vao::VAO {

    let mut verts = vec![];
    let mut uvs = vec![];

    // six triangles per block item
    for b in b {
        verts.extend_from_slice(&[
            // top
            0.5, 1., 0.,
            0., 0.75, 0.,
            1., 0.75, 0.,
            0., 0.75, 0.,
            0.5, 0.5, 0.,
            1., 0.75, 0.,
            // left
            0., 0.75, 0.,
            0.5, 0., 0.,
            0.5, 0.5, 0.,
            0.5, 0., 0.,
            0., 0.75, 0.,
            0.0, 0.25, 0.,
            // right
            0.5, 0.5, 0.,
            0.5, 0., 0.,
            1., 0.75, 0.,
            0.5, 0., 0.,
            1., 0.25, 0.,
            1., 0.75, 0.,
        ]);
        let (t,s,_) = b.texture;
        let (u,v) = a.get_uv(t);
        let d = a.uv_dif();
        uvs.extend_from_slice(&[
            // top
            u, v,
            u, v+d,
            u+d, v,
            u, v+d,
            u+d, v+d,
            u+d, v,
        ]);
        let (u,v) = a.get_uv(s);
        let d = a.uv_dif();
        uvs.extend_from_slice(&[
            // left
            u, v,
            u+d, v+d,
            u+d, v,
            u+d, v+d,
            u, v,
            u, v+d,
            // right
            u, v,
            u, v+d,
            u+d, v,
            u, v+d,
            u+d, v+d,
            u+d, v,
        ]);
    }

    crate::engine::vao::VAO::textured(&verts, &uvs)

}