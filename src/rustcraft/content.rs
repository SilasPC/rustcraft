
use crate::crafting::CraftingRegistry;
use crate::rustcraft::component::{Physics,Position,PlayerData,View};
use crate::prelude::*;

pub fn make_player() -> (impl hecs::DynamicBundle, util::AABB) {
    let pos = Position::from(Vector3 {x:50., y: 55., z: 50.}.as_coord());
    let mut phys = Physics::new(Vector3 {
        x: 0.8,
        y: 1.9,
        z: 0.8,
    });
    //cam_phys.set_flying(true);
    let aabb = phys.get_aabb(&pos);
    let view = View::from(Vector3 {
        x: 0.5,
        y: 1.8,
        z: 0.5,
    });
    ((pos, phys, view, PlayerData::new()), aabb)
}

pub fn make_registry(texture_atlas: Arc<TextureAtlas>) -> Arc<Registry> {
    let mut x: SerialItemRegistry = toml::from_str(&std::fs::read_to_string("assets/items.toml").unwrap()).unwrap();
    x.block.sort_by_key(|a| a.id);
    x.item.sort_by_key(|a| a.id);

    x.block[3].behavior = Some(Box::new(Behavior {
        on_rnd_tick: Some(grass_update),
        .. Default::default()
    }));
    /* x.block[8].behavior = Some(Box::new(Behavior {
        on_update: Some(fire_update),
        .. Default::default()
    })); */

    assert!(x.block.last().unwrap().id < x.item.first().unwrap().id);
    let blocks: Vec<_> = x.block.into_iter().map(Block::new_registered_as_shared).collect();
    let items: Vec<_> = x.item.into_iter().map(Item::new_registered_as_shared).collect();
    let items_offset = blocks.len();
    Arc::new(Registry {
        item_vao: util::gen_item_vao(&items, &texture_atlas),
        iso_block_vao: util::gen_block_vao(&blocks, &texture_atlas),
        blocks,
        items,
        items_offset,
        texture_atlas,
    })
}

#[derive(serde::Deserialize)]
struct SerialItemRegistry {
    block: Vec<BlockData>,
    item: Vec<ItemData>,
}

pub fn load_recipies(reg: &Registry) -> CraftingRegistry {
    let tomlstr = std::fs::read_to_string("assets/recipies.toml").unwrap();
    let saved: SavedRecipies = toml::from_str(tomlstr.as_str()).unwrap();
    let mut creg = CraftingRegistry::new();
    for shaped in saved.shaped {
        let input = shaped.input
            .into_iter()
            .map(|id| reg.get(id))
            .map(Option::from)
            .chain([None].iter().cycle().cloned())
            .take(9)
            .collect::<Vec<_>>();
        creg.register(true, input.as_slice(), ItemStack::of(reg.get(shaped.output), 1));
    }
    creg
}

#[derive(serde::Deserialize)]
struct SavedRecipies {
    shaped: Vec<SavedRecipe>,
}

#[derive(serde::Deserialize)]
struct SavedRecipe {
    input: Vec<usize>,
    output: usize,
    #[serde(default = "one")]
    count: usize
}

const fn one() -> usize {1}

fn grass_update(mut pos: WorldPos<i32>, data: &mut Data) {
    pos.0.y += 1;
    let mut turn_to_dirt = false;
    if let Some(block) = data.world.block_at(&pos) {
        turn_to_dirt = block.solid || !block.transparent;
    }
    if turn_to_dirt {
        pos.0.y -= 1;
        data.world.set_block_at(&pos, &data.registry[2]);
    }
}

fn fire_update(pos: WorldPos<i32>, data: &mut Data) {
    use rand::Rng;
    let mut r = rand::thread_rng();
    if r.gen::<f32>() < 0.9 {
        data.world.to_update.push(pos);
        return
    }
    let fire = &data.registry[8]; // tmp glowstone
    macro_rules! test {
        ($x:expr) => {test!($x, 0.2)};
        ($x:expr, always) => {{
            let npos = pos + $x.into();
            if data.world.block_at(&npos).map(|b| b.flammable).unwrap_or(false) {
                let above = npos + (0,1,0).into();
                if data.world.replace_at(&above, fire) {
                    data.world.to_update.push(above);
                }
            }
        }};
        ($x:expr, $p:expr) => {{
            if r.gen::<f32>() < $p {
                test!($x, always);
            }
        }};
    }
    for y in -1..=1 {
        test!((1,y,0));
        test!((-1,y,0));
        test!((0,y,1));
        test!((0,y,-1));
    }
    if r.gen::<f32>() < 0.9 {
        let below = pos + (0,-1,0).into();
        data.world.set_block_at(&pos, &data.registry[0]);
        data.world.set_block_at(&below, &data.registry[0]);
        test!((0,-2,0), 0.5);
    } else {
        data.world.to_update.push(pos);
    }
}