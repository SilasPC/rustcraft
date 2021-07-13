
use crate::game_loop::InventoryGUI;
use crate::ItemStack;
use crate::game_loop::Text;
use std::time::Instant;
use std::sync::Arc;

pub enum GameState {
    Inventory {
        inventory: Arc<InventoryGUI>,
        picked_item: Option<ItemStack>,
    },
    Playing,
    Paused,
    Chat {
        start_frame: Instant,
        text: Text,
    }
}

impl GameState {
    pub fn is_playing(&self) -> bool {
        match self { Self::Playing => true, _ => false }
    }
    pub fn is_paused(&self) -> bool {
        match self { Self::Paused {..} => true, _ => false }
    }
    pub fn is_chat(&self) -> bool {
        match self { Self::Chat {..} => true, _ => false }
    }
    pub fn show_inventory(&self) -> bool {
        match self {
            Self::Inventory {..} => true,
            _ => false
        }
    }
}
