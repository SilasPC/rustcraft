
use crate::worker::*;
use meshing::ChunkRenderer;
use crate::cmd::Cmd;
use crate::updates::Updates;
use crate::crafting::CraftingRegistry;
use crate::util::position_to_chunk_coordinates;
use crate::util::AABB;
use crate::text::text::Text;
use game::pgui::GUI;
use crate::gui::render::GUIRenderer;
use crate::display::GLDisplay;
use crate::component::*;
use std::time::{Instant, Duration};
use crate::texture::Texture;
use game::player::inventory::PlayerInventory;

use crate::prelude::*;

/// Tick interval duration
const TICK_DURATION: Duration = Duration::from_millis(50);
/// Number of random ticks per chunk per game tick
const RANDOM_TICK_SPEED: usize = 3;
/// Sky minimum brightness
const SKY_MIN_BRIGHTNESS: f32 = 0.4;
/// Sky color
const SKY: (f32,f32,f32) = (110./256., 160./256., 240./256.,);

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
    let mut guirend = GUIRenderer::new(display.size_i32());
    let mut block_updates = Updates::default();
    let mut pgui = GUI::new();
    let worker_data = WorkerData {
        registry: data.registry.clone()
    };
    let mut worker = JobDispatcher::new(worker_data);
    let mut last_tick = Instant::now();
    let text_rend = crate::engine::text::font::TextRenderer::new();
    let mut debug_text = rdata.font.build_text("RustCraft dev build".into());
    use crate::engine::lines::*;
    let box_vao = box_vao();
    let lines = LineProgram::new();
    let vign = data.loader.load_texture("assets/vign.png");
    let prg = Program::load(
        include_str!("vert.glsl"),
        include_str!("frag.glsl"),
        vec!["project","view","transform","uvScale","uvOffset"]
    );
    
    let mut state = GameState::Playing;
    let mut event_pump = display.event_pump();
    let mut last_tick_dur = 0.;

    let irenderer = ItemGUIRenderer::generate(data.registry.as_ref());
    
    data.world.load_around(&WorldPos::from(Vector3 {x:50., y: 55., z: 50.}));

    'main: loop {

        let mut do_chunk_load = false;

        data.delta = data.frame_time.elapsed().as_secs_f32().min(0.1);
        data.frame_time = Instant::now();

        if last_tick.elapsed() > TICK_DURATION {
            last_tick += TICK_DURATION;

            let start = Instant::now();
            if !state.is_paused() {
                block_updates.update(data);
                crate::rustcraft::component::ItemCmp::system_tick_age_items(data);
                data.world.ticks += 1;
            }

            let mut rng = rand::thread_rng();
            use rand::prelude::*;

            for p in std::mem::take(&mut data.world.to_update) {
                if let Some(on_update) = data.world.block_at(&p).map(|b| b.behavior.clone()).flatten().map(|beh| beh.on_update).flatten() {
                    on_update(p, data)
                }
            }

            let keys = data.world.chunks.iter().filter(|(_,c)| c.renderable()).map(|(k,_)| k.clone()).collect::<Vec<_>>();
            for cp in keys {
                // println!("{:?}",cp);
                for _ in 0..RANDOM_TICK_SPEED {
                    let random = rng.gen::<(i32,i32,i32)>();
                    let pos = cp.as_pos_i32() + Vector3::from(random).map(|x| x.abs() % 16).into();
                    assert_eq!(cp, pos.as_chunk());
                    if let Some(on_rnd_tick) = data.world.block_at(&pos).map(|b| b.behavior.clone()).flatten().map(|beh| beh.on_rnd_tick).flatten() {
                        on_rnd_tick(pos, data)
                    }
                }
            }
            last_tick_dur = start.elapsed().as_secs_f32() * 1000.;

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
                            let cmd: Option<Cmd> = text.text().parse().ok();
                            println!("{}\n => {:?}",text.text(),cmd);
                            if let Some(cmd) = cmd {
                                cmd.exec(data);
                            }
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
                            GameState::Chat { text: rdata.font.build_text("".into()), start_frame: data.frame_time }
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
        let mut on_use = None;
        let mut to_spawn = vec![];

        if let Ok((pos, phys, view, pdata)) = data.ecs.query_one_mut::<(&mut Position, &mut Physics, &View, &mut PlayerData)>(data.cam) {

            raycast_hit = raycast(pos.pos+view.offset().into(), &pos.heading(), 5., &data.registry, &data.world);
            if do_chunk_load {
                /* worker.send(WorkerJob::SaveChunk(
                    data.world.take_chunk()
                )); */
                data.world.load_around(&pos.pos);
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
    - tick: {:.1} ms
"#,
                    pos,
                    position_to_chunk_coordinates(&pos.pos),
                    raycast_hit.and_then(|(_,hit)| w.block_at(&hit)).map(|b| &b.name),
                    1. / data.delta,
                    last_tick_dur
                )
            );

            // println!("{}", data.world.area(&pos.pos).is_some());

            if state.is_playing() {

                if data.input.holding_jump() {
                    phys.try_jump();
                }
    
                if data.input.clicked_primary() {
                    if let Some(hit) = raycast_hit {
                        let block = data.world.block_at(&hit.1).unwrap().clone();
                        if data.world.set_block_at(&hit.1, data.registry.get("air").as_block().unwrap()) {
                            if let Some(drop_id) = &block.drops {
                                let mut stack = ItemStack::of(data.registry.get(&drop_id).clone(), 1);
                                // * merge directly
                                /* pdata.inventory.merge(&mut Some(stack)); */
                                let phys = Physics::new();
                                let pos = Position::new(hit.1.pos_center().into(),(0.3,0.3,0.3).into());
                                let aabb = pos.get_aabb();
                                let cmps = (
                                    pos,
                                    phys,
                                    ItemCmp::from(stack),
                                    Model::from(rdata.cube.clone()),
                                );
                                to_spawn.push((cmps,aabb));
                            }
                            block_updates.add_area(hit.1.0);
                            block_updates.add_single(hit.1.0);
                            data.world.to_update.push(hit.1.as_pos_i32());
                        }
                    }
                } else if data.input.clicked_secondary() {

                    if let Some(hit) = raycast_hit {
                        let maybe_item = &mut pdata.inventory.hotbar[pgui.selected_slot as usize];
                        let mut success = false;
                        if let Some(ref mut block) = maybe_item.as_mut().and_then(|item| item.item.as_block()) {
                            if data.world.set_block_at(&hit.0, &block) {
                                success = true;
                                block_updates.add_area(hit.0.0);
                                block_updates.add_single(hit.0.0);
                                data.world.to_update.push(hit.0.as_pos_i32());
                            }
                        } else {
                            let b = data.world.block_at(&hit.1).unwrap();
                            on_use = b.behavior.as_ref().and_then(|b| b.on_use.map(|f| (hit.1,f)));
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

            rdata.view_mat = Matrix4::from(pos.rot) * Matrix4::from_translation(-pos.pos.0-view.offset());

            /* let aabb = phys.get_aabb(pos);
            //println!("Player: {:?}",aabb);

            data.ent_tree.set(data.cam, &aabb); */
            
        }

        for (cmps, aabb) in to_spawn {
            let ent = data.ecs.spawn(cmps);
            data.ent_tree.insert(ent, (), &aabb);
        }

        if let Some((p,f)) = on_use {
            f(p.as_pos_i32(), data);
        }

        // interact with inventory
        if let Ok(pdata) = data.ecs.query_one_mut::<&mut PlayerData>(data.cam) {
            match state {
                GameState::Inventory { ref mut picked_item } => {
                    if data.input.clicked_primary() {
                        let slot = pgui.determine_hovered_slot(data.input.mouse_pos());
                        // println!("{:?}",slot);
                        if let Some(slot) = slot {
                            pdata.inventory.transfer(slot, picked_item, &data.registry, &data.crafting);
                        }
                    }
                },
                _ => {}
            }
        }
        
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

        // ! START RENDERING

        unsafe {
            let lf = (data.world.smooth_light_level() * (1. - SKY_MIN_BRIGHTNESS)) + SKY_MIN_BRIGHTNESS;
            gl::ClearColor(
                SKY.0 * lf,
                SKY.1 * lf,
                SKY.2 * lf,
                1.
            );
        }

        chunk_renderer.program.enable();
        unsafe {
            gl::Clear(
                gl::COLOR_BUFFER_BIT |
                gl::DEPTH_BUFFER_BIT
            );
            gl::Enable(gl::DEPTH_TEST);
            gl::ActiveTexture(gl::TEXTURE0);
        }
        
        // render chunks
        data.atlas.texture().bind();
        chunk_renderer.load_proj(&Matrix4::from(data.fov));
        chunk_renderer.load_view(&rdata.view_mat);
        chunk_renderer.load_glob_light(data.world.smooth_light_level().max(1./16.));
        chunk_renderer.render(&data.world);

        // render bounding boxes
        /* rdata.cube.bind();
        rdata.bbox.bind();
        Position::system_draw_bounding_boxes(data, &chunk_renderer.program, &rdata.cube); // ! change */

        // render entities
        prg.enable();
        prg.load_mat4(0, &Matrix4::from(data.fov));
        prg.load_mat4(1, &rdata.view_mat);
        Model::system_render(data, &prg);

        // render item in hand
        if let Ok(pdata) = data.ecs.query_one_mut::<&PlayerData>(data.cam) {
            if let Some(item) = &pdata.inventory.hotbar[pgui.selected_slot()] {
                unsafe {
                    gl::Disable(gl::DEPTH_TEST);
                }
                prg.enable();
                prg.load_mat4(0, &Matrix4::from(data.fov));
                prg.load_mat4(1, &Matrix4::one());
                let pos = (0.4, -1.5, -2.).into();
                let rot = Euler::new(Deg(0.),Deg(-60.),Deg(0.));
                prg.load_mat4(2, &(Matrix4::from_translation(pos) * Matrix4::from(rot)));
                data.atlas.bind();
                /* match &item.item {
                    ItemLike::Item(item) => {
                        data.registry.item_vao.bind();
                        data.registry.item_vao.draw_6((item.id - data.registry.items_offset) as i32); // hax
                    }
                    ItemLike::Block(block) => {
                        // rdata.cube.draw();
                    }
                } */
                unsafe {
                    gl::Enable(gl::DEPTH_TEST);
                }
            }
        }

        // tmp vignette solution
        guirend.start();
        guirend.set_pixels(0, 0);
        guirend.square.bind();
        vign.bind();
        let (w,h) = guirend.screen_size;
        let ps = guirend.pixel_scale;
        guirend.set_uniforms(w/ps,h/ps);
        guirend.square.draw();
        guirend.stop();

        // render inventory
        if let Ok(pdata) = data.ecs.query_one_mut::<&PlayerData>(data.cam) {
            pgui.render(&mut guirend, &data.registry, &pdata.inventory, state.show_inventory(), data.input.mouse_pos(), &irenderer);
            match state {
                GameState::Inventory { ref picked_item } => {
                    if let Some(picked_item) = picked_item {
                        let m = data.input.mouse_pos();
                        guirend.set_pixels(m.0, display.size_i32().1 - m.1);
                        guirend.move_pixels(-8, -8);
                        guirend.set_uniforms(16, 16);
                        unsafe {
                            gl::Enable(gl::BLEND);
                        }
                        irenderer.draw(&picked_item.item);
                        unsafe {
                            gl::Disable(gl::BLEND);
                        }
                    }
                },
                _ => {}
            }
        }

        if let Some(hit) = raycast_hit {
            let hit = hit.1.map(|v| v.floor());
            lines.program.enable();
            lines.program.load_mat4(0, &Matrix4::from_translation(hit));
            lines.program.load_mat4(1, &rdata.view_mat);
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

        // ! END RENDERING

        display.window.gl_swap_window();

    }

}

fn raycast(mut pos: WorldPos<f32>, heading: &Vector3<f32>, max_dist: f32, reg: &Registry, w: &super::world::WorldData) -> Option<(WorldPos<f32>,WorldPos<f32>)> {
    
    let mut dist = 0.;
    while dist < max_dist && !check_hit(&reg, w, &pos) {
        dist += 0.1;
        pos.0 += 0.1 * heading;
    }

    if dist < max_dist {
        return Some(((pos.0-0.1*heading).into(),pos))
    } else {
        return None
    }

    fn check_hit(reg: &Registry, w: &super::world::WorldData, pos: &Vector3<f32>) -> bool {
        w.block_at(&pos.as_coord())
            .map(|b| b.solid)
            .unwrap_or(false)
    }
}
