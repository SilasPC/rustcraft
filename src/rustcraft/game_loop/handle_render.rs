
use crate::static_prg::StaticProgram;
use crate::prelude::*;
use util::DebugText;
use crate::text::font::TextRenderer;
use crate::game_loop::InventoryRenderer;
use crate::lines::LineProgram;
use crate::game_loop::GameState;
use crate::PlayerData;
use cgmath::*;
use crate::Model;
use crate::pgui::GUI;
use crate::Program;
use crate::meshing::ChunkRenderer;

pub fn handle_render(
    display: &GLDisplay,
    data: &mut Data,
    rdata: &RenderData,
    chunk_renderer: &ChunkRenderer,
    invren: &mut InventoryRenderer,
    text_rend: &TextRenderer,
    debug_text: &mut DebugText,
    lines: &mut LineProgram,
    prg: &Program,
    pgui: &GUI,
    state: &GameState,
    raycast_hit: Option<WorldPos>,
    sprg: &mut StaticProgram,
    vign: &Texture,
    clouds: &Texture,
) {
    let light_factor = (data.world.smooth_light_level() * (1. - consts::SKY_MIN_BRIGHTNESS)) + consts::SKY_MIN_BRIGHTNESS;
    unsafe {
        gl::ClearColor(
            consts::SKY.0 * light_factor,
            consts::SKY.1 * light_factor,
            consts::SKY.2 * light_factor,
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
    chunk_renderer.load_proj(&rdata.proj_mat);
    chunk_renderer.load_view(&rdata.view_mat);
    let world_light = data.world.smooth_light_level().max(consts::MIN_BRIGHTNESS);
    chunk_renderer.load_glob_light(light_factor);
    chunk_renderer.render(&data.world);

    unsafe {
        gl::Disable(gl::BLEND);
    }

    // render bounding boxes
    if data.settings.debug {
        lines.enable();
        lines.load_view(
            &rdata.view_mat,
            &rdata.proj_mat,    
        );
        Position::system_draw_bounding_boxes(data, lines);
    }

    // render entities 
    sprg.enable();
    sprg.load_view(&rdata.view_mat, &rdata.proj_mat);
    sprg.load_light(1.0);
    sprg.load_uv((1.,1.), (0.,0.));
    Model::system_render(data, sprg);

    // render clouds
    sprg.enable();
    clouds.bind();
    sprg.load_view(&rdata.view_mat, &rdata.proj_mat);
    sprg.load_uv((1.,1.), (0.,0.));
    sprg.load_transform(&(Matrix4::from_translation((0.,consts::CLOUD_HEIGHT,0.).into()) * Matrix4::from_scale(clouds.size().0 * consts::CLOUD_SIZE)));
    sprg.load_light(light_factor);
    // ? use quad instead
    rdata.cube.bind();
    rdata.cube.draw();

    // render item in hand
    if let Ok(pdata) = data.ecs.query_one_mut::<&PlayerData>(data.cam) {
        if let Some(item) = &pdata.inventory.data[pgui.selected_slot()] {
            unsafe {
                gl::Disable(gl::DEPTH_TEST);
            }
            prg.enable();
            prg.load_mat4(0, &rdata.proj_mat);
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

    if let Some(hit) = raycast_hit {
        const E: f32 = 0.001;
        let hit = hit.corner_align().0.map(|v| v - E);
        lines.enable();
        let t = Matrix4::from_translation(hit) * Matrix4::from_scale(1. + 2.*E);
        lines.load_view(
            &rdata.view_mat,
            &rdata.proj_mat,    
        );
        lines.load_transform(&t);
        lines.load_color(&Vector4 {
            x: 0.2,
            y: 0.2,
            z: 0.2,
            w: 1.0,
        });
        unsafe {
            gl::LineWidth(3.0);
            gl::Enable(gl::LINE_SMOOTH | gl::DEPTH_TEST);
        }
        lines.bind_and_draw();

        match state {
            GameState::Playing { breaking: Some(breaking) } => {
                sprg.enable();
                sprg.load_transform(&t);
                rdata.break_atlas.bind();
                let uvd = rdata.break_atlas.uv_dif();
                let offset = rdata.break_atlas.get_uv((breaking.0 * 10.) as usize);
                sprg.load_uv((uvd, uvd), offset);
                rdata.cube.bind();
                rdata.cube.draw();
            },
            _ => {}
        }
    }

    // tmp vignette solution
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
    }
    sprg.enable();
    sprg.load_uv((1., 1.), (0.,0.));
    sprg.load_light(1.);
    sprg.load_view(&Matrix4::one(),&Matrix4::one());
    sprg.load_transform(&(Matrix4::from_translation((-1., -1., -1.,).into()) * Matrix4::from_scale(2.)));
    vign.bind();
    rdata.cube.bind();
    rdata.cube.draw();
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Disable(gl::BLEND);
    }
    
    // render inventory
    if let Ok(pdata) = data.ecs.query_one_mut::<&PlayerData>(data.cam) {
        // pgui.render(&mut guirend, &data.registry, &pdata.inventory, state.show_inventory(), data.input.mouse_pos(), &irenderer);
        let mpos = data.input.mouse_pos(display.size_i32().1);
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
        }
        match state {
            GameState::Inventory { ref picked_item, ref inventory } => {
                invren.render(&pgui, &pdata.inventory.data, mpos, picked_item, true);
            },
            _ => {
                invren.render(&pgui, &pdata.inventory.data, mpos, &None, false);
            }
        }
    }

    match &state {
        GameState::Chat { text, .. } => {
            text_rend.render(&text, -0.9, -0.9, display.size())
        },
        _ => {}
    };

    text_rend.render(&debug_text.text, -0.9, 0.9, display.size());
    
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

}