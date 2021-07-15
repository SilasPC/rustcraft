
use super::*;

#[derive(Clone)]
pub struct View {
    offset: Vector3<f32>,
}

impl From<Vector3<f32>> for View {
    fn from(offset: Vector3<f32>) -> Self {
        Self { offset }
    }
}

impl View {
    pub fn offset(&self) -> Vector3<f32> {self.offset}

    pub fn calc_view_mat(&self, pos: &Position) -> Matrix4<f32> {
        Matrix4::from(pos.rot) * Matrix4::from_translation(-pos.pos.0-self.offset())
    }

}
