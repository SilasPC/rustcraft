
use crate::block::Block;
use crate::chunk::*;
use cgmath::*;

pub struct TerrainGen {
    pub noise: crate::perlin::PerlinNoise,
    pub noise_basic: crate::perlin::PerlinNoise,
}

impl TerrainGen {
    pub fn is_cave(&self, x: isize, y: isize, z: isize) -> bool {
        let xf = x as f64 / 13.;
        let yf = y as f64 / 13.;
        let zf = z as f64 / 13.;
        let c = self.noise_basic.get3d([xf, yf, zf]);
        let c = (c+0.1).powf(1.5);
        c > 0.65
    }
    pub fn density(&self, x: isize, y: isize, z: isize) -> f64 {
        let xf = x as f64 / 70.;
        let yf = y as f64 / 40.;
        let zf = z as f64 / 70.;
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

pub struct WorldData {
    pub chunks: Vec<Vec<Vec<Chunk>>>,
}

impl WorldData {
    pub fn new() -> Self {
        let noise = crate::perlin::PerlinNoise::new("seed!".to_owned(), 4, 0.5);
        let noise_basic = crate::perlin::PerlinNoise::new("seed!".to_owned(), 1, 1.);
        let noise = TerrainGen {
            noise,
            noise_basic
        };
        let mut chunks = vec![];
        for x in 0..7isize {
            let mut ys = vec![];
            for y in 0..7isize {
                let mut zs = vec![];
                for z in 0..7isize {
                    let mut chunk = Chunk::new(Vector3 { x, y, z });
                    chunk.gen_terrain(&noise);
                    chunk.gen_detail();
                    zs.push(
                        chunk
                    );
                }
                ys.push(zs);
            }
            chunks.push(ys);
        }
        WorldData { chunks }
    }

    pub fn chunk_iter_mut(&mut self) -> impl std::iter::Iterator<Item=&mut Chunk> {
        self.chunks.iter_mut().flat_map(|inn| inn.iter_mut().flat_map(|i2| i2.iter_mut()))
    }

}
