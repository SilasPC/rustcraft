
use crate::block::BlockData;
use crate::util::sub_coords_from_i32;
use crate::registry::Registry;
use std::sync::Arc;
use crate::util::position_to_sub_coordinates;
use crate::util::AABB;
use cgmath::*;
use crate::engine::vao::*;
use crate::rustcraft::block::Block;

/// Signifies the current state of the chunk
#[derive(PartialOrd,PartialEq,Eq,Ord,Clone,Copy,Debug,serde::Serialize,serde::Deserialize)]
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

type Data = Vec<Vec<Vec<Block>>>;
type LightData = [[[u8; 16]; 16]; 16];

pub struct Chunk {
    pub chunk_state: ChunkState,
    pub needs_refresh: bool,
    pub pos: Vector3<i32>,
    pub data: Data,
    pub light: LightData,
    pub mesh: Option<VAO>,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum SavedBlock {
    Shared(usize),
    Unique(BlockData),
}

impl SavedBlock {
    pub fn by(block: &Block) -> Self {
        if block.is_shared() {
            Self::Shared(block.id)
        } else {
            Self::Unique(block.as_ref().clone())
        }
    }
    pub fn to(self, reg: &Registry) -> Block {
        match self {
            Self::Shared(id) => reg[id].clone(),
            Self::Unique(data) => Block::new_not_shared(data)
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedChunk {
    chunk_state: ChunkState,
    pos: Vector3<i32>,
    data: Vec<Vec<Vec<SavedBlock>>>,
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

    pub fn compare_position(&self, rhs: &Self) -> std::cmp::Ordering {
        (
            self.pos.x,
            self.pos.y,
            self.pos.z
        ).cmp(&(
            rhs.pos.x,
            rhs.pos.y,
            rhs.pos.z
        )).into()
    }

    pub fn load(x: i32, y: i32, z: i32, reg: &Registry) -> Option<Self> {
        let SavedChunk {
            chunk_state,
            data,
            pos,
        } = bincode::deserialize(&std::fs::read(format!("save/{:x}_{:x}_{:x}.chunk",x,y,z)).ok()?).ok()?;
        let data = data
            .into_iter()
            .map(|plane|
                plane
                    .into_iter()
                    .map(|row|
                        row
                            .into_iter()
                            .map(|b| match b {
                                SavedBlock::Unique(data) => Block::new_not_shared(data),
                                SavedBlock::Shared(id) => reg[id].clone()
                            }
                        )
                    .collect())
                .collect())
            .collect();
        let light = [[[0; 16]; 16]; 16];
        println!("Loading chunk {:?}",(x,y,z));
        Self { chunk_state, data, mesh: None, pos, needs_refresh: true, light }.into()
    }

    pub fn save(&self) -> Vec<u8> {
        let mut data = vec![];
        for x in 0..16 {
            let mut plane = vec![];
            for y in 0..16 {
                let mut row = vec![];
                for z in 0..16 {
                    row.push(SavedBlock::by(&self.data[x][y][z]))
                }
                plane.push(row)
            }
            data.push(plane)
        }
        let sc = SavedChunk {
            data,
            pos: self.pos,
            chunk_state: self.chunk_state
        };
        println!("Saving chunk {:?}", (self.pos.x, self.pos.y, self.pos.z));
        bincode::serialize(&sc).unwrap()
    }

    pub fn new(pos: Vector3<i32>, air: Block) -> Self {
        let data = vec![vec![vec![air;16];16];16];
        let light = [[[0; 16]; 16]; 16];
        Self { chunk_state: ChunkState::Empty, data, mesh: None, pos, needs_refresh: false, light }
    }

    pub fn world_pos(&self) -> Vector3<f32> {
        self.world_pos_i32().map(|x| x as f32)
    }
    pub fn world_pos_i32(&self) -> Vector3<i32> {
        self.pos.map(|x| x * 16)
    }

    pub fn world_pos_center(&self) -> Vector3<f32> {
        self.pos.map(|x| (x * 16 + 8) as f32)
    }

    pub fn block_at(&self, x: i32, y: i32, z: i32) -> &Block {
        &self.data[x.rem_euclid(16) as usize][y.rem_euclid(16) as usize][z.rem_euclid(16) as usize]
    }

    pub fn block_at_pos_mut(&mut self, pos: &Vector3<f32>) -> &mut Block {
        let sc = position_to_sub_coordinates(&pos).map(|c| c as usize);
        &mut self.data[sc.x][sc.y][sc.z]
    }
    pub fn block_at_pos(&self, pos: &Vector3<f32>) -> &Block {
        let sc = position_to_sub_coordinates(&pos).map(|c| c as usize);
        &self.data[sc.x][sc.y][sc.z]
    }
    
    pub fn set_at(&mut self, x: i32, y: i32, z: i32, block: &Block) -> bool {
        let sc = sub_coords_from_i32(x,y,z).map(|c| c as usize);
        let b = &mut self.data[sc.x][sc.y][sc.z];
        if b.ptr_eq(block) {
            false
        } else {
            *b = block.clone();
            self.needs_refresh = true;
            true
        }
    }
    pub fn set_at_pos(&mut self, pos: &Vector3<f32>, block: &Block) -> bool {
        let sc = position_to_sub_coordinates(&pos).map(|c| c as usize);
        let b = &mut self.data[sc.x][sc.y][sc.z];
        if b.ptr_eq(block) {
            false
        } else {
            *b = block.clone();
            self.needs_refresh = true;
            true
        }
    }

    pub fn gen_terrain(&mut self, noise: &crate::rustcraft::world::TerrainGen, reg: &Registry) {
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
                    reg[
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
                        }
                    ].clone();
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

    pub fn refresh(&mut self, reg: &Registry) {
        if !self.needs_refresh {return}
        let (verts, uvs) = make_mesh(&self.data, reg);
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

fn make_mesh(data: &Data, reg: &Registry) -> (Vec<f32>, Vec<f32>) {

    let block_map = &reg.blocks;
    let atlas = &reg.texture_atlas;

    let mut verts = vec![];
    let mut uvs = vec![];

    let uv_dif = atlas.uv_dif();

    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {

                let block = &data[x][y][z];

                let t = block.transparent;

                if block.no_render {continue};

                let xc = x as isize;
                let yc = y as isize + 1;
                let zc = z as isize;

                // y+ face
                if y == 15 || data[x][y+1][z].transparent /* != t */ {
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
                if y == 0 || data[x][y-1][z].transparent /* != t */ {
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
                if x == 0 || data[x-1][y][z].transparent /* != t */ {
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
                if x == 15 || data[x+1][y][z].transparent /* != t */ {
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
                if z == 0 || data[x][y][z-1].transparent /* != t */ {
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
                if z == 15 || data[x][y][z+1].transparent /* != t */ {
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
