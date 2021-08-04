
use crate::prelude::*;

pub trait TerrainGenerator {
    fn density(&self, x: isize, y: isize, z: isize) -> f64;
    fn is_cave(&self, x: isize, y: isize, z: isize) -> bool;
    fn palette(&self, x: isize, z: isize) -> &[&'static str; 3];
}

#[derive(Debug)]
pub struct TerrainGen {
    pub noise: crate::perlin::PerlinNoise,
    pub noise_basic: crate::perlin::PerlinNoise,
    pub palettes: [[&'static str; 3]; 2],
}

impl TerrainGenerator for TerrainGen {
    fn is_cave(&self, x: isize, y: isize, z: isize) -> bool {
        let xf = x.abs() as f64 / 13.;
        let yf = y.abs() as f64 / 13.;
        let zf = z.abs() as f64 / 13.;
        let c = self.noise_basic.get3d([xf, yf, zf]);
        c > 0.75
    }
    fn density(&self, x: isize, y: isize, z: isize) -> f64 {
        let xf = x.abs() as f64 / 70.;
        let yf = y.abs() as f64 / 40.;
        let zf = z.abs() as f64 / 70.;
        let d = self.noise.get3d([xf, yf, zf]);
        let d = d * 0.8 / yf;

        let r = (xf*xf+zf*zf).sqrt().max(1.);
        d / r
    }
    fn palette(&self, x: isize, z: isize) -> &[&'static str; 3] {
        let xf = (x.abs() as f64 + 0.5) / 10.;
        let zf = (z.abs() as f64 + 0.5) / 10.;
        let n = self.noise_basic.get2d([xf, zf]);
        
        &self.palettes[if n < 0.5 {0} else {1}]
    } 
}
