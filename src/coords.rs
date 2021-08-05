
use std::cmp::Ordering;
use derive_more::*;
use cgmath::*;
use serde::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    XPos,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
}

impl Face {
    #[inline(always)]
    pub fn iter_all() -> impl Iterator<Item = Face> {
        [
            Face::XNeg, Face::XPos,
            Face::YNeg, Face::YPos,
            Face::ZNeg, Face::ZPos,
        ].iter().copied()
    }
}

#[derive(From, Into, Clone, Copy, Debug)]
/// Y-axis is upwards.
pub struct PixelPos(pub (i32, i32));

#[derive(Neg, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Copy, Clone, From, Into, Serialize, Deserialize, PartialEq, Debug)]
pub struct WorldPos(pub Vector3<f32>);

#[derive(Neg, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Copy, Clone, From, Into, Serialize, Deserialize, Hash, PartialEq, Eq, Debug)]
pub struct BlockPos(pub Vector3<i32>);

#[derive(Neg, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Copy, Clone, From, Into, Serialize, Deserialize, Hash, PartialEq, Eq, Debug)]
pub struct ChunkPos(pub Vector3<i32>);

impl PartialOrd for WorldPos {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        AsRef::<[f32;3]>::as_ref(&self.0).partial_cmp(rhs.0.as_ref())
    }
}

impl Ord for BlockPos {
    fn cmp(&self, rhs: &Self) -> Ordering {
        AsRef::<[i32;3]>::as_ref(&self.0).cmp(rhs.0.as_ref())
    }
}
impl PartialOrd for BlockPos {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        AsRef::<[i32;3]>::as_ref(&self.0).partial_cmp(rhs.0.as_ref())
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

impl std::ops::Deref for WorldPos {
    type Target = Vector3<f32>;
    fn deref(&self) -> &Self::Target {&self.0}
}
impl std::ops::Deref for BlockPos {
    type Target = Vector3<i32>;
    fn deref(&self) -> &Self::Target {&self.0}
}
impl std::ops::Deref for ChunkPos {
    type Target = Vector3<i32>;
    fn deref(&self) -> &Self::Target {&self.0}
}

impl From<(f32,f32,f32)> for WorldPos {
    fn from(val: (f32,f32,f32)) -> Self {
        Vector3::from(val).into()
    }
}
impl From<(i32,i32,i32)> for WorldPos {
    fn from(val: (i32,i32,i32)) -> Self {
        BlockPos::from(val).as_world()
    }
}

impl From<(i32,i32,i32)> for BlockPos {
    fn from(val: (i32,i32,i32)) -> Self {
        Vector3::from(val).into()
    }
}

impl From<(i32,i32,i32)> for ChunkPos {
    fn from(val: (i32,i32,i32)) -> Self {
        Vector3::from(val).into()
    }
}

impl Coord for WorldPos {
    fn abs(&self) -> Self {
        self.0.map(f32::abs).into()
    }
    fn zero() -> Self {(0.,0.,0.).into()}
    fn as_block(&self) -> BlockPos {
        self.0.map(|v| v.floor() as i32).into()
    }
    fn as_world(&self) -> WorldPos {
        *self
    }
    fn as_chunk(&self) -> ChunkPos {
        self.0.map(|v| (v / 16.).floor() as i32).into()
    }
    fn as_sub(&self) -> Vector3<usize> {
        self.0.map(|v| (v.floor() as i32).rem_euclid(16) as usize)
    }
}

impl Coord for BlockPos {
    fn abs(&self) -> Self {
        self.0.map(i32::abs).into()
    }
    fn zero() -> Self {(0,0,0).into()}
    fn as_block(&self) -> BlockPos {
        *self
    }
    fn as_world(&self) -> WorldPos {
        self.0.map(|v| v as f32).into()
    }
    fn as_chunk(&self) -> ChunkPos {
        self.0.map(|v| (v as f32 / 16.).floor() as i32).into()
    }
    fn as_sub(&self) -> Vector3<usize> {
        self.0.map(|v| v.rem_euclid(16) as usize)
    }
}

impl WorldPos {
    pub fn as_tuple(&self) -> (f32,f32,f32) {
        self.0.into()
    }
}
impl BlockPos {
    pub fn shifted(&self, f: Face) -> Self {
        match f {
            Face::XNeg => *self + (-1,0,0).into(),
            Face::XPos => *self + (1,0,0).into(),
            Face::YNeg => *self + (0,-1,0).into(),
            Face::YPos => *self + (0,1,0).into(),
            Face::ZNeg => *self + (0,0,-1).into(),
            Face::ZPos => *self + (0,0,1).into(),
        }
    }
    pub fn as_tuple(&self) -> (i32,i32,i32) {
        self.0.into()
    }
    pub fn adjacent_to(&self, other: &Self) -> bool {
        (self.0.x - other.0.x).abs() +
        (self.0.y - other.0.y).abs() +
        (self.0.z - other.0.z).abs() == 1
    }
}
impl ChunkPos {
    pub fn as_tuple(&self) -> (i32,i32,i32) {
        self.0.into()
    }
}
impl Coord for ChunkPos {
    fn abs(&self) -> Self {
        self.0.map(i32::abs).into()
    }
    fn zero() -> Self {(0,0,0).into()}
    fn as_block(&self) -> BlockPos {
        self.0.map(|x| x * 16).into()
    }
    fn as_world(&self) -> WorldPos {
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
    fn dist(&self, other: &Self) -> f32 {
        (self.as_world() - other.as_world()).0.magnitude()
    }
    fn manhat_dist(&self, other: &Self) -> f32 {
        let (x,y,z) = (self.as_world() - other.as_world()).0.map(f32::abs).into();
        x+y+z
    }
    fn max_dist(&self, other: &Self) -> f32 {
        let (x,y,z) = (self.as_world() - other.as_world()).0.map(f32::abs).into();
        x.max(y).max(z)
    }
    fn abs(&self) -> Self;
    #[inline(always)]
    fn zero() -> Self;
    #[inline(always)]
    fn as_world(&self) -> WorldPos;
    #[inline(always)]
    fn as_block(&self) -> BlockPos;
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
    #[inline(always)]
    fn align_center(&self) -> WorldPos {
        self.as_block().as_world() + (0.5,0.5,0.5).into()
    }
    #[inline(always)]
    fn align_corner(&self) -> WorldPos {
        self.as_block().as_world()
    }
}

pub trait AsCoord {
    fn as_coord(&self) -> WorldPos;
}

impl AsCoord for Vector3<f32> {
    fn as_coord(&self) -> WorldPos {
        WorldPos(*self)
    }
}

impl AsCoord for (f32,f32,f32) {
    fn as_coord(&self) -> WorldPos {
        WorldPos(Vector3::from(*self))
    }
}
