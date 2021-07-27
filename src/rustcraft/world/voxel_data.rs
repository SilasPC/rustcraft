
use super::*;
use crate::prelude::*;

impl VoxelData {

    /// The caller has responsibility to register changes with
    /// the method `VoxelData::register_change(..)`, as well as
    /// registering changes to the chunk directly.
    pub fn block_at_mut(&mut self, pos: &impl Coord) -> Option<&mut Block> {
        self.chunk_at_mut(pos.as_chunk()).map(|c| c.block_at_mut(pos))
    }
    pub fn block_at(&self, pos: &impl Coord) -> Option<&Block> {
        self.chunk_at(pos.as_chunk()).map(|c| c.block_at(pos))
    }
    pub fn block_at_any_state(&self, pos: &impl Coord) -> Option<&Block> {
        self.chunks.get(&pos.as_chunk()).map(|c| c.chunk.block_at(pos))
    }
    pub fn set_block_at(&mut self, pos: &impl Coord, block: &Block) -> bool {
        let success = self.chunk_at_mut(pos.as_chunk())
            .map(|c| c.set_at(pos, block))
            .unwrap_or(false);
        if success {
            self.register_change(&pos.as_chunk());
        }
        success
    }
    pub fn set_block_at_any_state(&mut self, pos: &impl Coord, block: &Block) -> bool {
        let success = self.chunks.get_mut(&pos.as_chunk())
            .map(|c| c.chunk.set_at(pos, block))
            .unwrap_or(false);
        if success {
            self.changed_chunks.insert(pos.as_chunk());
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
            if c.chunk.block_at(pos).replacable {
                c.chunk.set_at(pos, block)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn register_change(&mut self, pos: &impl Coord) {
        self.changed_chunks.insert(pos.as_chunk());
    }

    pub fn light_at(&self, pos: &impl Coord) -> &Light {
        self.chunk_at(pos.as_chunk()).unwrap().light_at(pos)
    }
    pub fn light_at_mut(&mut self, pos: &impl Coord) -> &mut Light {
        self.chunk_at_mut(pos.as_chunk()).unwrap().light_at_mut(pos)
    }

    pub fn chunk_at(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos).filter(|c| c.chunk.chunk_state >= ChunkState::Detailed).map(|cd| cd.chunk.as_ref())
    }
    pub fn chunk_at_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos).filter(|c| c.chunk.chunk_state >= ChunkState::Detailed).map(|cd| cd.chunk.as_mut())
    }

    pub fn refresh(&mut self, reg: &Registry) {
        let mut cc = std::mem::take(&mut self.changed_chunks);
        let mut meshed = HashSet::new();
        cc.retain(|x| self.chunk_at(*x).map(Chunk::renderable).unwrap_or(false));
        // println!("refresh {:?}",cc);
        for cp in &cc {
            calc_light(*cp, self);
        }
        // println!("{}",cc.len());
        for cp in cc {
            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        let p = Vector3 {
                            x: x + cp.x,
                            y: y + cp.y,
                            z: z + cp.z,
                        };
                        // ! make_mesh is slow, old version faster
                        // ! need to make another solution here
                        // ! need to make a hybrid version as well
                        if !self.chunks.get(&p.into()).unwrap().chunk.renderable() || meshed.contains(&p) {continue}
                        // println!("{:?}", p);
                        let (m1,m2) = (meshing::make_mesh(p.into(), self, reg));
                        let c = &mut self.chunks.get_mut(&p.into()).unwrap().chunk;
                        {
                            let m = c.mesh.as_mut().unwrap();
                            m.0.update_lit(&m1.0, &m1.1, &m1.2);
                            m.1.update_lit(&m2.0, &m2.1, &m2.2);
                        }
                        c.needs_refresh = false;
                        c.chunk_state = ChunkState::Rendered;
                        meshed.insert(p);
                    }
                }
            }
        }
    }

}