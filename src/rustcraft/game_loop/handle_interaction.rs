

use crate::world::RayCastHit;
use crate::game_loop::handle_input::Return;
use crate::util::Drawable;
use crate::game_loop::InventoryRenderer;
use crate::util::DebugText;
use crate::game_loop::Text;
use crate::game_loop::GameState;
use crate::prelude::*;
use world::updates::Updates;

impl<'a> GameLoop<'a> {
    pub fn handle_interaction(&mut self, ret: Return) -> Option<RayCastHit> {

        let mut raycast_hit = None;
        let mut on_use = None;
        let mut to_spawn = vec![];

        let mut opos = None;

        if let Ok((pos, phys, view, pdata)) = self.world.entities.ecs.query_one_mut::<(&mut Position, &mut Physics, &View, &mut PlayerData)>(self.world.entities.player) {

            raycast_hit = self.world.blocks.raycast(pos.pos+view.offset().into(), &pos.heading(), 5.);
            opos = Some(pos.pos);

            let b = &self.world.blocks;
            self.debug_text.set_data(
                &pos.pos,
                raycast_hit.and_then(|hit| b.block_at(&hit.hit).map(|b| (&b.name, hit.hit.as_block()))),
                self.rdata.delta,
                self.last_tick_dur
            );

            if let GameState::Playing{ breaking } = &mut self.state {

                phys.set_edge_stop(self.data.input.holding_sneak());

                if self.data.input.holding_jump() {
                    phys.try_jump();
                }

                if !self.data.input.holding_primary() || raycast_hit.is_none() {
                    *breaking = None;
                }
                if self.data.input.holding_primary() {
                    if let Some(RayCastHit {hit, prev: hit_prev, ..}) = raycast_hit {

                        if let Some(breaking) = breaking {
                            if breaking.1 != hit.as_block() {
                                *breaking = (0., hit.as_block());
                            }
                        } else {
                            *breaking = Some((0., hit.as_block()));
                        }
                        
                        let block = self.world.blocks.block_at(&hit).unwrap().clone();

                        let broken = {
                            let t = {
                                let breaking = breaking.as_mut().unwrap();
                                breaking.0 += self.rdata.delta; // TODO mult by hardness factor
                                breaking.0
                            };
                            if t >= 1.0 {
                                *breaking = None;
                                true
                            } else {
                                false
                            }
                        };
                        
                        if broken {
                            if self.world.blocks.set_block_at(&hit, self.idata.content.items.get("air").as_block().unwrap()) {
                                if let Some(drop_id) = &block.drops {
                                    let mut stack = ItemStack::of(self.idata.content.items.get(&drop_id).clone(), 1);
                                    let phys = Physics::new();
                                    let pos = Position::new(hit.align_center(),(0.3,0.3,0.3).into());
                                    let aabb = pos.get_aabb();
                                    let model = box util::RenderedItem {
                                        vao: self.idata.item_cubes.clone(),
                                        offset: self.invren.iren.offsets[drop_id.as_ref()]
                                    };
                                    let cmps = (
                                        pos,
                                        phys,
                                        ItemCmp::from(stack),
                                        Model::from(model as Box<dyn Drawable>),
                                    );
                                    to_spawn.push((cmps,aabb));
                                }
                                self.world.block_updates.add_area(hit.as_block());
                            }
                        }
                    }
                } else if self.data.input.clicked_secondary() {

                    if let Some(RayCastHit {hit, prev: hit_prev, ..}) = raycast_hit {
                        let hit_prev = hit_prev.as_block();
                        let maybe_item = &mut pdata.inventory.data[self.pgui.selected_slot as usize];
                        let mut success = false;
                        if let Some(ref mut block) = maybe_item.as_mut().and_then(|item| item.item.as_block()) {
                            if self.world.blocks.set_block_at(&hit_prev, &block) {
                                success = true;
                                self.world.block_updates.add_area(hit_prev);
                                self.world.block_updates.add_single(hit_prev);
                                self.world.to_update.push(hit_prev);
                            }
                        } else {
                            let b = self.world.blocks.block_at(&hit).unwrap();
                            on_use = b.behavior.as_ref().and_then(|b| b.on_use.map(|f| (hit,f)));
                        }
                        if success {
                            ItemStack::deduct(maybe_item, 1);
                        }
                    }

                }

                pos.rotate(
                    self.data.input.mouse_y() as f32 * self.data.settings.mouse_sensitivity,
                    self.data.input.mouse_x() as f32 * self.data.settings.mouse_sensitivity
                );

                let force = self.data.input.compute_movement_vector(pos.yaw()) * 40.;
                phys.apply_force_continuous(self.rdata.delta, &force);
            }

            self.rdata.view_mat = view.calc_view_mat(&pos);
            if self.data.settings.third_person {
                let heading = pos.heading();
                let d = consts::THIRD_PERSON_DISTANCE;
                let trans = if let Some(RayCastHit { dist_prev, .. }) = self.world.blocks.raycast(pos.pos, &-heading, d) {
                    Matrix4::from_translation(dist_prev * heading)
                } else {
                    Matrix4::from_translation(heading * d)
                };
                self.rdata.view_mat = self.rdata.view_mat * trans;
            }
            
        }

        for (cmps, aabb) in to_spawn {
            let ent = self.world.entities.ecs.spawn(cmps);
            self.world.entities.tree.insert(ent, ent, &aabb);
        }

        if let Some((p,f)) = on_use {
            f(p.as_block(), &mut self.world);
        }

        if let Some(pos) = opos {
            if ret.do_chunk_load {
                /* worker.send(WorkerJob::SaveChunk(
                    self.data.self.world.take_chunk()
                )); */
                self.world.load_around(&pos);
            }
        }

        // interact with inventory
        if self.data.input.clicked_primary() {
            use inventory::*;
            match self.state {
                GameState::Inventory { ref mut picked_item, .. } => {
                    let mpos = self.data.input.mouse_pos(self.data.display.size_i32().1);
                    // compile_warning!(need corner anchor for determining hovered slot);
                    // ! hot fix
                    if let Some(hovered_slot) = self.invren.corner_cursor(&self.pgui.inventory, mpos) {
                        if let Some(inv_data) = self.pgui.inventory.borrow_data(&mut self.world) {
                            ItemStack::transfer_or_swap(picked_item, inv_data.slot_mut(hovered_slot));
                        }
                    }
                },
                _ => {}
            }
        }

        raycast_hit

    }
}