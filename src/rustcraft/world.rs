
use crate::registry::Registry;
use std::sync::Arc;
use crate::util::position_to_chunk_coordinates;
use crate::util::Proxy;
use crate::util::AABB;
use crate::BVH;
use aabb_tree::AabbTree;
use crate::block::Block;
use crate::chunk::*;
use cgmath::*;

#[derive(Debug)]
pub struct TerrainGen {
    pub noise: crate::perlin::PerlinNoise,
    pub noise_basic: crate::perlin::PerlinNoise,
}

impl TerrainGen {
    pub fn is_cave(&self, x: isize, y: isize, z: isize) -> bool {
        let xf = x.abs() as f64 / 13.;
        let yf = y.abs() as f64 / 13.;
        let zf = z.abs() as f64 / 13.;
        let c = self.noise_basic.get3d([xf, yf, zf]);
        let c = (c+0.1).powf(1.5);
        c > 0.65
    }
    pub fn density(&self, x: isize, y: isize, z: isize) -> f64 {
        let xf = x.abs() as f64 / 70.;
        let yf = y.abs() as f64 / 40.;
        let zf = z.abs() as f64 / 70.;
        let d = self.noise.get3d([xf, yf, zf]);
        let d = d * 0.8 / yf;
        d
    }
}

pub struct ChunkArea<'a> {
    world_data: &'a mut WorldData,
    pos: Vector3<i32>,
    area: Vec<Proxy>,
}

impl<'a> ChunkArea<'a> {
    pub fn edge_world_pos_i32(&self) -> Vector3<i32> {
        self.world_data.chunks_tree[self.area[0]].world_pos_i32()
    }
    pub fn center_world_pos_i32(&self) -> Vector3<i32> {
        self.world_data.chunks_tree[self.area[9*1+3*1+1]].world_pos_i32()
    }
    pub fn center_mut(&mut self) -> &mut Chunk {
        &mut self.world_data.chunks_tree[self.area[9*1+3*1+1]]
    }
    pub fn center(&self) -> &Chunk {
        &self.world_data.chunks_tree[self.area[9*1+3*1+1]]
    }
    pub fn center_state(&self) -> ChunkState {
        self.world_data.chunks_tree[self.area[9*2+3*2+1]].chunk_state
    }
    pub fn replace_at(&mut self, x: i32, y: i32, z: i32, block: &Block) -> bool {
        let c = &mut self.world_data.chunks_tree[
            self.area[( (x-self.pos.x*16)/16 * 9 + (y-self.pos.y*16)/16 * 3 + (z-self.pos.z*16)/16 ) as usize]];
        if c.block_at(x,y,z).replacable {
            c.set_at(x,y,z,block)
        } else {
            false
        }
    }
    pub fn set_block_at(&mut self, x: i32, y: i32, z: i32, block: &Block) -> bool {
        self.world_data.chunks_tree[
            self.area[( (x-self.pos.x*16)/16 * 9 + (y-self.pos.y*16)/16 * 3 + (z-self.pos.z*16)/16 ) as usize]].set_at(x,y,z,block)
    }
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> &Block {
        self.world_data.chunks_tree[
            self.area[( (x-self.pos.x*16)/16 * 9 + (y-self.pos.y*16)/16 * 3 + (z-self.pos.z*16)/16 ) as usize]].block_at(x,y,z)
    }
    pub fn block_at_pos(&self, pos: &Vector3<f32>) -> &Block {
        let cc = position_to_chunk_coordinates(pos) - self.pos;
        self.world_data.chunks_tree[self.area[(cc.x * 9 + cc.y * 3 + cc.z) as usize]].block_at_pos(pos)
    }
    pub fn light_at(&self, mut x: i32, mut y: i32, mut z: i32) -> u8 {
        x -= self.pos.x * 16;
        y -= self.pos.y * 16;
        z -= self.pos.z * 16;
        let sx = x.rem_euclid(16) as usize;
        let sy = y.rem_euclid(16) as usize;
        let sz = z.rem_euclid(16) as usize;
        self.world_data.chunks_tree[self.area[(x * 9 + y * 3 + z) as usize]].light[sx][sy][sz]
    }
    pub fn set_light_at(&mut self, mut x: i32, mut y: i32, mut z: i32, val: u8) {
        x -= self.pos.x * 16;
        y -= self.pos.y * 16;
        z -= self.pos.z * 16;
        let sx = x.rem_euclid(16) as usize;
        let sy = y.rem_euclid(16) as usize;
        let sz = z.rem_euclid(16) as usize;
        let c = &mut self.world_data.chunks_tree[self.area[(x * 9 + y * 3 + z) as usize]];
        c.light[sx][sy][sz] = val;
        c.needs_refresh = true;
    }
}

#[derive(Debug)]
pub struct WorldData {
    pub seed: String,
    pub air: Block,
    pub chunks_tree: BVH<Vector3<i32>, Box<Chunk>>,
    pub noise: TerrainGen
}

impl WorldData {
    
    pub fn new(seed: &str, air: Block) -> Self {
        let noise = crate::perlin::PerlinNoise::new(seed.to_owned(), 4, 0.5);
        let noise_basic = crate::perlin::PerlinNoise::new(seed.to_owned(), 1, 1.);
        let noise = TerrainGen {
            noise,
            noise_basic
        };
        let mut chunks_tree = BVH::new();
        WorldData { seed: seed.to_owned(), chunks_tree, noise, air }
    }

    pub fn block_at_pos(&self, pos: &Vector3<f32>) -> Option<&Block> {
        self.chunk_at_pos(pos).map(|c| c.block_at_pos(pos))
    }
    pub fn set_block_at_pos(&mut self, pos: &Vector3<f32>, block: &Block) -> bool {
        self.chunk_at_pos_mut(pos).map(|c| c.set_at_pos(pos, block)).unwrap_or(false)
    }
    pub fn block_at_pos_mut(&mut self, pos: &Vector3<f32>) -> Option<&mut Block> {
        self.chunk_at_pos_mut(pos).map(|c| c.block_at_pos_mut(pos))
    }
    pub fn set_block_at(&mut self, x: i32, y: i32, z: i32, block: &Block) -> bool {
        self.chunk_at_pos_mut(&Vector3 {x: x as f32, y: y as f32, z: z as f32})
            .map(|c| c.set_at(x,y,z, block)).unwrap_or(false)
    }
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> Option<&Block> {
        self.chunk_at_pos(&Vector3 {x: x as f32, y: y as f32, z: z as f32}).map(|c| c.block_at(x,y,z))
    }

    pub fn chunk_at_pos(&self, pos: &Vector3<f32>) -> Option<&Chunk> {
        let ps = position_to_chunk_coordinates(pos);
        self.chunks_tree.get(ps).filter(|c| c.chunk_state == ChunkState::Done).map(Box::as_ref)
    }
    pub fn chunk_at_pos_mut(&mut self, pos: &Vector3<f32>) -> Option<&mut Chunk> {
        let ps = position_to_chunk_coordinates(pos);
        self.chunks_tree.get_mut(ps).filter(|c| c.chunk_state == ChunkState::Done).map(Box::as_mut)
    }

    pub fn take_chunk(&mut self) -> Box<Chunk> {
        self.chunks_tree.remove(Vector3{x:3,y:1,z:3}).unwrap()
    }

    pub fn chunk_iter_mut(&mut self) -> impl std::iter::Iterator<Item=&mut Chunk> {
        self.chunks_tree.values_mut().filter(|c| c.chunk_state == ChunkState::Done).map(Box::as_mut)
    }

    pub fn area_exclusive(&self, proxy: Proxy) -> Vec<Proxy> {
        self.chunks_tree
            .query(&self.chunks_tree[proxy].aabb())
            .into_iter()
            .filter(|p| *p != proxy)
            .collect::<Vec<_>>()
    }

    pub fn area(&mut self, pos: &Vector3<f32>) -> Option<ChunkArea<'_>> {
        let mut area = self.chunks_tree
            .query(&AABB::radius(pos, 16.));
        if area.len() != 27 {
            return None
        }
        let mut area: Vec<_> = area.iter().map(|p| (&self.chunks_tree[*p], p)).collect();
        area.sort_unstable_by(|a,b| a.0.compare_position(&b.0));
        // println!("{:?}",area);
        let area = area.iter().map(|(_,p)| **p).collect::<Vec<_>>();
        ChunkArea {
            world_data: self,
            area,
            pos: position_to_chunk_coordinates(pos) - Vector3 { x: 1, y: 1, z: 1}
        }.into()
    }

    fn area_from_proxy(&mut self, proxy: Proxy) -> Option<ChunkArea<'_>> {
        self.area(&self.chunks_tree[proxy].world_pos_center())
    }
    fn fill_around(&mut self, pos: &Vector3<f32>, rad: f32, reg: &Registry) {

        let low = position_to_chunk_coordinates(&(pos - Vector3 { x: rad, y: rad, z: rad, }));
        let high = position_to_chunk_coordinates(&(pos + Vector3 { x: rad, y: rad, z: rad, }));

        for cx in low.x..=high.x {
            for cy in low.y..=high.y {
                for cz in low.z..=high.z {
                    if !self.chunks_tree.has(Vector3 {
                        x: cx,
                        y: cy,
                        z: cz,
                    }) {
                        // result.insert(i, (cx, cy, cz));
                        if let Some(chunk) = Chunk::load(cx, cy, cz, reg) {
                            let aabb = chunk.aabb();
                            self.chunks_tree.insert(chunk.pos, Box::new(chunk), &aabb);
                        } else {
                            let pos = Vector3 {
                                x: cx,
                                y: cy,
                                z: cz,
                            };
                            let chunk = Chunk::new(pos, /* self.air.clone() */ reg[0].clone());
                            let aabb = chunk.aabb();
                            self.chunks_tree.insert(pos, Box::new(chunk), &aabb);
                        }
                    }
                }
            }
        }

    }

    pub fn load_around2(&mut self, pos: &Vector3<f32>, rad: f32, reg: &Registry) {
        use std::cmp::Reverse as Rev;
        
        self.fill_around(pos, rad + 32., reg);

        let mut heap = std::collections::BinaryHeap::new(); // (Rev<target>, Rev<current>, proxy)
        let aabb = AABB::radius(pos, rad);

        for proxy in self.chunks_tree.query(&aabb) {
            let c = &self.chunks_tree[proxy];
            if c.chunk_state != ChunkState::Done {
                heap.push((Rev(ChunkState::Done), Rev(c.chunk_state), proxy))
            }
        }

        while let Some(mut x) = heap.peek().copied() {
            let mut aabb = self.chunks_tree[x.2].aabb();
            aabb.extend_radius(1.);
            let area = self.area_exclusive(x.2);
            let mut can_upgrade = true;
            let new_target = self.chunks_tree[x.2].chunk_state.prev();
            for proxy in &area {
                let state = self.chunks_tree[*proxy].chunk_state;
                if state < new_target {
                    heap.push((Rev(new_target), Rev(state), *proxy));
                    can_upgrade = false;
                }
            }
            if can_upgrade {
                x.1.0 = match x.1.0 {
                    ChunkState::Empty => {
                        self.chunks_tree[x.2].gen_terrain(&self.noise, reg);
                        ChunkState::Filled
                    },
                    ChunkState::Filled => {
                        let mut area = self.area_from_proxy(x.2).unwrap();
                        gen_detail(&mut area, &reg);
                        ChunkState::Done
                    }
                    ChunkState::Done => {
                        x.1.0
                    }
                };
                heap.pop();
                if x.1.0 != x.0.0 {
                    heap.push(x);
                }
            }
        }
    }

}

fn gen_detail(area: &mut ChunkArea, reg: &Registry) {
    let Vector3 {x,y,z} = area.center_world_pos_i32();
    let log = &reg[4];
    let leaves = &reg[6];
    for x in x..x+16 {
        for z in z..z+16 {
            for y in y..y+16 {
                if area.block_at(x,y-1,z).id == 3 {
                    let nx = x as f64 / 1.3;
                    let nz = z as f64 / 1.3;
                    let n = area.world_data.noise.noise_basic.get2d([nx,nz]);
                    const CUTOFF: f64 = 0.59;
                    if n > CUTOFF {
                        const INV: f64 = 1./1.3;
                        if area.world_data.noise.noise_basic.get2d([nx-INV,nz]) <= CUTOFF &&
                            area.world_data.noise.noise_basic.get2d([nx,nz-INV]) <= CUTOFF {
                                let h = 3 + (2. * area.world_data.noise.noise_basic.get2d([nx,nz])) as i32;
                                for y in y..=y+h {
                                    area.set_block_at(x,y,z,log);
                                }
                                for dx in -2..=2 {
                                    for dz in -2..=2 {
                                        if dx == 0 && dz == 0 {continue}
                                        for y in y+h-3..=y+h {
                                            area.replace_at(x+dx,y+4,z+dz,leaves);
                                        }
                                    }
                                }
                                area.replace_at(x,y+h,z,leaves);
                            }
                    }
                    break;
                }
            }
        }    
    }
    /* area.set_block_at(x, y, z, log);
    area.set_block_at(x, y+1, z, log);
    area.set_block_at(x, y+2, z, log);
    area.set_block_at(x, y+3, z, leaves);
    area.set_block_at(x+1, y+2, z, leaves);
    area.set_block_at(x-1, y+2, z, leaves);
    area.set_block_at(x, y+2, z+1, leaves);
    area.set_block_at(x, y+2, z-1, leaves); */
    area.center_mut().chunk_state = ChunkState::Done;
}

pub fn update_light(area: &mut ChunkArea) {
    let mut prop_queue = std::collections::VecDeque::new();
    while let Some((x,y,z,l)) = prop_queue.pop_front() {
        if prop(area, x+1, y, z, l) {
            prop_queue.push_back((x+1,y,z, l-1))}
        if prop(area, x-1, y, z, l) {
            prop_queue.push_back((x+1,y,z, l-1))}
        if prop(area, x, y+1, z, l) {
            prop_queue.push_back((x,y+1,z, l-1))}
        if prop(area, x, y-1, z, l) {
            prop_queue.push_back((x,y-1,z, l-1))}
        if prop(area, x, y, z+1, l) {
            prop_queue.push_back((x,y,z+1, l-1))}
        if prop(area, x, y, z-1, l) {
            prop_queue.push_back((x,y,z-1, l-1))}
    }
    fn prop(area: &mut ChunkArea, x: i32, y: i32, z: i32, new_light: u8) -> bool {
        let b = area.block_at(x,y,z);
        if !b.transparent || area.light_at(x,y,z)+2>new_light {return false}
        area.set_light_at(x,y,z,new_light-1);
        true
    }
}
