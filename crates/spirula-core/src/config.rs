//! The training flags this port reads. Defaults match src/config/TrainConfig.h.
//! A config.json key this struct does not know is reported and left unused;
//! a known key with the wrong JSON shape is an error.

use crate::error::{Error, Result};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct TrainConfig {
    pub data: String,
    pub resume: String,
    pub output_dir_prefix: String,
    pub output_dir_name: String,
    pub num_iterations: i32,
    pub steps_per_save: i32,
    pub viewer_port: i32,
    pub disable_viewer: bool,
    pub data_format: String,
    pub image_dir: String,
    pub mask_dir: String,
    pub colmap_recon_dir: String,
    pub seed_pointcloud: String,
    pub init_ply: String,
    pub max_resolution: i32,
    pub sh_degree: i32,
    pub sb_lobes: i32,
    pub primitive: String,
    pub background_mode: String,
    pub background_color: [f32; 3],
    pub cap_max: i32,
    pub min_opacity: f32,
    pub refine_every: i32,
    pub refine_start_iter: i32,
    pub refine_stop_iter: i32,
    pub max_split_fraction: f32,
    pub scale_init: f32,
    pub opacity_init: f32,
    pub ssim_lambda: f32,
    pub l1_weight: f32,
    pub l2_weight: f32,
    pub means_lr: f32,
    pub means_lr_final: f32,
    pub scales_lr: f32,
    pub quats_lr: f32,
    pub opacities_lr: f32,
    pub features_dc_lr: f32,
    pub features_sh_lr: f32,
    pub max_steps: Option<i32>,
    pub use_bilateral_grid: bool,
    pub use_ppisp: bool,
    pub image_color_is_linear: Option<bool>,
    pub depth_supervision_weight: f32,
    pub normal_supervision_weight: f32,
    pub orientation_method: String,
    pub load_masks: bool,
    pub apply_loss_for_mask: Option<bool>,
    pub random_init: String,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            data: String::new(),
            resume: String::new(),
            output_dir_prefix: "outputs".into(),
            output_dir_name: String::new(),
            num_iterations: 30_000,
            steps_per_save: 2_000,
            viewer_port: 7007,
            disable_viewer: false,
            data_format: String::new(),
            image_dir: "images".into(),
            mask_dir: "masks".into(),
            colmap_recon_dir: String::new(),
            seed_pointcloud: String::new(),
            init_ply: String::new(),
            max_resolution: 0,
            sh_degree: 3,
            sb_lobes: 0,
            primitive: "3dgs".into(),
            background_mode: "color".into(),
            background_color: [0.0, 0.0, 0.0],
            cap_max: 1_000_000,
            min_opacity: 0.005,
            refine_every: 100,
            refine_start_iter: 500,
            refine_stop_iter: 14_000,
            max_split_fraction: 0.1,
            scale_init: 0.35,
            opacity_init: 0.1,
            ssim_lambda: 0.2,
            l1_weight: 1.0,
            l2_weight: 0.0,
            means_lr: 0.000256,
            means_lr_final: 3.2e-6,
            scales_lr: 0.02,
            quats_lr: 0.0015,
            opacities_lr: 0.025,
            features_dc_lr: 0.005,
            features_sh_lr: 0.00025,
            max_steps: None,
            use_bilateral_grid: true,
            use_ppisp: true,
            image_color_is_linear: None,
            depth_supervision_weight: 0.0,
            normal_supervision_weight: 0.01,
            orientation_method: "up".into(),
            load_masks: true,
            apply_loss_for_mask: None,
            random_init: "auto".into(),
        }
    }
}

impl TrainConfig {
    pub fn known_keys() -> &'static [&'static str] {
        &[
            "data",
            "resume",
            "output_dir_prefix",
            "output_dir_name",
            "num_iterations",
            "steps_per_save",
            "viewer_port",
            "disable_viewer",
            "data_format",
            "image_dir",
            "mask_dir",
            "colmap_recon_dir",
            "seed_pointcloud",
            "init_ply",
            "max_resolution",
            "sh_degree",
            "sb_lobes",
            "primitive",
            "background_mode",
            "background_color",
            "cap_max",
            "min_opacity",
            "refine_every",
            "refine_start_iter",
            "refine_stop_iter",
            "max_split_fraction",
            "scale_init",
            "opacity_init",
            "ssim_lambda",
            "l1_weight",
            "l2_weight",
            "means_lr",
            "means_lr_final",
            "scales_lr",
            "quats_lr",
            "opacities_lr",
            "features_dc_lr",
            "features_sh_lr",
            "max_steps",
            "use_bilateral_grid",
            "use_ppisp",
            "image_color_is_linear",
            "random_init",
            "load_masks",
            "apply_loss_for_mask",
            "depth_supervision_weight",
            "normal_supervision_weight",
            "orientation_method",
        ]
    }

    pub fn apply_json(&mut self, root: &Value) -> Result<Vec<String>> {
        let obj = root.as_object().ok_or_else(|| {
            Error::new("config JSON must be an object of flat training flags")
        })?;
        let known: BTreeSet<&str> = Self::known_keys().iter().copied().collect();
        let mut unknown = Vec::new();
        for (key, value) in obj {
            if !known.contains(key.as_str()) {
                unknown.push(key.clone());
                continue;
            }
            self.set_key(key, value)?;
        }
        if !matches!(self.primitive.as_str(), "3dgs" | "mip" | "3dgut") {
            return Err(Error::new(format!(
                "primitive '{}' is not 3dgs, mip or 3dgut",
                self.primitive
            )));
        }
        Ok(unknown)
    }

    pub fn from_json_str(text: &str) -> Result<(Self, Vec<String>)> {
        let value: Value = serde_json::from_str(text)
            .map_err(|e| Error::new(format!("config JSON: {e}")))?;
        let mut cfg = Self::default();
        let unknown = cfg.apply_json(&value)?;
        Ok((cfg, unknown))
    }

    fn set_key(&mut self, key: &str, value: &Value) -> Result<()> {
        let bad = || Error::new(format!("config field {key} has the wrong JSON shape"));
        match key {
            "data" => self.data = json_string(value).ok_or_else(bad)?,
            "resume" => self.resume = json_string(value).ok_or_else(bad)?,
            "output_dir_prefix" => self.output_dir_prefix = json_string(value).ok_or_else(bad)?,
            "output_dir_name" => self.output_dir_name = json_string(value).ok_or_else(bad)?,
            "num_iterations" => self.num_iterations = json_i32(value).ok_or_else(bad)?,
            "steps_per_save" => self.steps_per_save = json_i32(value).ok_or_else(bad)?,
            "viewer_port" => self.viewer_port = json_i32(value).ok_or_else(bad)?,
            "disable_viewer" => self.disable_viewer = json_bool(value).ok_or_else(bad)?,
            "data_format" => self.data_format = json_string(value).ok_or_else(bad)?,
            "image_dir" => self.image_dir = json_string(value).ok_or_else(bad)?,
            "mask_dir" => self.mask_dir = json_string(value).ok_or_else(bad)?,
            "colmap_recon_dir" => self.colmap_recon_dir = json_string(value).ok_or_else(bad)?,
            "seed_pointcloud" => self.seed_pointcloud = json_string(value).ok_or_else(bad)?,
            "init_ply" => self.init_ply = json_string(value).ok_or_else(bad)?,
            "max_resolution" => self.max_resolution = json_i32(value).ok_or_else(bad)?,
            "sh_degree" => self.sh_degree = json_i32(value).ok_or_else(bad)?,
            "sb_lobes" => self.sb_lobes = json_i32(value).ok_or_else(bad)?,
            "primitive" => self.primitive = json_string(value).ok_or_else(bad)?,
            "background_mode" => self.background_mode = json_string(value).ok_or_else(bad)?,
            "background_color" => self.background_color = json_vec3(value).ok_or_else(bad)?,
            "cap_max" => self.cap_max = json_i32(value).ok_or_else(bad)?,
            "min_opacity" => self.min_opacity = json_f32(value).ok_or_else(bad)?,
            "refine_every" => self.refine_every = json_i32(value).ok_or_else(bad)?,
            "refine_start_iter" => self.refine_start_iter = json_i32(value).ok_or_else(bad)?,
            "refine_stop_iter" => self.refine_stop_iter = json_i32(value).ok_or_else(bad)?,
            "max_split_fraction" => self.max_split_fraction = json_f32(value).ok_or_else(bad)?,
            "scale_init" => self.scale_init = json_f32(value).ok_or_else(bad)?,
            "opacity_init" => self.opacity_init = json_f32(value).ok_or_else(bad)?,
            "ssim_lambda" => self.ssim_lambda = json_f32(value).ok_or_else(bad)?,
            "l1_weight" => self.l1_weight = json_f32(value).ok_or_else(bad)?,
            "l2_weight" => self.l2_weight = json_f32(value).ok_or_else(bad)?,
            "means_lr" => self.means_lr = json_f32(value).ok_or_else(bad)?,
            "means_lr_final" => self.means_lr_final = json_f32(value).ok_or_else(bad)?,
            "scales_lr" => self.scales_lr = json_f32(value).ok_or_else(bad)?,
            "quats_lr" => self.quats_lr = json_f32(value).ok_or_else(bad)?,
            "opacities_lr" => self.opacities_lr = json_f32(value).ok_or_else(bad)?,
            "features_dc_lr" => self.features_dc_lr = json_f32(value).ok_or_else(bad)?,
            "features_sh_lr" => self.features_sh_lr = json_f32(value).ok_or_else(bad)?,
            "max_steps" => {
                self.max_steps = if value.is_null() {
                    None
                } else {
                    Some(json_i32(value).ok_or_else(bad)?)
                }
            }
            "use_bilateral_grid" => self.use_bilateral_grid = json_bool(value).ok_or_else(bad)?,
            "use_ppisp" => self.use_ppisp = json_bool(value).ok_or_else(bad)?,
            "image_color_is_linear" => {
                self.image_color_is_linear = if value.is_null() {
                    None
                } else {
                    Some(json_bool(value).ok_or_else(bad)?)
                }
            }
            "random_init" => self.random_init = json_string(value).ok_or_else(bad)?,
            "load_masks" => self.load_masks = json_bool(value).ok_or_else(bad)?,
            "apply_loss_for_mask" => {
                self.apply_loss_for_mask = if value.is_null() {
                    None
                } else {
                    Some(json_bool(value).ok_or_else(bad)?)
                }
            }
            "depth_supervision_weight" => {
                self.depth_supervision_weight = json_f32(value).ok_or_else(bad)?
            }
            "normal_supervision_weight" => {
                self.normal_supervision_weight = json_f32(value).ok_or_else(bad)?
            }
            "orientation_method" => self.orientation_method = json_string(value).ok_or_else(bad)?,
            _ => {}
        }
        Ok(())
    }
}

fn json_string(v: &Value) -> Option<String> {
    v.as_str().map(|s| s.to_string())
}

fn json_bool(v: &Value) -> Option<bool> {
    v.as_bool()
}

fn json_f32(v: &Value) -> Option<f32> {
    v.as_f64().map(|n| n as f32)
}

fn json_i32(v: &Value) -> Option<i32> {
    if let Some(i) = v.as_i64() {
        return i32::try_from(i).ok();
    }
    v.as_f64().map(|n| n as i32)
}

fn json_vec3(v: &Value) -> Option<[f32; 3]> {
    let arr = v.as_array()?;
    if arr.len() != 3 {
        return None;
    }
    Some([json_f32(&arr[0])?, json_f32(&arr[1])?, json_f32(&arr[2])?])
}

/// `lr * (lr_final / lr) ^ min(step / max_steps, 1)`, from TrainerCore.cpp.
pub fn scheduled_lr(step: i32, max_steps: i32, lr: f32, lr_final: f32) -> f32 {
    if lr == 0.0 || lr_final == 0.0 {
        return lr;
    }
    let t = (step as f32 / max_steps.max(1) as f32).min(1.0);
    lr * (lr_final / lr).powf(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_train_config_h() {
        let c = TrainConfig::default();
        assert_eq!(c.num_iterations, 30_000);
        assert_eq!(c.sh_degree, 3);
        assert!((c.means_lr - 0.000256).abs() < 1e-12);
        assert!((c.means_lr_final - 3.2e-6).abs() < 1e-12);
        assert!((c.ssim_lambda - 0.2).abs() < 1e-6);
        assert!(c.use_bilateral_grid && c.use_ppisp);
        assert_eq!(c.viewer_port, 7007);
    }

    #[test]
    fn flat_json_overrides_known_keys_and_lists_the_rest() {
        let (c, unknown) = TrainConfig::from_json_str(
            r#"{"num_iterations": 10, "sh_degree": 1, "background_color": [1, 0.5, 0],
                "bilagrid_shape": [16, 16, 8], "max_steps": null}"#,
        )
        .unwrap();
        assert_eq!(c.num_iterations, 10);
        assert_eq!(c.sh_degree, 1);
        assert_eq!(c.background_color, [1.0, 0.5, 0.0]);
        assert_eq!(c.max_steps, None);
        assert_eq!(unknown, vec!["bilagrid_shape".to_string()]);
        assert_eq!(c.means_lr, TrainConfig::default().means_lr);
    }

    #[test]
    fn learning_rate_decays_geometrically() {
        let lr = scheduled_lr(0, 100, 1.0, 0.01);
        let mid = scheduled_lr(50, 100, 1.0, 0.01);
        let end = scheduled_lr(100, 100, 1.0, 0.01);
        assert!((lr - 1.0).abs() < 1e-6);
        assert!((end - 0.01).abs() < 1e-5);
        assert!(mid < 1.0 && mid > 0.01);
    }
}
