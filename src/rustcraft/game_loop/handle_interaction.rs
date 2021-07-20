

use crate::world::RayCastHit;
use crate::util::Drawable;
use crate::game_loop::InventoryRenderer;
use crate::pgui::GUI;
use crate::util::DebugText;
use crate::game_loop::Updates;
use crate::game_loop::Text;
use crate::game_loop::GameState;
use crate::prelude::*;

pub fn handle_interaction(
    data: &mut data::Data,
    rdata: &mut data::RData,
    idata: &data::IData,
    world: &mut WorldData,
    state: &mut GameState,
    debug_text: &mut DebugText,
    block_updates: &mut Updates,
    last_tick_dur: f32,
    pgui: &GUI,
    invren: &mut InventoryRenderer,
) -> Option<RayCastHit> {
    
    let mut raycast_hit = None;
    let mut on_use = None;
    let mut to_spawn = vec![];

    if let Ok((pos, phys, view, pdata)) = world.entities.ecs.query_one_mut::<(&mut Position, &mut Physics, &View, &mut PlayerData)>(world.entities.player) {

        raycast_hit = world.blocks.raycast(pos.pos+view.offset().into(), &pos.heading(), 5.);
        /* if ret.do_chunk_load {
            /* worker.send(WorkerJob::SaveChunk(
                data.world.take_chunk()
            )); */
            data.world.load_around(&pos.pos);
        } */

        let b = &world.blocks;
        debug_text.set_data(
            &pos.pos,
            raycast_hit.and_then(|hit| b.block_at(&hit.hit).map(|b| (&b.name, hit.hit.as_block()))),
            rdata.delta,
            last_tick_dur
        );

        if let GameState::Playing{ breaking } = state {

            phys.set_edge_stop(data.input.holding_sneak());

            if data.input.holding_jump() {
                phys.try_jump();
            }

            if !data.input.holding_primary() || raycast_hit.is_none() {
                *breaking = None;
            }
            if data.input.holding_primary() {
                if let Some(RayCastHit {hit, prev: hit_prev}) = raycast_hit {

                    if let Some(breaking) = breaking {
                        if breaking.1 != hit.as_block() {
                            *breaking = (0., hit.as_block());
                        }
                    } else {
                        *breaking = Some((0., hit.as_block()));
                    }
                    
                    let block = world.blocks.block_at(&hit).unwrap().clone();

                    let broken = {
                        let t = {
                            let breaking = breaking.as_mut().unwrap();
                            breaking.0 += rdata.delta; // TODO mult by hardness factor
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
                        if world.blocks.set_block_at(&hit, idata.registry.get("air").as_block().unwrap()) {
                            if let Some(drop_id) = &block.drops {
                                let mut stack = ItemStack::of(idata.registry.get(&drop_id).clone(), 1);
                                let phys = Physics::new();
                                let pos = Position::new(hit.center_align(),(0.3,0.3,0.3).into());
                                let aabb = pos.get_aabb();
                                let model = box util::RenderedItem {
                                    vao: idata.item_cubes.clone(),
                                    offset: invren.iren.offsets[drop_id.as_ref()]
                                };
                                let cmps = (
                                    pos,
                                    phys,
                                    ItemCmp::from(stack),
                                    Model::from(model as Box<dyn Drawable>),
                                );
                                to_spawn.push((cmps,aabb));
                            }
                            block_updates.add_area(hit.as_block());
                            // data.world.to_update.push(hit.as_block());
                        }
                    }
                }
            } else if data.input.clicked_secondary() {

                if let Some(RayCastHit {hit, prev: hit_prev}) = raycast_hit {
                    let hit_prev = hit_prev.as_block();
                    let maybe_item = &mut pdata.inventory.data[pgui.selected_slot as usize];
                    let mut success = false;
                    if let Some(ref mut block) = maybe_item.as_mut().and_then(|item| item.item.as_block()) {
                        if world.blocks.set_block_at(&hit_prev, &block) {
                            success = true;
                            block_updates.add_area(hit_prev);
                            block_updates.add_single(hit_prev);
                            world.to_update.push(hit_prev);
                        }
                    } else {
                        let b = world.blocks.block_at(&hit).unwrap();
                        on_use = b.behavior.as_ref().and_then(|b| b.on_use.map(|f| (hit,f)));
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
            phys.apply_force_continuous(rdata.delta, &force);
        }

        rdata.view_mat = view.calc_view_mat(&pos);
        /* if data.settings.third_person {
            if let Some(RayCastHit { prev, .. }) = world.blocks.raycast(pos.pos, &-pos.heading(), 5.) {
                rdata.view_mat = Matrix4::from_translation(prev.0) * rdata.view_mat; 
            }
        } */
        
    }

    for (cmps, aabb) in to_spawn {
        let ent = world.entities.ecs.spawn(cmps);
        world.entities.tree.insert(ent, ent, &aabb);
    }

    if let Some((p,f)) = on_use {
        f(p.as_block(), world);
    }

    // interact with inventory
    if let Ok(pdata) = world.entities.ecs.query_one_mut::<&mut PlayerData>(world.entities.player) {
        match state {
            GameState::Inventory { ref mut picked_item, ref inventory } => {
                if data.input.clicked_primary() {
                    let mpos = data.input.mouse_pos(data.display.size_i32().1);
                    // compile_warning!(need corner anchor for determining hovered slot);
                    if let Some(slot) = invren.corner_cursor(&pgui.inventory, mpos) {
                        pdata.inventory.transfer(slot as u32, picked_item, &idata.registry, &idata.crafting);
                    }
                }
            },
            _ => {}
        }
    }

    raycast_hit

}
