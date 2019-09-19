use crate::hyperminhash::{RegisterVector, NUM_REGISTERS};
use super::dma::CByteArray;
use std::mem::size_of;

/// RegisterVector impl which stores registers as 16-bit integer array.
/// Each integer is stored in little endian.
pub struct DenseVector {
    data: CByteArray,
}

impl DenseVector {
    pub const SINGLE_REGISTER_BYTES: usize = size_of::<u16>();
    pub const DENSE_BYTES: usize = NUM_REGISTERS * DenseVector::SINGLE_REGISTER_BYTES;

    pub fn wrap(data: CByteArray) -> Self {
        Self { data, }
    }
}

impl RegisterVector for DenseVector {
    fn register_at(&self, idx: usize) -> u32 {
        let offset = idx * DenseVector::SINGLE_REGISTER_BYTES;

        let mut result = 0u16;
        result |= u16::from(self.data[offset    ]);
        result |= u16::from(self.data[offset + 1]) << 8;

        u32::from(result)
    }

    fn set_register(&mut self, idx: usize, value: u32) {
        let offset = idx * DenseVector::SINGLE_REGISTER_BYTES;

        self.data[offset    ] = ((value     ) & 0xff) as u8;
        self.data[offset + 1] = ((value >> 8) & 0xff) as u8;
    }
}
