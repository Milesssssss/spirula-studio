//! Front-to-back EWA composite. The backward holds the 2D covariance fixed
//! and chains the mean through the pinhole Jacobian.

use crate::model::GaussianModel;
use spirula_core::color::lobe_color;
use spirula_core::project::{sh_viewdir, ProjectedSplat, ALPHA_THRESHOLD};
use spirula_core::sh::{display_color, sh_coeffs_to_color, SH_C0};

#[derive(Clone, Debug)]
pub struct RasterGrad {
    pub means: Vec<[f32; 3]>,
    pub opacity_logit: Vec<f32>,
    pub dc: Vec<[f32; 3]>,
}

pub fn render(
    model: &GaussianModel,
    projected: &[ProjectedSplat],
    view: &[f32; 12],
    fx: f32,
    fy: f32,
    width: u32,
    height: u32,
    background: [f32; 3],
    pixel_grad: &[f32],
) -> (Vec<f32>, RasterGrad) {
    let n = model.len();
    let mut order: Vec<usize> = (0..n).filter(|&i| projected[i].valid).collect();
    order.sort_by(|&a, &b| {
        projected[a]
            .depth
            .partial_cmp(&projected[b].depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let pixels = (width * height) as usize;
    let mut image = vec![0.0; pixels * 3];
    let mut grads = RasterGrad {
        means: vec![[0.0; 3]; n],
        opacity_logit: vec![0.0; n],
        dc: vec![[0.0; 3]; n],
    };
    let want_grad = pixel_grad.len() == image.len();
    for y in 0..height {
        for x in 0..width {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let pix = (y * width + x) as usize;
            let mut hits = Vec::new();
            let mut trans = 1.0_f32;
            let mut rgb = [0.0_f32; 3];
            for &i in &order {
                if trans < 1e-4 {
                    break;
                }
                let p = &projected[i];
                let dx = p.mean2d[0] - px;
                let dy = p.mean2d[1] - py;
                let sigma =
                    0.5 * (p.conic[0] * dx * dx + p.conic[2] * dy * dy) + p.conic[1] * dx * dy;
                if !(sigma >= 0.0) {
                    continue;
                }
                let vis = (-sigma).exp();
                let raw_alpha = p.opacity * vis;
                let alpha = raw_alpha.min(0.999);
                if alpha <= ALPHA_THRESHOLD {
                    continue;
                }
                let (raw_sh, color) = splat_color(model, i, view);
                for c in 0..3 {
                    rgb[c] += color[c] * alpha * trans;
                }
                hits.push(Hit {
                    index: i,
                    alpha,
                    trans,
                    color,
                    raw_sh,
                    dx,
                    dy,
                    conic: p.conic,
                    opacity: p.opacity,
                    clamped: raw_alpha > 0.999,
                });
                trans *= 1.0 - alpha;
            }
            for c in 0..3 {
                rgb[c] += background[c] * trans;
                image[pix * 3 + c] = rgb[c];
            }
            if want_grad {
                backward_pixel(
                    model,
                    view,
                    fx,
                    fy,
                    background,
                    &hits,
                    [
                        pixel_grad[pix * 3],
                        pixel_grad[pix * 3 + 1],
                        pixel_grad[pix * 3 + 2],
                    ],
                    &mut grads,
                );
            }
        }
    }
    (image, grads)
}

struct Hit {
    index: usize,
    alpha: f32,
    trans: f32,
    color: [f32; 3],
    raw_sh: [f32; 3],
    dx: f32,
    dy: f32,
    conic: [f32; 3],
    opacity: f32,
    clamped: bool,
}

fn backward_pixel(
    model: &GaussianModel,
    view: &[f32; 12],
    fx: f32,
    fy: f32,
    background: [f32; 3],
    hits: &[Hit],
    d_pixel: [f32; 3],
    grads: &mut RasterGrad,
) {
    let mut accum = background;
    let mut last_alpha = 0.0;
    let mut last_color = [0.0_f32; 3];
    for hit in hits.iter().rev() {
        accum = [
            last_alpha * last_color[0] + (1.0 - last_alpha) * accum[0],
            last_alpha * last_color[1] + (1.0 - last_alpha) * accum[1],
            last_alpha * last_color[2] + (1.0 - last_alpha) * accum[2],
        ];
        let mut d_alpha = 0.0;
        for c in 0..3 {
            d_alpha += (hit.color[c] - accum[c]) * d_pixel[c];
            let shown = hit.raw_sh[c] + 0.5;
            if shown > 0.0 && shown < 1.0 {
                grads.dc[hit.index][c] += SH_C0 * hit.alpha * hit.trans * d_pixel[c];
            }
        }
        d_alpha *= hit.trans;
        if !hit.clamped && hit.opacity > 1e-8 {
            let dsigma_dx = hit.conic[0] * hit.dx + hit.conic[1] * hit.dy;
            let dsigma_dy = hit.conic[1] * hit.dx + hit.conic[2] * hit.dy;
            let d_mx = d_alpha * (-hit.alpha * dsigma_dx);
            let d_my = d_alpha * (-hit.alpha * dsigma_dy);
            let vis = hit.alpha / hit.opacity;
            let d_opac = d_alpha * vis;
            grads.opacity_logit[hit.index] += d_opac * hit.opacity * (1.0 - hit.opacity);
            chain_mean(view, fx, fy, model.means[hit.index], d_mx, d_my, &mut grads.means[hit.index]);
        }
        last_alpha = hit.alpha;
        last_color = hit.color;
    }
}

fn chain_mean(
    view: &[f32; 12],
    fx: f32,
    fy: f32,
    mean: [f32; 3],
    d_mx: f32,
    d_my: f32,
    grad: &mut [f32; 3],
) {
    let pc = [
        view[0] * mean[0] + view[1] * mean[1] + view[2] * mean[2] + view[3],
        view[4] * mean[0] + view[5] * mean[1] + view[6] * mean[2] + view[7],
        view[8] * mean[0] + view[9] * mean[1] + view[10] * mean[2] + view[11],
    ];
    if !(pc[2] > 1e-8) {
        return;
    }
    let rz = 1.0 / pc[2];
    let dpc_x = d_mx * fx * rz;
    let dpc_y = d_my * fy * rz;
    let dpc_z = d_mx * (-fx * pc[0] * rz * rz) + d_my * (-fy * pc[1] * rz * rz);
    grad[0] += view[0] * dpc_x + view[4] * dpc_y + view[8] * dpc_z;
    grad[1] += view[1] * dpc_x + view[5] * dpc_y + view[9] * dpc_z;
    grad[2] += view[2] * dpc_x + view[6] * dpc_y + view[10] * dpc_z;
}

fn splat_color(model: &GaussianModel, i: usize, view: &[f32; 12]) -> ([f32; 3], [f32; 3]) {
    let dir = sh_viewdir(model.means[i], view);
    let row = model.sh_row(i);
    let sh_coeffs = model.basis().sh_coeffs() as usize * 3;
    let sh = if row.len() >= sh_coeffs { &row[..sh_coeffs] } else { &[] };
    let mut raw = sh_coeffs_to_color(model.sh_degree, dir, model.dc[i], sh);
    if model.sb_lobes > 0 && row.len() >= sh_coeffs + model.sb_lobes as usize * 6 {
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-8);
        let unit = [dir[0] / len, dir[1] / len, dir[2] / len];
        let lobes = &row[sh_coeffs..];
        for l in 0..model.sb_lobes as usize {
            let mut lobe = [0.0; 6];
            lobe.copy_from_slice(&lobes[l * 6..l * 6 + 6]);
            let add = lobe_color(&lobe, [unit[0] as f64, unit[1] as f64, unit[2] as f64]);
            for c in 0..3 {
                raw[c] += add[c] as f32;
            }
        }
    }
    (raw, display_color(raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use spirula_core::project::{project_pinhole, ProjectionBatch};

    #[test]
    fn mean_gradient_matches_a_finite_difference() {
        let view = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let model = GaussianModel {
            means: vec![[0.05, -0.02, 2.0]],
            quats: vec![[1.0, 0.0, 0.0, 0.0]],
            log_scales: vec![[0.4_f32.ln(); 3]],
            opacity_logit: vec![1.0],
            dc: vec![[0.4, -0.2, 0.1]],
            sh_degree: 0,
            sb_lobes: 0,
            features_sh: Vec::new(),
        };
        let (w, h) = (8u32, 8u32);
        let gt = vec![0.2; (w * h * 3) as usize];
        let eps = 1e-3_f32;
        let loss = |mean_x: f32| -> f32 {
            let mut m = model.clone();
            m.means[0][0] = mean_x;
            let batch = ProjectionBatch {
                means: &m.means,
                quats: &m.quats,
                log_scales: &m.log_scales,
                opacity_logits: &m.opacity_logit,
                view: &view,
                fx: 16.0,
                fy: 16.0,
                cx: 4.0,
                cy: 4.0,
                width: w,
                height: h,
                antialiased: false,
            };
            let proj = project_pinhole(&batch);
            let (img, _) = render(&m, &proj, &view, 16.0, 16.0, w, h, [0.0; 3], &[]);
            image_l1_loss(&img, &gt)
        };
        let base = model.means[0][0];
        let numerical = (loss(base + eps) - loss(base - eps)) / (2.0 * eps);
        let batch = ProjectionBatch {
            means: &model.means,
            quats: &model.quats,
            log_scales: &model.log_scales,
            opacity_logits: &model.opacity_logit,
            view: &view,
            fx: 16.0,
            fy: 16.0,
            cx: 4.0,
            cy: 4.0,
            width: w,
            height: h,
            antialiased: false,
        };
        let proj = project_pinhole(&batch);
        let (img, _) = render(&model, &proj, &view, 16.0, 16.0, w, h, [0.0; 3], &[]);
        let grad_px = l1_grad(&img, &gt);
        let (_, g) = render(&model, &proj, &view, 16.0, 16.0, w, h, [0.0; 3], &grad_px);
        let analytic = g.means[0][0];
        let scale = numerical.abs().max(analytic.abs()).max(1e-6);
        assert!(
            (numerical - analytic).abs() / scale < 0.05,
            "numerical {numerical} analytic {analytic}"
        );
    }

    fn image_l1_loss(pred: &[f32], gt: &[f32]) -> f32 {
        pred.iter().zip(gt).map(|(a, b)| (a - b).abs()).sum::<f32>() / pred.len() as f32
    }

    fn l1_grad(pred: &[f32], gt: &[f32]) -> Vec<f32> {
        let n = pred.len() as f32;
        pred.iter()
            .zip(gt)
            .map(|(a, b)| (a - b).signum() / n)
            .collect()
    }
}
