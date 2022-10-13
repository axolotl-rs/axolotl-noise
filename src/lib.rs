pub mod math;
/// An attempt to mimic the Minecraft Perlin noise generator.
/// The algorithm is designed by Mojang and this is just a Rust implementation.
#[cfg(feature = "minecraft")]
pub mod minecraft;

pub trait Noise<T, const DIM: usize> {
    fn get(&mut self, t: [T; DIM]) -> f64;
}
