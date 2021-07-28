#![allow(dead_code, unused)]
#![deny(unconditional_recursion)]
#![feature(box_patterns, box_syntax, generators, generator_trait, try_blocks, duration_constants, result_cloned)]

#[macro_use]
extern crate warn;

pub mod util;
pub mod engine;
pub mod coords;
pub mod rustcraft;
pub mod perlin;
pub mod consts;
use crate::game_loop::GameLoop;
use crate::util::gen_full_block_vao;
use crate::lines::box_vao;
use crate::crafting::CraftingRegistry;
use crate::text::font::Font;
use crate::display::GLDisplay;
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

pub mod prelude {
    pub use entity::template::{EntityRegistry, EntityTemplate};
    pub use hecs::Entity;
    pub use serde_json::Value as JSON;
    pub use crate::consts;
    pub use game::settings::Settings;
    pub use util::ArcStr;
    #[macro_use]
    pub use util;
    pub use engine;
    pub use rustcraft as game;
    pub use crate::rustcraft::world::{self, *};
    pub use crate::rustcraft::item::*;
    pub use crate::*;
    pub use crate::rustcraft::component;
    pub use crate::rustcraft::chunk::chunk::*;
    pub use cgmath::*;
    pub use crate::registry::Registry;
    pub use crate::coords::*;
    pub use std::collections::{HashSet, HashMap, VecDeque, BinaryHeap};
    pub use crate::vao::VAO;
    pub use crate::rustcraft::chunk::{chunk::{self, Chunk}, meshing};
    pub use std::sync::Arc;
    pub use std::time::{Duration, Instant};
}

use crate::prelude::*;

fn main() {

    let mut data = init_data();
    let mut rdata = init_rdata(&data);
    let idata = init_idata();

    let mut game_loop = GameLoop::new(&mut data, &mut rdata, &idata);
    
    game_loop.run_loop();
    
}

fn init_data() -> data::Data {
    let mut display = display::GLDisplay::new("Rustcraft", (900,700));
    let event_pump = display.event_pump();
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
    let settings = Settings::load();
    display.set_vsync(settings.vsync);
    display.set_fullscreen(settings.fullscreen);
    data::Data {
        display,
        input: Input::default(),
        settings,
        paused: false,
        event_pump,
    }
}

fn init_idata() -> data::IData {
    let atlas = TextureAtlas::new(
        Texture::from_path("assets/atlas.png"),
        6
    ).into();
    let break_atlas = TextureAtlas::new(
        Texture::from_path("assets/break_atlas.png"),
        4
    ).into();
    let mut content = ContentBuilder::new();
    
    use content::base::*;
    register_components(&mut content);
    register_entities(&mut content);
    register_items(&mut content);
    register_recipies(&mut content);

    let content: Arc<_> = content.finish(Arc::clone(&atlas)).into();

    let font = Font::from_font_files("assets/font.png", "assets/font.fnt").into();
    let line_box = lines::box_vao().into();
    let cube = meshing::cube_mesh().into();
    let item_cubes = gen_full_block_vao(
        content.items.items.values().filter_map(ItemLike::as_block),
        &mut HashMap::new(),
        &*atlas,
    ).into();
    let vign = Texture::from_path("assets/vign.png").into();
    let clouds = Texture::from_path("assets/clouds.png").into();

    data::IData {
        air: content.items.get("air").to_block().unwrap(),
        content,
        item_cubes,
        atlas,
        break_atlas,
        font,
        line_box,
        cube,
        vign,
        clouds,
    }
}

fn init_rdata(data: &data::Data) -> data::RData {
    let fov = PerspectiveFov {
        near: 0.1,
        far: 1000.,
        fovy: Rad::from(data.settings.fov),
        aspect: 900./700.
    };

    data::RData {
        frame_time: Instant::now(),
        delta: 0.,
        fov,
        view_mat: Matrix4::one(),
        proj_mat: Matrix4::from(fov)
    }
}