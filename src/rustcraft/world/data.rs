
use super::*;
use crate::prelude::*;

impl WorldData {

    pub fn smooth_light_level(&self) -> f32 {
        (self.time_of_day() * std::f32::consts::TAU).sin() + 0.5
    } 
    pub fn time_of_day(&self) -> f32 {
        (self.ticks as f32 / consts::DAY_NIGHT_DURATION_TICKS as f32) % 1.
    }

    pub fn load_around(&mut self, pos: &impl Coord) {
        let (x,y,z) = pos.as_chunk().as_tuple();
        let p = (x-5,y-5,z-5).into();
        println!("Filling from {:?}...", p);
        self.to_load.push_back(Loading::Filling(0, p))
    }

    pub fn load(&mut self, reg: &ItemRegistry, max_work: usize) {
        const DIAMETER: i32 = 10;
        if let Some(mut loading) = self.to_load.pop_front() {
            let mut work = 0;
            match loading {
                Loading::Filling(ref mut i, pos) => {
                    let (x,y,z) = pos.as_tuple();
                    const RAD: i32 = DIAMETER;
                    while *i < RAD*RAD*RAD && work < max_work {
                        let p = (
                            x + *i / (RAD*RAD),
                            y + (*i / RAD) % RAD,
                            z + *i % RAD
                        ).into();
                        if let Some(c) = self.blocks.chunks.get_mut(&p) {
                            if c.chunk.chunk_state == ChunkState::Empty {
                                c.chunk.gen_terrain(&*self.noise, reg);
                                work += 1;
                            }
                        } else {
                            let mut chunk = Box::new(Chunk::new(p, self.air.clone()));
                            chunk.gen_terrain(&*self.noise, reg);
                            let mut chunk_data = ChunkData {
                                chunk,
                                loaded_neighbours: 0, 
                            };
                            for dx in -1..=1 {
                                for dy in -1..=1 {
                                    for dz in -1..=1 {
                                        let p = (
                                            p.x+dx,
                                            p.y+dy,
                                            p.z+dz,
                                        ).into();
                                        if let Some(c) = self.blocks.chunks.get_mut(&p) {
                                            chunk_data.loaded_neighbours += 1;
                                            c.loaded_neighbours += 1;
                                        }
                                    }
                                }
                            }
                            self.blocks.chunks.insert(p, chunk_data);
                            work += 1;
                        }
                        // println!("generated for {:?}",p);
                        *i += 1;
                    }
                    if *i == RAD*RAD*RAD {
                        let pos = pos + Vector3{x:1,y:1,z:1}.into();
                        println!("Detailing from {:?}...",pos);
                        loading = Loading::Detailing(0, pos);
                    }
                },
                Loading::Detailing(ref mut i, pos) => {
                    let (x,y,z) = pos.as_tuple();
                    const RAD: i32 = DIAMETER-2;
                    while *i < RAD*RAD*RAD && work < max_work {
                        let p = (
                            x + *i / (RAD*RAD),
                            y + (*i / RAD) % RAD,
                            z + *i % RAD
                        ).into();
                        if self.blocks.chunks.get(&p).unwrap().chunk.chunk_state == ChunkState::Filled {
                            super::gen::gen_detail(p, self, reg);
                            work += 1;
                        }
                        // println!("detailed for {:?}",p);
                        *i += 1;
                    }
                    if *i == RAD*RAD*RAD {
                        let pos = pos + Vector3{x:1,y:1,z:1}.into();
                        println!("Meshing from {:?}...", pos);
                        loading = Loading::Meshing(0, pos);
                    }
                },
                Loading::Meshing(ref mut i, pos) => {
                    let (x,y,z) = pos.as_tuple();
                    const RAD: i32 = DIAMETER-4;
                    while *i < RAD*RAD*RAD && work < max_work {
                        let p = (
                            x + *i / (RAD*RAD),
                            y + (*i / RAD) % RAD,
                            z + *i % RAD
                        ).into();
                        {
                            let (m1, m2) = meshing::make_mesh(p, &self.blocks, reg);
                            let c = self.blocks.chunks.get_mut(&p).unwrap();
                            if let Some(mesh) = &mut c.chunk.mesh {
                                mesh.0.update_lit(&m1.0, &m1.1, &m1.2);
                                mesh.1.update_lit(&m2.0, &m2.1, &m2.2);
                            } else {
                                c.chunk.mesh = Some((
                                    VAO::textured_lit(&m1.0, &m1.1, &m1.2),
                                    VAO::textured_lit(&m2.0, &m2.1, &m2.2)
                                ));
                            }
                            c.chunk.needs_refresh = false;
                            c.chunk.chunk_state = ChunkState::Rendered;
                        }
                        // self.chunks.get_mut(&p).unwrap().refresh(reg);
                        // println!("detailed for {:?}",p);
                        work += 1;
                        *i += 1;
                    }
                    if *i == RAD*RAD*RAD {
                        println!("Done loading");
                        return
                    }
                },
            }
            self.to_load.push_front(loading);
        }
    }

}


