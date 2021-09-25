pub type RansState = u64;
pub const RANS64_L: RansState = 1 << 31;

pub fn mul_hi(a: u64, b: u64) -> u64 {
    ((a as u128 * b as u128) >> 64) as u64
}

pub const PROB_BITS: u32 = 14;
pub const PROB_SCALE: u32 = 1 << PROB_BITS;
