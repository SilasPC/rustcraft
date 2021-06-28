
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

    pub fn transfer(from: &mut Option<ItemStack>, into: &mut Option<ItemStack>) {
        if from.is_some() && into.is_some() {
            let a = from.as_mut().unwrap();
            let b = into.as_mut().unwrap();
            if a.item != b.item {
                std::mem::swap(a, b)
            } else {
                let to_move = a.count.min(64-b.count);
                b.count += to_move;
                a.count -= to_move;
                if a.count == 0 {
                    *from = None
                }
            }
        } else {
            std::mem::swap(from, into)
        }
    }

    pub fn merge(from: &mut Option<ItemStack>, into: &mut Option<ItemStack>) {
        if from.is_none() {return}
        match into {
            Some(ref mut inner) => {
                let mut from_inner = from.as_mut().unwrap();
                if from_inner.item != inner.item {
                    return
                }
                let to_move = from_inner.count.min(64-inner.count);
                inner.count += to_move;
                from_inner.count -= to_move;
                if from_inner.count == 0 {
                    *from = None
                }
            },
            _ => {
                std::mem::swap(from, into)
            }
        }
    }
    
}