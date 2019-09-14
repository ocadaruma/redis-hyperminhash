//! HyperMinHash data structure.

use crate::hash::murmur3_x64_128;

const HASH_BITS: usize = 128;
const HASH_SEED: u64 = 0x1fb03e03;
const P: usize = 14;
const Q: usize = 6;
const R: usize = 10;
const NUM_REGISTERS: usize = 1 << P as usize;
const HLL_Q: usize = 1 << Q as usize;
const HLL_BITS: usize = HASH_BITS - R;

/// constant for 0.5/ln(2)
const HLL_ALPHA_INF: f64 = 0.721347520444481703680;

pub struct MinHash {
    registers: [u32; NUM_REGISTERS],
}

impl MinHash {
    pub fn new() -> Self {
        MinHash {
            registers: [0; NUM_REGISTERS],
        }
    }

    pub fn add(&mut self, element: &[u8]) {
        let hash = murmur3_x64_128(element, HASH_SEED);

        let register = (hash >> (HASH_BITS - P) as u128) as usize;

        let pat_len = MinHash::pat_len(&hash);
        let rbits = hash as usize & ((1 << R) - 1);

        let packed = rbits as u32 | (pat_len << R as u32);
        if packed > self.registers[register] {
            self.registers[register] = packed;
        }
    }

    pub fn cardinality(&self) -> u64 {
        let m = NUM_REGISTERS as f64;

        let mut reg_histo = [0u32; HLL_BITS as usize];
        for i in 0..NUM_REGISTERS {
            reg_histo[self.registers[i] as usize >> R] += 1;
        }

        let mut z = m * MinHash::tau((m - reg_histo[HLL_Q + 1] as f64) / m);
        for i in (1..=HLL_Q).rev() {
            z += reg_histo[i] as f64;
            z *= 0.5;
        }

        z += m * MinHash::sigma(reg_histo[0] as f64 / m);

        let e = (HLL_ALPHA_INF * m * m / z).round();
        e as u64
    }

    fn pat_len(hash: &u128) -> u32 {
        let mut pat_len = 1u32;
        for i in 0..HLL_Q {
            if hash & (1 << (HASH_BITS - P - i - 1) as u128) != 0 {
                break;
            }
            pat_len += 1;
        }
        pat_len
    }

    fn tau(mut x: f64) -> f64 {
        if x == 0.0 || x == 1.0 {
            return 0.0;
        }

        let mut z_prime: f64;
        let mut y = 1.0;
        let mut z = 1.0 - x;

        loop {
            x = x.sqrt();
            z_prime = z;
            y *= 0.5;
            z -= (1.0 - x).powf(2.0) * y;

            if z_prime == z {
                break;
            }
        }

        z / 3.0
    }

    fn sigma(mut x: f64) -> f64 {
        if x == 1.0 {
            return std::f64::INFINITY;
        }

        let mut z_prime;
        let mut y = 1.0;
        let mut z = x;
        loop {
            x *= x;
            z_prime = z;
            z += x * y;
            y += y;

            if z_prime == z {
                break;
            }
        }

        z
    }
}

#[cfg(test)]
mod tests {
    use super::MinHash;

    #[test]
    fn test_new() {
        let minhash = MinHash::new();
    }

    #[test]
    fn test_pat_len() {
        assert_eq!(MinHash::pat_len(&0u128), 65);

        assert_eq!(MinHash::pat_len(&0x1_00000000_00000000u128), 51);
    }

    #[test]
    fn test_accuracy() {
        let mut minhash = MinHash::new();

        for i in 0..10 {
            minhash.add(format!("id{}", i).as_bytes());
        }
        assert_eq!(minhash.cardinality(), 10);

        for i in 0..1_000_000 {
            minhash.add(format!("id{}", i).as_bytes());
        }
        assert_eq!(minhash.cardinality(), 997689);
    }
}
