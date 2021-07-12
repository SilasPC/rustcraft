
pub use aabb_tree::Proxy;
use crate::TextureAtlas;
use aabb_tree::AabbTree;
use std::ffi::CString;

use crate::prelude::*;

use derive_more::*;
#[derive(Deref, DerefMut, From, Into, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ArcStr(Arc<String>);

impl serde::Serialize for ArcStr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>  {
        serializer.serialize_str(self.0.as_ref())
    }
}
impl<'de> serde::Deserialize<'de> for ArcStr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>
    {
        Ok(Self(Arc::new(deserializer.deserialize_str(StrVisit)?)))
    }
}

struct StrVisit;

impl<'de> serde::de::Visitor<'de> for StrVisit {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E>
    {
        Ok(s.to_owned())
    }
}

pub fn make_cstr(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct AABB(pub AABBTuple);
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
    pub fn remove(&mut self, key: K) -> Option<T> {
        if let Some(proxy) = self.keys.remove(&key) {
            self.tree.destroy_proxy(proxy);
            self.vals.remove(&proxy)
        } else {
            None
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

    pub fn proxy_entries(&self) -> impl std::iter::Iterator<Item=(&Proxy, &T)> {
        self.vals.iter()
    }

    pub fn query(&self, aabb: &AABB) -> Vec<Proxy> {
        let mut proxies = vec![];
        self.tree.query_aabb(&aabb.0, |x| {
            proxies.push(x);
            true
        });
        proxies
    }

    pub fn for_each(&self, f: impl Fn(&K, &T)) {
        for (k, p) in &self.keys {
            f(k, &self.vals[p]);
        }
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
    pos.map(|x| (x % 16.).floor() as i32).map(|x| (x+16)%16)
}
pub fn sub_coords_from_i32(x: i32, y: i32, z: i32) -> Vector3<i32> {
    Vector3 {x,y,z}.map(|x| x % 16).map(|x| (x+16)%16)
}

/* 
impl serde::Serialize for crate::Data {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::*;
        let mut ser = serializer.serialize_tuple(2)?;
        ser.serialize_element(&self.world.seed)?;
        ser.serialize_element(&self.cam)?;

        let mut bser = serializer.serialize_seq(None)?;
        let mut next_id = self.registry.blocks.len();
        let mut blocks: HashMap<_,_> = self.registry.blocks
            .iter()
            .enumerate()
            .map(|(i,b)| (Arc::as_ptr(b), i))
            .collect::<HashMap<_,_>>();
        self.world.chunks_tree.for_each(|pos, chunk| {
            pos.serialize(ser); // err
            let mut out = Vec::with_capacity(16*16*16);
            for plane in &chunk.data {
                for row in plane {
                    for block in row {
                        if let Some(id) = blocks.get(&Arc::as_ptr(block)) {
                            out.push(id);
                        } else {
                            blocks.insert(Arc::as_ptr(block), next_id);
                            next_id += 1;
                        }
                    } 
                }
            }
            bser.serialize_element(&out);
        });
        ser.serialize_u32(blocks.len() as u32);
        for (ptr, id) in blocks {
            let is_shared = id < self.registry.blocks.len();
            ser.serialize_bool(is_shared);
            if is_shared {
                ser.serialize_u32(id as u32);
            } else {
                /// safe because chunks must still contain their arc pointer,
                /// so data must still be alive
                let b: &Block = unsafe {Arc::from_raw(ptr).as_ref()};
                // serialize non-registered block here ...
            }
        };
        

        // hecs::serialize::column::serialize(&self.ecs, context, ser);

        self.world.seed.serialize(ser)

    }
} */
