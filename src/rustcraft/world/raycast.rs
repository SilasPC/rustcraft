
use crate::world::VoxelData;
use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct RayCastHit {
    pub hit: WorldPos,
    pub prev: WorldPos,
}

impl VoxelData {
    
    pub fn raycast(&self, mut pos: WorldPos, heading: &Vector3<f32>, max_dist: f32) -> Option<RayCastHit> {
        
        let mut dist = 0.;
        while dist < max_dist && !self.check_hit(&pos) {
            dist += 0.1;
            pos.0 += 0.1 * heading;
        }

        if dist < max_dist {
            let hit = pos;
            let mut prev = pos;
            prev.0 -= 0.1 * heading;
            compile_warning!(not always adjacent);
            /* if !prev.adjacent_to(&hit.as_block()) {
                dbg!((prev,hit));
            } */

            return Some(RayCastHit {
                hit,
                prev
            });
        } else {
            return None
        }

    }

    fn check_hit(&self, pos: &impl Coord) -> bool {
        self.block_at(pos)
            .map(|b| b.solid)
            .unwrap_or(false)
    }

}