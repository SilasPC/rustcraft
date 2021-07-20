
use super::*;
use crate::prelude::*;

impl VoxelData {
    
    pub fn block_at(&self, pos: &impl Coord) -> Option<&Block> {
        self.chunk_at(pos.as_chunk()).map(|c| c.block_at(pos))
    }
    pub fn block_at_any_state(&self, pos: &impl Coord) -> Option<&Block> {
        self.chunks.get(&pos.as_chunk()).map(|c| c.block_at(pos))
    }
    pub fn set_block_at(&mut self, pos: &impl Coord, block: &Block) -> bool {
        let success = self.chunk_at_mut(pos.as_chunk())
            .map(|c| c.set_at(pos, block))
            .unwrap_or(false);
        if success {
            // ! self.changed_chunks.insert(pos.as_chunk());
        }
        success
    }
    pub fn set_block_at_any_state(&mut self, pos: &impl Coord, block: &Block) -> bool {
        let success = self.chunks.get_mut(&pos.as_chunk())
            .map(|c| c.set_at(pos, block))
            .unwrap_or(false);
        if success {
            // ! self.changed_chunks.insert(pos.as_chunk());
        }
        success
    }
    pub fn replace_at(&mut self, pos: &impl Coord, block: &Block) -> bool {
        if let Some(c) = self.chunk_at_mut(pos.as_chunk()) {
            if c.block_at(pos).replacable {
                c.set_at(pos, block)
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn replace_at_any_state(&mut self, pos: &impl Coord, block: &Block) -> bool {
        if let Some(c) = self.chunks.get_mut(&pos.as_chunk()) {
            if c.block_at(pos).replacable {
                c.set_at(pos, block)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn light_at(&self, pos: &impl Coord) -> &Light {
        self.chunk_at(pos.as_chunk()).unwrap().light_at(pos)
    }
    pub fn light_at_mut(&mut self, pos: &impl Coord) -> &mut Light {
        self.chunk_at_mut(pos.as_chunk()).unwrap().light_at_mut(pos)
    }

    pub fn chunk_at(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos).filter(|c| c.chunk_state >= ChunkState::Detailed).map(Box::as_ref)
    }
    pub fn chunk_at_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos).filter(|c| c.chunk_state >= ChunkState::Detailed).map(Box::as_mut)
    }

}