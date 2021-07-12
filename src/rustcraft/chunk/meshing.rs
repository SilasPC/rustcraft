
use crate::BlocksData;
use cgmath::Matrix4;
use crate::Program;
use crate::Registry;
use crate::world::WorldData;
use crate::coords::*;
use crate::vao::VAO;

pub struct ChunkRenderer {
    pub program: Program
}

impl ChunkRenderer {
    pub fn new() -> Self {
        let program = Program::load(
            include_str!("vert.glsl"),
            include_str!("frag.glsl"),
            vec!["project","view","transform","globLight"]
        );
        ChunkRenderer {
            program
        }
    }
    pub fn load_glob_light(&self, light: f32) {
        self.program.load_f32(3, light);
    }
    pub fn load_proj(&self, mat: &Matrix4<f32>) {
        self.program.load_mat4(0, mat);
    }
    pub fn load_view(&self, mat: &Matrix4<f32>) {
        self.program.load_mat4(1, mat);
    }
    pub fn render(&self, w: &WorldData) {
        self.program.enable();
        for chunk in w.chunks.values().filter(|c| c.renderable()) {
            self.program.load_mat4(2, &Matrix4::from_translation(
                chunk.pos.as_pos_f32().0
            ));
            chunk.bind_and_draw();
        }
    }
}

/* pub fn square_mesh() -> VAO {
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
    ];
    let uvs = vec![
        0., 0.,
        0., 1.,
        1., 0.,
        0., 1.,
        1., 1.,
        1., 0.,
    ];
} */

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

pub fn make_mesh(pos: ChunkPos, w: &WorldData, reg: &Registry) -> (Vec<f32>, Vec<f32>, Vec<f32>) {

    let now = std::time::Instant::now();

    let atlas = &reg.texture_atlas;
    let air = reg.get("air");

    let mut verts = Vec::with_capacity(10000);
    let mut uvs = Vec::with_capacity(10000);
    let mut light = Vec::with_capacity(10000);

    let uv_dif = atlas.uv_dif();

    let (bx,by,bz) = pos.as_pos_i32().as_tuple();

    for x in 0..16i32 {
        for y in 0..16i32 {
            for z in 0..16i32 {

                macro_rules! get {
                    ($x:expr, $y:expr, $z:expr) => {{
                        let p: WorldPos<i32> = ($x+bx, $y+by, $z+bz).into();
                        w.block_at_any_state(&p)
                            .or_else(|| {println!("{:?} {:?}",pos,p);None})
                            .unwrap()
                    }};
                    (light $x:expr, $y:expr, $z:expr) => {{
                        let p: WorldPos<i32> = ($x+bx, $y+by, $z+bz).into();
                        w.light_at(&p) as f32 / 15.
                    }};
                }
                

                let block = get!(x,y,z);

                let t = block.transparent;

                if block.no_render {continue};

                let xc = x as isize;
                let yc = y as isize + 1;
                let zc = z as isize;

                // y+ face
                if get!(x,y+1,z).transparent /* != t */ {
                    let l = get!(light x,y+1,z);
                    light.extend([l].iter().cycle().take(6));
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
                if get!(x,y-1,z).transparent /* != t */ {
                    let yc = yc - 1;
                    let l = get!(light x,y-1,z);
                    light.extend([l].iter().cycle().take(6));
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
                if get!(x-1,y,z).transparent /* != t */ {
                    let l = get!(light x-1,y,z);
                    light.extend([l].iter().cycle().take(6));
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
                if get!(x+1,y,z).transparent /* != t */ {
                    let xc = xc + 1;
                    let l = get!(light x+1,y,z);
                    light.extend([l].iter().cycle().take(6));
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
                if get!(x,y,z-1).transparent /* != t */ {
                    let yc = yc - 1; //?
                    let l = get!(light x,y,z-1);
                    light.extend([l].iter().cycle().take(6));
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
                if get!(x,y,z+1).transparent /* != t */ {
                    let yc = yc - 1;//?
                    let zc = zc + 1;
                    let l = get!(light x,y,z+1);
                    light.extend([l].iter().cycle().take(6));
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

    // println!("{:?} values in {} ms",verts.len()+uvs.len()+light.len(), now.elapsed().as_millis());

    (verts, uvs, light)

}

pub fn make_mesh_old(pos: ChunkPos, w: &WorldData, reg: &Registry) -> (Vec<f32>, Vec<f32>, Vec<f32>) {

    let data = &w.chunks.get(&pos).unwrap().data;
    let atlas = &reg.texture_atlas;

    let mut verts = vec![];
    let mut uvs = vec![];
    let mut light = vec![];

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
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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

    (verts, uvs, light)

}


pub fn make_mesh_hybrid(pos: ChunkPos, w: &WorldData, reg: &Registry) -> (Vec<f32>, Vec<f32>, Vec<f32>) {

    let now = std::time::Instant::now();
    let atlas = &reg.texture_atlas;

    let mut verts = vec![];
    let mut uvs = vec![];
    let mut light = vec![];

    let uv_dif = atlas.uv_dif();

    let (bx,by,bz) = pos.as_pos_i32().as_tuple();

    let data = &w.chunks.get(&pos).unwrap().data;

    for x in 0..16 {
        for z in 0..16 {
            
            let yplus = &w.chunks.get(&(pos+(0,1,0).into())).unwrap().data;
            let y = 15;
            let xc = x as isize;
            let yc = y as isize + 1;
            let zc = z as isize;
            let block = &data[x][y][z];
            if !block.no_render {
    
                // y+ face
                if yplus[x][0][z].transparent /* != t */ {
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
            };

            let yneg = &w.chunks.get(&(pos+(0,-1,0).into())).unwrap().data;
            let y = 0;
            let xc = x as isize;
            let yc = y as isize + 1;
            let zc = z as isize;
            let block = &data[x][y][z];
            if !block.no_render {
                let xc = x as isize;
                let yc = y as isize + 1;
                let zc = z as isize;

                // y- face
                if yneg[x][15][z].transparent /* != t */ {
                    let yc = yc - 1;
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
            }

        }
    }
    
    for x in 0..16 {
        for y in 0..16 {
            
            let zplus = &w.chunks.get(&(pos+(0,0,1).into())).unwrap().data;
            let z = 15;
            let xc = x as isize;
            let yc = y as isize + 1;
            let zc = z as isize;
            let block = &data[x][y][z];
            if !block.no_render {
                
                // side faces are the same
                let (u,v) = atlas.get_uv(block.texture.1);
                let (uh,vh) = (u+uv_dif,v+uv_dif);
    
                // z+ face
                if zplus[x][y][0].transparent /* != t */ {
                    let yc = yc - 1;//?
                    let zc = zc + 1;
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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

            };

            let zneg = &w.chunks.get(&(pos+(0,0,-1).into())).unwrap().data;
            let z = 0;
            let xc = x as isize;
            let yc = y as isize + 1;
            let zc = z as isize;
            let block = &data[x][y][z];
            if !block.no_render {

                // side faces are the same
                let (u,v) = atlas.get_uv(block.texture.1);
                let (uh,vh) = (u+uv_dif,v+uv_dif);

                // z- face
                if zneg[x][y][15].transparent /* != t */ {
                    let yc = yc - 1; //?
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
            }

        }
    }

    for y in 0..16 {
        for z in 0..16 {
            
            let xplus = &w.chunks.get(&(pos+(1,0,0).into())).unwrap().data;
            let x = 15;
            let xc = x as isize;
            let yc = y as isize + 1;
            let zc = z as isize;
            let block = &data[x][y][z];
            if !block.no_render {
                
                // side faces are the same
                let (u,v) = atlas.get_uv(block.texture.1);
                let (uh,vh) = (u+uv_dif,v+uv_dif);
    
                // x+ face
                if xplus[0][y][z].transparent /* != t */ {
                    let xc = xc + 1;
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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

            };

            let xneg = &w.chunks.get(&(pos+(-1,0,0).into())).unwrap().data;
            let x = 0;
            let xc = x as isize;
            let yc = y as isize + 1;
            let zc = z as isize;
            let block = &data[x][y][z];
            if !block.no_render {

                // side faces are the same
                let (u,v) = atlas.get_uv(block.texture.1);
                let (uh,vh) = (u+uv_dif,v+uv_dif);

                // x- face
                if xneg[15][y][z].transparent /* != t */ {
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
            }

        }
    }
    
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
                if y != 15 && data[x][y+1][z].transparent /* != t */ {
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                if y != 0 && data[x][y-1][z].transparent /* != t */ {
                    let yc = yc - 1;
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                if x != 0 && data[x-1][y][z].transparent /* != t */ {
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                if x != 15 && data[x+1][y][z].transparent /* != t */ {
                    let xc = xc + 1;
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                if z != 0 && data[x][y][z-1].transparent /* != t */ {
                    let yc = yc - 1; //?
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
                if z != 15 && data[x][y][z+1].transparent /* != t */ {
                    let yc = yc - 1;//?
                    let zc = zc + 1;
                    light.extend(&[1.,1.,1.,1.,1.,1.,]);
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
    
    // println!("{:?} values in {} ms",verts.len()+uvs.len()+light.len(), now.elapsed().as_millis());

    (verts, uvs, light)

}