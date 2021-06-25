#![allow(dead_code)]
#![allow(unused)]

mod util;
mod chunk;
mod engine;
mod rustcraft;
mod perlin;
use engine::program::*;
use rustcraft::component::*;
use cgmath::*;
use std::time::Instant;
use crate::rustcraft::input::Input;
use crate::engine::texture::*;
use engine::*;
use rustcraft::*;
use aabb_tree::*;

pub struct Settings {
    pub fov: Deg<f32>,
    pub mouse_sensitivity: f32,
}

pub type AABB = ((f32,f32,f32,),(f32,f32,f32,));

pub struct EntTree {
    tree: AabbTree<hecs::Entity>,
    map: std::collections::HashMap<hecs::Entity, aabb_tree::Proxy>,
}

impl EntTree {
    pub fn new() -> Self {
        Self {
            tree: AabbTree::new(),
            map: Default::default()
        }
    }
    pub fn set(&mut self, ent: hecs::Entity, aabb: &AABB) {
        if let Some(proxy) = self.map.get(&ent) {
            self.tree.set_aabb(*proxy, aabb);
        } else {
            let proxy = self.tree.create_proxy(*aabb, ent);
            self.map.insert(ent, proxy);
        }
    }
    pub fn remove(&mut self, ent: hecs::Entity) {
        if let Some(proxy) = self.map.remove(&ent) {
            self.tree.destroy_proxy(proxy);
        }
    }
    pub fn any_overlaps(&self, aabb: &AABB) -> bool {
        let mut found = false;
        self.tree.query_aabb(aabb, |_| {
            found = true;
            false
        });
        found
    }
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
    pub ent_tree: EntTree
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
        
        let mut ent_tree = EntTree::new();
        let cam = {
            let pos = Position::from(Vector3 {x:50., y: 55., z: 50.});
            let mut phys = Physics::new(Vector3 {
                x: 0.8,
                y: 1.9,
                z: 0.8,
            });
            //cam_phys.set_flying(true);
            let aabb = phys.get_aabb(&pos);
            let cam = ecs.spawn((pos, phys, View::from(Vector3 {
                x: 0.5,
                y: 1.8,
                z: 0.5,
            })));
            ent_tree.set(cam, &aabb);
            cam
        };
        let atlas = loader.load_texture_atlas("assets/atlas.png", 4);
        use block::Block;
        let block_map = vec![
            Block { id: 0, transparent: true, solid: false, no_render: true, texture: (0,0,0), has_gravity: false, drops: None, }, // air
            Block { id: 1, transparent: false, solid: true, no_render: false, texture: (0,0,0), has_gravity: false, drops: Some(1), }, // stone
            Block { id: 2, transparent: false, solid: true, no_render: false, texture: (1,1,1), has_gravity: false, drops: Some(2), }, // dirt
            Block { id: 3, transparent: false, solid: true, no_render: false, texture: (3,2,1), has_gravity: false, drops: Some(2), }, // grass
            Block { id: 4, transparent: false, solid: true, no_render: false, texture: (5,4,5), has_gravity: false, drops: Some(4), }, // wood log
            Block { id: 5, transparent: false, solid: true, no_render: false, texture: (6,6,6), has_gravity: true, drops: Some(5), }, // sand
            Block { id: 6, transparent: true, solid: true, no_render: false, texture: (7,7,7), has_gravity: false, drops: None, }, // leaves
        ].into_iter().map(std::sync::Arc::new).collect();
        Data {
            loader,
            paused: false,
            settings,
            fov,
            input: Input::default(),
            cam,
            frame_time: Instant::now(),
            world: world::WorldData::new(),
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