use crate::minecraft::random::MinecraftRandom;
use log::debug;
use rand::Rng;
use std::fmt::Debug;
use std::ops::BitAnd;

/// From the Minecraft source code:
static GRADIENTS: [[i8; 3]; 16] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
    [1, 1, 0],
    [0, -1, 1],
    [-1, 1, 0],
    [0, -1, -1],
];

pub type PermutationType = i16;

#[derive(Debug, Clone)]
pub struct ImprovedNoise<Random: MinecraftRandom> {
    pub x_offset: f64,
    pub y_offset: f64,
    pub z_offset: f64,
    // Might be changed to i32
    pub permutation: [PermutationType; 512],
    pub random: Random,
}

impl<Random: MinecraftRandom> ImprovedNoise<Random> {
    pub fn new(mut rand: Random) -> Self {
        let permutation = [0; 512];
        let mut value = Self {
            x_offset: rand.gen::<f64>() * 256.0,
            y_offset: rand.gen::<f64>() * 256.0,
            z_offset: rand.gen::<f64>() * 256.0,
            permutation,
            random: rand,
        };

        for i in 0..256 {
            let random = value.random.gen_range(0..(256 - i)) as PermutationType;
            value.permutation[i] = random + i as PermutationType;
            value.permutation[i + random as usize] = i as PermutationType;
        }
        if cfg!(debug_assertions) {
            debug!("Permutation: {:#?}", value);
            debug!("Permutation Length: {:#?}", value.permutation.len());
        }

        value
    }
    #[inline(always)]
    pub fn map<B>(&self, v: B) -> PermutationType
    where
        B: BitAnd<PermutationType, Output = PermutationType>,
    {
        self.permutation[v.bitand(0xFF as PermutationType) as usize] & 0xFF
    }
    #[inline(always)]
    pub(crate) fn dot(g: [i8; 3], x: f64, y: f64, z: f64) -> f64 {
        g[0] as f64 * x + g[1] as f64 * y + g[2] as f64 * z
    }
    #[inline(always)]
    pub fn distance(hash: usize, x: f64, y: f64, z: f64, distance: f64) -> f64 {
        let d = distance - x.powi(2) - y.powi(2) - z.powi(2);
        if d < 0f64 {
            0f64
        } else {
            let d = d.powi(2);
            d.powi(2) * Self::dot(GRADIENTS[hash], x, y, z)
        }
    }
    #[inline(always)]
    pub fn grad<B>(hash: B, x: f64, y: f64, z: f64) -> f64
    where
        B: BitAnd<PermutationType, Output = PermutationType>,
    {
        Self::dot(GRADIENTS[hash.bitand(0xF) as usize], x, y, z)
    }
    pub fn noise(&self, x: f64, y: f64, z: f64, scale_y: f64, y_max: f64) -> f64 {
        let x_plus_offset = x + self.x_offset;
        let y_plus_offset = y + self.y_offset;
        let z_plus_offset = z + self.z_offset;

        let x = x_plus_offset.floor() as i32;
        let y = y_plus_offset.floor() as i32;
        let z = z_plus_offset.floor() as i32;

        let diffed_x = x_plus_offset - x as f64;
        let diffed_y = y_plus_offset - y as f64;
        let diffed_z = z_plus_offset - z as f64;

        let scaled_y = if scale_y != 0f64 {
            if y_max >= 0f64 && y_max < diffed_y {
                y_max
            } else {
                diffed_y
            }
        } else {
            diffed_y
        };
        self.sample_and_lerp(
            x as PermutationType,
            y as PermutationType,
            z as PermutationType,
            diffed_x,
            scaled_y - scaled_y,
            diffed_z,
            diffed_y,
        )
    }

    pub fn sample_and_lerp(
        &self,
        section_x: PermutationType,
        section_y: PermutationType,
        section_z: PermutationType,
        local_x: f64,
        local_y: f64,
        local_z: f64,
        fade_local_y: f64,
    ) -> f64 {
        let i = self.map(section_x);
        let j = self.map(section_x + 1);
        let k = self.map(i + section_y);
        let l = self.map((section_y + 1) + i);
        let m = self.map(j + section_y);
        let n = self.map((section_y + 1) + j);
        let d = Self::grad(self.map(k + section_z), local_x, local_y, local_z);
        let e = Self::grad(self.map(l + section_z), local_x - 1f64, local_y, local_z);
        let f = Self::grad(self.map(m + section_z), local_x, local_y - 1f64, local_z);
        let g = Self::grad(
            self.map(n + section_z),
            local_x - 1f64,
            local_y - 1f64,
            local_z,
        );
        let h = Self::grad(
            self.map(k + section_z + 1),
            local_x,
            local_y,
            local_z - 1f64,
        );
        let o = Self::grad(
            self.map(l + section_z + 1),
            local_x - 1f64,
            local_y,
            local_z - 1f64,
        );
        let p = Self::grad(
            self.map(m + section_z + 1),
            local_x,
            local_y - 1f64,
            local_z - 1f64,
        );
        let q = Self::grad(
            self.map(n + section_z + 1),
            local_x - 1f64,
            local_y - 1f64,
            local_z - 1f64,
        );

        let r = crate::math::perlin_fade(local_x);
        let s = crate::math::perlin_fade(fade_local_y);
        let t = crate::math::perlin_fade(local_z);

        crate::math::lerp3(r, s, t, d, e, f, g, h, o, p, q)
    }
}
