
use crate::content::inventory::PlayerGUI;
use inventory::render::*;
use crate::lines::LineProgram;
use crate::text::font::TextRenderer;
use crate::static_prg::StaticProgram;
use crate::worker::*;
use meshing::ChunkRenderer;
use crate::cmd::Cmd;
use crate::crafting::CraftingRegistry;
use crate::util::*;
use crate::text::text::Text;
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

pub struct GameLoop<'cnt> {
    pub data: &'cnt mut data::Data,
    pub rdata: &'cnt mut data::RData,
    pub idata: &'cnt data::IData,
    pub world: WorldData<'cnt>,
    pub chunk_renderer: ChunkRenderer,
    pub pgui: PlayerGUI,
    pub last_tick: Instant,
    pub text_rend: TextRenderer,
    pub debug_text: DebugText,
    pub lines: LineProgram,
    pub prg: Program,
    pub sprg: StaticProgram,
    pub state: GameState,
    pub last_tick_dur: f32,
    pub invren: InventoryRenderer,
    pub tx: mpsc::Sender<server::ClientMsg>,
    pub rx: mpsc::Receiver<server::ServerMsg>,
    pub player_pos: Position,
    pub player_phys: Physics,
    pub player_view: View,
}

impl<'cnt: 'b, 'b> GameLoop<'cnt> {
    pub fn new(conn: (mpsc::Sender<server::ClientMsg>, mpsc::Receiver<server::ServerMsg>), data: &'cnt mut data::Data, rdata: &'cnt mut data::RData, idata: &'cnt data::IData) -> Self {
            
        let mut world = WorldData::new(consts::DEBUG_SEED, idata.air());

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
        let mut pgui = PlayerGUI::new();
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
            iren: ItemGUIRenderer::generate(&idata.content.items, &idata.atlas),
            gui: GUIRenderer::new(data.display.size_i32()),
            atlas: idata.atlas.clone(),
            highlight: Texture::from_path("assets/slot_highlight.png").into()
        };

        world.load_around(&WorldPos::from(Vector3 {x:50., y: 55., z: 50.}));

        let (tx,rx) = conn;
        let (player_pos, player_phys, player_view,_) = make_player().0;
        Self {
            tx,
            rx,
            data,
            rdata,
            idata,
            world,
            chunk_renderer,
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
            player_pos,
            player_phys,
            player_view,
        }

    }

    pub fn run_loop(&'b mut self) {while self.run() {}}
    pub fn run(&'b mut self) -> bool {

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

            self.player_phys.update(&mut self.player_pos, self.rdata.delta, &self.world.blocks);

        }
        // ! STOP SYSTEMS

        // TODO this is too slow
        self.world.blocks.refresh(&self.idata.content.items, &self.idata.atlas);
        self.world.load(&self.idata.content, &self.idata.atlas, 5); // ! adjust for performance

        // RENDER
        let now = Instant::now();
        self.handle_render(raycast_hit);

        self.data.display.window.gl_swap_window();

        true

    }

}
