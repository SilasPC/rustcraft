
use crate::player::inventory::PlayerInventory;

#[derive(Default)]
pub struct PlayerData {
    pub inventory: PlayerInventory,
}

impl PlayerData {

    pub fn new() -> Self {Self::default()}

}