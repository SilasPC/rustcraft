
use crate::prelude::*;
use crate::game_loop::GameState;
use crate::game_loop::Updates;

pub fn handle_game_tick(
    data: &mut data::Data,
    world: &mut WorldData,
    last_tick: &mut Instant,
    block_updates: &mut Updates,
    state: &GameState,
    
) -> Option<Duration> {
    if last_tick.elapsed() > consts::TICK_DURATION {
        *last_tick += consts::TICK_DURATION;

        let start = Instant::now();
        if !state.is_paused() {
            block_updates.update(world);
            crate::rustcraft::component::ItemCmp::system_tick_age_items(world);
            world.ticks += 1;
        }

        let mut rng = rand::thread_rng();
        use rand::prelude::*;

        for p in std::mem::take(&mut world.to_update) {
            if let Some(on_update) = world.blocks.block_at(&p).map(|b| b.behavior.clone()).flatten().map(|beh| beh.on_update).flatten() {
                on_update(p, world)
            }
        }

        let keys = world.blocks.chunks.iter().filter(|(_,c)| c.renderable()).map(|(k,_)| k.clone()).collect::<Vec<_>>();
        for cp in keys {
            // println!("{:?}",cp);
            for _ in 0..consts::RANDOM_TICK_SPEED {
                let random = rng.gen::<(i32,i32,i32)>();
                let pos = cp.as_block() + Vector3::from(random).map(|x| x.abs() % 16).into();
                assert_eq!(cp, pos.as_chunk());
                if let Some(on_rnd_tick) = world.blocks.block_at(&pos).map(|b| b.behavior.clone()).flatten().map(|beh| beh.on_rnd_tick).flatten() {
                    on_rnd_tick(pos, world)
                }
            }
        }
        Some(start.elapsed())
    } else {
        None
    }
}