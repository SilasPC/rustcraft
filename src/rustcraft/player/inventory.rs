
use crate::Registry;
use crate::crafting::CraftingRegistry;
use crate::rustcraft::item::*;
use crate::engine::gui::{render::*, gui::*};

#[derive(Default)]
pub struct PlayerInventory {
   pub hotbar: [Option<ItemStack>; 9],
   pub inventory: [Option<ItemStack>; 27],
   pub crafting: [Option<ItemStack>; 10],
}

impl PlayerInventory {
    
    pub fn new() -> Self {Self::default()}

    pub fn slot(&mut self, slot: u32) -> &Option<ItemStack> {
        match slot {
            0..=8 => &mut self.hotbar[slot as usize],
            9..=35 => &mut self.inventory[slot as usize - 9],
            _ => panic!("Invalid slot no. {}", slot)
        }
    }

    pub fn transfer(&mut self, slot: u32, from: &mut Option<ItemStack>, reg: &Registry, cr: &CraftingRegistry) {
        match slot {
            0..=8 => ItemStack::transfer(from, &mut self.hotbar[slot as usize]),
            9..=35 => ItemStack::transfer(from, &mut self.inventory[slot as usize - 9]),
            36..=44 => {
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
            },
            _ => panic!("Invalid slot no. {}", slot)
        }
    }

    pub fn merge(&mut self, stack: &mut Option<ItemStack>) {
        *stack = self.merge_priv(std::mem::take(stack));
    }
    fn merge_priv(&mut self, stack: Option<ItemStack>) -> Option<ItemStack> {
        let mut stack = stack?;
        for place in self.hotbar.iter_mut().filter(|p| p.is_some()) {
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
        // repeat for inventory
        let mut stack = stack?;
        for place in self.inventory.iter_mut().filter(|p| p.is_some()) {
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
        for place in &mut self.inventory {
            ItemStack::merge(&mut stack, place);
        }
        stack
    }

}