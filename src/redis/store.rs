use crate::hyperminhash::RegVector;
use super::dma::CByteArray;
use libc::size_t;
use std::mem::size_of;

/// RegVector impl which stores registers as integer array.
/// Each integer is stored in BigEndian.
pub struct SimpleDMARegVector {
    data: CByteArray
}

impl SimpleDMARegVector {
    pub fn wrap(ptr: *mut u8, len: size_t) -> Self {
        Self {
            data: CByteArray::wrap(ptr, len)
        }
    }
}

impl RegVector for SimpleDMARegVector {
    fn get(&self, idx: usize) -> u32 {
        let offset = idx * size_of::<u32>();

        let mut result = 0u32;
        result |= (self.data[offset + 3] as u32) << 0;
        result |= (self.data[offset + 2] as u32) << 8;
        result |= (self.data[offset + 1] as u32) << 16;
        result |= (self.data[offset + 0] as u32) << 24;

        result
    }

    fn set(&mut self, idx: usize, value: u32) {
        let offset = idx * size_of::<u32>();

        self.data[offset + 3] = ((value >> 0) & 0xff) as u8;
        self.data[offset + 2] = ((value >> 8) & 0xff) as u8;
        self.data[offset + 1] = ((value >> 16) & 0xff) as u8;
        self.data[offset + 0] = ((value >> 24) & 0xff) as u8;
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}
