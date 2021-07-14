
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

/// Tick interval duration
const TICK_DURATION: Duration = Duration::from_millis(50);
/// Number of random ticks per chunk per game tick
const RANDOM_TICK_SPEED: usize = 3;
/// Sky minimum brightness
const SKY_MIN_BRIGHTNESS: f32 = 0.4;
/// Minimum block brightness
const MIN_BRIGHTNESS: f32 = 0.4;
/// Sky color
const SKY: (f32,f32,f32) = (110./256., 160./256., 240./256.,);

pub fn game_loop(display: &mut GLDisplay, data: &mut Data, rdata: &mut RenderData) {

    display.refresh();
    display.set_mouse_capture(true);
    display.video.text_input().start();

    unsafe {
        gl::ClearColor(
            SKY.0,
            SKY.1,
            SKY.2,
            1.
        );
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    let chunk_renderer = ChunkRenderer::new();
    let mut block_updates = Updates::default();
    let mut pgui = GUI::new();
    let worker_data = WorkerData {
        registry: data.registry.clone()
    };
    let mut worker = JobDispatcher::new(worker_data);
    let mut last_tick = Instant::now();
    let text_rend = crate::engine::text::font::TextRenderer::new();
    let mut debug_text = DebugText::from(&rdata.font);
    use crate::engine::lines::*;
    let lines = LineProgram::new();
    let vign = data.loader.load_texture("assets/vign.png");
    let prg = Program::load(
        include_str!("../vert.glsl"),
        include_str!("../frag.glsl"),
        vec!["project","view","transform","uvScale","uvOffset"]
    );
    
    let mut state = GameState::Playing;
    let mut event_pump = display.event_pump();
    let mut last_tick_dur = 0.;

    let mut invren = InventoryRenderer {
        iren: ItemGUIRenderer::generate(data.registry.as_ref()),
        gui: GUIRenderer::new(display.size_i32()),
        atlas: data.atlas.clone(),
        highlight: data.loader.load_texture("assets/slot_highlight.png")
    };

    data.world.load_around(&WorldPos::from(Vector3 {x:50., y: 55., z: 50.}));

    'main: loop {

        data.delta = data.frame_time.elapsed().as_secs_f32().min(0.1);
        data.frame_time = Instant::now();

        // GAME TICK
        if let Some(dur) = handle_game_tick::handle_game_tick(
            data,
            &mut last_tick,
            &mut block_updates,
            &state
        ) {
            last_tick_dur = dur.as_secs_f32() * 1000.;
        }

        // INPUT PROCESSING
        let ret = handle_input::handle_input(
            data,
            &mut state,
            &mut event_pump,
            display,
            &mut pgui,
            rdata
        );
        if ret.do_quit {break 'main}

        let raycast_hit = handle_interaction::handle_interaction(
            data,
            rdata,
            &mut state,
            &mut debug_text,
            &mut block_updates,
            last_tick_dur,
            &pgui
        );
        
        // ! START SYSTEMS
        if !state.is_paused() {
            WanderingAI::system_update(data);
            Physics::system_update(data);
            FallingBlock::system_collide_land(data);
        }
        // ! STOP SYSTEMS

        // TODO this is waaay to slow
        data.world.refresh(&data.registry);
        data.world.load(&data.registry, 100);

        // RENDER
        handle_render::handle_render(
            display,
            data,
            rdata,
            &chunk_renderer,
            &mut invren,
            &text_rend,
            &mut debug_text,
            &lines,
            &prg,
            &pgui,
            &state,
            raycast_hit
        );

        display.window.gl_swap_window();

    }

}
