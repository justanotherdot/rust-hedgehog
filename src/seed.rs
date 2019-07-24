use rand::distributions::{Distribution, Uniform};
use rand::*;
use rand_core::{impls, RngCore};

const GOLDEN_GAMMA: u64 = 0x9e3779b97f4a7c15;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Seed {
    value: u64,
    gamma: u64,
}

#[inline(never)]
pub fn global() -> Seed {
    let mut rng = rand::thread_rng();
    from(rng.gen())
}

pub fn random() -> Seed {
    let (_, s) = split(global());
    s
}

pub fn from(x: u64) -> Seed {
    let value = mix64(x);
    let gamma = mix_gamma(x.wrapping_add(GOLDEN_GAMMA));
    Seed { value, gamma }
}

pub fn next(Seed { value, gamma }: Seed) -> (u64, Seed) {
    let value = value.wrapping_add(gamma);
    (value, Seed { value, gamma })
}

pub fn split(s0: Seed) -> (Seed, Seed) {
    let (v0, s1) = next(s0);
    let (g0, s2) = next(s1);
    let value = mix64(v0);
    let gamma = mix_gamma(g0);
    (s2, Seed { value, gamma })
}

pub fn next_word64(s0: Seed) -> (u64, Seed) {
    let (v0, s1) = next(s0);
    (mix64(v0), s1)
}

pub fn next_word32(s0: Seed) -> (u32, Seed) {
    let (v0, s1) = next(s0);
    (mix32(v0 as u32), s1)
}

// XXX Should this be BigInt?
pub fn next_integer(lo: isize, hi: isize, mut s0: Seed) -> (isize, Seed) {
    let v = Uniform::from(lo..=hi).sample(&mut s0);
    (v, s0)
}

pub fn next_double(lo: f64, hi: f64, mut s0: Seed) -> (f64, Seed) {
    // Could use lo..hi.into()
    let v = Uniform::from(lo..=hi).sample(&mut s0);
    (v, s0)
}

pub fn next_float(lo: f32, hi: f32, mut s0: Seed) -> (f32, Seed) {
    // Could use lo..hi.into()
    let v = Uniform::from(lo..=hi).sample(&mut s0);
    (v, s0)
}

pub fn mix64(x: u64) -> u64 {
    let y = (x ^ (x >> 33)).wrapping_mul(0xff51afd7ed558ccd);
    let z = (y ^ (y >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53);
    z ^ (z >> 33)
}

#[allow(overflowing_literals)]
#[allow(exceeding_bitshifts)]
pub fn mix32(x: u32) -> u32 {
    let y = (x ^ (x >> 33)).wrapping_mul(0xff51afd7ed558ccd);
    let z = (y ^ (y >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53);
    z >> 32
}

pub fn mix64_variant13(x: u64) -> u64 {
    let y = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    let z = (y ^ (y >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

pub fn mix_gamma(x: u64) -> u64 {
    let y = mix64_variant13(x) | 1;
    let n = (y ^ (y >> 1)).count_ones();
    if n < 24 {
        y ^ 0xaaaaaaaaaaaaaaaa
    } else {
        y
    }
}

// Not sure if the RngCore config intelligently works between 32 and 64 bit
// arch otherwise we need `#[cfg(target_pointer_width = "64")]`
impl RngCore for Seed {
    fn next_u32(&mut self) -> u32 {
        next_word32(self.clone()).0
    }

    fn next_u64(&mut self) -> u64 {
        next_word64(self.clone()).0
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        Ok(self.fill_bytes(dest))
    }
}

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn stub() {
        assert_eq!(1 + 1, 2);
    }
}
