
use crate::prelude::*;

pub trait Dimension: World {
    fn skylight(&self) -> f32;

}

pub trait World {
    fn raycast(&self, pos: WorldPos, heading: &V3f) -> Option<RayCastHit>;
    fn replace(&mut self, pos: &impl Coord) -> bool;
    fn get_block(&self, pos: &impl Coord);
    fn set_block(&mut self, pos: &impl Coord) -> bool;
    fn get_light(&self, pos: &impl Coord) -> &Light;
    fn set_light(&mut self, pos: &impl Coord);
    fn get_chunk(&self, pos: &impl Coord) -> &Chunk;
    fn get_chunk_mut(&mut self, pos: &impl Coord) -> &mut Chunk;
}
