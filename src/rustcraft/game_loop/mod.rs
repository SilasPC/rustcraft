
use crate::static_prg::StaticProgram;
use crate::inv::*;
use crate::worker::*;
use meshing::ChunkRenderer;
use crate::cmd::Cmd;
use crate::updates::Updates;
use crate::crafting::CraftingRegistry;
use crate::util::*;
use crate::text::text::Text;
use game::pgui::GUI;
use crate::gui::render::GUIRenderer;
use crate::display::GLDisplay;
use crate::component::*;
use std::time::{Instant, Duration};
use crate::texture::Texture;
use game::player::inventory::PlayerInventory;

use crate::prelude::*;

mod handle_input;
mod handle_game_tick;
mod handle_render;
mod handle_interaction;
mod state;
use state::*;

pub fn game_loop(data: &mut data::Data, rdata: &mut data::RData, idata: &data::IData) {

    let mut world = WorldData::new(consts::DEBUG_SEED, idata.air.clone());

    data.display.refresh();
    data.display.set_mouse_capture(true);
    data.display.video.text_input().start();

    unsafe {
        gl::ClearColor(
            consts::SKY.0,
            consts::SKY.1,
            consts::SKY.2,
            1.
        );
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    let chunk_renderer = ChunkRenderer::new();
    let mut block_updates = Updates::default();
    let mut pgui = GUI::new();
    let worker_data = WorkerData {
        registry: idata.registry.clone()
    };
    let mut worker = JobDispatcher::new(worker_data);
    let mut last_tick = Instant::now();
    let text_rend = crate::engine::text::font::TextRenderer::new();
    let mut debug_text = DebugText::from(&idata.font);
    use crate::engine::lines::*;
    let mut lines = LineProgram::new(idata.line_box.clone());
    let prg = Program::load(
        include_str!("../vert.glsl"),
        include_str!("../frag.glsl"),
        vec!["project","view","transform","uvScale","uvOffset"]
    );
    let mut sprg = StaticProgram::new();
    
    let mut state = GameState::Playing { breaking: std::option::Option::None };
    let mut last_tick_dur = 0.;
    

    let mut invren = InventoryRenderer {
        iren: ItemGUIRenderer::generate(idata.registry.as_ref()),
        gui: GUIRenderer::new(data.display.size_i32()),
        atlas: idata.atlas.clone(),
        highlight: Texture::from_path("assets/slot_highlight.png").into()
    };

    world.load_around(&WorldPos::from(Vector3 {x:50., y: 55., z: 50.}));

    'main: loop {

        rdata.delta = rdata.frame_time.elapsed().as_secs_f32().min(0.1);
        rdata.frame_time = Instant::now();

        // GAME TICK
        if let Some(dur) = handle_game_tick::handle_game_tick(
            data,
            &mut world,
            &mut last_tick,
            &mut block_updates,
            &state
        ) {
            last_tick_dur = dur.as_secs_f32() * 1000.;
        }

        // INPUT PROCESSING
        let ret = handle_input::handle_input(
            data,
            &mut world,
            &mut state,
            &mut pgui,
            rdata,
            idata
        );
        if ret.do_quit {break 'main}

        let raycast_hit = handle_interaction::handle_interaction(
            data,
            rdata,
            idata,
            &mut world,
            &mut state,
            &mut debug_text,
            &mut block_updates,
            last_tick_dur,
            &pgui,
            &mut invren,
        );
        
        // ! START SYSTEMS
        if !state.is_paused() {
            WanderingAI::system_update(&mut world, rdata.delta);
            Physics::system_update(&mut world, rdata.delta);
            FallingBlock::system_collide_land(&mut world);
        }
        // ! STOP SYSTEMS

        // TODO this is waaay to slow
        world.refresh(&idata.registry);
        world.load(&idata.registry, 100);

        // RENDER
        let now = Instant::now();
        handle_render::handle_render(
            data,
            rdata,
            idata,
            &mut world,
            &chunk_renderer,
            &mut invren,
            &text_rend,
            &mut debug_text,
            &mut lines,
            &prg,
            &pgui,
            &state,
            raycast_hit.map(|hit| hit.hit),
            &mut sprg,
        );

        data.display.window.gl_swap_window();

    }

}
