
use crate::content::SavedRecipies;
use crate::content::SerialItemRegistry;
use crate::prelude::*;
use super::builder::*;

pub struct BaseMod;
impl ContentMod for BaseMod {

    fn name(&mut self) -> &str {"rustcraft"}
    
    fn register_components(&mut self, cnt: &mut ContentBuilder) {
        cnt.components.register("Physics".into(), Physics::new());
        cnt.components.register("Viewable".into(), View::default());
        compile_warning!(more stuff);
    }
    
    fn register_entities(&mut self, cnt: &mut ContentBuilder) {
        let ents = std::fs::read_to_string("base/entities.toml").unwrap();
        let ents = ents.parse::<toml::Value>().unwrap();
        let ents = ents.as_table().unwrap();
        for (k, v) in ents {
            let cmp_opts = v.as_table().unwrap();
            let mut cmps = vec![];
            for (cmp, opt) in cmp_opts {
                cmps.push(cnt.components.get(cmp));
            }
            assert!(cnt.entities.entities.insert(k.to_owned(), EntityTemplate {
                cmps
            }).is_none());
        };
    }

    fn register_behaviors(&mut self, cnt: &mut ContentBuilder) {
        assert!(
            cnt.behaviors.behaviors.insert("update/gravity".into(), FallingBlock::behaviour_on_update)
            .is_none()
        );
    }
    
    fn register_items(&mut self, reg: &mut ContentBuilder) {
        let map = SerialItemRegistry::from_path("base/items.toml").into_map();
        for (k, v) in map {
            assert!(reg.items.insert(k, v).is_none())
        }
        macro_rules! mut_shr {
            ($id:expr) => {
                reg.items.get_mut($id)
                    .unwrap()
                    .as_block_mut()
                    .unwrap()
                    .mutate_shared()
            };
        }
        mut_shr!("sand").behavior = Some(box Behavior {
            on_update: Some(FallingBlock::behaviour_on_update),
            ..Default::default()
        });
        /* mut_shr!("glowstone").behavior = Some(box Behavior {
            on_update: Some(rnd_glow_dec),
            ..Default::default()
        }); */
        /* add_behavior!("grass", Behavior {
            on_rnd_tick: Some(grass_rnd_tick),
            ..Default::default()
        }); */
    }
    
    fn register_recipies(&mut self, reg: &mut ContentBuilder) {
        let tomlstr = std::fs::read_to_string("base/recipies.toml").unwrap();
        let saved: SavedRecipies = toml::from_str(tomlstr.as_str()).unwrap();
        for shaped in saved.shaped {
            let input: Vec<Option<ItemLike>> = shaped.input
                .into_iter()
                .map(|id: String| reg.items.get(&id).cloned())
                .map(|o: Option<ItemLike>| o.filter(|item| item.id() != "air"))
                .chain([None].iter().cycle().cloned())
                .take(9)
                .collect();
            reg.crafting.register(true, input, ItemStack::of(reg.items.get(&shaped.output).cloned().unwrap(), 1));
        }
    }
}

/* fn rnd_glow_dec(pos: BlockPos, world: &mut WorldData) {
    let mut guard = world.blocks.block_at_mut(&pos);
    let b = guard.get_mut().unwrap();
    // println!("{:?} decrease?", b);
    if b.light > 1 {
        let mb = b.mutate();
        mb.light -= 5.min(mb.light);
        // println!("{:?} decreased", pos);
        if mb.light == 0 {
            mb.behavior.as_mut().unwrap().on_rnd_tick = None;
        }
    }
} */

/* fn grass_rnd_tick(pos: BlockPos, world: &mut WorldData) {
    let mut turn_to_dirt = false;
    if let Some(block) = world.blocks.block_at(&pos.shifted(Face::YPos)) {
        turn_to_dirt = block.solid || !block.transparent;
    }
    if turn_to_dirt {
        world.blocks.set_block_at(&pos, data.registry.get("dirt").as_block().unwrap());
    }
} */

/* fn fire_update(pos: BlockPos, data: &mut WorldData) {
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

fn register_base_blocks(blocks: &mut HashMap<String, BlockData>) {
    blocks.insert("air".into(), BlockData {
        id: "air".into(),
        name: "Air".into(),
        hitbox: true,
        no_render: true,
        transparent: true,
        replacable: true,
        solid: false,
        texture: (0, 0, 0),
        ..Default::default()
    }).assert_none();
    blocks.insert("stone".into(), BlockData {
        id: "stone".into(),
        name: "Stone".into(),
        texture: (0, 0, 0),
        drops: Some("cobblestone".into()),
        ..Default::default()
    }).assert_none();
}

trait AssertNone {
    fn assert_none(self);
}

impl<T> AssertNone for Option<T> {
    fn assert_none(self) {
        assert!(!self.is_none());
    }
}
