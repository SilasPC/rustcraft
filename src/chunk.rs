
use cgmath::*;
use crate::engine::vao::*;
use crate::rustcraft::block::Block;
use crate::Data as WData;

type Data = [[[usize; 16]; 16]; 16];

pub struct Chunk {
    pub needs_refresh: bool,
    pub pos: Vector3<f32>,
    pub data: Data,
    pub mesh: VAO,
}

impl Chunk {

    pub fn new(pos: Vector3<isize>, block_map: &Vec<Block>, atlas: &crate::texture::TextureAtlas) -> Self {

        let mut data = [[[0; 16]; 16]; 16];

        use std::ops::Mul;

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    let xf = 2. * (16. * pos.x as f32 + x as f32).mul(0.1).sin();
                    let zf = 2. * (16. * pos.z as f32 + z as f32).mul(0.2).sin();
                    if (y as f32) < 5. + xf + zf {
                        if y < 4 {
                            data[x][y][z] = 4;
                        } else if (1. + y as f32) >= 5. + xf + zf {
                            data[x][y][z] = 3;
                        } else {
                            data[x][y][z] = 2;
                        }
                    }
                    //if y == 0 {data[x][y][z] = true}
                }
            }
        }

        let pos = Vector3 {x: pos.x as f32, y: pos.y as f32, z: pos.z as f32};
        let (verts, uvs) = make_mesh(&data, block_map, atlas);
        let mesh = VAO::textured(&verts, &uvs);
        Self { data, mesh, pos, needs_refresh: false }

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