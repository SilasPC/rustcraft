
use crate::prelude::*;
use std::iter::Iterator as Iter;

pub struct ItemGUIRenderer {
    item: VAO,
    block: VAO,
    offsets: HashMap<String, i32>,
}

impl ItemGUIRenderer {
    pub fn generate(reg: &Registry) -> Self {
        let mut offsets = HashMap::new();
        let item = gen_item_vao(
            reg.items.values().filter_map(ItemLike::as_item),
            &mut offsets, reg.texture_atlas.as_ref()
        );
        let block = gen_block_vao(
            reg.items.values().filter_map(ItemLike::as_block),
            &mut offsets, reg.texture_atlas.as_ref()
        );
        Self {
            item,
            block,
            offsets
        }
    }
    pub fn draw(&self, item: &ItemLike) {
        let offset = self.offsets[item.id()];
        if item.is_item() {
            self.item.bind();
            self.item.draw_6(offset);
        } else {
            self.block.bind();
            self.item.draw_18(offset);
        }
    }
}


pub fn gen_block_vao<'a>(b: impl Iter<Item = &'a Block>, m: &mut HashMap<String, i32>, a: &TextureAtlas) -> VAO {

    let mut verts = vec![];
    let mut uvs = vec![];

    let mut offset = 0;

    // six triangles per block item => 18 verts
    for b in b {
        m.insert(b.id.to_string(), offset);
        offset += 1;
        verts.extend_from_slice(&[
            // top
            0.5, 1., 0.,
            0., 0.75, 0.,
            1., 0.75, 0.,
            0., 0.75, 0.,
            0.5, 0.5, 0.,
            1., 0.75, 0.,
            // left
            0., 0.75, 0.,
            0.5, 0., 0.,
            0.5, 0.5, 0.,
            0.5, 0., 0.,
            0., 0.75, 0.,
            0.0, 0.25, 0.,
            // right
            0.5, 0.5, 0.,
            0.5, 0., 0.,
            1., 0.75, 0.,
            0.5, 0., 0.,
            1., 0.25, 0.,
            1., 0.75, 0.,
        ]);
        let (t,s,_) = b.texture;
        let (u,v) = a.get_uv(t);
        let d = a.uv_dif();
        uvs.extend_from_slice(&[
            // top
            u, v,
            u, v+d,
            u+d, v,
            u, v+d,
            u+d, v+d,
            u+d, v,
        ]);
        let (u,v) = a.get_uv(s);
        let d = a.uv_dif();
        uvs.extend_from_slice(&[
            // left
            u, v,
            u+d, v+d,
            u+d, v,
            u+d, v+d,
            u, v,
            u, v+d,
            // right
            u, v,
            u, v+d,
            u+d, v,
            u, v+d,
            u+d, v+d,
            u+d, v,
        ]);
    }

    crate::engine::vao::VAO::textured(&verts, &uvs)

}

pub fn gen_item_vao<'a>(items: impl Iter<Item = &'a Item>, m: &mut HashMap<String, i32>, a: &TextureAtlas) -> VAO {

    let mut verts = vec![];
    let mut uvs = vec![];

    let mut offset = 0;

    // one face => 6 verts
    for item in items {
        m.insert(item.id.to_string(), offset);
        offset += 1;
        verts.extend_from_slice(&[
            0., 0., 0.,
            1., 1., 0.,
            0., 1., 0.,
            1., 1., 0.,
            0., 0., 0.,
            1., 0., 0.,
        ]);
        let (u,v) = a.get_uv(item.texture);
        let d = a.uv_dif();
        uvs.extend_from_slice(&[
            u, v+d,
            u+d, v,
            u, v,
            u+d, v,
            u, v+d,
            u+d, v+d,
        ]);
    }

    crate::engine::vao::VAO::textured(&verts, &uvs)

}