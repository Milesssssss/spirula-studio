//! What one `features_sh` row holds, and one spherical-beta lobe.
//! Layout: src/core/ColorBasis.h and src/core/SphericalBeta.h.

use crate::sh::SH_C0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorBasis {
    pub sh_degree: i32,
    pub sb_lobes: i32,
}

impl ColorBasis {
    pub const MAX_SH_DEGREE: i32 = 4;
    pub const MAX_SB_LOBES: i32 = 8;
    pub const SB_COEFFS_PER_LOBE: i32 = 2;
    pub const LOBE_FLOATS: usize = 6;

    pub fn sh_coeffs_of(degree: i32) -> i32 {
        degree * (degree + 2)
    }

    pub fn sh_coeffs(self) -> i32 {
        Self::sh_coeffs_of(self.sh_degree)
    }

    pub fn sb_coeffs(self) -> i32 {
        Self::SB_COEFFS_PER_LOBE * self.sb_lobes
    }

    pub fn coeffs(self) -> i32 {
        self.sh_coeffs() + self.sb_coeffs()
    }

    pub fn validate(self) -> Result<(), String> {
        if !(0..=Self::MAX_SH_DEGREE).contains(&self.sh_degree) {
            return Err(format!(
                "sh_degree {} is outside 0..={}",
                self.sh_degree,
                Self::MAX_SH_DEGREE
            ));
        }
        if !(0..=Self::MAX_SB_LOBES).contains(&self.sb_lobes) {
            return Err(format!(
                "sb_lobes {} is outside 0..={}",
                self.sb_lobes,
                Self::MAX_SB_LOBES
            ));
        }
        Ok(())
    }
}

pub const SOFTPLUS_BETA: f64 = 6.931471805599453;

pub fn softplus(x: f64) -> f64 {
    let bx = SOFTPLUS_BETA * x;
    (bx.max(0.0) + (-bx.abs()).exp().ln_1p()) / SOFTPLUS_BETA
}

pub fn lobe_axis(theta: f64, phi: f64) -> [f64; 3] {
    let st = theta.sin();
    [st * phi.cos(), st * phi.sin(), theta.cos()]
}

/// Colour one lobe adds for a unit view direction. `lobe` is
/// `(r, g, b, theta, phi, beta)`.
pub fn lobe_color(lobe: &[f32; 6], dir: [f64; 3]) -> [f64; 3] {
    let axis = lobe_axis(lobe[3] as f64, lobe[4] as f64);
    let d = dir[0] * axis[0] + dir[1] * axis[1] + dir[2] * axis[2];
    let w = if d > 0.0 {
        d.powf(4.0 * (lobe[5] as f64).exp())
    } else {
        0.0
    };
    [
        w * softplus(lobe[0] as f64),
        w * softplus(lobe[1] as f64),
        w * softplus(lobe[2] as f64),
    ]
}

pub fn init_lobe(u: f32, v: f32) -> [f32; 6] {
    [
        0.0,
        0.0,
        0.0,
        (1.0 - 2.0 * u as f64).acos() as f32,
        (2.0 * std::f64::consts::PI * v as f64) as f32,
        0.0,
    ]
}

pub fn dc_from_rgb(rgb: [f32; 3]) -> [f32; 3] {
    rgb.map(|c| (c - 0.5) / SH_C0)
}

pub fn rgb_from_dc(dc: [f32; 3]) -> [f32; 3] {
    dc.map(|c| (SH_C0 * c + 0.5).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basis_width_matches_color_basis_h() {
        let b = ColorBasis {
            sh_degree: 3,
            sb_lobes: 2,
        };
        assert_eq!(b.sh_coeffs(), 15);
        assert_eq!(b.sb_coeffs(), 4);
        assert_eq!(b.coeffs(), 19);
        assert!(ColorBasis {
            sh_degree: 5,
            sb_lobes: 0
        }
        .validate()
        .is_err());
    }

    #[test]
    fn a_fresh_lobe_is_dim_and_a_facing_lobe_adds_colour() {
        let lobe = init_lobe(0.0, 0.0);
        let along = lobe_axis(lobe[3] as f64, lobe[4] as f64);
        let add = lobe_color(&lobe, along);
        let away = lobe_color(&lobe, [-along[0], -along[1], -along[2]]);
        assert!(add[0] > 0.05 && add[0] < 0.2);
        assert!(away[0].abs() < 1e-12);
    }

    #[test]
    fn dc_roundtrip_through_the_sh0_offset() {
        let rgb = [0.2, 0.5, 0.9];
        let back = rgb_from_dc(dc_from_rgb(rgb));
        for i in 0..3 {
            assert!((rgb[i] - back[i]).abs() < 1e-5);
        }
    }
}
