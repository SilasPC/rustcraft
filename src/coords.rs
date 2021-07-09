
use std::cmp::Ordering;
use derive_more::*;
use cgmath::*;
use serde::*;

#[derive(Neg, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Copy, Clone, From, Into, Serialize, Deserialize, Hash, PartialEq, Debug)]
pub struct WorldPos<T: cgmath::BaseNum>(pub Vector3<T>);

#[derive(Neg, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Copy, Clone, From, Into, Serialize, Deserialize, Hash, PartialEq, Eq, Debug)]
pub struct ChunkPos(pub Vector3<i32>);

impl<T: cgmath::BaseNum + Eq> Eq for WorldPos<T> {}
impl<T: cgmath::BaseNum + Ord> Ord for WorldPos<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        Into::<(T,T,T)>::into(self.0).cmp(&Into::<(T,T,T)>::into(rhs.0))
    }
}
impl<T: cgmath::BaseNum> PartialOrd for WorldPos<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Into::<(T,T,T)>::into(self.0).partial_cmp(&Into::<(T,T,T)>::into(rhs.0))
    }
}

impl Ord for ChunkPos {
    fn cmp(&self, rhs: &Self) -> Ordering {
        AsRef::<[i32;3]>::as_ref(&self.0).cmp(rhs.0.as_ref())
    }
}
impl PartialOrd for ChunkPos {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        AsRef::<[i32;3]>::as_ref(&self.0).partial_cmp(rhs.0.as_ref())
    }
}

impl<T: cgmath::BaseNum> WorldPos<T> {
    pub fn as_tuple(&self) -> (T,T,T) {
        self.0.into()
    }
}

impl<T: cgmath::BaseNum> std::ops::Deref for WorldPos<T> {
    type Target = Vector3<T>;
    fn deref(&self) -> &Self::Target {&self.0}
}
impl std::ops::Deref for ChunkPos {
    type Target = Vector3<i32>;
    fn deref(&self) -> &Self::Target {&self.0}
}

impl<T: cgmath::BaseNum> From<(T,T,T)> for WorldPos<T> {
    fn from(val: (T,T,T)) -> WorldPos<T> {
        Vector3::from(val).into()
    }
}

impl From<(i32,i32,i32)> for ChunkPos {
    fn from(val: (i32,i32,i32)) -> ChunkPos {
        Vector3::from(val).into()
    }
}

impl Coord for WorldPos<i32> {
    fn as_pos_i32(&self) -> WorldPos<i32> {
        *self
    }
    fn as_pos_f32(&self) -> WorldPos<f32> {
        self.0.map(|v| v as f32).into()
    }
    fn as_chunk(&self) -> ChunkPos {
        self.0.map(|v| (v as f32 / 16.).floor() as i32).into()
    }
    fn as_sub(&self) -> Vector3<usize> {
        self.0.map(|v| v.rem_euclid(16) as usize)
    }
}

impl Coord for WorldPos<f32> {
    fn as_pos_i32(&self) -> WorldPos<i32> {
        self.0.map(|v| v.floor() as i32).into()
    }
    fn as_pos_f32(&self) -> WorldPos<f32> {
        *self
    }
    fn as_chunk(&self) -> ChunkPos {
        self.0.map(|v| (v / 16.).floor() as i32).into()
    }
    fn as_sub(&self) -> Vector3<usize> {
        self.0.map(|v| (v.floor() as i32).rem_euclid(16) as usize)
    }
}

impl ChunkPos {
    pub fn as_tuple(&self) -> (i32,i32,i32) {
        self.0.into()
    }
}
impl Coord for ChunkPos {
    fn as_pos_i32(&self) -> WorldPos<i32> {
        self.0.map(|x| x * 16).into()
    }
    fn as_pos_f32(&self) -> WorldPos<f32> {
        self.0.map(|x| (x * 16) as f32).into()
    }
    fn as_chunk(&self) -> ChunkPos {
        *self
    }
    fn as_sub(&self) -> Vector3<usize> {
        Vector3 { x: 0, y: 0, z: 0 }
    }
}

pub trait Coord {
    #[inline(always)]
    fn as_pos_f32(&self) -> WorldPos<f32>;
    #[inline(always)]
    fn as_pos_i32(&self) -> WorldPos<i32>;
    #[inline(always)]
    fn as_chunk(&self) -> ChunkPos;
    #[inline(always)]
    fn as_sub(&self) -> Vector3<usize>;
    fn is_on_chunk_border(&self) -> bool {
        let sub = self.as_sub();
        sub.x == 0 || sub.x == 15 ||
        sub.y == 0 || sub.y == 15 ||
        sub.z == 0 || sub.z == 15
    }
}

pub trait AsCoord<T: cgmath::BaseNum> {
    fn as_coord(&self) -> WorldPos<T>;
}

impl<T: cgmath::BaseNum> AsCoord<T> for Vector3<T> {
    fn as_coord(&self) -> WorldPos<T> {
        WorldPos(*self)
    }
}


impl<T: cgmath::BaseNum> AsCoord<T> for (T,T,T) {
    fn as_coord(&self) -> WorldPos<T> {
        WorldPos(Vector3::from(*self))
    }
}
