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
    gaps.push(gap("scale_quat_grad", "scale and quaternion gradients are not ported", false));
    gaps.push(gap("masks_depth_pose", "masks, depth, normals and pose normalization are not applied", false));
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
