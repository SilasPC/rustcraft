
use crate::prelude::*;
use inventory::*;
use crate::static_prg::StaticProgram;
use util::DebugText;
use crate::text::font::TextRenderer;
use crate::game_loop::InventoryRenderer;
use crate::lines::LineProgram;
use crate::game_loop::GameState;
use crate::PlayerData;
use crate::meshing::ChunkRenderer;

impl<'b: 'cnt, 'cnt> GameLoop<'cnt> {
    pub fn handle_render(&mut self, raycast_hit: Option<RayCastHit>) {
        let light_factor = (self.world.smooth_light_level() * (1. - consts::SKY_MIN_BRIGHTNESS)) + consts::SKY_MIN_BRIGHTNESS;
        let sky_col = Vector3::from(consts::SKY) * light_factor;
        unsafe {
            gl::ClearColor(
                sky_col.x,
                sky_col.y,
                sky_col.z,
                1.
            );
        }

        self.chunk_renderer.program.enable();
        self.chunk_renderer.load_fog_color(&sky_col);
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
        self.idata.atlas.texture().bind();
        self.chunk_renderer.load_proj(&self.rdata.proj_mat);
        self.chunk_renderer.load_view(&self.rdata.view_mat);
        let world_light = self.world.smooth_light_level().max(consts::MIN_BRIGHTNESS);
        self.chunk_renderer.load_glob_light(light_factor);
        self.chunk_renderer.render(&mut self.world);

        unsafe {
            gl::Disable(gl::BLEND);
        }

        // render bounding boxes
        if self.data.settings.debug {
            self.lines.enable();
            self.lines.load_view(
                &self.rdata.view_mat,
                &self.rdata.proj_mat,    
            );
        Model::system_render(&mut self.world, &mut self.sprg);
        Position::system_draw_bounding_boxes(&mut self.world, &mut self.lines);
        }

        // render entities 
        self.sprg.enable();
        self.sprg.load_view(&self.rdata.view_mat, &self.rdata.proj_mat);
        self.sprg.load_light(1.0);
        self.sprg.load_uv((1.,1.), (0.,0.));
        Model::system_render(&mut self.world, &mut self.sprg);

        // render clouds
        self.sprg.enable();
        self.idata.clouds.bind();
        self.sprg.load_view(&self.rdata.view_mat, &self.rdata.proj_mat);
        self.sprg.load_uv((1.,1.), (0.,0.));
        self.sprg.load_transform(&(Matrix4::from_translation((self.player_pos.pos.x,consts::CLOUD_HEIGHT,self.player_pos.pos.z).into()) * Matrix4::from_scale(self.idata.clouds.size().0 * consts::CLOUD_SIZE)));
        self.sprg.load_light(light_factor);
        // ? use quad instead
        self.idata.cube.bind();
        self.idata.cube.draw();


        // render item in hand
        /* if let Ok(pdata) = self.world.entities.ecs.query_one_mut::<&PlayerData>(self.world.entities.player) {
            if let Some(item) = &pdata.inventory.data[self.pgui.selected_slot()] {
                unsafe {
                    gl::Disable(gl::DEPTH_TEST);
                }
                self.prg.enable();
                self.prg.load_mat4(0, &self.rdata.proj_mat);
                self.prg.load_mat4(1, &Matrix4::one());
                let pos = (0.4, -1.5, -2.).into();
                let rot = Euler::new(Deg(0.),Deg(-60.),Deg(0.));
                self.prg.load_mat4(2, &(Matrix4::from_translation(pos) * Matrix4::from(rot)));
                self.idata.atlas.bind();
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
        } */

        if let Some(hit) = raycast_hit {
            const E: f32 = 0.001;
            let hit = hit.hit.align_corner().0.map(|v| v - E);
            self.lines.enable();
            let t = Matrix4::from_translation(hit) * Matrix4::from_scale(1. + 2.*E);
            self.lines.load_view(
                &self.rdata.view_mat,
                &self.rdata.proj_mat,    
            );
            self.lines.load_transform(&t);
            self.lines.load_color(&Vector4 {
                x: 0.2,
                y: 0.2,
                z: 0.2,
                w: 1.0,
            });
            unsafe {
                gl::LineWidth(3.0);
                gl::Enable(gl::LINE_SMOOTH | gl::DEPTH_TEST);
            }
            self.lines.bind_and_draw();

            match self.state {
                GameState::Playing { breaking: Some(breaking) } => {
                    self.sprg.enable();
                    self.sprg.load_transform(&t);
                    self.idata.break_atlas.bind();
                    let uvd = self.idata.break_atlas.uv_dif();
                    let offset = self.idata.break_atlas.get_uv((breaking.0 * 10.) as usize);
                    self.sprg.load_uv((uvd, uvd), offset);
                    self.idata.cube.bind();
                    self.idata.cube.draw();
                },
                _ => {}
            }
        }

        // tmp vignette solution
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
        }
        self.sprg.enable();
        self.sprg.load_uv((1., 1.), (0.,0.));
        self.sprg.load_light(1.);
        self.sprg.load_view(&Matrix4::one(),&Matrix4::one());
        self.sprg.load_transform(&(Matrix4::from_translation((-1., -1., -1.,).into()) * Matrix4::from_scale(2.)));
        self.idata.vign.bind();
        self.idata.cube.bind();
        self.idata.cube.draw();
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
        }
        
        // render inventory
        // ! factor out
        let mpos = self.data.input.mouse_pos(self.data.display.size_i32().1);
        self.invren.gui.start();
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
        }
        if let Some(d) = self.pgui.hotbar.borrow_data(&mut self.world) {
            self.invren.render_bottom(&self.pgui, &d, mpos);
        }
        match self.state {
            GameState::Inventory { ref picked_item, .. } => {
                if let Some(d) = self.pgui.inventory.borrow_data(&mut self.world) {
                    { 
                        self.invren.render_centered(&self.pgui.inventory, &d, mpos);
                        if let Some(item) = picked_item.as_ref().map(|s| &s.item) {
                            self.invren.render_floating_item(item, mpos);
                        }
                        
                    }
                }
                if let Some(picked_item) = picked_item {
                    self.invren.render_floating_item(&picked_item.item, mpos);
                }
            },
            _ => {}
        }

        /* unsafe {
            gl::Disable(gl::BLEND);
        } */

        /* if let Ok(pdata) = self.world.entities.ecs.query_one_mut::<&PlayerData>(self.world.entities.player) {
            // pgui.render(&mut guirend, &data.registry, &pdata.inventory, state.show_inventory(), data.input.mouse_pos(), &irenderer);
            let mpos = self.data.input.mouse_pos(self.data.display.size_i32().1);
            unsafe {
                gl::Disable(gl::DEPTH_TEST);
            }
            match self.state {
                GameState::Inventory { ref picked_item, ref inventory } => {
                    self.invren.render(&self.pgui, &pdata.inventory, mpos, picked_item, true);
                },
                _ => {
                    self.invren.render(&self.pgui, &pdata.inventory, mpos, &None, false);
                }
            }
        } */

        match &self.state {
            GameState::Chat { text, .. } => {
                self.text_rend.render(&text, -0.9, -0.9, self.data.display.size())
            },
            _ => {}
        };

        self.text_rend.render(&self.debug_text.text, -0.9, 0.9, self.data.display.size());
        
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
    }
}
