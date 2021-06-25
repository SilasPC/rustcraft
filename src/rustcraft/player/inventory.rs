
use crate::rustcraft::item::*;
use crate::engine::gui::{render::*, gui::*};

pub struct PlayerInventory {
   pub hotbar: [Option<ItemStack>; 9],
}

impl PlayerInventory {
    
    pub fn new() -> Self {
        Self {
            hotbar: [None, None, None, None, None, None, None, None, None]
        }
    }

    pub fn merge(&mut self, stack: Option<ItemStack>) -> Option<ItemStack> {
        let mut stack = stack?;
        for place in self.hotbar.iter_mut().filter(|p| p.is_some()) {
            if place.as_ref().map(|p| p.item.id).unwrap() == stack.item.id {
                let maybe_stack = ItemStack::merge(Some(stack), place);
                if maybe_stack.is_some() {
                    stack = maybe_stack.unwrap();
                } else {
                    return None;
                }
            }
        }
        let mut stack = Some(stack);
        for place in &mut self.hotbar {
            stack = ItemStack::merge(stack, place);
        }
        stack
    }

}