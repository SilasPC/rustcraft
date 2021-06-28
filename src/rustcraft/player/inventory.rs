
use crate::rustcraft::item::*;
use crate::engine::gui::{render::*, gui::*};

#[derive(Default)]
pub struct PlayerInventory {
   pub hotbar: [Option<ItemStack>; 9],
   pub inventory: [Option<ItemStack>; 27],
}

impl PlayerInventory {
    
    pub fn new() -> Self {Self::default()}

    pub fn slot_mut(&mut self, slot: u32) -> &mut Option<ItemStack> {
        match slot {
            0..=8 => &mut self.hotbar[slot as usize],
            9..=35 => &mut self.inventory[slot as usize - 9],
            _ => panic!("Invalid slot no. {}", slot)
        }
    }

    #[must_use]
    pub fn merge(&mut self, stack: Option<ItemStack>) -> Option<ItemStack> {
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