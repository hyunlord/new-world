//! Integer-hashed smooth value noise with no platform-dependent transcendentals.
use crate::runtime::agent::MovementRng;

pub(super) fn splitmix64(seed: u64) -> u64 {
    MovementRng::new(seed).next_u64()
}
pub(super) fn h64(seed: u64, salt: u64, x: u32, y: u32) -> u64 {
    splitmix64(
        seed ^ salt
            ^ u64::from(x).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ u64::from(y).wrapping_mul(0xC2B2_AE3D_27D4_EB4F),
    )
}
fn unit(hash: u64) -> f64 {
    (hash >> 11) as f64 / (1_u64 << 53) as f64
}
fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}
pub(super) fn value(seed: u64, salt: u64, x: u32, y: u32, spacing: u32) -> f64 {
    let lx = x / spacing;
    let ly = y / spacing;
    let tx = smooth(f64::from(x % spacing) / f64::from(spacing));
    let ty = smooth(f64::from(y % spacing) / f64::from(spacing));
    let a = unit(h64(seed, salt, lx, ly));
    let b = unit(h64(seed, salt, lx + 1, ly));
    let c = unit(h64(seed, salt, lx, ly + 1));
    let d = unit(h64(seed, salt, lx + 1, ly + 1));
    let top = a + (b - a) * tx;
    let bottom = c + (d - c) * tx;
    top + (bottom - top) * ty
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lattice_values_and_unit_interval() {
        assert_eq!(value(1, 2, 16, 32, 16), unit(h64(1, 2, 1, 2)));
        for y in 0..64 {
            for x in 0..64 {
                assert!((0.0..1.0).contains(&value(42, 3, x, y, 8)));
            }
        }
    }
}
