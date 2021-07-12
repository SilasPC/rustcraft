
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
    pub fn id(&self) -> &str {
        match self {
            Self::Block(inner) => inner.id.as_ref(),
            Self::Item(inner) => inner.id.as_ref(),
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Block(inner) => &inner.name,
            Self::Item(inner) => &inner.name,
        }
    }
    pub fn is_block(&self) -> bool {
        match self {
            Self::Block(_) => true,
            _ => false
        }
    }
    pub fn to_block(&self) -> Option<Block> {
        self.as_block().cloned()
    }
    pub fn as_block(&self) -> Option<&Block> {
        match self {
            Self::Block(inner) => Some(inner),
            Self::Item(_) => None
        }
    }
    pub fn is_item(&self) -> bool {
        match self {
            Self::Item(_) => true,
            _ => false
        }
    }
    pub fn to_item(&self) -> Option<Item> {
        self.as_item().cloned()
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