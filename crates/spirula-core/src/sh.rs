//! `sh_coeffs_to_color` from shaders/harmonics.slang (gsplat's basis).
//! `coeffs` is coefficient-major RGB: channel `c` of coefficient `j` is `3*j+c`.
//! The returned colour does not include the +0.5 display offset.

pub const SH_C0: f32 = 0.2820947917738781;

pub fn sh_coeffs_to_color(degree: i32, viewdir: [f32; 3], dc: [f32; 3], coeffs: &[f32]) -> [f32; 3] {
    [
        sh_channel(0, degree, viewdir, dc, coeffs),
        sh_channel(1, degree, viewdir, dc, coeffs),
        sh_channel(2, degree, viewdir, dc, coeffs),
    ]
}

fn sh_channel(c: usize, degree: i32, viewdir: [f32; 3], dc: [f32; 3], coeffs: &[f32]) -> f32 {
    let mut colors = SH_C0 * dc[c];
    if degree <= 0 {
        return colors;
    }
    let inv = 1.0
        / (viewdir[0] * viewdir[0] + viewdir[1] * viewdir[1] + viewdir[2] * viewdir[2]).sqrt();
    let (x, y, z) = (viewdir[0] * inv, viewdir[1] * inv, viewdir[2] * inv);
    let coef = |j: usize| coeffs[3 * j + c];

    let f_tmp0_a = 0.48860251190292;
    colors += f_tmp0_a * (-y * coef(0) + z * coef(1) - x * coef(2));
    if degree <= 1 {
        return colors;
    }

    let z2 = z * z;
    let f_tmp0_b = -1.092548430592079 * z;
    let f_tmp1_a = 0.5462742152960395;
    let f_c1 = x * x - y * y;
    let f_s1 = 2.0 * x * y;
    let p_sh6 = 0.9461746957575601 * z2 - 0.3153915652525201;
    let p_sh7 = f_tmp0_b * x;
    let p_sh5 = f_tmp0_b * y;
    let p_sh8 = f_tmp1_a * f_c1;
    let p_sh4 = f_tmp1_a * f_s1;
    colors += p_sh4 * coef(3) + p_sh5 * coef(4) + p_sh6 * coef(5) + p_sh7 * coef(6) + p_sh8 * coef(7);
    if degree <= 2 {
        return colors;
    }

    let f_tmp0_c = -2.285228997322329 * z2 + 0.4570457994644658;
    let f_tmp1_b = 1.445305721320277 * z;
    let f_tmp2_a = -0.5900435899266435;
    let f_c2 = x * f_c1 - y * f_s1;
    let f_s2 = x * f_s1 + y * f_c1;
    let p_sh12 = z * (1.865881662950577 * z2 - 1.119528997770346);
    let p_sh13 = f_tmp0_c * x;
    let p_sh11 = f_tmp0_c * y;
    let p_sh14 = f_tmp1_b * f_c1;
    let p_sh10 = f_tmp1_b * f_s1;
    let p_sh15 = f_tmp2_a * f_c2;
    let p_sh9 = f_tmp2_a * f_s2;
    colors += p_sh9 * coef(8)
        + p_sh10 * coef(9)
        + p_sh11 * coef(10)
        + p_sh12 * coef(11)
        + p_sh13 * coef(12)
        + p_sh14 * coef(13)
        + p_sh15 * coef(14);
    if degree <= 3 {
        return colors;
    }

    let f_tmp0_d = z * (-4.683325804901025 * z2 + 2.007139630671868);
    let f_tmp1_c = 3.31161143515146 * z2 - 0.47308734787878;
    let f_tmp2_b = -1.770130769779931 * z;
    let f_c3 = x * f_c2 - y * f_s2;
    let f_s3 = x * f_s2 + y * f_c2;
    let p_sh20 = 1.984313483298443 * z * p_sh12 - 1.006230589874905 * p_sh6;
    let p_sh21 = f_tmp0_d * x;
    let p_sh19 = f_tmp0_d * y;
    let p_sh22 = f_tmp1_c * f_c1;
    let p_sh18 = f_tmp1_c * f_s1;
    let p_sh23 = f_tmp2_b * f_c2;
    let p_sh17 = f_tmp2_b * f_s2;
    let p_sh24 = 0.6258357354491763 * f_c3;
    let p_sh16 = 0.6258357354491763 * f_s3;
    colors += p_sh16 * coef(15)
        + p_sh17 * coef(16)
        + p_sh18 * coef(17)
        + p_sh19 * coef(18)
        + p_sh20 * coef(19)
        + p_sh21 * coef(20)
        + p_sh22 * coef(21)
        + p_sh23 * coef(22)
        + p_sh24 * coef(23);
    colors
}

/// Display colour: clamp(sh + 0.5, 0, 1). `raw` is the pre-offset value,
/// which the backward uses for the `sh >= -0.5` gate.
pub fn display_color(raw: [f32; 3]) -> [f32; 3] {
    raw.map(|v| (v + 0.5).clamp(0.0, 1.0))
}

pub fn sh_degree_from_rest_coeffs(n_coeffs: usize) -> Option<i32> {
    if n_coeffs == 0 {
        return Some(0);
    }
    for d in 1..=4 {
        if d * (d + 2) == n_coeffs {
            return Some(d as i32);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degree_zero_is_c0_times_dc() {
        let raw = sh_coeffs_to_color(0, [1.0, 0.0, 0.0], [1.0, 0.0, -1.0], &[]);
        assert!((raw[0] - SH_C0).abs() < 1e-6);
        assert_eq!(raw[1], 0.0);
        assert!((raw[2] + SH_C0).abs() < 1e-6);
        let shown = display_color([0.0, 0.0, 0.0]);
        assert_eq!(shown, [0.5, 0.5, 0.5]);
    }

    #[test]
    fn degree_one_z_basis_matches_the_published_factor() {
        let mut coeffs = vec![0.0; 9];
        coeffs[3] = 1.0;
        let raw = sh_coeffs_to_color(1, [0.0, 0.0, 2.0], [0.0, 0.0, 0.0], &coeffs);
        assert!((raw[0] - 0.48860251190292).abs() < 1e-6);
        assert_eq!(raw[1], 0.0);
    }

    #[test]
    fn rest_count_selects_the_degree() {
        assert_eq!(sh_degree_from_rest_coeffs(0), Some(0));
        assert_eq!(sh_degree_from_rest_coeffs(3), Some(1));
        assert_eq!(sh_degree_from_rest_coeffs(15), Some(3));
        assert_eq!(sh_degree_from_rest_coeffs(24), Some(4));
        assert_eq!(sh_degree_from_rest_coeffs(7), None);
    }
}
