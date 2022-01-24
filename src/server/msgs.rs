
use crate::prelude::*;

pub enum ServerMsg {
    
}

pub enum ClientMsg {
    LoadAround(ChunkPos),
}
