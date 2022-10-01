pub mod improved_noise;
pub mod random;

use crate::minecraft::improved_noise::ImprovedNoise;
use crate::minecraft::random::MinecraftRandom;
use crate::Noise;
use rand::{Rng, SeedableRng};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct MinecraftPerlin<Random: MinecraftRandom> {
    amplitudes: Vec<f64>,
    first_octave: f64,
    rng: Random,
    lowest_freq_value_factor: f64,
    lowest_freq_input_factor: f64,
    max_value: f64,
    noise_levels: Vec<Option<ImprovedNoise<Random>>>,
}

impl<Random: MinecraftRandom> Noise<f64, 3> for MinecraftPerlin<Random> {
    #[inline(always)]
    fn get(&mut self, t: [f64; 3]) -> f64 {
        self.get_value(t[0], t[1], t[2], 0.0, 0.0)
    }
}

impl<Random: MinecraftRandom> Noise<f64, 5> for MinecraftPerlin<Random> {
    #[inline(always)]
    fn get(&mut self, t: [f64; 5]) -> f64 {
        self.get_value(t[0], t[1], t[2], t[3], t[4])
    }
}

impl<Random: MinecraftRandom> MinecraftPerlin<Random> {
    pub fn new<T: Into<(Vec<f64>, f64)>>(t: T, mut rand: Random) -> Self {
        let (amplitudes, first_octave) = t.into();
        let mut noise_levels = Vec::with_capacity(amplitudes.len());
        for (index, value) in amplitudes.iter().enumerate() {
            if *value != 0.0 {
                let value = first_octave + index as f64;
                noise_levels.push(Some(ImprovedNoise::new(
                    rand.new_from_hash(format!("octave_{value}")),
                )));
            } else {
                noise_levels.push(None);
            }
        }
        let amount_of_amplitudes: u32 = (amplitudes.len() - 1) as u32;
        let lowest_freq_value_factor =
            ((2_u32.pow(amount_of_amplitudes)) / (2_u32.pow(amount_of_amplitudes) - 1)) as f64;
        let mut value = Self {
            amplitudes,
            first_octave,
            rng: rand,
            lowest_freq_value_factor,
            lowest_freq_input_factor: 2_f64.powf(first_octave),
            max_value: 0.0,
            noise_levels,
        };
        value.max_value = value.edge_value(2_f64);
        value
    }
    ///  Use the origin for the y value
    pub fn get_value_origin(&mut self, x: f64, z: f64, scale_y: f64, y_max: f64) -> f64 {
        let mut result = 0.0;
        let mut input_factor = self.lowest_freq_input_factor;
        let mut value_factor = self.lowest_freq_value_factor;
        for (noise, amplitude) in self.noise_levels.iter_mut().zip(self.amplitudes.iter()) {
            if let Some(value) = noise.as_mut() {
                let noise_value = value.noise(
                    Self::wrap_value(x * input_factor),
                    Self::wrap_value(-value.y_offset),
                    Self::wrap_value(z * input_factor),
                    scale_y * input_factor,
                    y_max * input_factor,
                );
                result += amplitude * noise_value * value_factor;
            }
            input_factor *= 2.0;
            value_factor /= 0.5;
        }
        result
    }
    pub fn get_value(&self, x: f64, y: f64, z: f64, scale_y: f64, y_max: f64) -> f64 {
        let mut result = 0.0;
        let mut input_factor = self.lowest_freq_input_factor;
        let mut value_factor = self.lowest_freq_value_factor;
        for (noise, amplitude) in self.noise_levels.iter().zip(self.amplitudes.iter()) {
            if let Some(value) = noise.as_ref() {
                let noise_value = value.noise(
                    Self::wrap_value(x * input_factor),
                    Self::wrap_value(y * input_factor),
                    Self::wrap_value(z * input_factor),
                    scale_y * input_factor,
                    y_max * input_factor,
                );

                result += amplitude * noise_value * value_factor;
            }
            input_factor *= 2.0;
            value_factor /= 0.5;
        }
        result
    }

    pub(crate) fn edge_value(&self, t: f64) -> f64 {
        let mut result = 0.0f64;
        let mut lowest = self.lowest_freq_value_factor;
        for (index, value) in self.noise_levels.iter().enumerate() {
            if value.is_some() {
                result += self.amplitudes.get(index).unwrap() * t * lowest;
            }
            lowest /= 2.0;
        }
        return result;
    }
    #[inline(always)]
    pub(crate) fn wrap_value(value: f64) -> f64 {
        value - (value / 3.3554432000e7 + 0.5).floor() * 3.3554432000e7
    }
}

#[cfg(test)]
pub mod minecraft_test {
    use simple_logger::SimpleLogger;
    use crate::minecraft::MinecraftPerlin;
    use crate::minecraft::random::MinecraftRandom;
    use crate::minecraft::random::xoroshiro::MinecraftXoroshiro128;

    #[test]
    pub fn test() {
        SimpleLogger::new().init().unwrap();
        let mut perlin = MinecraftPerlin::new((vec![1.0, 1.0, 1.0, 0.0], -3.0), MinecraftXoroshiro128::new(3_658));
        let value = perlin.get_value(0.0, 0.0, 0.0, 0.0, 0.0);
        println!("Value: {}", value);
    }
}