
use pathfinding::prelude::*;
use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct FollowEntity {
    target: Option<Entity>,
}

impl FollowEntity {
    pub fn new(target: Option<Entity>) -> Self {
        Self {
            target
        }
    }
    pub fn system_update_tick(data: &mut crate::WorldData) {
        let stuff = data.entities.ecs.query_mut::<(&FollowEntity)>()
            .into_iter()
            .map(|(ent, flw)| (ent,flw.clone()))
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|(ent, flw)| try {
                let pos = data.entities.ecs.query_one_mut::<(&Position)>(flw.target?).ok()?;
                (ent, pos.pos.as_block())
            })
            .collect::<Vec<_>>();
        for (ent, pos) in stuff {
            if let Ok(pf) = data.entities.ecs.query_one_mut::<(&mut PathFinding)>(ent) {
                pf.target = Some(pos);
                pf.path = None;
            }
        }
    }
}

const MAX_RADIUS: f32 = 16.;

pub struct PathFinding {
    target: Option<BlockPos>,
    path: Option<Vec<BlockPos>>,
    search_delay: u32,
}

impl PathFinding {
    pub fn new() -> Self {
        Self {
            target: None,
            path: None,
            search_delay: 0,
        }
    }

    pub fn system_update_tick(data: &mut crate::WorldData) {
        for (_ent, (pos, pf)) in data.entities.ecs.query_mut::<(&mut Position, &mut PathFinding)>() {
            if pf.path.is_none() && pf.target.is_some() {
                if pf.search_delay == 0 {
                    pf.path = create_path(pos.pos.as_block(), pf.target.unwrap(), &data.blocks);
                    pf.search_delay = 40;
                } else {
                    pf.search_delay -= 1;
                }
            } else if let Some(path) = &pf.path {
                println!("got path");
                /* let _: Option<_> = try {
                    pos.pos = pf.path.as_mut().unwrap().pop()?.as_world();
                }; */
            }
        }
    }

}

fn create_path(from: BlockPos, to: BlockPos, data: &VoxelData) -> Option<Vec<BlockPos>> {
    astar(
        &from,
        |pos| {
            let mut ne = Vec::with_capacity(6);
            for pos in itertools::iproduct!([-1,1],[-1,1],[-1,1]).map(|p| *pos + p.into()) {
                let _: Option<_> = try {
                    if !data.block_at(&pos)?.solid && pos.dist(&from) <= MAX_RADIUS {
                        ne.push((pos, 1));
                    }
                };
            }
            ne
        },
        |pos| pos.dist(&to) as u32,
        |pos| *pos == to
    ).map(|(path, _dist)| path)
} 