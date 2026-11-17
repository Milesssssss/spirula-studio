use spirula_core::color::{dc_from_rgb, ColorBasis};
use spirula_core::math::quat_to_rotmat_f32;
use spirula_data::PointCloud;

#[derive(Clone, Debug)]
pub struct GaussianModel {
    pub means: Vec<[f32; 3]>,
    pub quats: Vec<[f32; 4]>,
    pub log_scales: Vec<[f32; 3]>,
    pub opacity_logit: Vec<f32>,
    pub dc: Vec<[f32; 3]>,
    pub sh_degree: i32,
    pub sb_lobes: i32,
    /// Coefficient-major SH then lobe-major spherical beta, one row per splat.
    pub features_sh: Vec<f32>,
}

impl GaussianModel {
    pub fn len(&self) -> usize {
        self.means.len()
    }

    pub fn basis(&self) -> ColorBasis {
        ColorBasis {
            sh_degree: self.sh_degree,
            sb_lobes: self.sb_lobes,
        }
    }

    pub fn sh_row(&self, i: usize) -> &[f32] {
        let stride = self.row_stride();
        if stride == 0 || self.features_sh.is_empty() {
            return &[];
        }
        let start = i * stride;
        &self.features_sh[start..start + stride]
    }

    pub fn row_stride(&self) -> usize {
        self.basis().sh_coeffs() as usize * 3 + self.sb_lobes as usize * 6
    }

    pub fn to_cloud(&self) -> spirula_data::SplatCloud {
        spirula_data::SplatCloud {
            sh_degree: self.sh_degree,
            sb_lobes: self.sb_lobes,
            primitive: "3dgs".into(),
            means: self.means.clone(),
            quats: self.quats.clone(),
            scales: self.log_scales.clone(),
            opacities: self.opacity_logit.clone(),
            features_dc: self.dc.clone(),
            features_sh: self.features_sh.clone(),
        }
    }
}

pub fn seed_from_points(points: &PointCloud, scale_init: f32, opacity_init: f32, cap: usize) -> GaussianModel {
    let n = (points.xyz.len()).min(cap);
    let mut means = Vec::with_capacity(n);
    for p in points.xyz.iter().take(n) {
        means.push([p[0] as f32, p[1] as f32, p[2] as f32]);
    }
    let dist = mean_neighbor_distance(&means);
    let logit = (opacity_init / (1.0 - opacity_init)).ln();
    let mut log_scales = Vec::with_capacity(n);
    let mut dc = Vec::with_capacity(n);
    for i in 0..n {
        let s = (scale_init * dist[i] + 1e-8).ln();
        log_scales.push([s, s, s]);
        let rgb = if let Some(c) = points.rgb.get(i) {
            [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]
        } else {
            [0.5, 0.5, 0.5]
        };
        dc.push(dc_from_rgb(rgb));
    }
    GaussianModel {
        means,
        quats: vec![[1.0, 0.0, 0.0, 0.0]; n],
        log_scales,
        opacity_logit: vec![logit; n],
        dc,
        sh_degree: 0,
        sb_lobes: 0,
        features_sh: Vec::new(),
    }
}

fn mean_neighbor_distance(means: &[[f32; 3]]) -> Vec<f32> {
    let n = means.len();
    let mut out = vec![1.0; n];
    if n < 2 {
        return out;
    }
    for i in 0..n {
        let mut best = [f32::MAX; 3];
        for j in 0..n {
            if i == j {
                continue;
            }
            let d = (0..3)
                .map(|k| means[i][k] - means[j][k])
                .map(|v| v * v)
                .sum::<f32>()
                .sqrt();
            if d < best[0] {
                best[2] = best[1];
                best[1] = best[0];
                best[0] = d;
            } else if d < best[1] {
                best[2] = best[1];
                best[1] = d;
            } else if d < best[2] {
                best[2] = d;
            }
        }
        let k = if n > 3 { 3 } else { n - 1 };
        let mut s = 0.0;
        for d in best.iter().take(k) {
            s += *d;
        }
        out[i] = (s / k as f32).max(1e-4);
    }
    out
}

pub fn longest_axis(quat: [f32; 4], log_scale: [f32; 3]) -> [f32; 3] {
    let rot = quat_to_rotmat_f32(quat);
    let axis = (0..3)
        .max_by(|&a, &b| log_scale[a].partial_cmp(&log_scale[b]).unwrap())
        .unwrap_or(0);
    let len = log_scale[axis].exp();
    [rot[0][axis] * len, rot[1][axis] * len, rot[2][axis] * len]
}
