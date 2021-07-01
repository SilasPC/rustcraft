
use crate::Registry;
use std::sync::Arc;
use crate::item::ItemStack;

pub struct CraftingRegistry;

impl CraftingRegistry {
    pub fn get_output(&self, input: &[Option<ItemStack>], reg: &Registry) -> Option<ItemStack> {
        if input.iter().skip(1).any(Option::is_some) || input[0].is_none() {
            return None
        };
        let stack = input[0].as_ref().unwrap();
        match stack.item.id() {
            2 => Some(ItemStack::of(reg.get(7), 1)),
            _ => None
        } 
    } 
}
