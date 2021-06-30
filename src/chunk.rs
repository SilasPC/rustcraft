
use crate::util::position_to_sub_coordinates;
use crate::util::AABB;
use cgmath::*;
use crate::engine::vao::*;
use crate::rustcraft::block::Block;

/// Signifies the current state of the chunk
#[derive(PartialOrd,PartialEq,Eq,Ord,Clone,Copy,Debug)]
pub enum ChunkState {
    Empty,
    Filled,
    Done,
}

impl ChunkState {
    pub fn prev(self) -> Self {
        match self {
            ChunkState::Done => ChunkState::Filled,
            ChunkState::Filled => ChunkState::Empty,
            ChunkState::Empty => ChunkState::Empty,
        }
    }
}

type Data = [[[usize; 16]; 16]; 16];
type LightData = [[[u8; 16]; 16]; 16];

pub struct Chunk {
    pub chunk_state: ChunkState,
    pub needs_refresh: bool,
    pub pos: Vector3<i32>,
    pub data: Data,
    pub light: LightData,
    pub mesh: Option<VAO>,
}


impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("pos", &self.pos)
            .field("chunk_state", &self.chunk_state)
            .finish()
    }
}

impl Chunk {

    pub fn new(pos: Vector3<i32>) -> Self {
        let data = [[[0; 16]; 16]; 16];
        let light = [[[0; 16]; 16]; 16];
        Self { chunk_state: ChunkState::Empty, data, mesh: None, pos, needs_refresh: false, light }
    }

    pub fn block_id_at(&self, x: i32, y: i32, z: i32) -> usize {
        self.data[x.rem_euclid(16) as usize][y.rem_euclid(16) as usize][z.rem_euclid(16) as usize]
    }

    pub fn block_id_at_pos(&self, pos: &Vector3<f32>) -> usize {
        let sc = position_to_sub_coordinates(&pos).map(|c| c as usize);
        self.data[sc.x][sc.y][sc.z]
    }
    
    pub fn set_at_pos(&mut self, pos: &Vector3<f32>, id: usize) -> bool {
        let sc = position_to_sub_coordinates(&pos).map(|c| c as usize);
        let bid = &mut self.data[sc.x][sc.y][sc.z];
        if *bid != id {
            self.needs_refresh = true;
            *bid = id;
            true
        } else {
            false
        }
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
        self.chunk_state = ChunkState::Filled;
    }

    pub fn gen_detail(&mut self) {
        self.chunk_state = ChunkState::Done;
    }

    pub fn renderable_after_refresh(&self) -> bool {
        self.chunk_state == ChunkState::Done
    }

    pub fn aabb(&self) -> AABB { AABB::from_corner(&self.pos.map(|x| x as f32 * 16.), 16.) }

    pub fn refresh(&mut self, wdata: &Vec<std::sync::Arc<Block>>, atlas: &crate::texture::TextureAtlas) {
        if !self.needs_refresh {return}
        let (verts, uvs) = make_mesh(&self.data, wdata, atlas);
        if let Some(mesh) = &mut self.mesh {
            mesh.update(&verts, &uvs);
        } else {
            self.mesh = Some(VAO::textured(&verts, &uvs));
        }
        self.needs_refresh = false;
    }

    pub fn bind_and_draw(&self) {
        if let Some(mesh) = &self.mesh {
            mesh.bind();
            mesh.draw();
        }
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

fn make_mesh(data: &Data, block_map: &Vec<std::sync::Arc<Block>>, atlas: &crate::texture::TextureAtlas) -> (Vec<f32>, Vec<f32>) {

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