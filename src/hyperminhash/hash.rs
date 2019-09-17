use std::ops::BitXor;

/// 128 bit version of MurmurHash3 for x64 architecture
/// Original cpp implementation: https://github.com/aappleby/smhasher/blob/master/src/MurmurHash3.cpp
pub fn murmur3_x64_128(element: &[u8], seed: u64) -> u128 {
    let len = element.len();
    let nblocks = len / 16;

    let mut h1 = seed;
    let mut h2 = seed;

    let c1 = 0x87c37b91_114253d5u64;
    let c2 = 0x4cf5ad43_2745937fu64;

    for i in 0..nblocks {
        let mut k1 = 0u64;
        let lsb1 = (i*2 + 0) * 8;
        k1 |= element[lsb1 + 0] as u64;
        k1 |= (element[lsb1 + 1] as u64) << 8;
        k1 |= (element[lsb1 + 2] as u64) << 16;
        k1 |= (element[lsb1 + 3] as u64) << 24;
        k1 |= (element[lsb1 + 4] as u64) << 32;
        k1 |= (element[lsb1 + 5] as u64) << 40;
        k1 |= (element[lsb1 + 6] as u64) << 48;
        k1 |= (element[lsb1 + 7] as u64) << 56;

        let mut k2 = 0u64;
        let lsb2 = (i*2 + 1) * 8;
        k2 |= element[lsb2 + 0] as u64;
        k2 |= (element[lsb2 + 1] as u64) << 8;
        k2 |= (element[lsb2 + 2] as u64) << 16;
        k2 |= (element[lsb2 + 3] as u64) << 24;
        k2 |= (element[lsb2 + 4] as u64) << 32;
        k2 |= (element[lsb2 + 5] as u64) << 40;
        k2 |= (element[lsb2 + 6] as u64) << 48;
        k2 |= (element[lsb2 + 7] as u64) << 56;

        k1 = k1
            .wrapping_mul(c1)
            .rotate_left(31)
            .wrapping_mul(c2);

        h1 = h1
            .bitxor(k1)
            .rotate_left(27)
            .wrapping_add(h2)
            .wrapping_mul(5)
            .wrapping_add(0x52dce729);

        k2 = k2
            .wrapping_mul(c2)
            .rotate_left(33)
            .wrapping_mul(c1);

        h2 = h2
            .bitxor(k2)
            .rotate_left(31)
            .wrapping_add(h1)
            .wrapping_mul(5)
            .wrapping_add(0x38495ab5);
    }

    let mut k1 = 0u64;
    let mut k2 = 0u64;

    let tail = nblocks * 16;

    for i in (1..=(len & 15)).rev() {
        match i {
            15 => k2 ^= (element[tail + 14] as u64) << 48,
            14 => k2 ^= (element[tail + 13] as u64) << 40,
            13 => k2 ^= (element[tail + 12] as u64) << 32,
            12 => k2 ^= (element[tail + 11] as u64) << 24,
            11 => k2 ^= (element[tail + 10] as u64) << 16,
            10 => k2 ^= (element[tail +  9] as u64) << 8,
            9 => {
                k2 ^= (element[tail +  8] as u64) << 0;
                k2 = k2
                    .wrapping_mul(c2)
                    .rotate_left(33)
                    .wrapping_mul(c1);
                h2 ^= k2;
            },
            8 => k1 ^= (element[tail +  7] as u64) << 56,
            7 => k1 ^= (element[tail +  6] as u64) << 48,
            6 => k1 ^= (element[tail +  5] as u64) << 40,
            5 => k1 ^= (element[tail +  4] as u64) << 32,
            4 => k1 ^= (element[tail +  3] as u64) << 24,
            3 => k1 ^= (element[tail +  2] as u64) << 16,
            2 => k1 ^= (element[tail +  1] as u64) << 8,
            1 => {
                k1 ^= (element[tail +  0] as u64) << 0;
                k1 = k1
                    .wrapping_mul(c1)
                    .rotate_left(31)
                    .wrapping_mul(c2);
                h1 ^= k1;
            },
            _ => {}
        }
    }

    h1 ^= len as u64; h2 ^= len as u64;
    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);

    h1 = fmix64(h1);
    h2 = fmix64(h2);

    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);

    (h1 as u128) << 64 | (h2 as u128)
}

fn fmix64(k: u64) -> u64 {
    let mut result = k;

    result ^= result >> 33;
    result = result.wrapping_mul(0xff5_1afd7ed5_58ccdu64);
    result ^= result >> 33;
    result = result.wrapping_mul(0xc4c_eb9fe1a8_5ec53u64);
    result ^= result >> 33;

    result
}

#[cfg(test)]
mod tests {
    use super::murmur3_x64_128;

    #[test]
    fn test_hash() {
        let element = "Lorem ipsum dolor sit amet, consectetur adipisicing elit".as_bytes();
        let result = murmur3_x64_128(element, 104729);

        assert_eq!(result, 0x6769dae0_ba0f9ccf_7e4bd221_908cfc07);
    }
}
