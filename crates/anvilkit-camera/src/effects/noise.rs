//! Inline gradient noise for camera effects.
//!
//! Provides deterministic 1D and 2D gradient noise functions that return
//! smooth, continuous values in `[-1.0, 1.0]`. Used by the trauma shake
//! system to produce natural-looking camera displacement.
//!
//! No external dependencies — pure `std` math.

#![allow(clippy::unreadable_literal)]

// 256-entry permutation table (Ken Perlin's original).
const PERM: [u8; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225,
    140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148,
    247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32,
    57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122,
    60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54,
    65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169,
    200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64,
    52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212,
    207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213,
    119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9,
    129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104,
    218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241,
    81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157,
    184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93,
    222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
];

/// Fade function for smooth interpolation: 6t^5 - 15t^4 + 10t^3.
#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Linear interpolation.
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Hash function using the permutation table.
#[inline]
fn hash(i: i32) -> u8 {
    PERM[(i & 255) as usize]
}

/// 1D gradient: returns +1 or -1.
#[inline]
fn grad_1d(hash: u8) -> f32 {
    if hash & 1 == 0 { 1.0 } else { -1.0 }
}

/// 2D gradient: returns one of 4 gradients based on hash.
#[inline]
fn grad_2d(hash: u8, x: f32, y: f32) -> f32 {
    match hash & 3 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        _ => -x - y,
    }
}

/// 1D gradient noise. Returns a value in `[-1.0, 1.0]`.
///
/// Deterministic: same input always produces the same output.
/// Smooth: adjacent inputs produce smoothly varying outputs.
pub fn gradient_noise_1d(x: f32) -> f32 {
    let xi = x.floor() as i32;
    let xf = x - x.floor();
    let u = fade(xf);

    let a = grad_1d(hash(xi)) * xf;
    let b = grad_1d(hash(xi + 1)) * (xf - 1.0);

    lerp(a, b, u)
}

/// 2D gradient noise. Returns a value in `[-1.0, 1.0]`.
///
/// Deterministic: same `(x, y)` always produces the same output.
/// Smooth: nearby inputs produce smoothly varying outputs.
pub fn gradient_noise_2d(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();

    let u = fade(xf);
    let v = fade(yf);

    // Hash the four corners
    let aa = hash(hash(xi) as i32 + yi);
    let ab = hash(hash(xi) as i32 + yi + 1);
    let ba = hash(hash(xi + 1) as i32 + yi);
    let bb = hash(hash(xi + 1) as i32 + yi + 1);

    // Compute gradients at each corner
    let g00 = grad_2d(aa, xf, yf);
    let g10 = grad_2d(ba, xf - 1.0, yf);
    let g01 = grad_2d(ab, xf, yf - 1.0);
    let g11 = grad_2d(bb, xf - 1.0, yf - 1.0);

    // Bilinear interpolation
    let x0 = lerp(g00, g10, u);
    let x1 = lerp(g01, g11, u);
    lerp(x0, x1, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_1d_deterministic() {
        let a = gradient_noise_1d(1.234);
        let b = gradient_noise_1d(1.234);
        assert_eq!(a, b);
    }

    #[test]
    fn test_noise_2d_deterministic() {
        let a = gradient_noise_2d(1.234, 5.678);
        let b = gradient_noise_2d(1.234, 5.678);
        assert_eq!(a, b);
    }

    #[test]
    fn test_noise_1d_range() {
        for i in 0..1000 {
            let x = i as f32 * 0.1 - 50.0;
            let v = gradient_noise_1d(x);
            assert!(
                v >= -1.0 && v <= 1.0,
                "noise_1d({x}) = {v}, out of range"
            );
        }
    }

    #[test]
    fn test_noise_2d_range() {
        for i in 0..100 {
            for j in 0..100 {
                let x = i as f32 * 0.1 - 5.0;
                let y = j as f32 * 0.1 - 5.0;
                let v = gradient_noise_2d(x, y);
                assert!(
                    v >= -1.5 && v <= 1.5,
                    "noise_2d({x}, {y}) = {v}, out of expected range"
                );
            }
        }
    }

    #[test]
    fn test_noise_1d_continuity() {
        for i in 0..1000 {
            let x = i as f32 * 0.01;
            let a = gradient_noise_1d(x);
            let b = gradient_noise_1d(x + 0.01);
            let diff = (a - b).abs();
            assert!(
                diff < 0.15,
                "noise_1d discontinuity at {x}: |{a} - {b}| = {diff}"
            );
        }
    }

    #[test]
    fn test_noise_2d_continuity() {
        for i in 0..100 {
            let x = i as f32 * 0.01;
            let y = 3.14;
            let a = gradient_noise_2d(x, y);
            let b = gradient_noise_2d(x + 0.01, y);
            let diff = (a - b).abs();
            assert!(
                diff < 0.15,
                "noise_2d discontinuity at ({x}, {y}): diff = {diff}"
            );
        }
    }

    #[test]
    fn test_noise_varies() {
        // Avoid integer lattice points (where gradient noise is always 0)
        let a = gradient_noise_2d(0.3, 0.7);
        let b = gradient_noise_2d(1.8, 2.4);
        assert_ne!(a, b, "noise should vary for different inputs");
    }
}
