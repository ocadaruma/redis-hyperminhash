use std::slice::{IterMut, Iter};

pub mod sketch;
mod hash;

const HASH_BITS: usize = 128;
const HASH_SEED: u64 = 0x1fb03e03;
const P: usize = 14;
const Q: usize = 6;
const R: usize = 10;
const NUM_REGISTERS: usize = 1 << P as usize;
const HLL_Q: usize = 1 << Q as usize;
const HLL_BITS: usize = HASH_BITS - R;

/// Provides abstraction of HyperMinHash registers.
pub trait RegVector {
    fn get(&self, idx: usize) -> u32;

    fn set(&mut self, idx: usize, value: u32);

    fn iterator(&self) -> Iter<u32>;

    fn iterator_mut(&mut self) -> IterMut<u32>;

    fn new() -> Self;
}

/// Plain array-backed RegVector impl.
/// For unit testing purpose only.
impl RegVector for [u32; NUM_REGISTERS] {
    fn get(&self, idx: usize) -> u32 {
        self[idx]
    }

    fn set(&mut self, idx: usize, value: u32) {
        self[idx] = value;
    }

    fn iterator(&self) -> Iter<'_, u32> {
        self.iter()
    }

    fn iterator_mut(&mut self) -> IterMut<'_, u32> {
        self.iter_mut()
    }

    fn new() -> Self {
        [0; NUM_REGISTERS]
    }
}
