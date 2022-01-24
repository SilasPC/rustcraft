
use crate::updates::Updates;
use crate::prelude::*;

mod msgs;
pub use msgs::*;

pub struct ServerLoop<'cnt> {
    pub content: &'cnt Content,
    pub world: WorldData<'cnt>,
    pub last_tick: Instant,
    pub last_tick_dur: f32,
    pub rx: Receiver<ClientMsg>,
    pub tx: Sender<ServerMsg>,
}

impl<'cnt: 'b, 'b> ServerLoop<'cnt> {

    pub fn new(conn: (Sender<ServerMsg>, Receiver<ClientMsg>), content: &'cnt Content) -> Self {
            
        let world = WorldData::new(consts::DEBUG_SEED, content.blocks.get("air").unwrap());
        let (tx, rx) = conn;
        Self {
            tx,
            rx,
            content,
            world,
            last_tick: Instant::now(),
            last_tick_dur: consts::TICK_DURATION.as_secs_f32() * 1000.
        }

    }

    pub fn run_and_sleep(&'b mut self) -> bool {
        let ret = self.run();
        if ret {
            let sleep: Duration = (self.last_tick + consts::TICK_DURATION) - Instant::now();
            if sleep.as_nanos() > 0 {
                std::thread::sleep(sleep)
            }
        }
        ret
    }

    pub fn run(&'b mut self) -> bool {

        // GAME TICK
        if let Some(dur) = self.handle_game_tick() {
            self.last_tick_dur = dur.as_secs_f32() * 1000.;
        }

        // ! START SYSTEMS
        WanderingAI::system_update(&mut self.world, self.last_tick_dur);
        Physics::system_update(&mut self.world, self.last_tick_dur);
        FallingBlock::system_collide_land(&mut self.world);
        // ! STOP SYSTEMS

        // TODO this is too slow
        //self.world.load(&self.content, 5); // ! load without meshing

        self.handle_msgs();

        true

    }

    pub fn handle_game_tick(&'b mut self) -> Option<Duration> {
        if self.last_tick.elapsed() > consts::TICK_DURATION {
            self.last_tick += consts::TICK_DURATION;

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
            Some(start.elapsed())
        } else {
            None
        }
    }

    pub fn handle_msgs(&'b mut self) {

        for msg in self.rx.try_iter() {
            match msg {
                ClientMsg::LoadAround(p) => {
                    self.world.load_around(&p);
                }
            }
        }

    }

}
