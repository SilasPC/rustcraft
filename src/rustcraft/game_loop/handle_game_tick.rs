
use crate::prelude::*;
use super::*;

impl<'a> GameLoop<'a> {

    pub fn handle_game_tick(&mut self) -> Option<Duration> {
        if self.last_tick.elapsed() > consts::TICK_DURATION {
            self.last_tick += consts::TICK_DURATION;
    
            if self.state.is_paused() {
                return None
            }

            let start = Instant::now();
            
            self.block_updates.update(&mut self.world);
            self.world.ticks += 1;

            component::ItemCmp::system_tick_age_items(&mut self.world);
            component::PathFinding::system_update_tick(&mut self.world);
    
            let mut rng = rand::thread_rng();
            use rand::prelude::*;
    
            for p in std::mem::take(&mut self.world.to_update) {
                if let Some(on_update) = self.world.blocks.block_at(&p).map(|b| b.behavior.clone()).flatten().map(|beh| beh.on_update).flatten() {
                    on_update(p, &mut self.world)
                }
            }
    
            let keys = self.world.blocks.chunks.iter()
                .filter(|(_,c)|
                    c.all_neighbours_loaded() &&
                    c.chunk.renderable()
                )
                .map(|(k,_)| k.clone())
                .collect::<Vec<_>>();
            for cp in keys {
                // println!("{:?}",cp);
                for _ in 0..consts::RANDOM_TICK_SPEED {
                    let random = rng.gen::<(i32,i32,i32)>();
                    let pos = cp.as_block() + Vector3::from(random).map(|x| x.abs() % 16).into();
                    assert_eq!(cp, pos.as_chunk());
                    if let Some(on_rnd_tick) = self.world.blocks.block_at(&pos).map(|b| b.behavior.clone()).flatten().map(|beh| beh.on_rnd_tick).flatten() {
                        on_rnd_tick(pos, &mut self.world)
                    }
                }
            }
            Some(start.elapsed())
        } else {
            None
        }
    }

}
