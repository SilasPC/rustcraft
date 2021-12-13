
use crate::world::VoxelData;
use crate::prelude::*;
use crate::util::AABB;
use super::lighting::*;

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

pub struct ChunkLoadLevel {
    pub spread: u8,
    pub source: u8,
}

pub type BlocksData<'cnt> = Vec<Vec<Vec<&'cnt BlockData>>>;
pub type LightData = [[[Light; 16]; 16]; 16];

pub struct Chunk<'cnt> {
    pub load_level_spread: u8,
    pub load_level_set: u8,
    pub chunk_state: ChunkState,
    pub needs_refresh: bool,
    pub pos: ChunkPos,
    pub data: BlocksData<'cnt>,
    pub light: LightData,
    pub light_updates: LightUpdates,
    pub mesh: Option<(VAO, VAO)>,
}

impl<'cnt> std::fmt::Debug for Chunk<'cnt> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("pos", &self.pos)
            .field("chunk_state", &self.chunk_state)
            .finish()
    }
}

impl<'cnt> Chunk<'cnt> {

    pub fn new(pos: ChunkPos, air: &'cnt BlockData) -> Self {
        let data = vec![vec![vec![air;16];16];16];
        let light = [[[Light::default(); 16]; 16]; 16];
        Self {
            light_updates: LightUpdates::default(),
            chunk_state: ChunkState::Empty,
            data, mesh: None,
            pos,
            needs_refresh: false,
            light,
            load_level_spread: 0,
            load_level_set: 0,
        }
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

    pub fn block_at(&self, pos: &impl Coord) -> &'cnt BlockData {&self[*pos]}

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
            self.light_updates.reg_block(&pos.as_block(), old_light.block(), block.light);
            self.light[sc.x][sc.y][sc.z].set_block(block.light);
            self.needs_refresh = true;
        }
    }

    pub fn set_at(&mut self, pos: &impl Coord, block: &'cnt BlockData) -> bool {
        let b = &mut self[*pos];
        if !std::ptr::eq(*b, block) {
            *b = block;
            self.light_update(pos);
            self.needs_refresh = true;
            true
        } else {
            false
        }
    }

    pub fn gen_terrain(&mut self, noise: &dyn TerrainGenerator, reg: &'cnt Content) {
        noise.gen_terrain(self, reg);
        self.needs_refresh = true;
        self.chunk_state = ChunkState::Filled;
    }

    pub fn renderable(&self) -> bool {
        self.chunk_state == ChunkState::Rendered
    }

    pub fn aabb(&self) -> AABB { AABB::from_corner(&self.pos.map(|x| x as f32 * 16.), 16.) }

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

impl<'cnt, C: Coord> std::ops::Index<C> for Chunk<'cnt> {
    type Output = &'cnt BlockData;
    fn index(&self, pos: C) -> &Self::Output {
        let (x,y,z) = pos.as_sub().into();
        &self.data[x][y][z]
    }
}

impl<'cnt, C: Coord> std::ops::IndexMut<C> for Chunk<'cnt> {
    fn index_mut(&mut self, pos: C) -> &mut Self::Output {
        let (x,y,z) = pos.as_sub().into();
        &mut self.data[x][y][z]
    }
}
