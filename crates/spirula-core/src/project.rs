//! Pinhole 3DGS projection, the no-distortion path of
//! shaders/primitive_3dgs.slang. The Vulkan compute shader repeats these
//! expressions; spirula-backend tests the two against each other.

use crate::math::quat_to_rotmat_f32;

pub const ALPHA_THRESHOLD: f32 = 1.0 / 255.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedSplat {
    pub mean2d: [f32; 2],
    /// Euclidean camera-space distance. Not the sort key.
    pub depth: f32,
    /// Pinhole and orthographic sort by camera-space z (`get_sorting_depth`).
    pub sorting_depth: f32,
    pub conic: [f32; 3],
    /// Activated opacity after the mip compensation.
    pub opacity: f32,
    /// sigmoid(logit), before mip compensation. The logit derivative uses this.
    pub sigmoid_opacity: f32,
    pub valid: bool,
}

impl ProjectedSplat {
    pub fn invalid() -> Self {
        Self {
            mean2d: [0.0; 2],
            depth: 0.0,
            sorting_depth: 0.0,
            conic: [0.0; 3],
            opacity: 0.0,
            sigmoid_opacity: 0.0,
            valid: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectionBatch<'a> {
    pub means: &'a [[f32; 3]],
    pub quats: &'a [[f32; 4]],
    pub log_scales: &'a [[f32; 3]],
    pub opacity_logits: &'a [f32],
    /// World-to-camera, OpenCV, row-major 3x4.
    pub view: &'a [f32; 12],
    pub fx: f32,
    pub fy: f32,
    pub cx: f32,
    pub cy: f32,
    pub width: u32,
    pub height: u32,
    pub antialiased: bool,
}

pub fn project_pinhole(batch: &ProjectionBatch<'_>) -> Vec<ProjectedSplat> {
    let n = batch.means.len();
    (0..n)
        .map(|i| {
            project_one(
                batch.means[i],
                batch.quats[i],
                batch.log_scales[i],
                batch.opacity_logits[i],
                batch,
            )
        })
        .collect()
}

pub fn project_one(
    mean: [f32; 3],
    quat: [f32; 4],
    log_scale: [f32; 3],
    opacity_logit: f32,
    batch: &ProjectionBatch<'_>,
) -> ProjectedSplat {
    let v = batch.view;
    let pc = [
        v[0] * mean[0] + v[1] * mean[1] + v[2] * mean[2] + v[3],
        v[4] * mean[0] + v[5] * mean[1] + v[6] * mean[2] + v[7],
        v[8] * mean[0] + v[9] * mean[1] + v[10] * mean[2] + v[11],
    ];
    if !(pc[2] > 0.0) {
        return ProjectedSplat::invalid();
    }
    let rot = quat_to_rotmat_f32(quat);
    let s = [log_scale[0].exp(), log_scale[1].exp(), log_scale[2].exp()];
    let mut m_rows = [[0.0; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            m_rows[r][c] = rot[r][c] * s[c];
        }
    }
    let mut cov = [[0.0; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            cov[r][c] = m_rows[r][0] * m_rows[c][0]
                + m_rows[r][1] * m_rows[c][1]
                + m_rows[r][2] * m_rows[c][2];
        }
    }
    let vr = [
        [v[0], v[1], v[2]],
        [v[4], v[5], v[6]],
        [v[8], v[9], v[10]],
    ];
    let mut cw = [[0.0; 3]; 3];
    for j in 0..3 {
        for r in 0..3 {
            cw[j][r] = cov[r][0] * vr[j][0] + cov[r][1] * vr[j][1] + cov[r][2] * vr[j][2];
        }
    }
    let mut cc = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            cc[i][j] = vr[i][0] * cw[j][0] + vr[i][1] * cw[j][1] + vr[i][2] * cw[j][2];
        }
    }

    let (fx, fy, cx, cy) = (batch.fx, batch.fy, batch.cx, batch.cy);
    let rz = 1.0 / pc[2];
    let j00 = fx * rz;
    let j11 = fy * rz;
    let mut j02 = -fx * pc[0] * rz * rz;
    let mut j12 = -fy * pc[1] * rz * rz;
    let width = batch.width as f32;
    let height = batch.height as f32;
    let lim_x_pos = width + 0.3 * (0.5 * width);
    let lim_x_neg = 0.3 * (0.5 * width);
    let lim_y_pos = height + 0.3 * (0.5 * height);
    let lim_y_neg = 0.3 * (0.5 * height);
    let max_jxz = -(-lim_x_neg - cx) * rz;
    let min_jxz = -(lim_x_pos - cx) * rz;
    let max_jyz = -(-lim_y_neg - cy) * rz;
    let min_jyz = -(lim_y_pos - cy) * rz;
    j02 = j02.clamp(min_jxz, max_jxz);
    j12 = j12.clamp(min_jyz, max_jyz);

    let jcv0 = [
        j00 * cc[0][0] + j02 * cc[2][0],
        j00 * cc[0][1] + j02 * cc[2][1],
        j00 * cc[0][2] + j02 * cc[2][2],
    ];
    let jcv1 = [
        j11 * cc[1][0] + j12 * cc[2][0],
        j11 * cc[1][1] + j12 * cc[2][1],
        j11 * cc[1][2] + j12 * cc[2][2],
    ];
    let mut a = j00 * jcv0[0] + j02 * jcv0[2];
    let b = j00 * jcv1[0] + j02 * jcv1[2];
    let mut c = j11 * jcv1[1] + j12 * jcv1[2];
    let det_orig = a * c - b * b;
    let eps = if batch.antialiased { 0.1 } else { 0.3 };
    a += eps;
    c += eps;
    let det = a * c - b * b;
    let sigmoid_opacity = 1.0 / (1.0 + (-opacity_logit).exp());
    let mut opacity = sigmoid_opacity;
    if !(det > 0.0) {
        return ProjectedSplat::invalid();
    }
    if batch.antialiased {
        opacity *= (det_orig / det).max(0.0).sqrt();
    }
    if !(opacity >= ALPHA_THRESHOLD) {
        return ProjectedSplat::invalid();
    }
    let inv_det = 1.0 / det;
    let conic = [c * inv_det, -b * inv_det, a * inv_det];
    let mean2d = [fx * pc[0] * rz + cx, fy * pc[1] * rz + cy];
    let extend = 3.33_f32.min((2.0 * (opacity / ALPHA_THRESHOLD).ln()).sqrt());
    let rx = extend * a.max(0.0).sqrt();
    let ry = extend * c.max(0.0).sqrt();
    if mean2d[0] + rx <= 0.0
        || mean2d[0] - rx >= width
        || mean2d[1] + ry <= 0.0
        || mean2d[1] - ry >= height
    {
        return ProjectedSplat::invalid();
    }
    let depth = (pc[0] * pc[0] + pc[1] * pc[1] + pc[2] * pc[2]).sqrt();
    ProjectedSplat {
        mean2d,
        depth,
        sorting_depth: pc[2],
        conic,
        opacity,
        sigmoid_opacity,
        valid: true,
    }
}

/// Direction from the camera to `mean`, the argument of `sh_coeffs_to_color`.
pub fn sh_viewdir(mean: [f32; 3], view: &[f32; 12]) -> [f32; 3] {
    let t = [view[3], view[7], view[11]];
    [
        mean[0] + view[0] * t[0] + view[4] * t[1] + view[8] * t[2],
        mean[1] + view[1] * t[0] + view[5] * t[1] + view[9] * t[2],
        mean[2] + view[2] * t[0] + view[6] * t[1] + view[10] * t[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_view() -> [f32; 12] {
        [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]
    }

    #[test]
    fn centered_isotropic_splat_lands_on_the_principal_point() {
        let view = identity_view();
        let mean = [[0.0, 0.0, 2.0]];
        let quat = [[1.0, 0.0, 0.0, 0.0]];
        let scale = [[0.1_f32.ln(), 0.1_f32.ln(), 0.1_f32.ln()]];
        let opac = [0.0];
        let batch = ProjectionBatch {
            means: &mean,
            quats: &quat,
            log_scales: &scale,
            opacity_logits: &opac,
            view: &view,
            fx: 100.0,
            fy: 80.0,
            cx: 32.0,
            cy: 24.0,
            width: 64,
            height: 48,
            antialiased: false,
        };
        let p = project_pinhole(&batch);
        assert!(p[0].valid);
        assert!((p[0].mean2d[0] - 32.0).abs() < 1e-4);
        assert!((p[0].mean2d[1] - 24.0).abs() < 1e-4);
        assert!((p[0].opacity - 0.5).abs() < 1e-5);
        let s2 = 0.01_f32;
        let a = (100.0_f32 / 2.0).powi(2) * s2 + 0.3;
        let c = (80.0_f32 / 2.0).powi(2) * s2 + 0.3;
        let det = a * c;
        assert!((p[0].conic[0] - c / det).abs() < 1e-4);
        assert!(p[0].conic[1].abs() < 1e-4);
        assert!((p[0].conic[2] - a / det).abs() < 1e-4);
        assert!((p[0].depth - 2.0).abs() < 1e-5);
        assert!((p[0].sorting_depth - 2.0).abs() < 1e-5);
        assert!((p[0].sigmoid_opacity - 0.5).abs() < 1e-5);
    }

    #[test]
    fn off_axis_pinhole_sorts_by_z_not_radius() {
        let view = identity_view();
        let mean = [[1.0, 0.0, 2.0]];
        let quat = [[1.0, 0.0, 0.0, 0.0]];
        let scale = [[0.0, 0.0, 0.0]];
        let opac = [0.0];
        let batch = ProjectionBatch {
            means: &mean,
            quats: &quat,
            log_scales: &scale,
            opacity_logits: &opac,
            view: &view,
            fx: 30.0,
            fy: 30.0,
            cx: 32.0,
            cy: 32.0,
            width: 64,
            height: 64,
            antialiased: false,
        };
        let p = project_pinhole(&batch)[0];
        assert!(p.valid);
        assert!((p.sorting_depth - 2.0).abs() < 1e-5);
        let radius = (1.0_f32 + 4.0).sqrt();
        assert!((p.depth - radius).abs() < 1e-4);
        assert!((p.sorting_depth - p.depth).abs() > 0.1);
    }

    #[test]
    fn a_splat_behind_the_camera_is_invalid() {
        let view = identity_view();
        let mean = [[0.0, 0.0, -1.0]];
        let quat = [[1.0, 0.0, 0.0, 0.0]];
        let scale = [[0.0, 0.0, 0.0]];
        let opac = [2.0];
        let batch = ProjectionBatch {
            means: &mean,
            quats: &quat,
            log_scales: &scale,
            opacity_logits: &opac,
            view: &view,
            fx: 50.0,
            fy: 50.0,
            cx: 16.0,
            cy: 16.0,
            width: 32,
            height: 32,
            antialiased: false,
        };
        assert!(!project_pinhole(&batch)[0].valid);
    }
}
