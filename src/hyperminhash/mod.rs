//! Module contains Redis-independent HyperMinHash features.

pub mod sketch;
mod hash;

pub const HASH_BITS: usize = 128;
pub const P: usize = 14;
pub const Q: usize = 6;
pub const R: usize = 10;
pub const NUM_REGISTERS: usize = 1 << P as usize;
pub const HLL_Q: usize = 1 << Q as usize;
pub const HLL_BITS: usize = HASH_BITS - R;

pub type ArrayRegisters = [u32; NUM_REGISTERS];

/// Provides abstraction of HyperMinHash registers.
pub trait RegisterVector {
    fn get(&self, idx: usize) -> u32;

    fn set(&mut self, idx: usize, value: u32);

    fn len(&self) -> usize;
}

/// Plain array-backed RegisterVector impl.
impl RegisterVector for ArrayRegisters {
    fn get(&self, idx: usize) -> u32 {
        self[idx]
    }

    fn set(&mut self, idx: usize, value: u32) {
        self[idx] = value;
    }

    fn len(&self) -> usize {
        NUM_REGISTERS
    }
}
