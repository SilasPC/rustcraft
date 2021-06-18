
use crate::display::GLDisplay;
use crate::Program;
use crate::Data;
use crate::component::*;
use cgmath::*;
use std::time::Instant;

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

    use crate::texture::Texture;
    use crate::gui::{gui::*,render::*};

    let bbox = Texture::from_path("assets/bbox.png");
    let cube = crate::chunk::cube_mesh();
    let mut guirend = GUIRenderer::new();

    let item_bar = Texture::from_path("assets/item_bar.png");
    let item_bar = FlatGUI {
        texture: std::rc::Rc::from(item_bar),
        anchor: Anchor::Bottom,
        pos: -Vector2::unit_y(),
        scale: Scale::FixedWidth(1.3)
    };
    // width == 1.3 => height == 1.3/9.

    let item = Texture::from_path("assets/item_selected.png");
    let mut item = FlatGUI {
        texture: std::rc::Rc::from(item),
        anchor: Anchor::Bottom.add(Anchor::Offset(-4.,0.)),
        pos: -Vector2::unit_y(),
        scale: Scale::FixedWidth(1.3/9.)
    };
    let mut scroll_pos = 0i32;

    let heart = std::rc::Rc::from(Texture::from_path("assets/heart.png"));
    let mut hearts = vec![];
    for i in 0..10 {
        hearts.push(FlatGUI {
            texture: std::rc::Rc::clone(&heart),
            anchor: Anchor::BottomLeft.add(Anchor::Offset(1.1 * i as f32, 0.)),
            pos: Vector2 {x: -1.3/2., y: -0.8 },
            scale: Scale::FixedWidth(1.3/9. / 2.)
        })
    }
    let hearts = ContainerGUI(hearts);

    let mut view_mat = Matrix4::one();
    'main: loop {

        data.delta = data.frame_time.elapsed().as_secs_f32();
        data.frame_time = Instant::now();

        data.input.start_new_frame();
        
        let mut do_toggle = false;
        for event in display.event_pump.poll_iter() {
            use sdl2::event::Event::*;
            use sdl2::keyboard::Keycode::*;
            use sdl2::event::WindowEvent::*;
            data.input.update(&event);
            match event {
                Quit {..} => break 'main,
                KeyDown {keycode: Some(Escape), ..} => break 'main,
                KeyDown {keycode: Some(F11), ..} => do_toggle = true,
                Window { win_event: Resized(..), .. } => {},
                _ => {}
            }
        }

        if do_toggle {display.toggle_fullscren()}

        //let mut pos_phys = None;
        if let Ok((pos, phys, view)) = data.ecs.query_one_mut::<(&mut Position, &mut Physics, &View)>(data.cam) {

            // pos_phys = Some((pos.clone(),phys.clone(),view.clone()));

            if data.input.jump() {
                phys.try_jump();
            }

            if data.input.primary() {
                if let Some(hit) = raycast(pos.pos+view.offset(), &pos.heading(), 5., &data.block_map, &data.world) {
                    set_block(&mut data.world, &hit.1, 0);
                }
            } else if data.input.secondary() {
                if let Some(hit) = raycast(pos.pos+view.offset(), &pos.heading(), 5., &data.block_map, &data.world) {
                    set_block(&mut data.world, &hit.0, 1);
                }
            }

            pos.rotate(
                data.input.mouse_y() as f32,
                data.input.mouse_x() as f32
            );

            let force = data.input.compute_movement_vector(pos.yaw()) * 40.;
            phys.apply_force_continuous(data.delta, &force);

            view_mat = Matrix4::from(pos.rot) * Matrix4::from_translation(-pos.pos-view.offset());

        }

        // throw cubes debugging
        /* if pos_phys.is_some() && rand::random::<f32>() < 0.01 {
            let (mut pos, mut phys, view) = pos_phys.unwrap();
            pos.add(view.offset());
            phys.set_size(&Vector3 { x: 0.1, y: 0.1, z: 0.1 });
            phys.apply_force(&(5.*pos.heading()));
            data.ecs.spawn((pos,phys));
        } */

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
        program.load_mat4(1, &view_mat);
        
        for chunk in data.world.chunks.iter_mut().flat_map(|inn| inn.iter_mut()) {

            chunk.refresh(&data.block_map, &data.atlas);

            program.load_mat4(2, &Matrix4::from_translation(
                chunk.pos * 16.
            ));

            chunk.mesh.bind();
            chunk.mesh.draw();

        }

        cube.bind();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, bbox.id());
            gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as f32);
            gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as f32);
        }

        Position::system_draw_bounding_boxes(data, &program, &cube);

        item.anchor = Anchor::Bottom.add(Anchor::Offset(scroll_pos as f32 - 4.,0.));

        guirend.render(&item_bar, display.aspect_ratio());
        guirend.render(&item, display.aspect_ratio());
        guirend.render(&hearts, display.aspect_ratio());

        WanderingAI::system_update(data);
        Physics::system_update(data);

        display.window.gl_swap_window();

    }

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
        let cc = (pos / 16.).map(|c| c.floor() as isize);
        let mut sc = (pos % 16.).map(|c| c.floor() as isize);
        if cc.x < 0 || pos.y < 0. || cc.z < 0 {
            return false;
        }
        let chunk = &w.chunks[cc.x as usize][cc.z as usize];
        if sc.x < 0 {sc.x += 16}
        if sc.z < 0 {sc.z += 16}
        let id = chunk.data[sc.x as usize][sc.y as usize][sc.z as usize];
        block_map[id].solid
    }
}

fn set_block(w: &mut crate::world::WorldData, pos: &Vector3<f32>, val: usize) {
    let cc = (pos / 16.).map(|c| c.floor() as isize);
    let mut sc = (pos % 16.).map(|c| c.floor() as isize);
    if cc.x < 0 || pos.y < 0. || cc.z < 0 {
        return
    }
    let chunk = &mut w.chunks[cc.x as usize][cc.z as usize];
    if sc.x < 0 {sc.x += 16}
    if sc.z < 0 {sc.z += 16}
    chunk.data[sc.x as usize][sc.y as usize][sc.z as usize] = val;
    chunk.needs_refresh = true;
}