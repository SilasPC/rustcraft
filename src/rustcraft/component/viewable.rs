
use cgmath::*;

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
}
