
use cgmath::Vector3;
use aabb_tree::AabbTree;
pub use aabb_tree::Proxy;
use std::ffi::CString;

pub fn make_cstr(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct AABB(AABBTuple);
pub type AABBTuple = ((f32,f32,f32,),(f32,f32,f32,));

impl From<AABBTuple> for AABB {
    fn from(tuple: AABBTuple) -> Self { Self(tuple) }
}

impl AABB {
    pub fn radius(pos: &Vector3<f32>, rad: f32) -> Self {
        Self(((
            pos.x - rad,
            pos.y - rad,
            pos.z - rad,
        ),(
            pos.x + rad,
            pos.y + rad,
            pos.z + rad,
        )))
    }
    pub fn from_corner(pos: &Vector3<f32>, extent: f32) -> Self {
        Self(((
            pos.x,
            pos.y,
            pos.z,
        ),(
            pos.x + extent,
            pos.y + extent,
            pos.z + extent,
        )))
    }
    pub fn extend_radius(&mut self, rad: f32) {
        self.0.0.0 -= rad;
        self.0.0.1 -= rad;
        self.0.0.2 -= rad;
        self.0.1.0 += rad;
        self.0.1.1 += rad;
        self.0.1.2 += rad;
    }
}

pub struct BVH<K,T> {
    keys: std::collections::HashMap<K, aabb_tree::Proxy>,
    vals: std::collections::HashMap<aabb_tree::Proxy, T>,
    tree: AabbTree<()>,
}

impl<T: std::fmt::Debug, K: std::fmt::Debug> std::fmt::Debug for BVH<T,K> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("BVH")
            .field("keys", &self.keys)
            .field("vals", &self.vals)
            .finish()
    }
}

impl<K: Copy + Eq + std::hash::Hash, T> BVH<K,T> {
    pub fn new() -> Self {
        Self {
            tree: AabbTree::new(),
            keys: Default::default(),
            vals: Default::default(),
        }
    }
    pub fn insert(&mut self, key: K, val: T, aabb: &AABB) {
        let proxy = self.tree.create_proxy(aabb.0, ());
        assert!(self.keys.insert(key, proxy).is_none());
        assert!(self.vals.insert(proxy, val).is_none());
    }
    pub fn update(&mut self, key: K, aabb: &AABB) {
        let proxy = self.keys[&key];
        self.tree.set_aabb(proxy, &aabb.0);
    }
    pub fn remove(&mut self, key: K) {
        if let Some(proxy) = self.keys.remove(&key) {
            self.tree.destroy_proxy(proxy);
        }
    }
    pub fn any_overlaps(&self, aabb: &AABB) -> bool {
        let mut found = false;
        self.tree.query_aabb(&aabb.0, |_| {
            found = true;
            false
        });
        found
    }
    pub fn get(&self, key: K) -> Option<&T> {
        self.vals.get(self.keys.get(&key)?)
    }
    pub fn has(&self, key: K) -> bool {
        self.keys.contains_key(&key)
    }
    pub fn get_mut(&mut self, key: K) -> Option<&mut T> {
        self.vals.get_mut(self.keys.get(&key)?)
    }

    pub fn values_mut(&mut self) -> impl std::iter::Iterator<Item=&mut T> {
        self.vals.values_mut()
    }

    pub fn query(&self, aabb: &AABB) -> Vec<Proxy> {
        let mut proxies = vec![];
        self.tree.query_aabb(&aabb.0, |x| {
            proxies.push(x);
            true
        });
        proxies
    }
}

impl<K, T> std::ops::Index<Proxy> for BVH<K, T> {
    type Output = T;
    fn index(&self, idx: Proxy) -> &Self::Output {
        &self.vals[&idx]
    }
}

impl<K, T> std::ops::IndexMut<Proxy> for BVH<K, T> {
    fn index_mut(&mut self, idx: Proxy) -> &mut Self::Output {
        self.vals.get_mut(&idx).unwrap()
    }
}

pub fn position_to_chunk_coordinates(pos: &Vector3<f32>) -> Vector3<i32> {
    pos.map(|x| (x / 16.).floor() as i32)
}
pub fn position_to_sub_coordinates(pos: &Vector3<f32>) -> Vector3<i32> {
    pos.map(|x| (x % 16.).floor() as i32).map(|x| x.max((x+16)%16))
}