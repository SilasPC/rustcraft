
use crate::static_prg::StaticProgram;
use rand::prelude::*;
use super::*;

#[derive(derive_more::From)]
pub struct Model {
    pub model: Arc<VAO>,
}

impl Model {
    pub fn system_render(data: &mut crate::Data, program: &mut StaticProgram) {
        for (ent, (model, pos)) in data.ecs.query_mut::<(&Model, &Position)>() {
            program.load_transform(&Matrix4::from_translation(pos.pos.0));
            model.model.bind();
            model.model.draw();
        }
    }
}