
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
) -> Option<(WorldPos<f32>,WorldPos<f32>)> {
    
    let mut raycast_hit = None;
    let mut on_use = None;
    let mut to_spawn = vec![];

    if let Ok((pos, phys, view, pdata)) = data.ecs.query_one_mut::<(&mut Position, &mut Physics, &View, &mut PlayerData)>(data.cam) {

        raycast_hit = raycast(pos.pos+view.offset().into(), &pos.heading(), 5., &data.registry, &data.world);
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
            raycast_hit.and_then(|(_,hit)| w.block_at(&hit)).map(|b| &b.name),
            data.delta,
            last_tick_dur
        );

        // println!("{}", data.world.area(&pos.pos).is_some());

        if let GameState::Playing{ breaking } = state {

            phys.set_edge_stop(data.input.holding_sneak());

            if data.input.holding_jump() {
                phys.try_jump();
            }

            if !data.input.holding_primary() {
                *breaking = None;
            }
            if data.input.holding_primary() {
                if let Some(hit) = raycast_hit {

                    if let Some(breaking) = breaking {
                        if breaking.1 != hit.1.as_pos_i32() {
                            *breaking = (0., hit.1.as_pos_i32());
                        }
                    } else {
                        *breaking = Some((0., hit.1.as_pos_i32()));
                    }
                    
                    let block = data.world.block_at(&hit.1).unwrap().clone();

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
                        if data.world.set_block_at(&hit.1, data.registry.get("air").as_block().unwrap()) {
                            if let Some(drop_id) = &block.drops {
                                let mut stack = ItemStack::of(data.registry.get(&drop_id).clone(), 1);
                                // * merge directly
                                /* pdata.inventory.merge(&mut Some(stack)); */
                                let phys = Physics::new();
                                let pos = Position::new(hit.1.pos_center().into(),(0.3,0.3,0.3).into());
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
                            block_updates.add_area(hit.1.0);
                            block_updates.add_single(hit.1.0);
                            data.world.to_update.push(hit.1.as_pos_i32());
                        }
                    }
                }
            } else if data.input.clicked_secondary() {

                if let Some(hit) = raycast_hit {
                    let maybe_item = &mut pdata.inventory.data[pgui.selected_slot as usize];
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

        rdata.view_mat = view.calc_view_mat(&pos);

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
