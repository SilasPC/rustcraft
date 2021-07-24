
use crate::lines::LineProgram;
use crate::text::font::TextRenderer;
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

pub struct GameLoop<'a> {
    pub data: &'a mut data::Data,
    pub rdata: &'a mut data::RData,
    pub idata: &'a data::IData,
    pub world: WorldData,
    pub chunk_renderer: ChunkRenderer,
    pub block_updates: Updates,
    pub pgui: GUI,
    pub last_tick: Instant,
    pub text_rend: TextRenderer,
    pub debug_text: DebugText,
    pub lines: LineProgram,
    pub prg: Program,
    pub sprg: StaticProgram,
    pub state: GameState,
    pub last_tick_dur: f32,
    pub invren: InventoryRenderer,
}

impl<'a> GameLoop<'a> {
    pub fn new(data: &'a mut data::Data, rdata: &'a mut data::RData, idata: &'a data::IData) -> Self {
            
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
        /* let worker_data = WorkerData {
            registry: idata.registry.clone()
        };
        let mut worker = JobDispatcher::new(worker_data); */
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

        Self {
            data,
            rdata,
            idata,
            world,
            chunk_renderer,
            block_updates,
            pgui,
            last_tick,
            text_rend,
            debug_text,
            lines,
            prg,
            sprg,
            state,
            last_tick_dur,
            invren,
        }

    }

    pub fn run_loop(&mut self) {while self.run() {}}
    pub fn run(&mut self) -> bool {

        self.rdata.delta = self.rdata.frame_time.elapsed().as_secs_f32().min(0.1);
        self.rdata.frame_time = Instant::now();

        // GAME TICK
        if let Some(dur) = self.handle_game_tick() {
            self.last_tick_dur = dur.as_secs_f32() * 1000.;
        }

        // INPUT PROCESSING
        let ret = self.handle_input();
        if ret.do_quit {return false}

        let raycast_hit = self.handle_interaction(ret);
        
        // ! START SYSTEMS
        if !self.state.is_paused() {
            WanderingAI::system_update(&mut self.world, self.rdata.delta);
            Physics::system_update(&mut self.world, self.rdata.delta);
            FallingBlock::system_collide_land(&mut self.world);
        }
        // ! STOP SYSTEMS

        // TODO this is too slow
        self.world.blocks.refresh(&self.idata.registry);
        self.world.load(&self.idata.registry, 100);

        // RENDER
        let now = Instant::now();
        self.handle_render(raycast_hit);

        self.data.display.window.gl_swap_window();

        true

    }

}
