pub mod inventory;

use crate::util::fdiv;
use crate::loader::Loader;
use crate::crafting::CraftingRegistry;
use crate::rustcraft::component::{Physics,Position,PlayerData,View};
use crate::prelude::*;

pub fn make_player() -> (impl hecs::DynamicBundle, util::AABB) {
    let pos = Position::new(Vector3 {x:50., y: 55., z: 50.}.as_coord(), (0.8,1.9,0.8).into());
    let aabb = pos.get_aabb();
    let view = View::from(Vector3 {
        x: 0.5,
        y: 1.8,
        z: 0.5,
    });
    ((pos, Physics::new(), view, PlayerData::new()), aabb)
}

pub fn make_registry(texture_atlas: Arc<TextureAtlas>) -> Arc<Registry> {
    let mut x: SerialItemRegistry = toml::from_str(&std::fs::read_to_string("assets/items.toml").unwrap()).unwrap();
    x.block.sort_by_cached_key(|a| a.id.clone());
    x.item.sort_by_cached_key(|a| a.id.clone());

    /* x.block[3].behavior = Some(Box::new(Behavior {
        on_rnd_tick: Some(grass_update),
        .. Default::default()
    })); */
    /* x.block[8].behavior = Some(Box::new(Behavior {
        on_update: Some(fire_update),
        .. Default::default()
    })); */
    
    let items = x.block.into_iter()
        .map(Block::new_registered_as_shared)
        .map(ItemLike::from)
        .map(|il| (il.id().to_owned(), il))
        .chain(
            x.item.into_iter()
                .map(Item::new_registered_as_shared)
                .map(ItemLike::from)
                .map(|il| (il.id().to_owned(), il))
        ).collect::<HashMap<String, ItemLike>>();
    
    Arc::new(Registry {
        items,
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
            .map(|id| reg.get(&id).clone())
            .map(Option::from)
            .map(|o| o.filter(|item| item.id() != "air"))
            .chain([None].iter().cycle().cloned())
            .take(9)
            .collect();
        creg.register(true, input, ItemStack::of(reg.get(&shaped.output).clone(), 1));
    }
    creg
}

#[derive(serde::Deserialize)]
struct SavedRecipies {
    shaped: Vec<SavedRecipe>,
}

#[derive(serde::Deserialize)]
struct SavedRecipe {
    input: Vec<String>,
    output: String,
    #[serde(default = "one")]
    count: usize
}

fn chest_use(pos: BlockPos, data: &mut WorldData) {
    // TODO somehow trigger inventory to open
}

const fn one() -> usize {1}
/* 
fn grass_update(mut pos: BlockPos, data: &mut WorldData) {
    pos.0.y += 1;
    let mut turn_to_dirt = false;
    if let Some(block) = world.block_at(&pos) {
        turn_to_dirt = block.solid || !block.transparent;
    }
    if turn_to_dirt {
        pos.0.y -= 1;
        world.set_block_at(&pos, data.registry.get("dirt").as_block().unwrap());
    }
}

fn fire_update(pos: BlockPos, data: &mut WorldData) {
    use rand::Rng;
    let mut r = rand::thread_rng();
    if r.gen::<f32>() < 0.9 {
        world.to_update.push(pos);
        return
    }
    let fire = data.registry.get("glowstone").as_block().unwrap(); // tmp glowstone
    let air = data.registry.get("air").as_block().unwrap();
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
        data.world.set_block_at(&pos, &air);
        data.world.set_block_at(&below, &air);
        test!((0,-2,0), 0.5);
    } else {
        data.world.to_update.push(pos);
    }
} */
