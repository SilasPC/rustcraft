
use crate::prelude::*;

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub enum ItemLike {
    Block(Block),
    Item(Item)
}

impl From<Block> for ItemLike {
    fn from(arc: Block) -> Self {
        Self::Block(arc)
    }
}
impl From<Item> for ItemLike {
    fn from(arc: Item) -> Self {
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
    pub fn as_block(&self) -> Option<&Block> {
        match self {
            Self::Block(inner) => Some(inner),
            Self::Item(_) => None
        }
    }
    pub fn as_item(&self) -> Option<&Item> {
        match self {
            Self::Item(inner) => Some(inner),
            Self::Block(_) => None
        }
    }
    pub fn as_block_mut(&mut self) -> Option<&mut Block> {
        match self {
            Self::Block(inner) => Some(inner),
            Self::Item(_) => None
        }
    }
    pub fn as_item_mut(&mut self) -> Option<&mut Item> {
        match self {
            Self::Item(inner) => Some(inner),
            Self::Block(_) => None
        }
    }
}