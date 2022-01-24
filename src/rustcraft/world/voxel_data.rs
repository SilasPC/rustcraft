
use super::*;
use crate::prelude::*;
use game::chunk::*;

/*// Automatically handles update logic needed with `VoxelData::block_at_mut_unguarded` on drop
pub struct BlockMutGuard<'w> {
    pos: BlockPos,
    voxels: &'w mut VoxelData,
}

impl<'w> BlockMutGuard<'w> {
    pub fn get_mut(&mut self) -> Option<&mut Block> {
        let pos = self.pos;
        self.voxels.chunk_at_mut(pos.as_chunk()).map(|c| c.block_at_mut(&pos))
    }
}

impl<'w> Drop for BlockMutGuard<'w> {
    fn drop(&mut self) {
        self.voxels.register_mesh_change(self.pos);
        self.voxels.chunk_at_mut(self.pos.as_chunk()).unwrap().light_update(&self.pos);
    }
}*/

impl<'cnt: 'b, 'b> VoxelData<'cnt> {

    /* pub fn block_at_mut(&mut self, pos: &impl Coord) -> BlockMutGuard<'_> {
        BlockMutGuard {
            voxels: self,
            pos: pos.as_block()
        }
    } */

    pub fn block_at(&'b self, pos: &impl Coord) -> Option<&'cnt BlockData> {
        self.chunk_at(pos.as_chunk()).map(|c| c.block_at(pos))
    }
    pub fn block_at_any_state(&'b self, pos: &impl Coord) -> Option<&'cnt BlockData> {
        self.chunks.get(&pos.as_chunk()).map(|c| c.chunk.block_at(pos))
    }
    pub fn set_block_at(&'b mut self, pos: &impl Coord, block: &'cnt BlockData) -> bool {
        let success = self.chunk_at_mut(pos.as_chunk())
            .map(|c| c.set_at(pos, block))
            .unwrap_or(false);
        if success {
            self.register_mesh_change(pos.as_block());
        }
        success
    }
    pub fn set_block_at_any_state(&mut self, pos: &impl Coord, block: &'cnt BlockData) -> bool {
        let success = self.chunks.get_mut(&pos.as_chunk())
            .map(|c| c.chunk.set_at(pos, block))
            .unwrap_or(false);
        if success {
            self.changed_chunks.insert(pos.as_chunk());
        }
        success
    }
    pub fn replace_at(&'b mut self, pos: &impl Coord, block: &'cnt BlockData) -> bool {
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
    pub fn replace_at_any_state(&mut self, pos: &impl Coord, block: &'cnt BlockData) -> bool {
        if let Some(c) = self.chunks.get_mut(&pos.as_chunk()) {
            if c.chunk.block_at(pos).replacable {
                c.chunk.set_at(pos, block)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn register_mesh_change(&mut self, pos: BlockPos) {
        for f in Face::iter_all() {
            if let Some(b) = self.block_at(&pos.shifted(f)) {
                if b.transparent {
                    self.changed_chunks.insert(pos.as_chunk());
                    for f in Face::iter_all() {
                        self.changed_chunks.insert(
                            pos.shifted(f).as_chunk()
                        );
                    }
                    break
                }
            }
        }
    }

    pub fn light_at(&self, pos: &impl Coord) -> &Light {
        self.chunk_at(pos.as_chunk()).unwrap().light_at(pos)
    }
    pub fn light_at_mut(&mut self, pos: &impl Coord) -> &mut Light {
        self.chunk_at_mut(pos.as_chunk()).unwrap().light_at_mut(pos)
    }

    pub fn chunk_at(&self, pos: ChunkPos) -> Option<&Chunk<'cnt>> {
        self.chunks.get(&pos).filter(|c| c.chunk.chunk_state >= ChunkState::Detailed).map(|cd| cd.chunk.as_ref())
    }
    pub fn chunk_at_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk<'cnt>> {
        self.chunks.get_mut(&pos).filter(|c| c.chunk.chunk_state >= ChunkState::Detailed).map(|cd| cd.chunk.as_mut())
    }

    pub fn refresh(&'b mut self, reg: &ItemRegistry, atlas: &TextureAtlas) {
        let mut cc = std::mem::take(&mut self.changed_chunks);
        let mut meshed = HashSet::new();
        cc.retain(|x| self.chunk_at(*x).map(Chunk::renderable).unwrap_or(false));
        // println!("refresh {:?}",cc);
        for cp in &cc {
            calc_light(*cp, self);
        }
        // println!("{}",cc.len());
        for cp in cc {
            /* for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 { */
                        let p = cp; /* Vector3 {
                            x: x + cp.x,
                            y: y + cp.y,
                            z: z + cp.z,
                        }; */
                        // ! make_mesh is slow, old version faster
                        // ! need to make another solution here
                        // ! need to make a hybrid version as well
                        if !self.chunks.get(&p.into()).unwrap().chunk.renderable() || meshed.contains(&p) {continue}
                        // println!("{:?}", p);
                        let (m1,m2) = (meshing::make_mesh(p.into(), self, reg, atlas));
                        let c = &mut self.chunks.get_mut(&p.into()).unwrap().chunk;
                        {
                            let m = c.mesh.as_mut().unwrap();
                            m.0.update_lit(&m1.0, &m1.1, &m1.2);
                            m.1.update_lit(&m2.0, &m2.1, &m2.2);
                        }
                        c.needs_refresh = false;
                        c.chunk_state = ChunkState::Rendered;
                        meshed.insert(p);
                    /* }
                }
            } */
        }
    }

}