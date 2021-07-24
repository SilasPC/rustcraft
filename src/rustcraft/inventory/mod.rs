
pub mod render;

use crate::util::fdiv;
use crate::gui::render::Cursor;
use crate::player::inventory::PlayerInventory;
use crate::gui::render::GUIRenderer;
use crate::prelude::*;

#[derive(Clone)]
pub struct InventoryGUI {
    pub texture: Arc<Texture>,
    pub slots: Vec<PixelPos>,
    pub slot_at: fn(p: PixelPos) -> Option<usize>,
}

impl InventoryShell for InventoryGUI {
    fn dyn_clone(&self) -> Box<dyn InventoryShell> {box self.clone() as Box<dyn InventoryShell>}
    fn texture(&self) -> &Texture {&self.texture}
    fn slots(&self) -> &[PixelPos] {self.slots.as_ref()}
    fn slot_at(&self, p: PixelPos) -> Option<usize> {(self.slot_at)(p)}
    fn borrow_data<'w>(&self, w: &'w mut WorldData) -> Option<&'w mut dyn InventoryData> {
        compile_warning!(wrong?);
        let pdata: &mut PlayerData = w.entities.ecs.query_one_mut::<&mut PlayerData>(w.entities.player).ok()?;
        let d: &mut dyn InventoryData = &mut pdata.inventory.data as &mut dyn InventoryData;
        Some(d)
    }
}

pub trait InventoryShell {
    fn dyn_clone(&self) -> Box<dyn InventoryShell>;
    fn texture(&self) -> &Texture;
    fn slots(&self) -> &[PixelPos];
    fn slot_at(&self, p: PixelPos) -> Option<usize> {
        self.slots()
            .iter()
            .enumerate()
            .find_map(|(slot,sp)|
                Some(slot).filter(|_|
                    (sp.0.0..sp.0.0+16).contains(&p.0.0) &&
                    (sp.0.1..sp.0.1+16).contains(&p.0.1)
                )
            )
    }
    fn borrow_data<'w>(&self, w: &'w mut WorldData) -> Option<&'w mut dyn InventoryData>;
    fn on_close(&self, w: &mut WorldData) {}
}
impl InventoryData for Vec<Option<ItemStack>> {
    fn slot(&self, slot: usize) -> &Option<ItemStack> {
        &self[slot]
    }
    fn slot_mut(&mut self, slot: usize) -> &mut Option<ItemStack> {
        &mut self[slot]
    }
}
impl InventoryData for [Option<ItemStack>] {
    fn slot(&self, slot: usize) -> &Option<ItemStack> {
        &self[slot]
    }
    fn slot_mut(&mut self, slot: usize) -> &mut Option<ItemStack> {
        &mut self[slot]
    }
}
impl InventoryData for &mut [Option<ItemStack>] {
    fn slot(&self, slot: usize) -> &Option<ItemStack> {
        &self[slot]
    }
    fn slot_mut(&mut self, slot: usize) -> &mut Option<ItemStack> {
        &mut self[slot]
    }
}

pub trait InventoryData {
    /// Get a reference to the slot data
    fn slot(&self, slot: usize) -> &Option<ItemStack>;
    /// Get a mutable reference to the slot data
    fn slot_mut(&mut self, slot: usize) -> &mut Option<ItemStack>;
}

impl InventoryData for &mut dyn InventoryData {
    fn slot(&self, slot: usize) -> &Option<ItemStack> {(**self).slot(slot)}
    fn slot_mut(&mut self, slot: usize) -> &mut Option<ItemStack> {(**self).slot_mut(slot)}
}