use rand::{Error, Rng, RngCore, SeedableRng};
use rand_xoshiro::Xoroshiro128PlusPlus;
use std::fmt::Debug;

pub trait MinecraftRandom: Rng + Debug + Clone {
    fn new(seed: i128) -> Self;
    fn new_from_hash<V: AsRef<[u8]>>(&self, v: V) -> Self;
}
pub mod legacy {
    use std::sync::atomic::AtomicI64;

    pub struct LegacyRandom {
        seed: AtomicI64,
    }
}

pub mod xoroshiro {
    use crate::minecraft::random::MinecraftRandom;
    use rand::{Error, Rng, RngCore, SeedableRng};
    use rand_xoshiro::Xoroshiro128PlusPlus;

    #[derive(Debug, Clone)]
    pub struct MinecraftXoroshiro128 {
        pub seed_low: i64,
        pub seed_high: i64,
        pub rand: Xoroshiro128PlusPlus,
    }

    impl RngCore for MinecraftXoroshiro128 {
        fn next_u32(&mut self) -> u32 {
            self.rand.next_u32()
        }

        fn next_u64(&mut self) -> u64 {
            self.rand.next_u64()
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.rand.fill_bytes(dest)
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
            self.rand.try_fill_bytes(dest)
        }
    }

    impl MinecraftRandom for MinecraftXoroshiro128 {
        fn new(seed: i128) -> Self {
            let seed_low = (seed & 0xFFFF_FFFF_FFFF_FFFF) as i64;
            let seed_high = ((seed >> 64) & 0xFFFF_FFFF_FFFF_FFFF) as i64;

            let rand = Xoroshiro128PlusPlus::from_seed(seed.to_be_bytes());
            Self {
                seed_low,
                seed_high,
                rand,
            }
        }

        fn new_from_hash<V: AsRef<[u8]>>(&self, v: V) -> Self {
            let hash = md5::compute(v).0;

            let seed_low = i64::from_be_bytes(hash[0..8].try_into().unwrap()) ^ self.seed_low;
            let seed_high = i64::from_be_bytes(hash[8..16].try_into().unwrap()) ^ self.seed_high;
            let seed_low_bytes: [u8; 8] = seed_low.to_be_bytes();
            let seed_high_bytes: [u8; 8] = seed_high.to_be_bytes();

            let seed: [u8; 16] = [
                seed_low_bytes[0],
                seed_low_bytes[1],
                seed_low_bytes[2],
                seed_low_bytes[3],
                seed_low_bytes[4],
                seed_low_bytes[5],
                seed_low_bytes[6],
                seed_low_bytes[7],
                seed_high_bytes[0],
                seed_high_bytes[1],
                seed_high_bytes[2],
                seed_high_bytes[3],
                seed_high_bytes[4],
                seed_high_bytes[5],
                seed_high_bytes[6],
                seed_high_bytes[7],
            ];
            return Self {
                seed_low,
                seed_high,
                rand: Xoroshiro128PlusPlus::from_seed(seed),
            };
        }
    }
}
