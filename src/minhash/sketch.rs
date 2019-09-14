//! HyperMinHash data structure.

use super::hash::murmur3_x64_128;
use crate::minhash::*;

/// constant for 0.5/ln(2)
const HLL_ALPHA_INF: f64 = 0.721347520444481703680;

pub struct MinHash<T : RegVector> {
    registers: T,
}

impl <T : RegVector> MinHash<T> {
    pub fn new() -> Self {
        MinHash {
            registers: T::new()
        }
    }

    pub fn merge(sketches: &[Self]) -> Self {
        let mut target = Self::new();
        for sketch in sketches {
            for i in 0..NUM_REGISTERS {
                let reg = target.registers.get(i);
                if sketch.registers.get(i) > reg {
                    target.registers.set(i, sketch.registers.get(i))
                }
            }
        }
        target
    }

    pub fn intersection(sketches: &[Self]) -> f64 {
        if sketches.is_empty() {
            panic!("sketches must not be empty");
        }

        Self::similarity(sketches) * Self::merge(sketches).cardinality()
    }

    pub fn similarity(sketches: &[Self]) -> f64 {
        if sketches.is_empty() {
            panic!("sketches must not be empty");
        }

        if sketches.len() == 1 {
            return 1.0;
        }

        let mut c = 0u64;
        let mut n = 0u64;
        let head = &sketches[0];

        for (i, reg) in head.registers.iterator().enumerate() {
            if *reg != 0 {
                let mut contains = true;
                for sketch in sketches {
                    contains = contains && (*reg == sketch.registers.get(i));
                }
                if contains {
                    c += 1;
                }
            }

            for sketch in sketches {
                if sketch.registers.get(i) != 0 {
                    n += 1;
                    break;
                }
            }
        }

        if c == 0 {
            return 0.0;
        }

        let mut cs = vec![0.0; sketches.len()];
        for (i, sketch) in sketches.iter().enumerate() {
            cs[i] = sketch.cardinality();
        }

        let n_e = expected_collision(&cs);
        if (c as f64) < n_e {
            return 0.0;
        }
        (c as f64 - n_e) / n as f64
    }

    pub fn add(&mut self, element: &[u8]) {
        let hash = murmur3_x64_128(element, HASH_SEED);

        let register = (hash >> (HASH_BITS - P) as u128) as usize;

        let pat_len = pat_len(&hash);
        let rbits = hash as usize & ((1 << R) - 1);

        let packed = rbits as u32 | (pat_len << R as u32);
        if packed > self.registers.get(register) {
            self.registers.set(register, packed);
        }
    }

    pub fn cardinality(&self) -> f64 {
        let m = NUM_REGISTERS as f64;

        let mut reg_histo = [0u32; HLL_BITS as usize];
        for i in 0..NUM_REGISTERS {
            reg_histo[self.registers.get(i) as usize >> R] += 1;
        }

        let mut z = m * tau((m - reg_histo[HLL_Q + 1] as f64) / m);
        for i in (1..=HLL_Q).rev() {
            z += reg_histo[i] as f64;
            z *= 0.5;
        }

        z += m * sigma(reg_histo[0] as f64 / m);

        (HLL_ALPHA_INF * m * m / z).round()
    }
}


fn expected_collision(cs: &[f64]) -> f64 {
    let _2r = 1 << R;

    let mut x = 0.0;
    let mut b1: f64;
    let mut b2: f64;

    for i in 1..=HLL_Q {
        for j in 1..=_2r {
            if i != HLL_Q {
                let den = 2f64.powf((P + R + i) as f64);
                b1 = (_2r + j) as f64 / den;
                b2 = (_2r + j + 1) as f64 / den;
            } else {
                let den = 2f64.powf((P + R + i - 1) as f64);
                b1 = j as f64 / den;
                b2 = (j + 1) as f64 / den;
            }

            let mut product = 1.0;
            for c in cs {
                product *= (1.0 - b2).powf(*c) - (1.0 - b1).powf(*c);
            }

            x += product;
        }
    }

    x * NUM_REGISTERS as f64
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let minhash: MinHash<[u32; NUM_REGISTERS]> = MinHash::new();

        assert_eq!(minhash.registers.len(), NUM_REGISTERS);
    }

    #[test]
    fn test_pat_len() {
        assert_eq!(pat_len(&0u128), 65);

        assert_eq!(pat_len(&0x1_00000000_00000000u128), 50);
    }

    #[test]
    fn test_cardinality() {
        let mut minhash: MinHash<[u32; NUM_REGISTERS]> = MinHash::new();

        for i in 0..10 {
            minhash.add(format!("id{}", i).as_bytes());
        }
        assert_eq!(minhash.cardinality() as u64, 10);

        for i in 0..1_000_000 {
            minhash.add(format!("id{}", i).as_bytes());
        }
        assert_eq!(minhash.cardinality() as u64, 997689);
    }

    #[test]
    fn test_intersection() {
        let mut minhash_1: MinHash<[u32; NUM_REGISTERS]> = MinHash::new();
        for i in 0..10000 {
            minhash_1.add(format!("a_{}", i).as_bytes());
        }

        let mut minhash_2: MinHash<[u32; NUM_REGISTERS]> = MinHash::new();
        for i in 0..10000 {
            minhash_2.add(format!("b_{}", i).as_bytes());
        }

        for i in 0..100 {
            minhash_1.add(format!("ab_{}", i).as_bytes());
            minhash_2.add(format!("ab_{}", i).as_bytes());
        }

        assert_eq!(MinHash::intersection(&[minhash_1, minhash_2]) as u64, 106);
    }
}
