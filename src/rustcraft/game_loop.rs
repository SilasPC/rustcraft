
use crate::display::GLDisplay;
use crate::Program;
use crate::Data;
use crate::component::*;
use cgmath::*;
use std::time::Instant;
use crate::texture::Texture;
use crate::gui::render::*;

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

    let bbox = Texture::from_path("assets/bbox.png");
    let cube = crate::chunk::cube_mesh();
    let mut guirend = GUIRenderer::new();

    let mut view_mat = Matrix4::one();

    let mut block_updates = Updates::default();

    let mut pgui = super::player_gui::PlayerGUI::new();

    'main: loop {

        data.delta = data.frame_time.elapsed().as_secs_f32().min(0.1);
        data.frame_time = Instant::now();

        // START INPUT PROCESSING

        data.input.start_new_frame();
        
        let mut do_toggle = false;
        let mut do_refresh = false;
        for event in display.event_pump.poll_iter() {
            use sdl2::event::Event::*;
            use sdl2::keyboard::Keycode::*;
            use sdl2::event::WindowEvent::*;
            data.input.update(&event);
            match event {
                Quit {..} => break 'main,
                KeyDown {keycode: Some(Escape), ..} => break 'main,
                KeyDown {keycode: Some(F11), ..} => do_toggle = true,
                Window { win_event: Resized(..), .. } => do_refresh = true,
                _ => {}
            }
        }

        if do_toggle {display.toggle_fullscren();}
        if do_refresh {display.refresh();}

        pgui.scroll(-data.input.scroll());

        if let Ok((pos, phys, view)) = data.ecs.query_one_mut::<(&mut Position, &mut Physics, &View)>(data.cam) {

            if data.input.holding_jump() {
                phys.try_jump();
            }

            if data.input.clicked_primary() {
                if let Some(hit) = raycast(pos.pos+view.offset(), &pos.heading(), 5., &data.block_map, &data.world) {
                    if set_block(&mut data.world, &data.ent_tree, &hit.1, 0, true) {
                        block_updates.add_defered(hit.1);
                    }
                }
            } else if data.input.clicked_secondary() {
                if let Some(hit) = raycast(pos.pos+view.offset(), &pos.heading(), 5., &data.block_map, &data.world) {
                    if set_block(&mut data.world, &data.ent_tree, &hit.0, 5, /* false */true) {
                        block_updates.add_defered(hit.0);
                    }
                }
            }

            pos.rotate(
                data.input.mouse_y() as f32,
                data.input.mouse_x() as f32
            );

            let force = data.input.compute_movement_vector(pos.yaw()) * 40.;
            phys.apply_force_continuous(data.delta, &force);

            view_mat = Matrix4::from(pos.rot) * Matrix4::from_translation(-pos.pos-view.offset());

            let aabb = phys.get_aabb(pos);
            //println!("Player: {:?}",aabb);

            data.ent_tree.set(data.cam, &aabb);

        }

        // START WORLD PROCESSING
        block_updates.update(data);
        // STOP WORLD PROCESSING

        
        // START SYSTEMS
        WanderingAI::system_update(data);
        Physics::system_update(data);
        FallingBlock::system_collide_land(data);
        // STOP SYSTEMS


        // START RENDERING
        render(&program, data, &view_mat, &cube, &bbox);
        guirend.render(&mut pgui, display.aspect_ratio());
        // END RENDERING

        display.window.gl_swap_window();

    }

}

#[derive(Default)]
struct Updates {
    pub area: Vec<Vector3<f32>>,
    pub def1: Vec<Vector3<f32>>,
    pub def2: Vec<Vector3<f32>>,
    pub one: bool,
}

impl Updates {
    pub fn add_area(&mut self, pos: Vector3<f32>) {
        self.area.push(pos);
    }
    pub fn add_defered(&mut self, pos: Vector3<f32>) {
        if self.one {
            self.def2.push(pos)
        } else {
            self.def1.push(pos)
        }
    }
    pub fn update(&mut self, data: &mut Data) {
        let mut new_block_updates = vec![];
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
                                data.ent_tree.set(falling_block, &aabb);
                                new_block_updates.push(pos);
                            }
                        }
                    }
                }
            };
        }
        for pos in &mut self.area {
            let pos = *pos;
            update!(pos+Vector3::unit_x());
            update!(pos-Vector3::unit_x());
            update!(pos+Vector3::unit_y());
            update!(pos-Vector3::unit_y());
            update!(pos+Vector3::unit_z());
            update!(pos-Vector3::unit_z());
        }
        let def = if self.one {&mut self.def1} else {&mut self.def2};
        for pos in def.iter() {
            update!(*pos);
        }
        def.clear();
        self.one = !self.one;
        self.area = new_block_updates;
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
    
    for chunk in data.world.chunks.iter_mut().flat_map(|inn| inn.iter_mut().flat_map(|i2| i2.iter_mut())) {

        chunk.refresh(&data.block_map, &data.atlas);

        program.load_mat4(2, &Matrix4::from_translation(
            chunk.pos * 16.
        ));

        chunk.mesh.bind();
        chunk.mesh.draw();

    }

    cube.bind();
    bbox.bind();

    Position::system_draw_bounding_boxes(data, &program, &cube);

}

fn raycast(mut pos: Vector3<f32>, heading: &Vector3<f32>, max_dist: f32, block_map: &Vec<super::block::Block>, w: &super::world::WorldData) -> Option<(Vector3<f32>,Vector3<f32>)> {
    
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

    fn check_hit(block_map: &Vec<super::block::Block>, w: &super::world::WorldData, pos: &Vector3<f32>) -> bool {
        block_at(w, pos)
            .map(|id| block_map[id].solid)
            .unwrap_or(false)
    }
}

pub fn block_at(w: &super::world::WorldData, pos: &Vector3<f32>) -> Option<usize> {
    let cc = (pos / 16.).map(|c| c.floor() as isize);
    let mut sc = (pos % 16.).map(|c| c.floor() as isize);
    if cc.x < 0 || pos.y < 0. || cc.z < 0 {
        return None
    }
    let chunk = &w.chunks[cc.x as usize][cc.y as usize][cc.z as usize];
    if sc.x < 0 {sc.x += 16}
    if sc.z < 0 {sc.z += 16}

    chunk.data[sc.x as usize][sc.y as usize][sc.z as usize].into()
}

pub fn set_block(w: &mut crate::world::WorldData, t: &crate::EntTree, pos: &Vector3<f32>, val: usize, ignore_ents: bool) -> bool {
    if !ignore_ents {
        const EPSILON: f32 = 0.1;
        let bpos = pos.map(|v| v.floor());
        if t.any_overlaps(&(
            (
                bpos.x+EPSILON,
                bpos.y+EPSILON,
                bpos.z+EPSILON,
            ),(
                bpos.x+1.-EPSILON,
                bpos.y+1.-EPSILON,
                bpos.z+1.-EPSILON,
            )
        )) {
            return false
        }
    }
    let cc = (pos / 16.).map(|c| c.floor() as isize);
    let mut sc = (pos % 16.).map(|c| c.floor() as isize);
    if cc.x < 0 || pos.y < 0. || cc.z < 0 {
        return false
    }
    let chunk = &mut w.chunks[cc.x as usize][cc.y as usize][cc.z as usize];
    if sc.x < 0 {sc.x += 16}
    if sc.z < 0 {sc.z += 16}
    let rf = &mut chunk.data[sc.x as usize][sc.y as usize][sc.z as usize];
    if *rf == val {
        false
    } else {
        *rf = val;
        chunk.needs_refresh = true;
        true
    }
}