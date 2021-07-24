
use crate::prelude::*;
use crate::engine::text::text::Text;
use crate::ItemStack;
use crate::rustcraft::inventory::*;

pub enum GameState {
    Inventory {
        //aux_inventory: Box<dyn InventoryShell>,
        inventory: Box<dyn InventoryShell>,
        picked_item: Option<ItemStack>,
    },
    Playing {
        breaking: Option<(f32, BlockPos)>
    },
    Paused,
    Chat {
        start_frame: Instant,
        text: Text,
    }
}

impl GameState {
    pub fn is_playing(&self) -> bool {
        match self { Self::Playing {..} => true, _ => false }
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
