
use crate::world::VoxelData;
use crate::prelude::*;
use crate::util::AABB;

#[derive(Clone, Copy, Default)]
pub struct Light(u8);
impl Light {
    #[inline(always)]
    pub fn block(&self) -> u8 {
        self.0 & 0xf
    }
    #[inline(always)]
    pub fn set_block(&mut self, val: u8) {
        self.0 &= 0xf0;
        self.0 |= val;
    }
    #[inline(always)]
    pub fn sky(&self) -> u8 {
        self.0 >> 4
    }
    #[inline(always)]
    pub fn set_sky(&mut self, val: u8) {
        self.0 &= 0xf;
        self.0 |= val << 4;
    }
}

/// Signifies the current state of the chunk
#[derive(PartialOrd,PartialEq,Eq,Ord,Clone,Copy,Debug,serde::Serialize,serde::Deserialize)]
pub enum ChunkState {
    /// No data has been filled in
    Empty,
    /// First stage (terrain) generation done
    Filled,
    /// Second stage (detailing) done
    Detailed,
    /// Rendered and interactable
    Rendered
}

/* impl ChunkState {
    pub fn prev(self) -> Self {
        match self {
            ChunkState::Done => ChunkState::Filled,
            ChunkState::Filled => ChunkState::Empty,
            ChunkState::Empty => ChunkState::Empty,
        }
    }
} */

pub type BlocksData = Vec<Vec<Vec<Block>>>;
pub type LightData = [[[Light; 16]; 16]; 16];

pub struct Chunk {
    pub chunk_state: ChunkState,
    pub needs_refresh: bool,
    pub pos: ChunkPos,
    pub data: BlocksData,
    pub light: LightData,
    pub light_updates: VecDeque<BlockPos>,
    pub light_remove_updates: VecDeque<(BlockPos, u8)>,
    pub mesh: Option<(VAO, VAO)>,
}
/* 
#[derive(serde::Serialize, serde::Deserialize)]
enum SavedBlock {
    Shared(usize),
    Unique(BlockData),
}

impl SavedBlock {
    pub fn by(block: &Block) -> Self {
        if block.is_shared() {
            Self::Shared(block.id)
        } else {
            Self::Unique(block.as_ref().clone())
        }
    }
    pub fn to(self, reg: &Registry) -> Block {
        match self {
            Self::Shared(id) => reg[id].clone(),
            Self::Unique(data) => Block::new_not_shared(data)
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedChunk {
    chunk_state: ChunkState,
    pos: ChunkPos,
    data: Vec<Vec<Vec<SavedBlock>>>,
} */

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("pos", &self.pos)
            .field("chunk_state", &self.chunk_state)
            .finish()
    }
}

impl Chunk {

    /* pub fn compare_position(&self, rhs: &Self) -> std::cmp::Ordering {
        self.pos.cmp(&rhs.pos)
    } */

    /* pub fn load(x: i32, y: i32, z: i32, reg: &Registry) -> Option<Self> {
        let SavedChunk {
            chunk_state,
            data,
            pos,
        } = bincode::deserialize(&std::fs::read(format!("save/{:x}_{:x}_{:x}.chunk",x,y,z)).ok()?).ok()?;
        let data = data
            .into_iter()
            .map(|plane|
                plane
                    .into_iter()
                    .map(|row|
                        row
                            .into_iter()
                            .map(|b| match b {
                                SavedBlock::Unique(data) => Block::new_not_shared(data),
                                SavedBlock::Shared(id) => reg[id].clone()
                            }
                        )
                    .collect())
                .collect())
            .collect();
        let light = [[[0; 16]; 16]; 16];
        println!("Loading chunk {:?}",(x,y,z));
        Self { light_remove_updates: VecDeque::new(), light_updates: VecDeque::new(), chunk_state, data, mesh: None, pos, needs_refresh: true, light }.into()
    } */

    /* pub fn save(&self) -> Vec<u8> {
        let mut data = vec![];
        for x in 0..16 {
            let mut plane = vec![];
            for y in 0..16 {
                let mut row = vec![];
                for z in 0..16 {
                    row.push(SavedBlock::by(&self.data[x][y][z]))
                }
                plane.push(row)
            }
            data.push(plane)
        }
        let sc = SavedChunk {
            data,
            pos: self.pos,
            chunk_state: self.chunk_state
        };
        println!("Saving chunk {:?}", (self.pos.x, self.pos.y, self.pos.z));
        bincode::serialize(&sc).unwrap()
    } */

    pub fn new(pos: ChunkPos, air: Block) -> Self {
        let data = vec![vec![vec![air;16];16];16];
        let light = [[[Light::default(); 16]; 16]; 16];
        Self { light_remove_updates: VecDeque::new(), light_updates: VecDeque::new(), chunk_state: ChunkState::Empty, data, mesh: None, pos, needs_refresh: false, light }
    }

    pub fn world_pos(&self) -> Vector3<f32> {
        self.world_pos_i32().map(|x| x as f32)
    }
    pub fn world_pos_i32(&self) -> Vector3<i32> {
        self.pos.map(|x| x * 16)
    }

    pub fn world_pos_center(&self) -> Vector3<f32> {
        self.pos.map(|x| (x * 16 + 8) as f32)
    }

    /// The caller has the responsibility to ensure
    /// lighting updates are executed via `Chunk::light_update(..)`
    pub fn block_at_mut(&mut self, pos: &impl Coord) -> &mut Block {
        let (x,y,z) = pos.as_sub().into();
        &mut self.data[x][y][z]
    }
    pub fn block_at(&self, pos: &impl Coord) -> &Block {
        let (x,y,z) = pos.as_sub().into();
        &self.data[x][y][z]
    }

    pub fn light_at_mut(&mut self, pos: &impl Coord) -> &mut Light {
        let (x,y,z) = pos.as_sub().into();
        &mut self.light[x][y][z]
    }
    pub fn light_at(&self, pos: &impl Coord) -> &Light {
        let (x,y,z) = pos.as_sub().into();
        &self.light[x][y][z]
    }

    pub fn light_update(&mut self, pos: &impl Coord) {
        let sc = pos.as_sub();
        let old_light = &mut self.light[sc.x][sc.y][sc.z];
        let block = &mut self.data[sc.x][sc.y][sc.z];
        if old_light.block() != block.light {
            let remove = old_light.block() > block.light;
            let val = pos.as_block();
            if remove {
                self.light_remove_updates.push_back((val, block.light));
            } else {
                self.light_updates.push_back(val);
            }
            self.light[sc.x][sc.y][sc.z].set_block(block.light);
            self.needs_refresh = true;
        }
    }

    pub fn set_at(&mut self, pos: &impl Coord, block: &Block) -> bool {
        let sc = pos.as_sub();
        let b = &mut self.data[sc.x][sc.y][sc.z];
        if b != block {
            *b = block.clone();
            self.light_update(pos);
            self.needs_refresh = true;
            true
        } else {
            false
        }
    }

    pub fn gen_terrain(&mut self, noise: &crate::rustcraft::world::TerrainGen, reg: &ItemRegistry) {
        for x in 0..16 {
            for z in 0..16 {
                let palette = noise.palette(x,z);
                for y in 0..16 {
                    let pos = self.pos.map(|x| x as isize);

                    let ax = 16 * pos.x + x;
                    let ay = 16 * pos.y + y;
                    let az = 16 * pos.z + z;

                    if noise.is_cave(ax, ay, az) {continue}

                    let d = noise.density(ax,ay,az);
                    let da = noise.density(ax,ay+1,az);
                    
                    self.data[x as usize][y as usize][z as usize] = 
                    reg.get(
                        if d > 0.56 {
                            palette[0]
                        } else if d > 0.52 {
                            if da > 0.52 {
                                palette[1]
                            } else if ay >= 20 {
                                palette[2]
                            } else {
                                palette[1]
                            }
                        } else if ay > 20 {
                            "air"
                        } else {
                            "water"
                        }
                    ).as_block().unwrap().clone();
                    /* if db > 0.52 && db < 0.56 && d < 0.51 && !(cb > 0.57) {
                        let t = noise.get2d([x as f64 / 1.5, z as f64 / 1.5]);
                        if t > 0.52 {
                            self.data[x as usize][y as usize][z as usize] = 4;
                        }
                    } */
                }
            }
        }
        self.needs_refresh = true;
        self.chunk_state = ChunkState::Filled;
    }

    pub fn renderable(&self) -> bool {
        self.chunk_state == ChunkState::Rendered
    }

    pub fn aabb(&self) -> AABB { AABB::from_corner(&self.pos.map(|x| x as f32 * 16.), 16.) }

    /* pub fn refresh(&mut self, reg: &Registry) {
        if !self.needs_refresh {return}
        let (verts, uvs, lights) = make_mesh(&self.data, reg);
        if let Some(mesh) = &mut self.mesh {
            mesh.update_lit(&verts, &uvs, &lights);
        } else {
            self.mesh = Some(VAO::textured_lit(&verts, &uvs, &lights));
        }
        self.needs_refresh = false;
        self.chunk_state = ChunkState::Rendered;
    } */

    pub fn bind_and_draw(&self) {
        if let Some(mesh) = &self.mesh {
            mesh.0.bind();
            mesh.0.draw();
        }
    }

    pub fn bind_and_draw_second_pass(&self) {
        if let Some(mesh) = &self.mesh {
            mesh.1.bind();
            mesh.1.draw();
        }
    }

}

pub fn calc_light(pos: ChunkPos, world: &mut VoxelData) {
    let Vector3 {x, y, z} = pos.into();
    let (mut removal_queue, mut queue) = world.chunk_at_mut(pos).map(|c| (
        std::mem::take(&mut c.light_remove_updates),
        std::mem::take(&mut c.light_updates)
    )).unwrap();

    // removal loop
    while let Some((pos,old_light)) = removal_queue.pop_front() {
        macro_rules! prop {
            ($x:expr, $y:expr, $z:expr) => {
                let pos = Into::<BlockPos>::into(($x, $y, $z));
                let light = world.light_at_mut(&pos);
                let new_light = light.block();
                if new_light != 0 && new_light < old_light {
                    light.set_block(0);
                    removal_queue.push_back((($x,$y,$z).into(),new_light));
                } else if new_light >= old_light {
                    queue.push_back(($x,$y,$z).into());
                }
            };
        }
        let (x,y,z) = pos.as_tuple();
        prop!(x+1,y,z);
        prop!(x-1,y,z);
        prop!(x,y+1,z);
        prop!(x,y-1,z);
        prop!(x,y,z+1);
        prop!(x,y,z-1);
    }
    
    // propagation loop
    while let Some(pos) = queue.pop_front() {
        let pos_light = world.light_at_mut(&pos).block();
        if pos_light == 0 {continue}
        macro_rules! prop {
            ($x:expr, $y:expr, $z:expr) => {
                let prop_pos = Into::<BlockPos>::into(($x, $y, $z));
                if world.block_at(&prop_pos).unwrap().transparent {
                    let l = world.light_at_mut(&prop_pos);
                    if l.block() + 2 <= pos_light {
                        l.set_block(pos_light - 1);
                        if pos_light > 2 {
                            queue.push_back(($x, $y, $z).into());
                        }
                    }
                }
            };
        }
        let (x,y,z) = pos.as_tuple();
        prop!(x+1,y,z);
        prop!(x-1,y,z);
        prop!(x,y+1,z);
        prop!(x,y-1,z);
        prop!(x,y,z+1);
        prop!(x,y,z-1);
    }
}
