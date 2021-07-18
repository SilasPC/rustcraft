
use crate::prelude::*;

#[derive(Eq, PartialEq, Clone, Debug, Hash, serde::Deserialize)]
pub struct ItemData {
    pub id: ArcStr,
    pub name: String,
    pub texture: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Item(Arc<(ItemData,bool)>);

impl Item {
    pub fn new_registered_as_shared(data: ItemData) -> Self {
        Self(Arc::new((data,true)))
    }
    pub fn new_not_shared(data: ItemData) -> Self {
        Self(Arc::new((data,false)))
    }
    pub fn mutate(&mut self) -> &mut ItemData {
        let mt = Arc::make_mut(&mut self.0);
        mt.1 = false;
        &mut mt.0
    }
    pub fn is_shared(&self) -> bool {self.0.1}
    pub fn ptr_eq(&self, rhs: &Self) -> bool {Arc::ptr_eq(&self.0, &rhs.0)}
    pub unsafe fn inc_arc_count(&self) {
        Arc::increment_strong_count(&self.0)
    }
    pub unsafe fn dec_arc_count(&self) {
        Arc::decrement_strong_count(&self.0)
    }
}

impl AsRef<ItemData> for Item {
    fn as_ref(&self) -> &ItemData {
        &self.0.0
    }
}


impl std::ops::Deref for Item {
    type Target = ItemData;
    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}

