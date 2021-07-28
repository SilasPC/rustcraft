
use crate::content::SavedRecipies;
use crate::content::SerialItemRegistry;
use crate::prelude::*;

pub fn register_components(cnt: &mut ContentBuilder) {
    cnt.components.register("Physics".into(), Physics::new());
    compile_warning!(more stuff);
}

pub fn register_entities(cnt: &mut ContentBuilder) {
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

pub fn register_items(reg: &mut ContentBuilder) {
    let map = SerialItemRegistry::from_path("base/items.toml").into_map();
    for (k, v) in map {
        assert!(reg.items.insert(k, v).is_none())
    }
}

pub fn register_recipies(reg: &mut ContentBuilder) {
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
