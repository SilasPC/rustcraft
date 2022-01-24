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
pub mod server;
use crate::builder::ContentBuilder;
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
use crate::rustcraft::input::Input;
use crate::engine::texture::*;
use engine::*;
use rustcraft::*;

pub mod prelude;
use crate::prelude::*;

fn main() {

    println!("Loading...");
    let mut data = init_data();
    let mut rdata = init_rdata(&data);
    let idata = init_idata();

    let (conn, server) = server(idata.content.clone());
    let mut game_loop = GameLoop::new(conn, &mut data, &mut rdata, &idata);
    
    println!("Client started");
    game_loop.run_loop();
    println!("Client stopped");

    server.stop();
}


use mpsc::{Sender,Receiver,channel};
use server::*;
struct Server {
    kill: Sender<()>,
    thread: std::thread::JoinHandle<()>,
}
impl Server {
    pub fn stop(self) {
        self.kill.send(());
        self.thread.join();
    }
}
fn server(content: Arc<Content>) -> ((Sender<ClientMsg>, Receiver<ServerMsg>), Server) {
    
    let (stx,rx) = channel();
    let (tx,srx) = channel();
    let (kill,krx) = channel();

    let thread = std::thread::spawn(move || {

        println!("Server start");
        let mut server = ServerLoop::new((stx, srx), &content);
        while server.run_and_sleep() && !krx.try_recv().is_ok() {} 
        println!("Server stop");

    });

    ((tx,rx), Server {
        kill,
        thread,
    })

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
        audio: AudioSys::new(),
        input: Input::default(),
        settings,
        paused: false,
        event_pump,
    }
}

fn init_idata() -> data::IData {
    let atlas = Arc::new(TextureAtlas::new(
        Texture::from_path("assets/atlas.png"),
        6
    ));
    let break_atlas = TextureAtlas::new(
        Texture::from_path("assets/break_atlas.png"),
        4
    ).into();

    let mut content = ContentBuilder::new();
    content.load_mod(&mut content::base::BaseMod);
    let content: Arc<_> = content.finish().into();

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