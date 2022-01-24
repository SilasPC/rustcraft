
use game::world::updates::Updates;
use crate::prelude::*;
use super::*;

impl<'cnt: 'b, 'b> GameLoop<'cnt> {

    pub fn handle_game_tick(&'b mut self) -> Option<Duration> {
        if self.last_tick.elapsed() > consts::TICK_DURATION {
            self.last_tick += consts::TICK_DURATION;
    
            if self.state.is_paused() {
                return None
            }

            let start = Instant::now();
            
            Updates::update(&mut self.world);
            self.world.ticks += 1;

            component::ItemCmp::system_tick_age_items(&mut self.world);
            component::PathFinding::system_update_tick(&mut self.world);
    
            let mut rng = rand::thread_rng();
            use rand::prelude::*;
    
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
                    if let Some(on_rnd_tick) = self.world.blocks.block_at(&pos).and_then(|b| b.behavior.as_ref()).and_then(|beh| beh.on_rnd_tick.as_ref()) {
                        on_rnd_tick(pos, &mut self.world)
                    }
                }
            }

            // tmp load around player every tick
            // self.tx.send(ClientMsg::LoadAround(self.player_pos.pos.as_chunk()));

            Some(start.elapsed())
        } else {
            None
        }
    }

}
