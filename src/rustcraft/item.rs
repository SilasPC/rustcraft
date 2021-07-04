
use std::sync::Arc;
use crate::rustcraft::block::*;

#[derive(Eq, PartialEq, Clone, Debug, Hash, serde::Deserialize)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub texture: usize,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub enum ItemLike {
    Block(Arc<Block>),
    Item(Arc<Item>)
}

impl From<Arc<Block>> for ItemLike {
    fn from(arc: Arc<Block>) -> Self {
        Self::Block(arc)
    }
}
impl From<Arc<Item>> for ItemLike {
    fn from(arc: Arc<Item>) -> Self {
        Self::Item(arc)
    }
}

impl ItemLike {
    pub fn id(&self) -> usize {
        match self {
            Self::Block(inner) => inner.id,
            Self::Item(inner) => inner.id,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Block(inner) => &inner.name,
            Self::Item(inner) => &inner.name,
        }
    }
    pub fn as_block(&self) -> Option<&Arc<Block>> {
        match self {
            Self::Block(inner) => Some(inner),
            Self::Item(_) => None
        }
    }
    pub fn as_item(&self) -> Option<&Arc<Item>> {
        match self {
            Self::Item(inner) => Some(inner),
            Self::Block(_) => None
        }
    }
    pub fn as_block_mut(&mut self) -> Option<&mut Arc<Block>> {
        match self {
            Self::Block(inner) => Some(inner),
            Self::Item(_) => None
        }
    }
    pub fn as_item_mut(&mut self) -> Option<&mut Arc<Item>> {
        match self {
            Self::Item(inner) => Some(inner),
            Self::Block(_) => None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub count: usize,
    pub item: ItemLike,
}

impl ItemStack {

    pub fn of(item: impl Into<ItemLike>, count: usize) -> Self {
        Self { item: item.into(), count }
    }

    pub fn stack_of(item: impl Into<ItemLike>) -> Self {
        Self { item: item.into(), count: 64 }
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

    pub fn transfer_no_swap(from: &mut Option<ItemStack>, into: &mut Option<ItemStack>) {
        if from.is_some() && into.is_some() {
            let a = from.as_mut().unwrap();
            let b = into.as_mut().unwrap();
            if a.item == b.item {
                let to_move = a.count.min(64-b.count);
                b.count += to_move;
                a.count -= to_move;
                if a.count == 0 {
                    *from = None
                }
            }
        } else if into.is_none() {
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