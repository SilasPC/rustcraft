
use crate::world::RayCastHit;
use crate::util::Drawable;
use crate::game_loop::InventoryRenderer;
use crate::pgui::GUI;
use crate::util::DebugText;
use crate::game_loop::Updates;
use crate::game_loop::Text;
use crate::game_loop::GameState;
use crate::Data;
use crate::prelude::*;

pub fn handle_interaction(
    data: &mut Data,
    rdata: &mut RenderData,
    state: &mut GameState,
    debug_text: &mut DebugText,
    block_updates: &mut Updates,
    last_tick_dur: f32,
    pgui: &GUI,
    invren: &mut InventoryRenderer,
    display: &GLDisplay,
) -> Option<RayCastHit> {
    
    let mut raycast_hit = None;
    let mut on_use = None;
    let mut to_spawn = vec![];

    if let Ok((pos, phys, view, pdata)) = data.ecs.query_one_mut::<(&mut Position, &mut Physics, &View, &mut PlayerData)>(data.cam) {

        raycast_hit = data.world.raycast(pos.pos+view.offset().into(), &pos.heading(), 5.);
        /* if ret.do_chunk_load {
            /* worker.send(WorkerJob::SaveChunk(
                data.world.take_chunk()
            )); */
            data.world.load_around(&pos.pos);
        } */

        let w = &data.world;
        let bm = &data.registry;
        debug_text.set_data(
            &pos.pos,
            raycast_hit.and_then(|hit| w.block_at(&hit.hit).map(|b| (&b.name, hit.hit.as_block()))),
            data.delta,
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
                    
                    let block = data.world.block_at(&hit).unwrap().clone();

                    let broken = {
                        let t = {
                            let breaking = breaking.as_mut().unwrap();
                            breaking.0 += data.delta; // TODO mult by hardness factor
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
                        if data.world.set_block_at(&hit, data.registry.get("air").as_block().unwrap()) {
                            if let Some(drop_id) = &block.drops {
                                let mut stack = ItemStack::of(data.registry.get(&drop_id).clone(), 1);
                                let phys = Physics::new();
                                let pos = Position::new(hit.center_align(),(0.3,0.3,0.3).into());
                                let aabb = pos.get_aabb();
                                let model = box util::RenderedItem {
                                    vao: rdata.item_cubes.clone(),
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
                    let maybe_item = &mut pdata.inventory.data[pgui.selected_slot as usize];
                    let mut success = false;
                    if let Some(ref mut block) = maybe_item.as_mut().and_then(|item| item.item.as_block()) {
                        if data.world.set_block_at(&hit_prev, &block) {
                            success = true;
                            block_updates.add_area(hit_prev);
                            block_updates.add_single(hit_prev);
                            data.world.to_update.push(hit_prev);
                        }
                    } else {
                        let b = data.world.block_at(&hit).unwrap();
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
            phys.apply_force_continuous(data.delta, &force);
        }

        rdata.view_mat = view.calc_view_mat(&pos);
        
    }

    for (cmps, aabb) in to_spawn {
        let ent = data.ecs.spawn(cmps);
        data.ent_tree.insert(ent, ent, &aabb);
    }

    if let Some((p,f)) = on_use {
        f(p.as_block(), data);
    }

    // interact with inventory
    if let Ok(pdata) = data.ecs.query_one_mut::<&mut PlayerData>(data.cam) {
        match state {
            GameState::Inventory { ref mut picked_item, ref inventory } => {
                if data.input.clicked_primary() {
                    let mpos = data.input.mouse_pos(display.size_i32().1);
                    // compile_warning!(need corner anchor for determining hovered slot);
                    if let Some(slot) = invren.corner_cursor(&pgui.inventory, mpos) {
                        pdata.inventory.transfer(slot as u32, picked_item, &data.registry, &data.crafting);
                    }
                }
            },
            _ => {}
        }
    }

    raycast_hit

}
