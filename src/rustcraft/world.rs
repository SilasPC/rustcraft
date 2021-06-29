
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

pub struct ChunkArea<'a>(pub (i32,i32), pub [[&'a Chunk; 3]; 3]);

impl<'a> ChunkArea<'a> {
    pub fn block_at(&self, pos: Vector3<f32>) -> Option<usize> {
        let cc = (pos / 16.).map(|c| c.floor() as i32);
        let mut sc = (pos % 16.).map(|c| c.floor() as i32);
        if cc.x < self.0.0 || cc.x > self.0.0 + 2 || cc.z < self.0.1 || cc.z > self.0.1 + 2 {
            return None
        }
        let chunk = self.1[cc.x as usize - self.0.0 as usize][cc.z as usize - self.0.1 as usize];
        if sc.x < 0 {sc.x += 16}
        if sc.z < 0 {sc.z += 16}
        Some(chunk.data[sc.x as usize][sc.y as usize][sc.z as usize])
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
        for x in 0..7i32 {
            for y in 0..7 {
                for z in 0..7 {
                    /* let pos = Vector3 { x, y, z };
                    let mut chunk = Chunk::new(pos);
                    chunks_tree.insert(pos * 16, chunk, &AABB::from_corner(&pos.map(|x| (x * 16) as f32), 16.));
                    chunks_tree.get(16 * pos); */
                }
            }
        }
        let mut w = WorldData { chunks_tree, noise };
        w
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
