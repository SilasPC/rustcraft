
use crate::util::Drawable;
use crate::static_prg::StaticProgram;
use rand::prelude::*;
use super::*;

#[derive(derive_more::From)]
pub struct Model {
    pub model: Box<dyn Drawable>
}

impl Model {
    pub fn system_render(data: &mut crate::Data, program: &mut StaticProgram) {
        program.load_light(1.);
        for (ent, (model, pos)) in data.ecs.query_mut::<(&Model, &Position)>() {
            program.load_transform(&(Matrix4::from_translation(pos.pos.0) * Matrix4::from_scale(pos.size.x))); // tmp scale
            model.model.bind();
            model.model.draw();
        }
    }
}