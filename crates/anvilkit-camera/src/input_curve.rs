//! Input curve utilities for camera input processing.
//!
//! Provides dead zone filtering and power-curve response mapping
//! for mouse and gamepad stick input.

use glam::Vec2;

/// Input processing curve with dead zone and power response.
///
/// Transforms raw input values through:
/// 1. **Dead zone** — input below the threshold maps to zero
/// 2. **Re-normalization** — remaining range is scaled to `[0, 1]`
/// 3. **Power curve** — applies `x^power` for non-linear response
///
/// # Example
///
/// ```
/// use anvilkit_camera::input_curve::InputCurve;
///
/// let curve = InputCurve::new(0.1, 2.0); // 10% dead zone, quadratic
/// assert_eq!(curve.apply(0.05), 0.0);    // inside dead zone
/// assert!(curve.apply(0.5) < 0.5);       // quadratic dampens small values
/// ```
#[derive(Debug, Clone, Copy)]
pub struct InputCurve {
    /// Dead zone threshold `[0.0, 1.0]`. Input with absolute value below this returns 0.
    pub dead_zone: f32,
    /// Response curve power. `1.0` = linear, `2.0` = quadratic, `3.0` = cubic.
    pub power: f32,
}

impl Default for InputCurve {
    fn default() -> Self {
        Self {
            dead_zone: 0.0,
            power: 1.0,
        }
    }
}

impl InputCurve {
    /// Create a new input curve with the given dead zone and power.
    pub fn new(dead_zone: f32, power: f32) -> Self {
        Self { dead_zone, power }
    }

    /// Linear response with no dead zone (pass-through).
    pub fn linear() -> Self {
        Self::default()
    }

    /// Quadratic response with given dead zone.
    pub fn quadratic(dead_zone: f32) -> Self {
        Self::new(dead_zone, 2.0)
    }

    /// Cubic response with given dead zone.
    pub fn cubic(dead_zone: f32) -> Self {
        Self::new(dead_zone, 3.0)
    }

    /// Apply the curve to a raw input value. Preserves sign.
    ///
    /// Returns `0.0` if `|raw| < dead_zone`.
    pub fn apply(&self, raw: f32) -> f32 {
        let magnitude = raw.abs();
        if magnitude < self.dead_zone {
            return 0.0;
        }
        let range = 1.0 - self.dead_zone;
        if range <= f32::EPSILON {
            return raw.signum();
        }
        let normalized = (magnitude - self.dead_zone) / range;
        let curved = normalized.powf(self.power);
        curved * raw.signum()
    }

    /// Apply the curve to a 2D input (e.g., mouse delta or stick).
    pub fn apply_vec2(&self, raw: Vec2) -> Vec2 {
        Vec2::new(self.apply(raw.x), self.apply(raw.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_passthrough() {
        let curve = InputCurve::linear();
        assert!((curve.apply(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((curve.apply(-0.3) - (-0.3)).abs() < f32::EPSILON);
        assert!((curve.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_dead_zone_filters() {
        let curve = InputCurve::new(0.1, 1.0);
        assert_eq!(curve.apply(0.05), 0.0);
        assert_eq!(curve.apply(-0.05), 0.0);
        assert_eq!(curve.apply(0.0), 0.0);
    }

    #[test]
    fn test_dead_zone_edge() {
        let curve = InputCurve::new(0.1, 1.0);
        // Value just below dead zone
        assert_eq!(curve.apply(0.09), 0.0);
        // Value at dead zone boundary
        assert_eq!(curve.apply(0.1), 0.0);
        // Value just above dead zone
        assert!(curve.apply(0.11) > 0.0);
    }

    #[test]
    fn test_sign_preserved() {
        let curve = InputCurve::quadratic(0.0);
        let pos = curve.apply(0.5);
        let neg = curve.apply(-0.5);
        assert!(pos > 0.0);
        assert!(neg < 0.0);
        assert!((pos + neg).abs() < f32::EPSILON);
    }

    #[test]
    fn test_quadratic_response() {
        let curve = InputCurve::quadratic(0.0);
        // Quadratic: 0.5^2 = 0.25
        assert!((curve.apply(0.5) - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cubic_response() {
        let curve = InputCurve::cubic(0.0);
        // Cubic: 0.5^3 = 0.125
        assert!((curve.apply(0.5) - 0.125).abs() < f32::EPSILON);
    }

    #[test]
    fn test_full_input() {
        // Input of 1.0 should always map to 1.0
        for power in [1.0, 2.0, 3.0, 0.5] {
            let curve = InputCurve::new(0.0, power);
            assert!(
                (curve.apply(1.0) - 1.0).abs() < f32::EPSILON,
                "power={power}: apply(1.0) = {}",
                curve.apply(1.0)
            );
        }
        // With dead zone too
        let curve = InputCurve::new(0.2, 2.0);
        assert!(
            (curve.apply(1.0) - 1.0).abs() < f32::EPSILON,
            "with dead zone: apply(1.0) = {}",
            curve.apply(1.0)
        );
    }

    #[test]
    fn test_apply_vec2() {
        let curve = InputCurve::new(0.1, 1.0);
        let result = curve.apply_vec2(Vec2::new(0.05, 0.5));
        assert_eq!(result.x, 0.0); // inside dead zone
        assert!(result.y > 0.0); // outside dead zone
    }
}
