//! HyperMinHash data structure representation.
//!
//! The data structure is consists of 16-byte header and HyperMinHash registers.
//!
//! ## Header
//!
//! ```
//!  +------+---+-----+----------+
//!  | HYMH | E | N/U | Cardin.  |
//!  +------+---+-----+----------+
//! ```
//!
//! - HYMH: 4 byte magic string.
//! - E: 1 byte flag represents register encoding. (currently DENSE only)
//! - N/U: 3 byte reserved for future use
//! - Cardin.: 8 byte cached cardinality of the sketch
//!
//! ## Registers
//!
//! ### Dense representation
//!
//! Unlike Redis built-in HyperLogLog, only dense representation is supported now.
//!
//! In dense representation, registers are encoded as plain 16-bit integer array.
//!
//! NOTE: If you want to change HyperMinHash P,Q,R parameters, you may have to change encoding.

use super::dense::DenseVector;
use super::dma::CByteArray;

const MAGIC: [u8; 4] = [b'H',b'Y',b'M',b'H'];
const HEADER_LEN: usize = 16;

pub enum Encoding {
    Dense,
}

impl Encoding {
    pub const DENSE: u8 = 0;
}

pub enum Registers {
    Dense(DenseVector),
}

pub struct HyperMinHashRepr {
    encoding: Encoding,
    data: CByteArray,
}

impl HyperMinHashRepr {
    pub fn dense_len() -> usize {
        HEADER_LEN + DenseVector::DENSE_BYTES
    }

    pub fn initialize(bytes: &mut CByteArray) {
        // set magic
        for i in 0..4 {
            bytes[i] = MAGIC[i]
        }
    }

    pub fn parse(bytes: CByteArray) -> Option<HyperMinHashRepr> {
        // check length
        if bytes.len() < HEADER_LEN {
            return None;
        }

        // check magic
        for i in 0..4 {
            if bytes[i] != MAGIC[i] {
                return None;
            }
        }

        match bytes[4] {
            Encoding::DENSE if bytes.len() == Self::dense_len() => {
                Some(HyperMinHashRepr {
                    encoding: Encoding::Dense,
                    data: bytes,
                })
            },
            _ => None,
        }
    }

    pub fn registers(&self) -> Registers {
        match self.encoding {
            Encoding::Dense => Registers::Dense(
                DenseVector::wrap(self.data.offset(HEADER_LEN))
            ),
        }
    }

    pub fn invalidate_cache(&mut self) {
        self.data[15] |= 1 << 7;
    }

    pub fn cache_valid(&self) -> bool {
        self.data[15] & (1 << 7) == 0
    }

    pub fn get_cache(&self) -> u64 {
        let mut result = 0u64;


        result |= u64::from(self.data[8]);
        result |= u64::from(self.data[9]) << 8;
        result |= u64::from(self.data[10]) << 16;
        result |= u64::from(self.data[11]) << 24;
        result |= u64::from(self.data[12]) << 32;
        result |= u64::from(self.data[13]) << 40;
        result |= u64::from(self.data[14]) << 48;
        result |= u64::from(self.data[15]) << 56;

        result
    }

    pub fn set_cache(&mut self, cardinality: u64) {
        self.data[8] = (cardinality & 0xff) as u8;
        self.data[9] = ((cardinality >> 8) & 0xff) as u8;
        self.data[10] = ((cardinality >> 16) & 0xff) as u8;
        self.data[11] = ((cardinality >> 24) & 0xff) as u8;
        self.data[12] = ((cardinality >> 32) & 0xff) as u8;
        self.data[13] = ((cardinality >> 40) & 0xff) as u8;
        self.data[14] = ((cardinality >> 48) & 0xff) as u8;
        self.data[15] = ((cardinality >> 56) & 0xff) as u8;
    }
}
