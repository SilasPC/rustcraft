
use crate::rustcraft::block::*;

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub count: usize,
    pub item: std::sync::Arc<Block>,
}

impl ItemStack {

    pub fn of(item: std::sync::Arc<Block>, count: usize) -> Self {
        Self { item, count }
    }

    pub fn stack_of(item: std::sync::Arc<Block>) -> Self {
        Self { item, count: 64 }
    }

    pub fn deduct(item: &mut Option<ItemStack>, num: usize) {
        match item {
            Some(ref mut inner) => {
                inner.count -= inner.count.min(num);
                if inner.count == 0 {
                    *item = None;
                }
            },
            None => {}
        }
    }

    pub fn split(item: &mut Option<ItemStack>) -> Option<ItemStack> {
        match item {
            Some(ref mut inner) => {
                if inner.count <= 1 {
                    std::mem::replace(item, None)
                } else {
                    let mut count = inner.count;
                    inner.count >>= 1;
                    count -= inner.count;
                    ItemStack {
                        count,
                        item: inner.item.clone(),
                    }.into()
                }
            },
            _ => None,
        }
    }

    pub fn merge(from: Option<ItemStack>, into: &mut Option<ItemStack>) -> Option<ItemStack> {
        if from.is_none() {return None}
        match into {
            Some(ref mut inner) => {
                let mut from = from.unwrap();
                let to_move = from.count.min(64-inner.count);
                inner.count += to_move;
                from.count -= to_move;
                if from.count > 0 {
                    Some(from)
                } else {
                    None
                }
            },
            _ => {
                *into = from;
                None
            }
        }
    }
    
}