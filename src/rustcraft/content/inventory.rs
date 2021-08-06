
use crate::util::fdiv;
use crate::rustcraft::inventory::*;
use crate::render_gui::ItemGUIRenderer;
use crate::registry::ItemRegistry;
use crate::player::inventory::PlayerInventory;
use crate::engine::texture::Texture;
use crate::engine::gui::{gui::*, render::*};

use crate::prelude::*;

pub struct PlayerGUI {
    pub hotbar: PlayerHotbarShell,
    pub selector: Texture,
    pub crosshairs: Texture,
    pub inventory: PlayerInventoryShell,
    pub selected_slot: i32,
}

impl PlayerGUI {
    pub fn new() -> Self {

        let hotbar = player_hotbar();
        let selector = Texture::from_path("assets/item_selected.png").into();
        let crosshairs = Texture::from_path("assets/crosshairs.png").into();
        let inventory = player_inventory();

        /* let heart = std::rc::Rc::from(Texture::from_path("assets/heart.png"));
        let mut hearts = vec![];
        for i in 0..10 {
            hearts.push(FlatGUI {
                texture: std::rc::Rc::clone(&heart),
                pos: Vector2 {x: -1.3/2., y: -0.8 },
            })
        }
        let hearts = ContainerGUI(hearts); */

        Self {
            selected_slot: 0i32,
            hotbar,
            crosshairs,
            selector,
            inventory
        }

    }

    pub fn scroll(&mut self, scroll: i32) {
        self.selected_slot = ((self.selected_slot + scroll) % 9 + 9) % 9
    }

    pub fn selected_slot(&self) -> usize {
        self.selected_slot as usize
    }
}

#[derive(Clone)]
pub struct PlayerHotbarShell {
    pub texture: Arc<Texture>,
    pub slots: Vec<PixelPos>,
}

impl InventoryShell for PlayerHotbarShell {
    fn dyn_clone(&self) -> Box<dyn InventoryShell> {box self.clone() as Box<dyn InventoryShell>}
    fn texture(&self) -> &Texture {&self.texture}
    fn slots(&self) -> &[PixelPos] {self.slots.as_ref()}
    fn borrow_data<'w>(&self, w: &'w mut WorldData) -> Option<&'w mut dyn InventoryData> {
        let pdata: &mut PlayerData = w.entities.ecs.query_one_mut::<&mut PlayerData>(w.entities.player).ok()?;
        let d: &mut dyn InventoryData = &mut pdata.inventory.data as &mut dyn InventoryData;
        Some(d)
    }
}

#[derive(Clone)]
pub struct PlayerInventoryShell {
    pub texture: Arc<Texture>,
    pub slots: Vec<PixelPos>,
}

impl InventoryShell for PlayerInventoryShell {
    fn dyn_clone(&self) -> Box<dyn InventoryShell> {box self.clone() as Box<dyn InventoryShell>}
    fn texture(&self) -> &Texture {&self.texture}
    fn slots(&self) -> &[PixelPos] {self.slots.as_ref()}
    fn borrow_data<'w>(&self, w: &'w mut WorldData) -> Option<&'w mut dyn InventoryData> {
        let pdata: &mut PlayerData = w.entities.ecs.query_one_mut::<&mut PlayerData>(w.entities.player).ok()?;
        let d: &mut dyn InventoryData = &mut pdata.inventory.data as &mut dyn InventoryData;
        Some(d)
    }
}


pub fn player_hotbar() -> PlayerHotbarShell {
    let texture = Texture::from_path("assets/item_bar.png").into();

    let mut slots = vec![];

    let mut c = (4, 4);
    for i in 0..9 {
        slots.push(c.into());
        c.0 += 20;
    }

    /* fn slot_at(p: PixelPos) -> Option<usize> {

        fn grid(x: i32, y: i32) -> Option<(i32, i32)> {
            if x.rem_euclid(20) < 16 && y.rem_euclid(20) < 16 {
                (fdiv(x,20),fdiv(y,20)).into()
            } else {
                None
            }
        }
            
        let mut p = p.0;
        p.0 -= 4;
        p.1 -= 4;
        if let Some((x@0..=8,0)) = grid(p.0, p.1) {
            Some(x as usize)
        } else {
            None
        }

    } */
    PlayerHotbarShell {
        texture,
        slots,
        /* slot_at, */
    }
}

pub fn player_inventory() -> PlayerInventoryShell {
    let texture = Texture::from_path("assets/inventory.png").into();

    let mut slots = vec![];

    let mut c = (4, 4);
    for i in 0..9 {
        slots.push(c.into());
        c.0 += 20;
    }

    c = (4,4);
    c.1 += 25 + 2 * 20;
    for i in 0..27 {
        if i % 9 == 0 && i > 0 {
            c.0 -= 9 * 20;
            c.1 -= 20;
        }

        slots.push(c.into());

        c.0 += 20;
    }

    /* fn slot_at(p: PixelPos) -> Option<usize> {

        fn grid(x: i32, y: i32) -> Option<(i32, i32)> {
            if x.rem_euclid(20) < 16 && y.rem_euclid(20) < 16 {
                (fdiv(x,20),fdiv(y,20)).into()
            } else {
                None
            }
        }
            
        let mut p = p.0;
        p.0 -= 4;
        p.1 -= 4;
        if let Some((x@0..=8,0)) = grid(p.0, p.1) {
            Some(x as usize)
        } else {
            p.1 -= 25;
            if let Some((x@0..=8, y@0..=2)) = grid(p.0, p.1) {
                Some(x as usize + 9 * (2 - y) as usize + 9)
            } else {
                None
            }
        }

    } */
    PlayerInventoryShell {
        texture,
        slots,
    }
}

#[derive(Clone)]
pub struct ChestGUI {
    pub texture: Arc<Texture>,
    pub slots: Vec<PixelPos>,
    pub chest: BlockPos,
}

impl ChestGUI {
    pub fn new() -> Self {

        let texture = Texture::from_path("assets/chest.png").into();

        let mut slots = vec![];

        let mut c = (4, 4);
        c.1 += 2 * 20;
        for i in 0..27 {
            if i % 9 == 0 && i > 0 {
                c.0 -= 9 * 20;
                c.1 -= 20;
            }

            slots.push(c.into());

            c.0 += 20;
        }

        Self {
            texture,
            slots,
            chest: BlockPos::zero()
        }
        
    }
}

impl InventoryShell for ChestGUI {
    fn dyn_clone(&self) -> Box<dyn InventoryShell> {box self.clone() as Box<dyn InventoryShell>}
    fn texture(&self) -> &Texture {&self.texture}
    fn slots(&self) -> &[PixelPos] {&self.slots}
    fn borrow_data<'w>(&self, w: &'w mut WorldData) -> Option<&'w mut dyn InventoryData> {
        let block = w.blocks.block_at_mut_unguarded(&self.chest)?;
        if block.id.as_ref() != "chest" {
            return None
        }
        Some(block.mutate().data.as_mut().unwrap())
    }
}

#[derive(Clone)]
pub struct CraftingGUI {
    pub texture: Arc<Texture>,
    pub slots: [PixelPos; 10],
}

impl CraftingGUI {
    pub fn new() -> Self {
        CraftingGUI {
            texture: Texture::from_path("assets/craft_big.png").into(),
            slots: [
                (44,59).into(),(64,59).into(),(84,59).into(),
                (44,39).into(),(64,39).into(),(84,39).into(),
                (44,19).into(),(64,19).into(),(84,19).into(),
                (124,38).into()
            ]
        }
    }
}

impl InventoryShell for CraftingGUI {
    fn dyn_clone(&self) -> Box<dyn InventoryShell> {box self.clone() as Box<dyn InventoryShell>}
    fn texture(&self) -> &Texture {&self.texture}
    fn slots(&self) -> &[PixelPos] {&self.slots}
    fn borrow_data<'w>(&self, w: &'w mut WorldData) -> Option<&'w mut dyn InventoryData> {
        compile_warning!(wrong);
        let pdata: &mut PlayerData = w.entities.ecs.query_one_mut::<&mut PlayerData>(w.entities.player).ok()?;
        let d: &mut dyn InventoryData = &mut pdata.inventory.data as &mut dyn InventoryData;
        Some(d)
    }
}

impl<'a> InventoryData for Value {
    fn slot(&self, slot: usize) -> &Option<ItemStack> {
        self.as_arr()[slot].as_item_stack()
    }
    fn slot_mut(&mut self, slot: usize) -> &mut Option<ItemStack> {
        self.as_arr_mut()[slot].as_item_stack_mut()
    }
}