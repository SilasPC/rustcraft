
use crate::prelude::*;
use inventory::*;

pub struct PlayerInventory {
    pub data: Vec<Option<ItemStack>>,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self {
            data: vec![None; 36]
        }
    }
}

impl InventoryData for PlayerInventory {
    fn slot(&self, slot: usize) -> &Option<ItemStack> {
        &self.data[slot]
    }
    fn slot_mut(&mut self, slot: usize) -> &mut Option<ItemStack> {
        &mut self.data[slot]
    }
}

impl PlayerInventory {

    pub fn slot(&mut self, slot: u32) -> &Option<ItemStack> {
        match slot {
            0..=35 => &self.data[slot as usize],
            _ => panic!("Invalid slot no. {}", slot)
        }
    }

    pub fn transfer(&mut self, slot: u32, from: &mut Option<ItemStack>, reg: &ItemRegistry, cr: &CraftingRegistry) {
        match slot {
            0..=35 => ItemStack::transfer(from, &mut self.data[slot as usize]),
            /* 36..=44 => {
                ItemStack::transfer(from, &mut self.crafting[slot as usize - 36]);
                self.crafting[9] = cr.search(&self.crafting[..9]).cloned();
            },
            45 => {
                if self.crafting[9].is_some() {
                    ItemStack::transfer_no_swap(&mut self.crafting[9], from);
                    for stack in self.crafting.iter_mut().take(9) {
                        ItemStack::deduct(stack, 1);
                    }
                    self.crafting[9] = cr.search(&self.crafting[..9]).cloned();
                }
            }, */
            _ => panic!("Invalid slot no. {}", slot)
        }
    }

    pub fn merge(&mut self, stack: &mut Option<ItemStack>) {
        for place in &mut self.data {
            ItemStack::merge(stack, place);
            if stack.is_none() {return}
        }
        //*stack = self.merge_priv(std::mem::take(stack));
    }
    /* fn merge_priv(&mut self, data: &mut [Option<ItemStack>], stack: Option<ItemStack>) -> Option<ItemStack> {
        let mut stack = stack?;
        for place in data.iter_mut().filter(|p| p.is_some()) {
            if place.as_ref().unwrap().item == stack.item {
                let mut maybe_stack = Some(stack);
                ItemStack::merge(&mut maybe_stack, place);
                if maybe_stack.is_some() {
                    stack = maybe_stack.unwrap();
                } else {
                    return None;
                }
            }
        }
        let mut stack = Some(stack);
        for place in &mut self.hotbar {
            ItemStack::merge(&mut stack, place);
        }
        stack
    } */

}