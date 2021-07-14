
use crate::game_loop::MIN_BRIGHTNESS;
use crate::util::DebugText;
use crate::text::font::TextRenderer;
use crate::*;
use crate::game_loop::InventoryRenderer;
use crate::lines::LineProgram;
use crate::game_loop::GameState;
use crate::PlayerData;
use cgmath::*;
use crate::Model;
use crate::pgui::GUI;
use crate::Program;
use crate::RenderData;
use crate::game_loop::SKY;
use crate::meshing::ChunkRenderer;
use crate::game_loop::SKY_MIN_BRIGHTNESS;
use crate::Data;

pub fn handle_render(
    display: &GLDisplay,
    data: &mut Data,
    rdata: &RenderData,
    chunk_renderer: &ChunkRenderer,
    invren: &mut InventoryRenderer,
    text_rend: &TextRenderer,
    debug_text: &mut DebugText,
    lines: &LineProgram,
    prg: &Program,
    pgui: &GUI,
    state: &GameState,
    raycast_hit: Option<(WorldPos<f32>,WorldPos<f32>)>
) {
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
        gl::Enable(gl::BLEND);
    }
    
    // render chunks
    data.atlas.texture().bind();
    chunk_renderer.load_proj(&Matrix4::from(data.fov));
    chunk_renderer.load_view(&rdata.view_mat);
    chunk_renderer.load_glob_light(data.world.smooth_light_level().max(MIN_BRIGHTNESS));
    chunk_renderer.render(&data.world);

    unsafe {
        gl::Disable(gl::BLEND);
    }

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
        if let Some(item) = &pdata.inventory.data[pgui.selected_slot()] {
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
    /* guirend.start();
    guirend.set_pixels(0, 0);
    guirend.square.bind();
    vign.bind();
    let (w,h) = guirend.screen_size;
    let ps = guirend.pixel_scale;
    guirend.set_uniforms(w/ps,h/ps);
    guirend.square.draw();
    guirend.stop(); */

    // render inventory
    if let Ok(pdata) = data.ecs.query_one_mut::<&PlayerData>(data.cam) {
        // pgui.render(&mut guirend, &data.registry, &pdata.inventory, state.show_inventory(), data.input.mouse_pos(), &irenderer);
        match state {
            GameState::Inventory { ref picked_item, ref inventory } => {
                let mpos = data.input.mouse_pos(display.size_i32().1);
                invren.render(&pgui, &pdata.inventory.data, mpos, picked_item);
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
        rdata.line_box.bind();
        rdata.line_box.draw();
    }

    match &state {
        GameState::Chat { text, .. } => {
            text_rend.render(&text, -0.9, -0.9, display.size())
        },
        _ => {}
    };

    text_rend.render(&debug_text.text, -0.9, 0.9, display.size());
}