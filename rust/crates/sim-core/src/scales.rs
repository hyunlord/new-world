/// Type-safe scale wrappers for stress subsystem values.
///
/// ECS stores 0-1 normalized f64. Kernel functions (stat_curve, body)
/// operate on GDScript-native scales:
///   stress:     0-2000
///   reserve:    0-100
///   allostatic: 0-100
///   resilience: 0-100

const STRESS_SCALE: f64 = 2000.0;
const PERCENT_SCALE: f64 = 100.0;

/// Stress level on normalized 0..1 scale (ECS storage).
#[derive(Debug, Clone, Copy)]
pub struct NormStress(pub f64);

/// Stress level on native 0..2000 scale (kernel computation).
#[derive(Debug, Clone, Copy)]
pub struct NativeStress(pub f32);

/// Percent value on normalized 0..1 scale (reserve, allostatic, resilience).
#[derive(Debug, Clone, Copy)]
pub struct NormPercent(pub f64);

/// Percent value on native 0..100 scale (kernel computation).
#[derive(Debug, Clone, Copy)]
pub struct NativePercent(pub f32);

impl NormStress {
    pub fn to_native(self) -> NativeStress {
        NativeStress((self.0 * STRESS_SCALE) as f32)
    }
}

impl NativeStress {
    /// Convert to normalized. Division in f32 matches the original code's
    /// precision path (`(value / 2000.0).clamp(0.0, 1.0) as f64`).
    pub fn to_norm(self) -> NormStress {
        NormStress((self.0 / STRESS_SCALE as f32).clamp(0.0, 1.0) as f64)
    }
}

impl NormPercent {
    pub fn to_native(self) -> NativePercent {
        NativePercent((self.0 * PERCENT_SCALE) as f32)
    }
}

impl NativePercent {
    /// Convert to normalized. Division in f32 matches the original code's
    /// precision path (`(value / 100.0).clamp(0.0, 1.0) as f64`).
    pub fn to_norm(self) -> NormPercent {
        NormPercent((self.0 / PERCENT_SCALE as f32).clamp(0.0, 1.0) as f64)
    }
}

impl From<NormStress> for NativeStress {
    fn from(v: NormStress) -> Self {
        v.to_native()
    }
}

impl From<NativeStress> for NormStress {
    fn from(v: NativeStress) -> Self {
        v.to_norm()
    }
}

impl From<NormPercent> for NativePercent {
    fn from(v: NormPercent) -> Self {
        v.to_native()
    }
}

impl From<NativePercent> for NormPercent {
    fn from(v: NativePercent) -> Self {
        v.to_norm()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// f32 division roundtrip tolerance — matches the precision path used
    /// by the original GDScript-bridge code (`(f32 / scale).clamp() as f64`).
    const F32_ROUNDTRIP_EPS: f64 = 1e-7;

    #[test]
    fn stress_roundtrip_midpoint() {
        let norm = NormStress(0.5);
        let native = norm.to_native();
        assert!((native.0 - 1000.0).abs() < 0.01);
        let back = native.to_norm();
        assert!((back.0 - 0.5).abs() < F32_ROUNDTRIP_EPS);
    }

    #[test]
    fn stress_boundary_zero() {
        let native = NormStress(0.0).to_native();
        assert_eq!(native.0, 0.0);
        assert_eq!(native.to_norm().0, 0.0);
    }

    #[test]
    fn stress_boundary_one() {
        let native = NormStress(1.0).to_native();
        assert!((native.0 - 2000.0).abs() < 0.01);
        assert!((native.to_norm().0 - 1.0).abs() < F32_ROUNDTRIP_EPS);
    }

    #[test]
    fn stress_clamping_over() {
        let norm = NativeStress(2500.0).to_norm();
        assert_eq!(norm.0, 1.0);
    }

    #[test]
    fn stress_clamping_under() {
        let norm = NativeStress(-100.0).to_norm();
        assert_eq!(norm.0, 0.0);
    }

    #[test]
    fn percent_roundtrip() {
        let norm = NormPercent(0.8);
        let native = norm.to_native();
        assert!((native.0 - 80.0).abs() < 0.01);
        let back = native.to_norm();
        assert!((back.0 - 0.8).abs() < F32_ROUNDTRIP_EPS);
    }

    #[test]
    fn percent_boundary_zero() {
        let native = NormPercent(0.0).to_native();
        assert_eq!(native.0, 0.0);
        assert_eq!(native.to_norm().0, 0.0);
    }

    #[test]
    fn percent_boundary_one() {
        let native = NormPercent(1.0).to_native();
        assert!((native.0 - 100.0).abs() < 0.01);
        assert!((native.to_norm().0 - 1.0).abs() < F32_ROUNDTRIP_EPS);
    }

    #[test]
    fn percent_clamping_over() {
        let norm = NativePercent(150.0).to_norm();
        assert_eq!(norm.0, 1.0);
    }

    #[test]
    fn percent_clamping_under() {
        let norm = NativePercent(-50.0).to_norm();
        assert_eq!(norm.0, 0.0);
    }

    #[test]
    fn from_trait_stress() {
        let native: NativeStress = NormStress(0.25).into();
        assert!((native.0 - 500.0).abs() < 0.01);
        let norm: NormStress = NativeStress(500.0).into();
        assert!((norm.0 - 0.25).abs() < F32_ROUNDTRIP_EPS);
    }

    #[test]
    fn from_trait_percent() {
        let native: NativePercent = NormPercent(0.6).into();
        assert!((native.0 - 60.0).abs() < 0.01);
        let norm: NormPercent = NativePercent(60.0).into();
        assert!((norm.0 - 0.6).abs() < F32_ROUNDTRIP_EPS);
    }
}
