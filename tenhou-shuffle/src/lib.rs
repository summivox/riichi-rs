#![doc = include_str!("../README.md")]
#![no_std]

pub mod mt19937_nostd;

use sha2::{Digest, Sha512};

use crate::mt19937_nostd::MT19937;

pub const MT19937_SEED_SIZE: usize = 624;

pub const NUM_CHUNKS: usize = 9;
pub const SRC_LEN: usize = 1024 / 32 * NUM_CHUNKS;
pub const RND_LEN: usize = 512 / 32 * NUM_CHUNKS;

fn u8_to_u32_le<const NUM_U32: usize>(bytes: &[u8]) -> [u32; NUM_U32] {
    assert_eq!(bytes.len(), NUM_U32 * 4);
    core::array::from_fn(|i|
        u32::from_le_bytes(bytes[(i * 4)..((i + 1) * 4)].try_into().unwrap()))
}

fn u32_le_to_u8<const NUM_U8: usize>(words: &[u32]) -> [u8; NUM_U8] {
    assert_eq!(NUM_U8, words.len() * 4);
    let mut bytes = [0u8; NUM_U8];
    for (i, word) in words.iter().enumerate() {
        bytes[(i * 4)..((i + 1) * 4)].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

/// Seeds a new MT19937 RNG.
pub fn mt_from_seed(seed: &[u32; MT19937_SEED_SIZE]) -> MT19937 {
    MT19937::new_with_slice_seed(&seed[..])
}

/// Decodes MT19937 seed from little-endian base-64-encoded string.
pub fn seed_from_base64(base64: &str) -> Result<[u32; MT19937_SEED_SIZE], base64::DecodeSliceError> {
    use base64::Engine as _;
    let mut seed_u8 = [0u8; MT19937_SEED_SIZE * 4];
    base64::engine::general_purpose::STANDARD.decode_slice(base64, &mut seed_u8)?;
    assert_eq!(seed_u8.len(), MT19937_SEED_SIZE * 4);
    Ok(u8_to_u32_le(&seed_u8))
}

/// Generates seed for another MT19937 RNG from the given one.
pub fn seed_from_mt(mt: &mut MT19937) -> [u32; MT19937_SEED_SIZE] {
    core::array::from_fn(|_| mt.gen_u32())
}

/// Seeds a new MT19937 RNG from the given one.
pub fn derive_new_mt(mt: &mut MT19937) -> MT19937 {
    mt_from_seed(&seed_from_mt(mt))
}

/// Generates the `src` array from the given MT19937 RNG.
/// See [mod-level docs](crate) for definition of the arrays.
pub fn src_from_mt(mt: &mut MT19937) -> [u32; SRC_LEN] {
    core::array::from_fn(|_| mt.gen_u32())
}

/// Generates the `rnd` array from the `src` array by chunk-wise hashing.
/// See [mod-level docs](crate) for definition of the arrays.
pub fn rnd_from_src(src: &[u32; SRC_LEN]) -> [u32; RND_LEN] {
    let mut rnd = [0u32; RND_LEN];
    let mut hasher = Sha512::new();
    for i in 0..NUM_CHUNKS {
        let src_block = &src[(i * 32)..((i + 1) * 32)];
        hasher.update(&u32_le_to_u8::<128>(src_block));
        let hash_block: [u8; 64] = hasher.finalize_reset().into();
        rnd[(i * 16)..((i + 1) * 16)].copy_from_slice(&u8_to_u32_le::<16>(&hash_block));
    }
    rnd
}

/// Generates the `rnd` array from the given MT19937 RNG.
/// See [mod-level docs](crate) for definition of the arrays.
pub fn rnd_from_mt(mt: &mut MT19937) -> [u32; RND_LEN] {
    rnd_from_src(&src_from_mt(mt))
}

/// Shuffles the given wall array using randomness from the `rnd` array.
/// See [mod-level docs](crate) for definition of the arrays and the specific shuffling algorithm.
pub fn shuffle_with_rnd<T>(wall: &mut [T], rnd: &[u32; RND_LEN]) {
    let n = wall.len();
    assert!(n == 136 || n == 108);
    for i in 0..(n - 1) {
        wall.swap(i, (i + rnd[i] as usize % (n - i)) as usize)
    }
}

/// Shuffles the given wall array using randomness from the given MT19937 RNG.
/// See [mod-level docs](crate) for the algorithm.
pub fn shuffle_with_mt<T>(wall: &mut [T], mt: &mut MT19937) {
    shuffle_with_rnd(wall, &rnd_from_mt(mt));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_b327da61() {
        let seed_base64 = include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/2016022509gm-0009-0000-b327da61.seed_str"
        ));
        let expected_seed = include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/2016022509gm-0009-0000-b327da61.seed_u32"
        ));
        let expected_src = include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/2016022509gm-0009-0000-b327da61.src_u32"
        ));
        let expected_rnd = include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/2016022509gm-0009-0000-b327da61.rnd_u32"
        ));
        let expected_wall136 = include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/2016022509gm-0009-0000-b327da61.wall136"
        ));

        let seed = seed_from_base64(seed_base64).unwrap();
        assert_eq!(seed, expected_seed);

        let mut game_mt = mt_from_seed(&seed);

        let src = src_from_mt(&mut game_mt);
        assert_eq!(src, expected_src);

        let rnd = rnd_from_src(&src);
        assert_eq!(rnd, expected_rnd);

        let mut wall136: [u8; 136] = core::array::from_fn(|i| i as u8);
        shuffle_with_rnd(&mut wall136, &rnd);
        assert_eq!(wall136, expected_wall136);
    }
}
