
use crate::updates::Updates;
use crate::chunk::Chunk;
use crate::make_crafting_registry;
use crate::crafting::CraftingRegistry;
use crate::registry::Registry;
use crate::block::Block;
use std::sync::Arc;
use crate::util::position_to_chunk_coordinates;
use crate::util::AABB;
use crate::text::text::Text;
use crate::rustcraft::pgui::GUI;
use crate::gui::render::GUIRenderer;
use crate::display::GLDisplay;
use crate::Program;
use crate::Data;
use crate::component::*;
use cgmath::*;
use std::time::{Instant, Duration};
use crate::texture::Texture;
use crate::rustcraft::player::inventory::PlayerInventory;
use crate::rustcraft::item::{ItemStack,Item,ItemLike};

pub enum GameState {
    Inventory {
        picked_item: Option<ItemStack>,
    },
    Playing,
    Paused,
    Chat {
        start_frame: Instant,
        text: Text,
    }
}

impl GameState {
    pub fn is_playing(&self) -> bool {
        match self { Self::Playing => true, _ => false }
    }
    pub fn is_paused(&self) -> bool {
        match self { Self::Paused {..} => true, _ => false }
    }
    pub fn is_chat(&self) -> bool {
        match self { Self::Chat {..} => true, _ => false }
    }
    pub fn show_inventory(&self) -> bool {
        match self {
            Self::Inventory {..} => true,
            _ => false
        }
    }
}

pub fn game_loop(display: &mut GLDisplay, data: &mut Data) {

    display.refresh();

    unsafe {
        gl::ClearColor(
            110./256.,
            160./256.,
            240./256.,
            1.
        );
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    let program = Program::load(
        include_str!("vert.glsl"),
        include_str!("frag.glsl"),
        vec!["project","view","transform"]
    );

    let bbox = data.loader.load_texture("assets/bbox.png");
    let cube = crate::chunk::cube_mesh();
    let mut guirend = GUIRenderer::new(display.size_i32());

    let mut view_mat = Matrix4::one();

    let mut block_updates = Updates::default();

    let mut pgui = GUI::new();

    let worker_data = WorkerData {
        registry: data.registry.clone()
    };
    let mut worker = JobDispatcher::new(worker_data);

    let mut last_tick = Instant::now();
    let tick_duration = Duration::from_millis(50);

    let font = data.loader.load_font("assets/font.png", "assets/font.fnt");
    let text_rend = crate::engine::text::font::TextRenderer::new();
    let mut debug_text = font.build_text("RustCraft dev build".into());

    use crate::engine::lines::*;
    let box_vao = box_vao();
    let lines = LineProgram::new();

    display.set_mouse_capture(true);

    let mut state = GameState::Playing;
    let mut event_pump = display.event_pump();

    display.video.text_input().start();

    data.world.load_around2(&Vector3 {x:50., y: 55., z: 50.}, 40., &data.registry);

    let cr = make_crafting_registry(&data.registry);

    'main: loop {

        let mut do_chunk_load = false;

        data.delta = data.frame_time.elapsed().as_secs_f32().min(0.1);
        data.frame_time = Instant::now();

        if last_tick.elapsed() > tick_duration {
            last_tick += tick_duration;

            if !state.is_paused() {
                block_updates.update(data);
                crate::rustcraft::component::Item::system_tick_age_items(data);
            }    

        }

        // START INPUT PROCESSING

        data.input.start_new_frame();
        display.video.text_input().start();
        //data.input.update_scancodes(event_pump.keyboard_state());
        for event in event_pump.poll_iter() {
            use sdl2::event::Event::*;
            use sdl2::keyboard::Keycode::*;
            use sdl2::event::WindowEvent::*;
            data.input.update(&event);
            match event {
                TextInput { ref text, .. } => {
                    let input_text = text;
                    match &mut state {
                        GameState::Chat { text, start_frame } if *start_frame != data.frame_time => {
                            let mut txt = text.text().to_owned();
                            txt.push_str(&input_text);
                            text.set_text(txt);
                        },
                        _ => {}
                    }
                },
                Quit {..} => break 'main,
                KeyDown {keycode: Some(Escape), ..} => {
                    state = match state {
                        GameState::Paused => {
                            display.set_mouse_capture(true);
                            GameState::Playing
                        },
                        GameState::Chat { .. } => {
                            display.set_mouse_capture(true);
                            GameState::Playing
                        },
                        _ => {
                            display.set_mouse_capture(false);
                            GameState::Paused
                        }
                    };
                },
                KeyDown {keycode: Some(F11), ..} => display.toggle_fullscren(),
                KeyDown {keycode: Some(R), ..} => do_chunk_load = true,
                KeyDown {keycode: Some(E), ..} => {
                    state = match state {
                        GameState::Playing => {
                            display.set_mouse_capture(false);
                            GameState::Inventory {picked_item: Option::None}
                        },
                        GameState::Inventory { .. } => {
                            display.set_mouse_capture(true);
                            GameState::Playing
                        },
                        _ => state
                    };
                },
                KeyDown {keycode: Some(Return), ..} => {
                    state = match state {
                        GameState::Chat { text, .. } => {
                            println!("{}", text.text());
                            display.set_mouse_capture(true);
                            GameState::Playing
                        },
                        _ => state
                    };
                },
                KeyDown {keycode: Some(T), ..} => {
                    state = match state {
                        GameState::Playing => {
                            display.set_mouse_capture(false);
                            GameState::Chat { text: font.build_text("".into()), start_frame: data.frame_time }
                        },
                        _ => state
                    };
                },
                KeyDown {keycode: Some(Backspace), ..} => {
                    state = match state {
                        GameState::Chat { mut text, start_frame } => {
                            let mut txt = text.text().to_owned();
                            txt.pop();
                            text.set_text(txt);
                            GameState::Chat { text, start_frame }
                        },
                        _ => state
                    };
                },
                Window { win_event: Resized(..), .. } => display.refresh(),
                _ => {}
            }
        }

        pgui.scroll(-data.input.scroll());

        let mut raycast_hit = None;

        if let Ok((pos, phys, view, pdata)) = data.ecs.query_one_mut::<(&mut Position, &mut Physics, &View, &mut PlayerData)>(data.cam) {

            raycast_hit = raycast(pos.pos+view.offset(), &pos.heading(), 5., &data.registry, &data.world);
            if do_chunk_load {
                /* worker.send(WorkerJob::SaveChunk(
                    data.world.take_chunk()
                )); */
                data.world.load_around2(&pos.pos, 30., &data.registry);
            }

            let w = &data.world;
            let bm = &data.registry;
            debug_text.set_text(
                format!(
r#"
RustCraft dev build
    - {:?}
    - Chunk {:?}
    - Looking at {:?}
    - fps: {:.0}
"#,
                    pos,
                    position_to_chunk_coordinates(&pos.pos),
                    raycast_hit.and_then(|(_,hit)| w.block_at_pos(&hit)).map(|b| &b.name),
                    1. / data.delta,
                )
            );

            // println!("{}", data.world.area(&pos.pos).is_some());

            if state.is_playing() {

                if data.input.holding_jump() {
                    phys.try_jump();
                }
    
                if data.input.clicked_primary() {
                    if let Some(hit) = raycast_hit {
                        let block = data.world.block_at_pos(&hit.1).unwrap().clone();
                        if data.world.set_block_at_pos(&hit.1, &data.registry[0]) {
                            if let Some(drop_id) = block.drops {
                                let stack = ItemStack::of(data.registry.get(drop_id), 1);
                                pdata.inventory.merge(Some(stack));
                            }
                            block_updates.add_area(hit.1);
                            block_updates.add_single(hit.1);
                        }
                    }
                } else if data.input.clicked_secondary() {

                    if let Some(hit) = raycast_hit {
                        let maybe_item = &mut pdata.inventory.hotbar[pgui.selected_slot as usize];
                        let mut success = false;
                        if let Some(ref mut block) = maybe_item.as_mut().and_then(|item| item.item.as_block()) {
                            if data.world.set_block_at_pos(&hit.0, &block) {
                                success = true;
                                block_updates.add_area(hit.0);
                                block_updates.add_single(hit.0);
                            }
                        } else {
                            let b = data.world.block_at_pos_mut(&hit.1).unwrap();
                            if let Some(on_use) = b.behavior.as_ref().and_then(|b| b.on_use) {
                                on_use(b);
                            }
                        }
                        if success {
                            ItemStack::deduct(maybe_item, 1);
                        }
                    }
    
                }
    
                pos.rotate(
                    data.input.mouse_y() as f32 * data.settings.mouse_sensitivity,
                    data.input.mouse_x() as f32 * data.settings.mouse_sensitivity
                );
    
                let force = data.input.compute_movement_vector(pos.yaw()) * 40.;
                phys.apply_force_continuous(data.delta, &force);
            }

            view_mat = Matrix4::from(pos.rot) * Matrix4::from_translation(-pos.pos-view.offset());

            /* let aabb = phys.get_aabb(pos);
            //println!("Player: {:?}",aabb);

            data.ent_tree.set(data.cam, &aabb); */
            
        }
        
        // START SYSTEMS
        if !state.is_paused() {
            WanderingAI::system_update(data);
            Physics::system_update(data);
            FallingBlock::system_collide_land(data);
        }
        // STOP SYSTEMS


        // START RENDERING
        render(&program, data, &view_mat, &cube, &bbox);

        if let Ok(pdata) = data.ecs.query_one_mut::<&mut PlayerData>(data.cam) {
            pgui.render(&mut guirend, &data.registry, &pdata.inventory, state.show_inventory(), data.input.mouse_pos());
            match state {
                GameState::Inventory { ref mut picked_item } => {
                    if data.input.clicked_primary() {
                        let slot = pgui.determine_hovered_slot(data.input.mouse_pos());
                        println!("{:?}",slot);
                        if let Some(slot) = slot {
                            pdata.inventory.transfer(slot, picked_item, &data.registry, &cr);
                        }
                    }
                    if let Some(picked_item) = picked_item {
                        let m = data.input.mouse_pos();
                        guirend.set_pixels(m.0, display.size_i32().1 - m.1);
                        guirend.move_pixels(-8, -8);
                        guirend.set_uniforms(16, 16);
                        unsafe {
                            gl::Enable(gl::BLEND);
                        }
                        match &picked_item.item {
                            ItemLike::Item(inner) => {
                                data.registry.item_vao.bind();
                                data.registry.item_vao.draw_6((inner.id - data.registry.blocks.len()) as i32);
                            },
                            ItemLike::Block(inner) => {
                                data.registry.iso_block_vao.bind();
                                data.registry.iso_block_vao.draw_18(inner.id as i32);
                            }
                        }
                        unsafe {
                            gl::Disable(gl::BLEND);
                        }
                    }
                },
                _ => {}
            }
        }

        program.load_mat4(0, &Matrix4::one());
        program.load_mat4(1, &Matrix4::one());
        program.load_mat4(2, &Matrix4::from_translation(-Vector3::unit_z()));

        if let Some(hit) = raycast_hit {
            let hit = hit.1.map(|v| v.floor());
            lines.program.enable();
            lines.program.load_mat4(0, &Matrix4::from_translation(hit));
            lines.program.load_mat4(1, &view_mat);
            lines.program.load_mat4(2, &Matrix4::from(data.fov));
            lines.program.load_vec4(3, &Vector4 {
                x: 0.2,
                y: 0.2,
                z: 0.2,
                w: 1.0,
            });
            box_vao.bind();
            box_vao.draw();
        }

        match &state {
            GameState::Chat { text, .. } => {
                text_rend.render(&text, -0.9, -0.9, display.size())
            },
            _ => {}
        };

        text_rend.render(&debug_text, -0.9, 0.9, display.size());

        // END RENDERING

        display.window.gl_swap_window();

    }

}


fn render(program: &Program, data: &mut Data, view_mat: &Matrix4<f32>, cube: &crate::engine::vao::VAO, bbox: &crate::engine::texture::Texture) {
    program.enable();
        
    unsafe {
        gl::Clear(
            gl::COLOR_BUFFER_BIT |
            gl::DEPTH_BUFFER_BIT
        );
        gl::Enable(gl::DEPTH_TEST);
        gl::ActiveTexture(gl::TEXTURE0);
        data.atlas.texture().bind();
    }
    
    program.load_mat4(0, &Matrix4::from(data.fov));
    program.load_mat4(1, view_mat);
    
    for chunk in data.world
        .chunk_iter_mut()
        .filter(|c| c.renderable_after_refresh())
    {
        
        program.load_mat4(2, &Matrix4::from_translation(
            chunk.pos.map(|x| x as f32 * 16.)
        ));

        chunk.refresh(&data.registry);
        chunk.bind_and_draw();

    }

    cube.bind();
    bbox.bind();

    Position::system_draw_bounding_boxes(data, &program, &cube);

}

fn raycast(mut pos: Vector3<f32>, heading: &Vector3<f32>, max_dist: f32, reg: &Registry, w: &super::world::WorldData) -> Option<(Vector3<f32>,Vector3<f32>)> {
    
    let mut dist = 0.;
    while dist < max_dist && !check_hit(&reg, w, &pos) {
        dist += 0.1;
        pos += 0.1 * heading;
    }

    if dist < max_dist {
        return Some((pos-0.1*heading,pos))
    } else {
        return None
    }

    fn check_hit(reg: &Registry, w: &super::world::WorldData, pos: &Vector3<f32>) -> bool {
        w.block_at_pos(pos)
            .map(|b| b.solid)
            .unwrap_or(false)
    }
}

struct WorkerData {
    registry: Arc<Registry>,
}

use std::sync::mpsc::*;

struct JobDispatcher {
    tx: Sender<WorkerJob>,
    rx: Receiver<WorkerResponse>,
}

impl JobDispatcher {

    pub fn iter_responses(&mut self) -> TryIter<'_, WorkerResponse> {
        self.rx.try_iter()
    }

    pub fn new(wdata: WorkerData) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (dtx, drx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            worker_thread(rx, dtx, wdata)
        });
        JobDispatcher {
            tx,
            rx: drx
        }
    }
    pub fn send(&self, work: WorkerJob) {
        self.tx.send(work);
    }
}

enum WorkerJob {
    SaveChunk(Box<Chunk>),
    LoadChunk(i32,i32,i32),
}
enum WorkerResponse {
    LoadedChunk(Option<Box<Chunk>>),
}
fn worker_thread(rx: Receiver<WorkerJob>, tx: Sender<WorkerResponse>, data: WorkerData) {
    'work: loop {
        let job = match rx.recv() {
            Err(_) => {break 'work}
            Ok(job) => job
        };
        use WorkerJob::*;
        match job {
            SaveChunk(chunk) => {
                std::fs::write(format!("save/{:x}_{:x}_{:x}.chunk", chunk.pos.x, chunk.pos.y, chunk.pos.z), chunk.save());
            },
            LoadChunk(x,y,z) => {
                tx.send(
                    WorkerResponse::LoadedChunk(
                        Chunk::load(x, y, z, data.registry.as_ref()).map(Box::new)
                    )
                ).unwrap();
            }
        };
    }
}