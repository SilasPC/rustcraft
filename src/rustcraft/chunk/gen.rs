
use crate::prelude::*;

pub struct ChunkGenerator {
    srcs: HashSet<ChunkPos>,
    deque: MultiDeque<(ChunkPos,usize)>,
    done: bool,
    last_rank: usize,
}

impl ChunkGenerator {
    pub fn step_n(&mut self, mut n: usize) {
        while n > 0 {
            if self.done {return};
            let rank = self.deque.get_rank();
            let p = self.deque.poll();
            if p.is_none() {
                self.done = true;
                return;
            }
            self.last_rank = rank;
            let (p, spread) = p.unwrap();
            let cur_state = self.get_state(p);
            let too_high = cur_state >= rank;
            if !too_high {
                self.set_state(p, rank);
            }
            if spread > 1 {
                for n in self.around(p) {
                    self.deque.push(rank, (n, spread-1));
                }
            }
            if !too_high {
                n -= 1;
            }
        }
    }
    fn around(&self, p: ChunkPos) -> Vec<ChunkPos> {unimplemented!()}
    fn get_state(&self, p: ChunkPos) -> usize {0}
    fn set_state(&self, p: ChunkPos, n: usize) {}
}

struct MultiDeque<E> {
    map: std::collections::BTreeMap<usize, VecDeque<E>>,
}

impl<E> MultiDeque<E> {
    pub fn push(&mut self, rank: usize, e: E) {
        if let Some(deque) = self.map.get_mut(&rank) {
            deque.push_back(e);
        } else {
            let mut deque = VecDeque::new();
            deque.push_back(e);
            self.map.insert(rank, deque);
        }
    }
    pub fn get_rank(&self) -> usize {
        self.map.keys().next().copied().unwrap_or_default()
    }
    pub fn poll(&mut self) -> Option<E> {
        let k = self.map.keys().next().copied()?;
        let deque = self.map.get_mut(&k).unwrap();
        let res = deque.pop_front();
        if deque.len() == 0 {self.map.remove(&k);}
        res
    }
    pub fn empty(&self) -> bool {self.map.len() == 0}
}