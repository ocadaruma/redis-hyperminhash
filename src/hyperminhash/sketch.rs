//! HyperMinHash data structure.

use super::*;
use super::hash::murmur3_x64_128;

/// constant for 0.5/ln(2)
const HLL_ALPHA_INF: f64 = 0.721347520444481703680;
const HASH_SEED: u64 = 0x1fb03e03;

/// Represents HyperMinHash sketch
pub struct HyperMinHash<T : RegisterVector> {
    pub registers: T,
}

/// HyperLogLog-part of HyperMinHash.
/// Cardinality estimation is based on Otmar Ertl, arXiv:1702.01284 "New cardinality estimation algorithms for HyperLogLog sketches"
/// which is adopted in Redis.
impl <T : RegisterVector> HyperMinHash<T> {
    pub fn wrap(registers: T) -> Self {
        Self { registers, }
    }

    /// Merge given sketch into this sketch destructively.
    pub fn merge<U : RegisterVector>(&mut self, other: &HyperMinHash<U>) {
        for i in 0..NUM_REGISTERS {
            let reg = other.registers.register_at(i);
            if reg > self.registers.register_at(i) {
                self.registers.set_register(i, reg);
            }
        }
    }

    pub fn add(&mut self, element: &[u8]) -> bool {
        let hash = murmur3_x64_128(element, HASH_SEED);

        let PatLen { register, len: pat_len } = pat_len(&hash);

        // take rightmost R bits
        let r_mask = ((1 << R) - 1) as u128;
        let rbits = hash & r_mask;

        let packed = rbits as u32 | (pat_len << R as u32);
        if packed > self.registers.register_at(register) {
            self.registers.set_register(register, packed);
            return true
        }

        false
    }

    pub fn cardinality(&self) -> f64 {
        let mut reg_histo = [0u32; HLL_BITS];
        for i in 0..NUM_REGISTERS {
            reg_histo[self.registers.register_at(i) as usize >> R] += 1;
        }

        cardinality(&reg_histo)
    }
}

/// MinHash-part of HyperMinHash.
/// Combines multiple sketches, estimate their similarity and intersection cardinality.
pub struct MinHashCombiner {
    union: HyperMinHash<ArrayRegisters>,
    reg_intersection: ArrayRegisters,
    cardinalities: Vec<f64>,
}

impl MinHashCombiner {
    pub fn new() -> MinHashCombiner {
        Self {
            union: HyperMinHash::wrap(new_array_registers()),
            reg_intersection: new_array_registers(),
            cardinalities: Vec::new(),
        }
    }

    pub fn combine<T : RegisterVector>(&mut self, sketch: &HyperMinHash<T>) {
        // number of sketches merged so far
        let num_sketch = self.cardinalities.len();
        let mut reg_histo = [0u32; HLL_BITS];

        for i in 0..NUM_REGISTERS {
            let reg = sketch.registers.register_at(i);

            // merge into self
            if reg > self.union.registers[i] {
                self.union.registers[i] = reg;
            }

            // update reg_histo for cardinality estimation
            reg_histo[reg as usize >> R] += 1;

            // update reg_intersection for similarity estimation
            // retain only if register values are equal
            if num_sketch < 1 {
                self.reg_intersection[i] = reg;
            } else if self.reg_intersection[i] != 0 && self.reg_intersection[i] != reg {
                self.reg_intersection[i] = 0;
            }
        }

        self.cardinalities.push(cardinality(&reg_histo))
    }

    pub fn similarity(&self) -> f64 {
        if self.cardinalities.is_empty() {
            return 0.0
        }

        if self.cardinalities.len() == 1 {
            return 1.0;
        }

        let mut c = 0u64;
        let mut n = 0u64;

        for i in 0..self.reg_intersection.len() {
            if self.reg_intersection[i] != 0 {
                c += 1;
            }
            if self.union.registers[i] != 0 {
                n += 1;
            }
        }

        if c == 0 {
            return 0.0;
        }

        // do collision error correction only when estimating similarity of 2 sets.
        // otherwise, expected collision computation in original paper (algorithm 2.1.5) does not works.
        // see discussion in: https://github.com/LiveRamp/HyperMinHash-java/issues/13
        if self.cardinalities.len() == 2 {
            let ec = approx_expected_collision(self.cardinalities[0], self.cardinalities[1]);
            (c as f64 - ec) / n as f64
        } else {
            c as f64 / n as f64
        }
    }

    pub fn intersection(&self) -> f64 {
        self.similarity() * self.union.cardinality()
    }
}

fn expected_collision(n: f64, m: f64, p: usize, q: usize, r: usize) -> f64 {
    let _2r = 1 << r;
    let _2q = 1 << q;

    let mut x = 0.0;
    let mut b1: f64;
    let mut b2: f64;

    for i in 1..=_2q {
        for j in 1..=_2r {
            if i != _2q {
                let den = 2f64.powi((p + r + i) as i32);
                b1 = (_2r + j) as f64 / den;
                b2 = (_2r + j + 1) as f64 / den;
            } else {
                let den = 2f64.powi((p + r + i - 1) as i32);
                b1 = j as f64 / den;
                b2 = (j + 1) as f64 / den;
            }

            let prx = (1.0 - b2).powf(n) - (1.0 - b1).powf(n);
            let pry = (1.0 - b2).powf(m) - (1.0 - b1).powf(m);

            x += prx * pry;
        }
    }

    x * NUM_REGISTERS as f64
}

fn approx_expected_collision(n: f64, m: f64) -> f64 {
    let (n, m) = if n < m { (m, n) } else { (n, m) };

    if n > 2f64.powi((HLL_Q + R) as i32) {
        // return 0 instead of panic if n is too large
        0.0
    } else if n > 2f64.powi((P + 5) as i32) {
        let phi = (4.0 * n / m) / (1.0 + n / m).powi(2);

        0.169919487159739093975315012348f64 * 2f64.powi((P - R) as i32) * phi
    } else {
        expected_collision(n, m, P, Q, 0) / 2f64.powi(R as i32)
    }
}

#[derive(Debug, PartialEq)]
struct PatLen {
    register: usize,
    len: u32,
}

/// Use leftmost P bits to determine register.
/// Find leftmost 1-bit position in next Q bits.
fn pat_len(hash: &u128) -> PatLen {
    let register = (hash >> (HASH_BITS - P) as u128) as usize;

    let mut pat_len = 1u32;
    for i in 1..=HLL_Q {
        if hash & (1 << (HASH_BITS - P - i) as u128) != 0 {
            break;
        }
        pat_len += 1;
    }

    PatLen { register, len: pat_len, }
}

fn cardinality(reg_histo: &[u32]) -> f64 {
    let m = NUM_REGISTERS as f64;

    let mut z = m * tau((m - reg_histo[HLL_Q + 1] as f64) / m);
    for i in (1..=HLL_Q).rev() {
        z += reg_histo[i] as f64;
        z *= 0.5;
    }

    z += m * sigma(reg_histo[0] as f64 / m);

    (HLL_ALPHA_INF * m * m / z).round()
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
    fn test_wrap() {
        let sketch: HyperMinHash<ArrayRegisters> = HyperMinHash::wrap(new_array_registers());

        assert_eq!(sketch.registers.len(), NUM_REGISTERS);
    }

    #[test]
    fn test_pat_len() {
        assert_eq!(pat_len(&0u128),
                   PatLen { register: 0, len: 65, });

        assert_eq!(pat_len(&0x1_00000000_00000000u128),
                   PatLen { register: 0, len: 50, });
    }

    #[test]
    fn test_add() {
        let mut sketch: HyperMinHash<ArrayRegisters> = HyperMinHash::wrap(new_array_registers());

        assert!(sketch.add("a".as_bytes()));
        assert!(!sketch.add("a".as_bytes()));
    }

    #[test]
    fn test_cardinality() {
        let mut sketch = HyperMinHash::wrap(new_array_registers());

        for i in 0..10 {
            sketch.add(format!("id{}", i).as_bytes());
        }
        assert_eq!(sketch.cardinality() as u64, 10);

        for i in 0..1_000_000 {
            sketch.add(format!("id{}", i).as_bytes());
        }
        assert_eq!(sketch.cardinality() as u64, 997689);
    }

    #[test]
    fn test_intersection_10000() {
        let mut sketch_1 = HyperMinHash::wrap(new_array_registers());
        for i in 0..10000 {
            sketch_1.add(format!("a_{}", i).as_bytes());
        }

        let mut sketch_2 = HyperMinHash::wrap(new_array_registers());
        for i in 0..10000 {
            sketch_2.add(format!("b_{}", i).as_bytes());
        }

        for i in 0..100 {
            sketch_1.add(format!("ab_{}", i).as_bytes());
            sketch_2.add(format!("ab_{}", i).as_bytes());
        }

        let mut combiner = MinHashCombiner::new();
        combiner.combine(&sketch_1);
        combiner.combine(&sketch_2);

        assert_eq!(combiner.intersection() as u64, 107);
    }

    #[test]
    fn test_intersection_1_000_000() {
        let mut sketch_1 = HyperMinHash::wrap(new_array_registers());
        for i in 0..1_000_000 {
            sketch_1.add(format!("a_{}", i).as_bytes());
        }

        let mut sketch_2 = HyperMinHash::wrap(new_array_registers());
        for i in 0..1_000_000 {
            sketch_2.add(format!("b_{}", i).as_bytes());
        }

        for i in 0..10000 {
            sketch_1.add(format!("ab_{}", i).as_bytes());
            sketch_2.add(format!("ab_{}", i).as_bytes());
        }

        let mut combiner = MinHashCombiner::new();
        combiner.combine(&sketch_1);
        combiner.combine(&sketch_2);

        assert_eq!(combiner.intersection() as u64, 9182);
    }
}
