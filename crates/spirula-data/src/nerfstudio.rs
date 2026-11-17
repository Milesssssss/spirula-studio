//! transforms.json. Poses stay in the file's frame: `applied_transform` is
//! recorded and not inverted, which is the frame a sibling point cloud uses.
//! A rational denominator or a nonzero skew is refused; the C++ reader fits those.

use crate::ply::{self, PointCloud};
use spirula_core::camera::{self, CameraModel, Distortion, DISTORTION_PARAMS};
use spirula_core::{Error, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct NsFrame {
    pub file_path: String,
    pub image_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub model: CameraModel,
    pub distortion: Distortion,
    pub dist: [f32; DISTORTION_PARAMS],
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64,
    pub c2w_gl: [f64; 12],
}

#[derive(Clone, Debug)]
pub struct NsDataset {
    pub frames: Vec<NsFrame>,
    pub points: PointCloud,
    pub notes: Vec<String>,
}

pub fn read_transforms(dataset_dir: &Path) -> Result<NsDataset> {
    let path = dataset_dir.join("transforms.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    let meta: Value = serde_json::from_str(&text)
        .map_err(|e| Error::new(format!("{}: {e}", path.display())))?;
    let frames_v = meta
        .get("frames")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::new(format!("{}: no frames array", path.display())))?;
    if frames_v.is_empty() {
        return Err(Error::new("NerfstudioParser: no frames in transforms.json"));
    }
    let mut notes = Vec::new();
    if meta.get("applied_transform").is_some() {
        notes.push(
            "applied_transform is present and was not inverted; cameras stay in the transforms.json frame"
                .into(),
        );
    }
    let mut frames = Vec::with_capacity(frames_v.len());
    for (i, fr) in frames_v.iter().enumerate() {
        frames.push(read_frame(dataset_dir, &meta, fr, i)?);
    }
    let mut points = PointCloud::default();
    if let Some(ply_name) = meta.get("ply_file_path").and_then(|v| v.as_str()) {
        let ply_path = dataset_dir.join(ply_name);
        if ply_path.is_file() {
            points = ply::read_point_ply(&ply_path).or_else(|_| {
                let splat = ply::read_splat_ply(&ply_path)?;
                let mut pc = PointCloud {
                    xyz: splat.means.iter().map(|m| [m[0] as f64, m[1] as f64, m[2] as f64]).collect(),
                    rgb: Vec::new(),
                };
                for dc in &splat.features_dc {
                    let rgb = spirula_core::color::rgb_from_dc(*dc);
                    pc.rgb.push(rgb.map(|c| (c * 255.0).round().clamp(0.0, 255.0) as u8));
                }
                Ok::<_, Error>(pc)
            })?;
        } else {
            notes.push(format!("ply_file_path '{ply_name}' was not found"));
        }
    }
    Ok(NsDataset { frames, points, notes })
}

fn read_frame(root: &Path, meta: &Value, fr: &Value, index: usize) -> Result<NsFrame> {
    let file_path = fr
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::new(format!("frame {index} has no file_path")))?
        .to_string();
    let fx = num(fr, meta, "fl_x").ok_or_else(|| Error::new(format!("frame {index} has no fl_x")))?;
    let fy = num(fr, meta, "fl_y").unwrap_or(fx);
    let cx = num(fr, meta, "cx").ok_or_else(|| Error::new(format!("frame {index} has no cx")))?;
    let cy = num(fr, meta, "cy").ok_or_else(|| Error::new(format!("frame {index} has no cy")))?;
    let width = num(fr, meta, "w").ok_or_else(|| Error::new(format!("frame {index} has no w")))? as u32;
    let height = num(fr, meta, "h").ok_or_else(|| Error::new(format!("frame {index} has no h")))? as u32;
    let model_name = fr
        .get("camera_model")
        .or_else(|| meta.get("camera_model"))
        .and_then(|v| v.as_str())
        .unwrap_or("OPENCV");
    let model = CameraModel::from_name(model_name).ok_or_else(|| {
        Error::new(format!("NerfstudioParser: unsupported camera model {model_name}"))
    })?;
    let mut fx = fx;
    let b1 = num(fr, meta, "b1").unwrap_or(0.0);
    let b2 = num(fr, meta, "b2").unwrap_or(0.0);
    if b1 != 0.0 {
        fx *= 1.0 + b1;
    }
    let skew_px = b2 * fx;
    let k4 = num(fr, meta, "k4").unwrap_or(0.0);
    let k5 = num(fr, meta, "k5").unwrap_or(0.0);
    let k6 = num(fr, meta, "k6").unwrap_or(0.0);
    if skew_px != 0.0 || k5 != 0.0 || k6 != 0.0 {
        return Err(Error::new(
            "transforms.json uses skew or a rational distortion; that fit is not in the Rust port",
        ));
    }
    let mut dist = [0.0; DISTORTION_PARAMS];
    dist[0] = num(fr, meta, "k1").unwrap_or(0.0) as f32;
    dist[1] = num(fr, meta, "k2").unwrap_or(0.0) as f32;
    dist[2] = num(fr, meta, "k3").unwrap_or(0.0) as f32;
    dist[3] = k4 as f32;
    dist[4] = num(fr, meta, "p1").unwrap_or(0.0) as f32;
    dist[5] = num(fr, meta, "p2").unwrap_or(0.0) as f32;
    dist[6] = num(fr, meta, "sx1").unwrap_or(0.0) as f32;
    dist[7] = num(fr, meta, "sy1").unwrap_or(0.0) as f32;
    let mut distortion = Distortion::ThinPrism;
    if model == CameraModel::Equirectangular {
        distortion = Distortion::None;
        dist = [0.0; DISTORTION_PARAMS];
    }
    distortion = camera::demote_distortion(distortion, &mut dist);
    let (fx, fy, cx, cy) = if model == CameraModel::Equirectangular {
        let w = width as f64;
        let h = height as f64;
        let pi = std::f64::consts::PI;
        (w / (2.0 * pi), h / pi, w / 2.0, h / 2.0)
    } else {
        (fx, fy, cx, cy)
    };
    let tm = fr
        .get("transform_matrix")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::new(format!("frame {index} has no transform_matrix")))?;
    if tm.len() < 3 {
        return Err(Error::new(format!("frame {index} has a short transform_matrix")));
    }
    let mut c2w = [0.0; 12];
    for r in 0..3 {
        let row = tm[r].as_array().ok_or_else(|| {
            Error::new(format!("frame {index} transform row {r} is not an array"))
        })?;
        if row.len() < 4 {
            return Err(Error::new(format!("frame {index} transform row {r} is short")));
        }
        for c in 0..4 {
            c2w[r * 4 + c] = row[c].as_f64().ok_or_else(|| {
                Error::new(format!("frame {index} transform is not numeric"))
            })?;
        }
    }
    Ok(NsFrame {
        image_path: resolve_file(root, &file_path),
        file_path,
        width,
        height,
        model,
        distortion,
        dist,
        fx,
        fy,
        cx,
        cy,
        c2w_gl: c2w,
    })
}

fn num(frame: &Value, meta: &Value, key: &str) -> Option<f64> {
    frame.get(key).and_then(|v| v.as_f64()).or_else(|| meta.get(key).and_then(|v| v.as_f64()))
}

fn resolve_file(root: &Path, file_path: &str) -> PathBuf {
    let raw = PathBuf::from(file_path);
    let candidates = [
        root.join(&raw),
        root.join("images").join(&raw),
        root.join(format!("{file_path}.png")),
        root.join(format!("{file_path}.jpg")),
        root.join("images").join(format!("{file_path}.png")),
        root.join("images").join(format!("{file_path}.jpg")),
    ];
    for c in &candidates {
        if c.is_file() {
            return c.clone();
        }
    }
    candidates[0].clone()
}
