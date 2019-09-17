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

/// Provides abstraction of HyperMinHash registers.
pub trait RegisterVector {
    fn register_at(&self, idx: usize) -> u32;

    fn set_register(&mut self, idx: usize, value: u32);

    fn num_registers(&self) -> usize;
}

pub type ArrayRegisters = [u32; NUM_REGISTERS];

pub fn new_array_registers() -> ArrayRegisters {
    [0u32; NUM_REGISTERS]
}

/// Plain array-backed RegisterVector impl.
impl RegisterVector for ArrayRegisters {
    fn register_at(&self, idx: usize) -> u32 {
        self[idx]
    }

    fn set_register(&mut self, idx: usize, value: u32) {
        self[idx] = value;
    }

    fn num_registers(&self) -> usize {
        self.len()
    }
}
