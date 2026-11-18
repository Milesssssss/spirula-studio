//! Features that change a C++ run and are not applied by this step.

use spirula_core::config::TrainConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gap {
    pub name: &'static str,
    pub detail: String,
    /// The loaded config asks for this and the step does not apply it.
    pub affects_result: bool,
}

pub fn port_gaps(cfg: &TrainConfig) -> Vec<Gap> {
    let mut gaps = Vec::new();
    if cfg.use_bilateral_grid {
        gaps.push(gap("bilateral_grid", "use_bilateral_grid is on; the grid is not applied", true));
    }
    if cfg.use_ppisp {
        gaps.push(gap("ppisp", "use_ppisp is on; per-image colour correction is not applied", true));
    }
    if cfg.ssim_lambda != 0.0 {
        gaps.push(gap(
            "ssim_loss",
            &format!(
                "ssim_lambda is {}; the gradient uses l1_weight and l2_weight only",
                cfg.ssim_lambda
            ),
            true,
        ));
    }
    if cfg.sh_degree > 0 {
        gaps.push(gap(
            "sh_bands",
            &format!(
                "sh_degree {} is evaluated; bands above DC are not optimized",
                cfg.sh_degree
            ),
            true,
        ));
    }
    if cfg.sb_lobes > 0 {
        gaps.push(gap("spherical_beta", "spherical-beta lobes are evaluated and not optimized", true));
    }
    if cfg.primitive != "3dgs" && cfg.primitive != "mip" {
        gaps.push(gap(
            "primitive",
            &format!("primitive '{}' is not the pinhole EWA projector", cfg.primitive),
            true,
        ));
    }
    if cfg.load_masks {
        gaps.push(gap("masks", "load_masks is on; mask files are not applied", true));
    }
    if cfg.apply_loss_for_mask.is_some() {
        gaps.push(gap(
            "mask_loss",
            "apply_loss_for_mask is set; the mask loss policy is not applied",
            true,
        ));
    }
    if cfg.depth_supervision_weight != 0.0 {
        gaps.push(gap(
            "depth_supervision",
            "depth_supervision_weight is nonzero; depth loss is not applied",
            true,
        ));
    }
    if cfg.normal_supervision_weight != 0.0 {
        gaps.push(gap(
            "normal_supervision",
            "normal_supervision_weight is nonzero; normal loss is not applied",
            true,
        ));
    }
    if cfg.background_mode != "color" {
        gaps.push(gap(
            "background_mode",
            &format!(
                "background_mode is '{}'; only color is applied",
                cfg.background_mode
            ),
            true,
        ));
    }
    if !cfg.resume.is_empty() {
        gaps.push(gap(
            "resume",
            "resume is set; checkpoint resume is not implemented",
            true,
        ));
    }
    if cfg.orientation_method != "none" {
        gaps.push(gap(
            "pose_normalization",
            &format!(
                "orientation_method is '{}'; pose normalization is not applied",
                cfg.orientation_method
            ),
            true,
        ));
    }
    if cfg.random_init != "auto" && cfg.random_init != "never" && cfg.random_init != "always" {
        gaps.push(gap(
            "random_init",
            &format!(
                "random_init '{}' is not never, auto, or always",
                cfg.random_init
            ),
            true,
        ));
    }
    if !cfg.init_ply.is_empty() && cfg.random_init == "always" {
        gaps.push(gap(
            "random_init",
            "random_init always does not replace an init ply",
            true,
        ));
    }
    if cfg.scales_lr != TrainConfig::default().scales_lr || cfg.quats_lr != TrainConfig::default().quats_lr
    {
        gaps.push(gap(
            "scale_quat_lr",
            "scales_lr and quats_lr are not applied; those parameters are not stepped",
            true,
        ));
    }
    if cfg.steps_per_save != TrainConfig::default().steps_per_save {
        gaps.push(gap(
            "steps_per_save",
            "intermediate checkpoints are not written",
            false,
        ));
    }
    if cfg.disable_viewer {
        gaps.push(gap(
            "disable_viewer",
            "disable_viewer is not read; omit --viewer to keep the status server off",
            false,
        ));
    }
    if cfg.features_sh_lr != TrainConfig::default().features_sh_lr && cfg.sh_degree == 0 {
        gaps.push(gap(
            "features_sh_lr",
            "features_sh_lr has no effect at sh_degree 0",
            false,
        ));
    }
    gaps.push(gap("scale_quat_grad", "scale and quaternion gradients are not ported", false));
    gaps.push(gap("mcmc_densify", "MCMC / IGS+ / MRNF densification is the C++ trainer", false));
    gaps
}

fn gap(name: &'static str, detail: &str, affects_result: bool) -> Gap {
    Gap {
        name,
        detail: detail.to_string(),
        affects_result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spirula_core::config::TrainConfig;

    #[test]
    fn explicit_supervision_blocks_a_strict_run() {
        let mut cfg = TrainConfig::default();
        cfg.use_bilateral_grid = false;
        cfg.use_ppisp = false;
        cfg.ssim_lambda = 0.0;
        cfg.sh_degree = 0;
        cfg.normal_supervision_weight = 0.0;
        cfg.load_masks = true;
        cfg.apply_loss_for_mask = Some(true);
        cfg.depth_supervision_weight = 1.0;
        cfg.orientation_method = "up".into();
        let names: Vec<_> = port_gaps(&cfg)
            .into_iter()
            .filter(|g| g.affects_result)
            .map(|g| g.name)
            .collect();
        assert!(names.contains(&"masks"));
        assert!(names.contains(&"mask_loss"));
        assert!(names.contains(&"depth_supervision"));
        assert!(names.contains(&"pose_normalization"));
    }

    #[test]
    fn random_background_and_resume_block_a_strict_run() {
        let mut cfg = TrainConfig::default();
        cfg.use_bilateral_grid = false;
        cfg.use_ppisp = false;
        cfg.ssim_lambda = 0.0;
        cfg.sh_degree = 0;
        cfg.normal_supervision_weight = 0.0;
        cfg.load_masks = false;
        cfg.orientation_method = "none".into();
        cfg.background_mode = "random".into();
        cfg.resume = "missing".into();
        let names: Vec<_> = port_gaps(&cfg)
            .into_iter()
            .filter(|g| g.affects_result)
            .map(|g| g.name)
            .collect();
        assert!(names.contains(&"background_mode"));
        assert!(names.contains(&"resume"));
    }

    #[test]
    fn random_init_modes_are_not_a_gap_and_max_steps_is_consumed() {
        let mut cfg = quiet();
        cfg.random_init = "never".into();
        cfg.max_steps = Some(100);
        let names: Vec<_> = port_gaps(&cfg)
            .into_iter()
            .filter(|g| g.affects_result)
            .map(|g| g.name)
            .collect();
        assert!(!names.contains(&"random_init"), "{names:?}");
        assert!(!names.contains(&"max_steps"), "{names:?}");
        cfg.random_init = "sideways".into();
        assert!(port_gaps(&cfg).iter().any(|g| g.name == "random_init" && g.affects_result));
    }

    fn quiet() -> TrainConfig {
        let mut cfg = TrainConfig::default();
        cfg.use_bilateral_grid = false;
        cfg.use_ppisp = false;
        cfg.ssim_lambda = 0.0;
        cfg.sh_degree = 0;
        cfg.normal_supervision_weight = 0.0;
        cfg.load_masks = false;
        cfg.orientation_method = "none".into();
        cfg
    }
}
