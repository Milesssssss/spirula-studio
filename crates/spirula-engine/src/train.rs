use crate::adam::Adam;
use crate::densify::{densify, optimizer_reset, DensifyMode};
use crate::gaps::{port_gaps, Gap};
use crate::metrics::{image_l1, image_psnr};
use crate::model::GaussianModel;
use crate::raster::render;
use spirula_backend::{Backend, ProjectionBatch};
use spirula_core::camera::{CameraModel, Distortion};
use spirula_core::color::dc_from_rgb;
use spirula_core::config::{scheduled_lr, TrainConfig};
use spirula_core::{Error, Result};
use spirula_data::{decode_rgb, resize_max_edge, Dataset};

#[derive(Clone, Debug)]
pub struct View {
    pub name: String,
    pub w2c: [f32; 12],
    pub fx: f32,
    pub fy: f32,
    pub cx: f32,
    pub cy: f32,
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct TrainOptions {
    pub iterations: i32,
    pub densify: DensifyMode,
    pub background: [f32; 3],
    pub antialiased: bool,
    pub l1: f32,
    pub l2: f32,
    pub means_lr: f32,
    pub means_lr_final: f32,
    pub opacities_lr: f32,
    pub features_dc_lr: f32,
    /// Learning-rate horizon. `max_steps` when set, otherwise the run length.
    pub schedule_steps: i32,
    pub refine_every: i32,
    pub refine_start: i32,
    pub refine_stop: i32,
    pub min_opacity: f32,
    pub max_split_fraction: f32,
    pub cap_max: usize,
}

impl TrainOptions {
    pub fn from_config(cfg: &TrainConfig, iterations: i32, densify: DensifyMode) -> Self {
        Self {
            iterations,
            densify,
            background: cfg.background_color,
            antialiased: cfg.primitive == "mip",
            l1: cfg.l1_weight,
            l2: cfg.l2_weight,
            means_lr: cfg.means_lr,
            means_lr_final: cfg.means_lr_final,
            opacities_lr: cfg.opacities_lr,
            features_dc_lr: cfg.features_dc_lr,
            schedule_steps: cfg.max_steps.filter(|n| *n > 0).unwrap_or(iterations),
            refine_every: cfg.refine_every.max(1),
            refine_start: cfg.refine_start_iter,
            refine_stop: cfg.refine_stop_iter,
            min_opacity: cfg.min_opacity,
            max_split_fraction: cfg.max_split_fraction,
            cap_max: cfg.cap_max.max(0) as usize,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StepReport {
    pub step: i32,
    pub loss: f32,
    pub l1: f32,
    pub psnr: f32,
    pub splats: usize,
    pub view: String,
}

#[derive(Clone, Debug)]
pub struct TrainOutput {
    pub model: GaussianModel,
    pub steps: Vec<StepReport>,
    pub gaps: Vec<Gap>,
    pub notes: Vec<String>,
    pub preview: Vec<f32>,
    pub preview_width: u32,
    pub preview_height: u32,
}

pub fn train(
    mut model: GaussianModel,
    views: &[View],
    opts: &TrainOptions,
    backend: &mut dyn Backend,
    gaps: Vec<Gap>,
    mut notes: Vec<String>,
) -> Result<TrainOutput> {
    if views.is_empty() {
        return Err(Error::new("no trainable pinhole views"));
    }
    if model.len() == 0 {
        return Err(Error::new("no splats to train"));
    }
    let dropped = model.cap_to(opts.cap_max);
    if dropped > 0 {
        notes.push(format!(
            "initial model capped by {dropped} splats to {}",
            opts.cap_max
        ));
    }
    if model.len() == 0 {
        return Err(Error::new("cap_max left no splats"));
    }
    let mut adam_m = Adam::new(model.len() * 3);
    let mut adam_o = Adam::new(model.len());
    let mut adam_c = Adam::new(model.len() * 3);
    let mut steps = Vec::new();
    let mut preview = Vec::new();
    let mut preview_width = 0;
    let mut preview_height = 0;
    for step in 0..opts.iterations {
        let view = &views[step as usize % views.len()];
        let batch = ProjectionBatch {
            means: &model.means,
            quats: &model.quats,
            log_scales: &model.log_scales,
            opacity_logits: &model.opacity_logit,
            view: &view.w2c,
            fx: view.fx,
            fy: view.fy,
            cx: view.cx,
            cy: view.cy,
            width: view.width,
            height: view.height,
            antialiased: opts.antialiased,
        };
        let projected = backend.project(&batch)?;
        let (image, _) = render(
            &model,
            &projected,
            &view.w2c,
            view.fx,
            view.fy,
            view.width,
            view.height,
            opts.background,
            &[],
        );
        let (loss, grad) = loss_grad(&image, &view.rgb, opts.l1, opts.l2);
        let (_, g) = render(
            &model,
            &projected,
            &view.w2c,
            view.fx,
            view.fy,
            view.width,
            view.height,
            opts.background,
            &grad,
        );
        let lr = scheduled_lr(step, opts.schedule_steps, opts.means_lr, opts.means_lr_final);
        step_means(&mut model, &g.means, &mut adam_m, lr);
        step_vec(&mut model.opacity_logit, &g.opacity_logit, &mut adam_o, opts.opacities_lr);
        step_dc(&mut model, &g.dc, &mut adam_c, opts.features_dc_lr);
        if opts.densify != DensifyMode::Off
            && step >= opts.refine_start
            && step < opts.refine_stop
            && opts.refine_every > 0
            && step % opts.refine_every == 0
        {
            let stats = densify(
                &mut model,
                opts.densify,
                opts.min_opacity,
                opts.max_split_fraction,
                opts.cap_max,
            )?;
            if optimizer_reset(&stats) {
                notes.push(format!(
                    "step {step}: pruned {} and split {}, optimizer moments reset",
                    stats.pruned, stats.split
                ));
                adam_m = Adam::new(model.len() * 3);
                adam_o = Adam::new(model.len());
                adam_c = Adam::new(model.len() * 3);
            }
        }
        preview = image;
        preview_width = view.width;
        preview_height = view.height;
        steps.push(StepReport {
            step,
            loss,
            l1: image_l1(&preview, &view.rgb),
            psnr: image_psnr(&preview, &view.rgb),
            splats: model.len(),
            view: view.name.clone(),
        });
    }
    Ok(TrainOutput {
        model,
        steps,
        gaps,
        notes,
        preview,
        preview_width,
        preview_height,
    })
}

pub fn views_from_dataset(ds: &Dataset, max_edge: u32, linear: bool) -> Result<(Vec<View>, Vec<String>)> {
    let mut views = Vec::new();
    let mut notes = Vec::new();
    if linear {
        notes.push(
            "image_color_is_linear is set; exported config.json marks the splats linear".into(),
        );
    }
    let mut skipped_model = 0;
    let mut skipped_dist = 0;
    let mut skipped_image = 0;
    for frame in &ds.frames {
        if frame.model != CameraModel::Pinhole {
            skipped_model += 1;
            continue;
        }
        if frame.distortion != Distortion::None {
            skipped_dist += 1;
            continue;
        }
        let bytes = match std::fs::read(&frame.image_path) {
            Ok(b) => b,
            Err(_) => {
                skipped_image += 1;
                continue;
            }
        };
        let image = decode_rgb(&bytes)?;
        let image = resize_max_edge(&image, max_edge);
        let sx = image.width as f32 / frame.width.max(1) as f32;
        let sy = image.height as f32 / frame.height.max(1) as f32;
        let w2c = frame.w2c_cv();
        views.push(View {
            name: frame.name.clone(),
            w2c: w2c.map(|v| v as f32),
            fx: frame.fx as f32 * sx,
            fy: frame.fy as f32 * sy,
            cx: frame.cx as f32 * sx,
            cy: frame.cy as f32 * sy,
            width: image.width,
            height: image.height,
            rgb: image.rgb,
        });
    }
    if skipped_model > 0 {
        notes.push(format!(
            "{skipped_model} frames are not pinhole; the Rust trainer skips them"
        ));
    }
    if skipped_dist > 0 {
        notes.push(format!(
            "{skipped_dist} frames have lens distortion; the Rust projector skips them"
        ));
    }
    if skipped_image > 0 {
        notes.push(format!("{skipped_image} frames have no readable image"));
    }
    Ok((views, notes))
}

pub fn gaps_for(cfg: &TrainConfig) -> Vec<Gap> {
    port_gaps(cfg)
}

pub fn synthetic_scene() -> (GaussianModel, Vec<View>, TrainOptions) {
    let view_m = [1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let truth = GaussianModel {
        means: vec![[0.0, 0.0, 2.2], [0.35, -0.2, 2.6]],
        quats: vec![[1.0, 0.0, 0.0, 0.0]; 2],
        log_scales: vec![[0.18_f32.ln(); 3], [0.12_f32.ln(); 3]],
        opacity_logit: vec![1.2, 0.8],
        dc: vec![dc_from_rgb([0.85, 0.2, 0.15]), dc_from_rgb([0.2, 0.45, 0.9])],
        sh_degree: 0,
        sb_lobes: 0,
        features_sh: Vec::new(),
    };
    let (w, h) = (16u32, 12u32);
    let batch = ProjectionBatch {
        means: &truth.means,
        quats: &truth.quats,
        log_scales: &truth.log_scales,
        opacity_logits: &truth.opacity_logit,
        view: &view_m,
        fx: 22.0,
        fy: 22.0,
        cx: 8.0,
        cy: 6.0,
        width: w,
        height: h,
        antialiased: false,
    };
    let projected = spirula_core::project::project_pinhole(&batch);
    let (rgb, _) = render(&truth, &projected, &view_m, 22.0, 22.0, w, h, [0.0; 3], &[]);
    let mut init = truth;
    init.means[0][0] += 0.25;
    init.means[1][1] -= 0.2;
    init.dc[0] = dc_from_rgb([0.5, 0.5, 0.5]);
    let view = View {
        name: "synthetic".into(),
        w2c: view_m,
        fx: 22.0,
        fy: 22.0,
        cx: 8.0,
        cy: 6.0,
        width: w,
        height: h,
        rgb,
    };
    let mut cfg = TrainConfig::default();
    cfg.use_bilateral_grid = false;
    cfg.use_ppisp = false;
    cfg.ssim_lambda = 0.0;
    cfg.sh_degree = 0;
    cfg.sb_lobes = 0;
    cfg.primitive = "3dgs".into();
    cfg.l1_weight = 1.0;
    cfg.means_lr = 0.05;
    cfg.means_lr_final = 0.01;
    cfg.features_dc_lr = 0.05;
    cfg.opacities_lr = 0.05;
    let opts = TrainOptions::from_config(&cfg, 40, DensifyMode::Off);
    (init, vec![view], opts)
}

fn loss_grad(pred: &[f32], gt: &[f32], l1: f32, l2: f32) -> (f32, Vec<f32>) {
    let n = pred.len().min(gt.len()).max(1) as f32;
    let mut loss = 0.0_f32;
    let mut grad = vec![0.0; pred.len()];
    for i in 0..pred.len().min(gt.len()) {
        let d = pred[i] - gt[i];
        loss += l1 * d.abs() + l2 * d * d;
        let mut g = 0.0;
        if l1 != 0.0 {
            g += l1 * d.signum();
        }
        if l2 != 0.0 {
            g += l2 * 2.0 * d;
        }
        grad[i] = g / n;
    }
    (loss / n, grad)
}

fn step_means(model: &mut GaussianModel, grad: &[[f32; 3]], adam: &mut Adam, lr: f32) {
    let mut flat_p = Vec::with_capacity(model.len() * 3);
    let mut flat_g = Vec::with_capacity(model.len() * 3);
    for i in 0..model.len() {
        flat_p.extend_from_slice(&model.means[i]);
        flat_g.extend_from_slice(&grad[i]);
    }
    adam.step(&mut flat_p, &flat_g, lr);
    for i in 0..model.len() {
        model.means[i] = [flat_p[i * 3], flat_p[i * 3 + 1], flat_p[i * 3 + 2]];
    }
}

fn step_dc(model: &mut GaussianModel, grad: &[[f32; 3]], adam: &mut Adam, lr: f32) {
    let mut flat_p = Vec::with_capacity(model.len() * 3);
    let mut flat_g = Vec::with_capacity(model.len() * 3);
    for i in 0..model.len() {
        flat_p.extend_from_slice(&model.dc[i]);
        flat_g.extend_from_slice(&grad[i]);
    }
    adam.step(&mut flat_p, &flat_g, lr);
    for i in 0..model.len() {
        model.dc[i] = [flat_p[i * 3], flat_p[i * 3 + 1], flat_p[i * 3 + 2]];
    }
}

fn step_vec(param: &mut [f32], grad: &[f32], adam: &mut Adam, lr: f32) {
    adam.step(param, grad, lr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use spirula_backend::CpuBackend;

    #[test]
    fn synthetic_l1_falls() {
        let (model, views, opts) = synthetic_scene();
        let gaps = port_gaps(&TrainConfig {
            use_bilateral_grid: false,
            use_ppisp: false,
            ssim_lambda: 0.0,
            sh_degree: 0,
            primitive: "3dgs".into(),
            ..TrainConfig::default()
        });
        assert!(gaps.iter().all(|g| g.name != "bilateral_grid" && g.name != "ssim_loss"));
        let out = train(model, &views, &opts, &mut CpuBackend, gaps, Vec::new()).unwrap();
        let first = out.steps.first().unwrap().loss;
        let last = out.steps.last().unwrap().loss;
        assert!(last < first * 0.7, "first {first} last {last}");
        assert!(out.preview.iter().any(|v| *v > 0.0));
    }

    #[test]
    fn prune_and_split_with_the_same_count_drops_adam_state() {
        use crate::adam::Adam;
        use crate::densify::{densify, optimizer_reset};
        let mut model = GaussianModel {
            means: vec![[0.0; 3], [1.0, 0.0, 0.0]],
            quats: vec![[1.0, 0.0, 0.0, 0.0]; 2],
            log_scales: vec![[-6.0; 3], [0.0, -4.0, -4.0]],
            opacity_logit: vec![-8.0, 2.0],
            dc: vec![[0.0; 3]; 2],
            sh_degree: 0,
            sb_lobes: 0,
            features_sh: Vec::new(),
        };
        let mut adam = Adam::new(2);
        adam.step(&mut model.opacity_logit, &[1.0, -1.0], 0.1);
        let stats = densify(
            &mut model,
            DensifyMode::SplitPrune,
            0.05,
            1.0,
            8,
        )
        .unwrap();
        assert_eq!(stats.pruned, 1);
        assert_eq!(stats.split, 1);
        assert_eq!(model.len(), 2);
        if optimizer_reset(&stats) {
            adam = Adam::new(model.len());
        }
        let snapshot = model.opacity_logit.clone();
        adam.step(&mut model.opacity_logit, &[0.0, 0.0], 0.1);
        assert_eq!(model.opacity_logit, snapshot);
    }

    #[test]
    fn cap_applies_before_the_first_step() {
        let (mut model, views, mut opts) = synthetic_scene();
        model.means = vec![[0.0, 0.0, 2.0]; 4];
        model.quats = vec![[1.0, 0.0, 0.0, 0.0]; 4];
        model.log_scales = vec![[0.1_f32.ln(); 3]; 4];
        model.opacity_logit = vec![0.0, 1.0, 2.0, 3.0];
        model.dc = vec![[0.0; 3]; 4];
        opts.iterations = 3;
        opts.cap_max = 2;
        opts.densify = DensifyMode::Off;
        opts.refine_start = 500;
        let out = train(model, &views, &opts, &mut CpuBackend, Vec::new(), Vec::new()).unwrap();
        assert!(out.steps.iter().all(|s| s.splats == 2));
        assert!(out.notes.iter().any(|n| n.contains("capped")));
    }

    #[test]
    fn max_steps_is_the_learning_rate_horizon() {
        let (model, views, mut opts) = synthetic_scene();
        opts.iterations = 3;
        opts.means_lr = 0.05;
        opts.means_lr_final = 0.00001;
        opts.schedule_steps = 3;
        opts.densify = DensifyMode::Off;
        let short = train(
            model.clone(),
            &views,
            &opts,
            &mut CpuBackend,
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        opts.schedule_steps = 100;
        let long = train(model, &views, &opts, &mut CpuBackend, Vec::new(), Vec::new()).unwrap();
        assert_ne!(short.model.means, long.model.means);
    }
}
