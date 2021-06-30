
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
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> usize {
        self.world_data.chunks_tree[
            self.area[((x-self.pos.x*16) * 9 + (y-self.pos.y*16) * 3 + (z-self.pos.z*16)) as usize]].block_id_at(x,y,z)
    }
    pub fn block_at_pos(&self, pos: &Vector3<f32>) -> usize {
        let cc = position_to_chunk_coordinates(pos) - self.pos;
        self.world_data.chunks_tree[self.area[(cc.x * 9 + cc.y * 3 + cc.z) as usize]].block_id_at_pos(pos)
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
    pub chunks_tree: BVH<Vector3<i32>, Chunk>,
    pub noise: TerrainGen
}

impl WorldData {
    
    pub fn new(seed: &str) -> Self {
        let noise = crate::perlin::PerlinNoise::new(seed.to_owned(), 4, 0.5);
        let noise_basic = crate::perlin::PerlinNoise::new(seed.to_owned(), 1, 1.);
        let noise = TerrainGen {
            noise,
            noise_basic
        };
        let mut chunks_tree = BVH::new();
        WorldData { chunks_tree, noise }
    }

    pub fn block_id_at_pos(&self, pos: &Vector3<f32>) -> Option<usize> {
        self.chunk_at_pos(pos).map(|c| c.block_id_at_pos(pos))
    }

    pub fn chunk_at_pos(&self, pos: &Vector3<f32>) -> Option<&Chunk> {
        let ps = position_to_chunk_coordinates(pos);
        self.chunks_tree.get(ps).filter(|c| c.chunk_state == ChunkState::Done)
    }
    pub fn chunk_at_pos_mut(&mut self, pos: &Vector3<f32>) -> Option<&mut Chunk> {
        self.chunks_tree.get_mut(position_to_chunk_coordinates(pos))
            .filter(|c| c.chunk_state == ChunkState::Done)
    }

    pub fn chunk_iter_mut(&mut self) -> impl std::iter::Iterator<Item=&mut Chunk> {
        self.chunks_tree.values_mut().filter(|c| c.chunk_state == ChunkState::Done)
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
        area.sort_unstable();
        ChunkArea {
            world_data: self,
            area,
            pos: position_to_chunk_coordinates(pos) - Vector3 { x: 1, y: 1, z: 1}
        }.into()
    }

    fn fill_empty_around(&mut self, pos: &Vector3<f32>, rad: f32) {

        let mut aabb = AABB::radius(pos, rad);
        //aabb.extend_radius(-0.1);
        println!("{:?}", aabb);
        let mut result = self.chunks_tree
            .query(&aabb)
            .iter()
            .map(|proxy| self.chunks_tree[*proxy].pos)
            .map(|p| (p.x, p.y, p.z))
            .collect::<Vec<_>>();
        result.sort_unstable();

        let low = position_to_chunk_coordinates(&(pos - Vector3 { x: rad, y: rad, z: rad, }));
        let high = position_to_chunk_coordinates(&(pos + Vector3 { x: rad, y: rad, z: rad, }));

        let mut i = 0;
        for cx in low.x..=high.x {
            for cy in low.y..=high.y {
                for cz in low.z..=high.z {
                    let pos = result.get(i).unwrap_or(&(100,100,100));
                    if /* pos.0 != cx || pos.1 != cy || pos.2 != cz */ !self.chunks_tree.has(Vector3 {
                        x: cx,
                        y: cy,
                        z: cz,
                    }) {
                        // result.insert(i, (cx, cy, cz));
                        let pos = Vector3 {
                            x: cx,
                            y: cy,
                            z: cz,
                        };
                        let chunk = Chunk::new(pos);
                        let aabb = chunk.aabb();
                        self.chunks_tree.insert(pos, chunk,&aabb);
                    } else {
                        i += 1;
                    }
                }
            }
        }

    }

    pub fn load_around2(&mut self, pos: &Vector3<f32>, rad: f32) {
        use std::cmp::Reverse as Rev;
        
        self.fill_empty_around(pos, rad + 32.);

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
                let mut c = &mut self.chunks_tree[x.2];
                match c.chunk_state {
                    ChunkState::Empty => c.gen_terrain(&self.noise),
                    ChunkState::Filled => c.gen_detail(),
                    ChunkState::Done => {},
                }
                heap.pop();
                if c.chunk_state != x.0.0 {
                    x.1 = Rev(c.chunk_state);
                    heap.push(x);
                }
            }
        }
    }

}

pub fn update_light(area: &mut ChunkArea, reg: &Vec<Arc<Block>>) {
    let mut prop_queue = std::collections::VecDeque::new();
    while let Some((x,y,z,l)) = prop_queue.pop_front() {
        if prop(area, reg, x+1, y, z, l) {
            prop_queue.push_back((x+1,y,z, l-1))}
        if prop(area, reg, x-1, y, z, l) {
            prop_queue.push_back((x+1,y,z, l-1))}
        if prop(area, reg, x, y+1, z, l) {
            prop_queue.push_back((x,y+1,z, l-1))}
        if prop(area, reg, x, y-1, z, l) {
            prop_queue.push_back((x,y-1,z, l-1))}
        if prop(area, reg, x, y, z+1, l) {
            prop_queue.push_back((x,y,z+1, l-1))}
        if prop(area, reg, x, y, z-1, l) {
            prop_queue.push_back((x,y,z-1, l-1))}
    }
    fn prop(area: &mut ChunkArea, reg: &Vec<Arc<Block>>, x: i32, y: i32, z: i32, new_light: u8) -> bool {
        let b = &reg[area.block_at(x,y,z)];
        if !b.transparent || area.light_at(x,y,z)+2>new_light {return false}
        area.set_light_at(x,y,z,new_light-1);
        true
    }
}
