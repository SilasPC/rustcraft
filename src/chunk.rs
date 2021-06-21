
use cgmath::*;
use crate::engine::vao::*;
use crate::rustcraft::block::Block;
use crate::Data as WData;

/// Signifies the current state of the chunk generation
pub enum GenState {
    Empty,
    Terrain,
    Detail,
    Done,
}

type Data = [[[usize; 16]; 16]; 16];

pub struct Chunk {
    pub gen_state: GenState,
    pub needs_refresh: bool,
    pub pos: Vector3<f32>,
    pub data: Data,
    pub mesh: VAO,
}

impl Chunk {

    pub fn new(pos: Vector3<isize>, block_map: &Vec<Block>, atlas: &crate::texture::TextureAtlas) -> Self {

        let data = [[[0; 16]; 16]; 16];
        let pos = Vector3 {x: pos.x as f32, y: pos.y as f32, z: pos.z as f32};
        let (verts, uvs) = make_mesh(&data, block_map, atlas);
        let mesh = VAO::textured(&verts, &uvs);
        Self { gen_state: GenState::Empty, data, mesh, pos, needs_refresh: false }

    }

    pub fn gen_terrain(&mut self, noise: &crate::rustcraft::world::TerrainGen) {
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    let pos = self.pos.map(|x| x as isize);

                    let ax = 16 * pos.x + x;
                    let ay = 16 * pos.y + y;
                    let az = 16 * pos.z + z;

                    // if noise.is_cave(ax, ay, az) {continue}

                    let d = noise.density(ax,ay,az);
                    let da = noise.density(ax,ay+1,az);
                    
                    self.data[x as usize][y as usize][z as usize] = 
                    if d > 0.56 {
                        1
                    } else if d > 0.52 {
                        if da > 0.52 {
                            2
                        } else {
                            3
                        }
                    } else {
                        0
                    };
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
        self.gen_state = GenState::Terrain;
    }

    pub fn refresh(&mut self, wdata: &Vec<Block>, atlas: &crate::texture::TextureAtlas) {
        if !self.needs_refresh {return}
        let (verts, uvs) = make_mesh(&self.data, wdata, atlas);
        self.mesh.update(&verts, &uvs);
        self.needs_refresh = false;
    }

}

pub fn cube_mesh() -> VAO {
    let xc = 0;
    let yc = 1;
    let zc = 0;
    let verts = vec![
        xc, yc, zc,
        xc, yc, zc+1,
        xc+1, yc, zc,
        xc, yc, zc+1,
        xc+1, yc, zc+1,
        xc+1, yc, zc,

        xc, yc-1, zc,
        xc+1, yc-1, zc,
        xc, yc-1, zc+1,
        xc, yc-1, zc+1,
        xc+1, yc-1, zc,
        xc+1, yc-1, zc+1,

        xc, yc, zc,
        xc, yc-1, zc,
        xc, yc, zc+1,
        xc, yc-1, zc,
        xc, yc-1, zc+1,
        xc, yc, zc+1,

        xc+1, yc, zc,
        xc+1, yc, zc+1,
        xc+1, yc-1, zc,
        xc+1, yc-1, zc,
        xc+1, yc, zc+1,
        xc+1, yc-1, zc+1,

        xc, yc-1, zc,
        xc, yc, zc,
        xc+1, yc-1, zc,
        xc+1, yc-1, zc,
        xc, yc, zc,
        xc+1, yc, zc,

        xc, yc-1, zc+1,
        xc+1, yc-1, zc+1,
        xc, yc, zc+1,
        xc+1, yc-1, zc+1,
        xc+1, yc, zc+1,
        xc, yc, zc+1,
    ];
    let uvs = vec![
        0., 0.,
        0., 1.,
        1., 0.,
        0., 1.,
        1., 1.,
        1., 0.,
        0., 0.,
        1., 0.,
        0., 1.,
        0., 1.,
        1., 0.,
        1., 1.,
        0., 0.,
        0., 1.,
        1., 0.,
        0., 1.,
        1., 1.,
        1., 0.,
        0., 0.,
        1., 0.,
        0., 1.,
        0., 1.,
        1., 0.,
        1., 1.,
        1., 1.,
        1., 0.,
        0., 1.,
        0., 1.,
        1., 0.,
        0., 0.,
        1., 1.,
        0., 1.,
        1., 0.,
        0., 1.,
        0., 0.,
        1., 0.,
    ];
    
    let verts = verts.into_iter().map(|v: isize| v as f32).collect::<Vec<_>>();

    VAO::textured(&verts, &uvs)

}

fn make_mesh(data: &Data, block_map: &Vec<Block>, atlas: &crate::texture::TextureAtlas) -> (Vec<f32>, Vec<f32>) {

    let mut verts = vec![];
    let mut uvs = vec![];

    let uv_dif = atlas.uv_dif();

    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {

                let block = &block_map[data[x][y][z]];

                let t = block.transparent;

                if block.no_render {continue};

                let xc = x as isize;
                let yc = y as isize + 1;
                let zc = z as isize;

                // y+ face
                if y == 15 || block_map[data[x][y+1][z]].transparent != t {
                    verts.extend_from_slice(&[
                        xc, yc, zc,
                        xc, yc, zc+1,
                        xc+1, yc, zc,
                        xc, yc, zc+1,
                        xc+1, yc, zc+1,
                        xc+1, yc, zc,
                    ]);
                    let (u,v) = atlas.get_uv(block.texture.0);
                    let (uh,vh) = (u+uv_dif,v+uv_dif);
                    uvs.extend_from_slice(&[
                        u, v,
                        u, vh,
                        uh, v,
                        u, vh,
                        uh, vh,
                        uh, v,
                    ]);
                }

                // y- face
                if y == 0 || block_map[data[x][y-1][z]].transparent != t {
                    let yc = yc - 1;
                    verts.extend_from_slice(&[
                        xc, yc, zc,
                        xc+1, yc, zc,
                        xc, yc, zc+1,
                        xc, yc, zc+1,
                        xc+1, yc, zc,
                        xc+1, yc, zc+1,
                    ]);
                    let (u,v) = atlas.get_uv(block.texture.2);
                    let (uh,vh) = (u+uv_dif,v+uv_dif);
                    uvs.extend_from_slice(&[
                        u, v,
                        uh, v,
                        u, vh,
                        u, vh,
                        uh, v,
                        uh, vh,
                    ]);
                }

                // side faces are the same
                let (u,v) = atlas.get_uv(block.texture.1);
                let (uh,vh) = (u+uv_dif,v+uv_dif);

                // x- face
                if x == 0 || block_map[data[x-1][y][z]].transparent != t {
                    verts.extend_from_slice(&[
                        xc, yc, zc,
                        xc, yc-1, zc,
                        xc, yc, zc+1,
                        xc, yc-1, zc,
                        xc, yc-1, zc+1,
                        xc, yc, zc+1,
                    ]);
                    uvs.extend_from_slice(&[
                        u, v,
                        u, vh,
                        uh, v,
                        u, vh,
                        uh, vh,
                        uh, v,
                    ]);
                }

                // x+ face
                if x == 15 || block_map[data[x+1][y][z]].transparent != t {
                    let xc = xc + 1;
                    verts.extend_from_slice(&[
                        xc, yc, zc,
                        xc, yc, zc+1,
                        xc, yc-1, zc,
                        xc, yc-1, zc,
                        xc, yc, zc+1,
                        xc, yc-1, zc+1,
                    ]);
                    uvs.extend_from_slice(&[
                        u, v,
                        uh, v,
                        u, vh,
                        u, vh,
                        uh, v,
                        uh, vh,
                    ]);
                }

                // z- face
                if z == 0 || block_map[data[x][y][z-1]].transparent != t {
                    let yc = yc - 1; //?
                    verts.extend_from_slice(&[
                        xc, yc, zc,
                        xc, yc+1, zc,
                        xc+1, yc, zc,
                        xc+1, yc, zc,
                        xc, yc+1, zc,
                        xc+1, yc+1, zc,
                    ]);
                    uvs.extend_from_slice(&[
                        uh, vh,
                        uh, v,
                        u, vh,
                        u, vh,
                        uh, v,
                        u, v,
                    ]);
                }

                // z+ face
                if z == 15 || block_map[data[x][y][z+1]].transparent != t {
                    let yc = yc - 1;//?
                    let zc = zc + 1;
                    verts.extend_from_slice(&[
                        xc, yc, zc,
                        xc+1, yc, zc,
                        xc, yc+1, zc,
                        xc+1, yc, zc,
                        xc+1, yc+1, zc,
                        xc, yc+1, zc,
                    ]);
                    uvs.extend_from_slice(&[
                        uh, vh,
                        u, vh,
                        uh, v,
                        u, vh,
                        u, v,
                        uh, v,
                    ]);
                }
                
            }
        }
    }

    let verts = verts.into_iter().map(|v: isize| v as f32).collect::<Vec<_>>();

    (verts, uvs)

}