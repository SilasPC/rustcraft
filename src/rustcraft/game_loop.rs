
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
use crate::rustcraft::item::ItemStack;

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

    let mut last_tick = Instant::now();
    let tick_duration = Duration::from_millis(50);

    let font = data.loader.load_font("assets/font.png", "assets/font.fnt");
    let text_rend = crate::engine::text::font::TextRenderer::new();
    let mut sum_text = font.build_text("0 fps".into());

    use crate::engine::lines::*;
    let box_vao = box_vao();
    let lines = LineProgram::new();

    let item_vao = crate::gen_item_vao(&data.block_map, data.atlas.as_ref());

    display.set_mouse_capture(true);

    let mut state = GameState::Playing;
    let mut event_pump = display.event_pump();

    display.video.text_input().start();

    data.world.load_around2(&Vector3 {x:50., y: 55., z: 50.}, 40.);

    'main: loop {

        let mut do_chunk_load = false;

        data.delta = data.frame_time.elapsed().as_secs_f32().min(0.1);
        data.frame_time = Instant::now();

        if last_tick.elapsed() > tick_duration {
            last_tick += tick_duration;

            if !state.is_paused() {
                block_updates.update(data);
                Item::system_tick_age_items(data);
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

            raycast_hit = raycast(pos.pos+view.offset(), &pos.heading(), 5., &data.block_map, &data.world);
            if do_chunk_load {
                data.world.load_around2(&pos.pos, 30.);
            }

            if state.is_playing() {

                if data.input.holding_jump() {
                    phys.try_jump();
                }
    
                if data.input.clicked_primary() {
                    if let Some(hit) = raycast_hit {
                        let bid = block_at(&data.world, &hit.1).unwrap();
                        if set_block(&mut data.world, &data.ent_tree, &hit.1, 0, true) {
                            let stack = ItemStack::of(data.block_map[bid].clone(), 1);
                            pdata.inventory.merge(Some(stack));
                            block_updates.add_area(hit.1);
                            block_updates.add_single(hit.1);
                        }
                    }
                } else if data.input.clicked_secondary() {
    
                    let maybe_item = &mut pdata.inventory.hotbar[pgui.selected_slot as usize];
                    let mut success = false;
                    if let Some(ref mut item) = maybe_item {
                        if let Some(hit) = raycast_hit {
                            if set_block(&mut data.world, &data.ent_tree, &hit.0, item.item.id, /* false */true) {
                                success = true;
                                block_updates.add_area(hit.0);
                                block_updates.add_single(hit.0);
                            }
                        }
                    }
                    if success {
                        ItemStack::deduct(maybe_item, 1);
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
            pgui.render(&mut guirend, &item_vao, &data.atlas, &pdata.inventory, state.show_inventory(), data.input.mouse_pos());
            match state {
                GameState::Inventory { ref mut picked_item } => {
                    if data.input.clicked_primary() {
                        let slot = pgui.determine_hovered_slot(data.input.mouse_pos());
                        println!("{:?}",slot);
                        if let Some(slot) = slot {
                            ItemStack::transfer(picked_item, pdata.inventory.slot_mut(slot));
                        }
                    }
                    if let Some(picked_item) = picked_item {
                        let m = data.input.mouse_pos();
                        guirend.set_pixels(m.0, display.size_i32().1 - m.1);
                        guirend.move_pixels(-8, -8);
                        guirend.set_uniforms(16, 16);
                        item_vao.draw_18(picked_item.item.id as i32);
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
                text_rend.render(&text)
            },
            _ => {}
        };

        /* sum_text.set_text(format!("{:.0} \nfps", 1. / data.delta));
        text_rend.render(&sum_text); */

        // END RENDERING

        display.window.gl_swap_window();

    }

}

#[derive(Default)]
struct Updates {
    pub area1: Vec<Vector3<f32>>,
    pub area2: Vec<Vector3<f32>>,
    pub def1: Vec<Vector3<f32>>,
    pub def2: Vec<Vector3<f32>>,
}

impl Updates {
    pub fn add_area(&mut self, pos: Vector3<f32>) {
        self.area2.push(pos);
    }
    pub fn add_single(&mut self, pos: Vector3<f32>) {
        self.def2.push(pos);
    }
    pub fn update(&mut self, data: &mut Data) {
        macro_rules! update {
            ($pos:expr) => {
                let pos: Vector3<f32> = $pos;
                if let Some(id) = block_at(&data.world, &pos) {
                    let block = &data.block_map[id];
                    if block.has_gravity {
                        if let Some(id) = block_at(&data.world, &(pos - Vector3::unit_y())) {
                            let below = &data.block_map[id];
                            if !below.solid {
                                set_block(&mut data.world, &data.ent_tree, &pos, 0, true);
                                let fall_pos = pos.map(|v| v.floor());
                                let fall_size = Vector3 {
                                    x: 1.,
                                    y: 1.,
                                    z: 1.,
                                };
                                let pos_comp = Position::from(fall_pos);
                                let phys = Physics::new(fall_size);
                                let aabb = phys.get_aabb(&pos_comp);
                                let falling_block = data.ecs.spawn((
                                    pos_comp, phys, FallingBlock::of(block.id)
                                ));
                                data.ent_tree.insert(falling_block, (), &aabb);
                                self.area2.push(pos);
                            }
                        }
                    }
                }
            };
        }
        for pos in &mut self.area1 {
            let pos = *pos;
            update!(pos+Vector3::unit_x());
            update!(pos-Vector3::unit_x());
            update!(pos+Vector3::unit_y());
            update!(pos-Vector3::unit_y());
            update!(pos+Vector3::unit_z());
            update!(pos-Vector3::unit_z());
        }
        for pos in self.def1.iter() {
            update!(*pos);
        }
        self.def1.clear();
        self.area1.clear();
        std::mem::swap(&mut self.def1, &mut self.def2);
        std::mem::swap(&mut self.area1, &mut self.area2);
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

        chunk.refresh(&data.block_map, &data.atlas);
        chunk.bind_and_draw();

    }

    cube.bind();
    bbox.bind();

    Position::system_draw_bounding_boxes(data, &program, &cube);

}

fn raycast(mut pos: Vector3<f32>, heading: &Vector3<f32>, max_dist: f32, block_map: &Vec<std::sync::Arc<super::block::Block>>, w: &super::world::WorldData) -> Option<(Vector3<f32>,Vector3<f32>)> {
    
    let mut dist = 0.;
    while dist < max_dist && !check_hit(block_map, w, &pos) {
        dist += 0.1;
        pos += 0.1 * heading;
    }

    if dist < max_dist {
        return Some((pos-0.1*heading,pos))
    } else {
        return None
    }

    fn check_hit(block_map: &Vec<std::sync::Arc<super::block::Block>>, w: &super::world::WorldData, pos: &Vector3<f32>) -> bool {
        block_at(w, pos)
            .map(|id| block_map[id].solid)
            .unwrap_or(false)
    }
}

pub fn block_at(w: &super::world::WorldData, pos: &Vector3<f32>) -> Option<usize> {
    let chunk = w.chunk_at_pos(&pos)?;
    chunk.block_id_at_pos(&pos).into()
}

pub fn set_block(w: &mut crate::world::WorldData, t: &crate::util::BVH<hecs::Entity, ()>, pos: &Vector3<f32>, val: usize, ignore_ents: bool) -> bool {
    if !ignore_ents {
        // TODO: not working properly?
        const EPSILON: f32 = 0.1;
        let bpos = pos.map(|v| v.floor());
        let mut aabb = AABB::from_corner(&bpos, 1.);
        aabb.extend_radius(-EPSILON);
        if t.any_overlaps(&aabb) {
            return false
        }
    }
    let chunk = w.chunk_at_pos_mut(&pos);
    if let Some(chunk) = chunk {
        chunk.set_at_pos(&pos, val)
    } else {
        false
    }
}