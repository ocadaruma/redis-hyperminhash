use libc::size_t;
use std::ops::{Index, IndexMut};

/// Provides familiar interface for raw Redis StringDMA
pub struct CByteArray {
    underlying: *mut u8,
    len: size_t,
}

impl CByteArray {
    pub fn wrap(ptr: *mut u8, len: size_t) -> Self {
        CByteArray { underlying: ptr, len, }
    }

    pub fn offset(&self, offset: size_t) -> Self {
        Self::wrap(unsafe {
            self.underlying.add(offset)
        }, self.len - offset)
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Index<usize> for CByteArray {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            &*self.underlying.add(index)
        }
    }
}

impl IndexMut<usize> for CByteArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            &mut *self.underlying.add(index)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::redis::dma::CByteArray;

    #[test]
    fn test_new() {
        let mut arr = [0u8; 10];
        let dma = CByteArray::wrap(arr.as_mut_ptr(), arr.len());

        assert_eq!(dma.len, 10);
    }

    #[test]
    fn test_index() {
        let mut arr = [2u8, 3, 5, 7, 11];
        let mut dma = CByteArray::wrap(arr.as_mut_ptr(), arr.len());

        assert_eq!(dma.len, 5);
        for i in 0..arr.len() {
            assert_eq!(dma[i], arr[i]);
        }

        dma[3] = 42;
        assert_eq!(dma[3], 42);
        assert_eq!(arr[3], 42);
    }
}
