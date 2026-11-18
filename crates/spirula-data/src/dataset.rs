//! One dataset view over COLMAP and Nerfstudio. Metashape is refused.

use crate::colmap::{self, BakedCamera};
use crate::nerfstudio;
use crate::ply::{self, PointCloud};
use spirula_core::camera::{self, CameraModel, Distortion, DISTORTION_PARAMS};
use spirula_core::config::TrainConfig;
use spirula_core::{Error, Result};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Frame {
    pub name: String,
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
    /// OpenGL camera-to-world, row-major 3x4, the layout the C++ parsers store.
    pub c2w_gl: [f64; 12],
}

impl Frame {
    pub fn w2c_cv(&self) -> [f64; 12] {
        camera::c2w_gl_to_w2c_cv(&self.c2w_gl)
    }

    pub fn w2c_cv_f32(&self) -> [f32; 12] {
        self.w2c_cv().map(|v| v as f32)
    }
}

#[derive(Clone, Debug)]
pub struct Dataset {
    pub root: PathBuf,
    pub format: &'static str,
    pub frames: Vec<Frame>,
    pub points: PointCloud,
    pub notes: Vec<String>,
}

pub fn load_dataset(root: &Path, cfg: &TrainConfig) -> Result<Dataset> {
    if !root.is_dir() {
        return Err(Error::new(format!("{} is not a dataset directory", root.display())));
    }
    let format = detect_format(root, cfg)?;
    match format {
        "nerfstudio" => load_nerfstudio(root, cfg),
        "colmap" => load_colmap(root, cfg),
        "metashape" => Err(Error::new(
            "Metashape datasets are not in the Rust port; use the C++ spirula binary",
        )),
        other => Err(Error::new(format!("unknown data_format '{other}'"))),
    }
}

fn detect_format(root: &Path, cfg: &TrainConfig) -> Result<&'static str> {
    let asked = cfg.data_format.to_ascii_lowercase();
    if !asked.is_empty() {
        return match asked.as_str() {
            "nerfstudio" | "colmap" | "metashape" => Ok(match asked.as_str() {
                "nerfstudio" => "nerfstudio",
                "colmap" => "colmap",
                _ => "metashape",
            }),
            "none" => Err(Error::new("data_format none has no images to train on")),
            other => Err(Error::new(format!("unknown data_format '{other}'"))),
        };
    }
    if root.join("transforms.json").is_file() {
        return Ok("nerfstudio");
    }
    if colmap::find_recon_dir(root, &cfg.colmap_recon_dir).is_ok() {
        return Ok("colmap");
    }
    if has_metashape(root) {
        return Ok("metashape");
    }
    Err(Error::new(format!(
        "could not detect a Nerfstudio or COLMAP dataset in {}",
        root.display()
    )))
}

fn has_metashape(root: &Path) -> bool {
    let Ok(rd) = std::fs::read_dir(root) else { return false };
    rd.flatten().any(|e| {
        let n = e.file_name().to_string_lossy().to_ascii_lowercase();
        n.ends_with(".xml") || n.ends_with(".psx")
    })
}

fn load_nerfstudio(root: &Path, cfg: &TrainConfig) -> Result<Dataset> {
    let ns = nerfstudio::read_transforms(root)?;
    let frames = ns
        .frames
        .into_iter()
        .map(|f| Frame {
            name: f.file_path,
            image_path: f.image_path,
            width: f.width,
            height: f.height,
            model: f.model,
            distortion: f.distortion,
            dist: f.dist,
            fx: f.fx,
            fy: f.fy,
            cx: f.cx,
            cy: f.cy,
            c2w_gl: f.c2w_gl,
        })
        .collect();
    let mut notes = ns.notes;
    let mut points = ns.points;
    if !cfg.seed_pointcloud.is_empty() {
        let path = root.join(&cfg.seed_pointcloud);
        points = ply::read_point_ply(&path)?;
        notes.push(format!("seed points replaced from {}", path.display()));
    }
    Ok(Dataset {
        root: root.to_path_buf(),
        format: "nerfstudio",
        frames,
        points,
        notes,
    })
}

fn load_colmap(root: &Path, cfg: &TrainConfig) -> Result<Dataset> {
    let recon = colmap::find_recon_dir(root, &cfg.colmap_recon_dir)?;
    let (cameras, images, points) = colmap::read_model(&recon)?;
    if images.is_empty() {
        return Err(Error::new(format!("{} has no images", recon.display())));
    }
    let mut baked: Vec<(i32, BakedCamera)> = Vec::new();
    for cam in cameras.values() {
        baked.push((cam.id, colmap::bake_camera(cam)?));
    }
    let mut frames = Vec::with_capacity(images.len());
    for im in images.values() {
        let cam = cameras.get(&im.camera_id).ok_or_else(|| {
            Error::new(format!("image {} cites missing camera {}", im.name, im.camera_id))
        })?;
        let b = baked.iter().find(|(id, _)| *id == im.camera_id).map(|(_, b)| b).ok_or_else(|| {
            Error::new(format!("image {} cites missing camera {}", im.name, im.camera_id))
        })?;
        let width = u32::try_from(cam.width).map_err(|_| {
            Error::new(format!("camera {} width does not fit in 32 bits", cam.id))
        })?;
        let height = u32::try_from(cam.height).map_err(|_| {
            Error::new(format!("camera {} height does not fit in 32 bits", cam.id))
        })?;
        frames.push(Frame {
            image_path: resolve_colmap_image(root, &cfg.image_dir, &im.name),
            name: im.name.clone(),
            width,
            height,
            model: b.model,
            distortion: b.distortion,
            dist: b.dist,
            fx: b.fx,
            fy: b.fy,
            cx: b.cx,
            cy: b.cy,
            c2w_gl: camera::colmap_to_c2w_gl(im.qvec, im.tvec),
        });
    }
    frames.sort_by(|a, b| a.name.cmp(&b.name));
    let mut pc = PointCloud {
        xyz: Vec::with_capacity(points.xyz.len() / 3),
        rgb: Vec::with_capacity(points.rgb.len() / 3),
    };
    for i in 0..points.xyz.len() / 3 {
        pc.xyz.push([points.xyz[i * 3], points.xyz[i * 3 + 1], points.xyz[i * 3 + 2]]);
        if points.rgb.len() >= (i + 1) * 3 {
            pc.rgb.push([points.rgb[i * 3], points.rgb[i * 3 + 1], points.rgb[i * 3 + 2]]);
        } else {
            pc.rgb.push([255, 255, 255]);
        }
    }
    let mut notes = Vec::new();
    if !cfg.seed_pointcloud.is_empty() {
        let path = root.join(&cfg.seed_pointcloud);
        pc = ply::read_point_ply(&path)?;
        notes.push(format!("seed points replaced from {}", path.display()));
    }
    Ok(Dataset {
        root: root.to_path_buf(),
        format: "colmap",
        frames,
        points: pc,
        notes,
    })
}

fn resolve_colmap_image(root: &Path, image_dir: &str, name: &str) -> PathBuf {
    let in_dir = root.join(image_dir).join(name);
    if in_dir.is_file() {
        return in_dir;
    }
    let direct = root.join(name);
    if direct.is_file() {
        return direct;
    }
    in_dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use spirula_core::config::TrainConfig;

    #[test]
    fn colmap_text_pinhole_and_one_point() {
        let dir = std::env::temp_dir().join(format!("spirula-colmap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let sparse = dir.join("sparse").join("0");
        std::fs::create_dir_all(&sparse).unwrap();
        std::fs::write(
            sparse.join("cameras.txt"),
            "# cameras\n1 PINHOLE 64 48 50 40 32 24\n",
        )
        .unwrap();
        std::fs::write(
            sparse.join("images.txt"),
            "# images\n1 1 0 0 0 0 0 0 1 frame.png\n\n",
        )
        .unwrap();
        std::fs::write(
            sparse.join("points3D.txt"),
            "7 1.5 2.5 3.5 9 8 7 0.1\n",
        )
        .unwrap();
        let ds = load_dataset(&dir, &TrainConfig::default()).unwrap();
        assert_eq!(ds.format, "colmap");
        assert_eq!(ds.frames.len(), 1);
        let f = &ds.frames[0];
        assert_eq!(f.model, CameraModel::Pinhole);
        assert_eq!(f.distortion, Distortion::None);
        assert!((f.fx - 50.0).abs() < 1e-9 && (f.cx - 32.0).abs() < 1e-9);
        let w2c = f.w2c_cv();
        assert!((w2c[0] - 1.0).abs() < 1e-9);
        assert!((w2c[5] - 1.0).abs() < 1e-9);
        assert_eq!(ds.points.xyz[0], [1.5, 2.5, 3.5]);
        assert_eq!(ds.points.rgb[0], [9, 8, 7]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn colmap_binary_matches_the_text_twin() {
        let dir = std::env::temp_dir().join(format!("spirula-colbin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut cam = Vec::new();
        cam.extend_from_slice(&1u64.to_le_bytes());
        cam.extend_from_slice(&1i32.to_le_bytes());
        cam.extend_from_slice(&1i32.to_le_bytes());
        cam.extend_from_slice(&8u64.to_le_bytes());
        cam.extend_from_slice(&6u64.to_le_bytes());
        for v in [10.0_f64, 12.0, 4.0, 3.0] {
            cam.extend_from_slice(&v.to_le_bytes());
        }
        std::fs::write(dir.join("cameras.bin"), cam).unwrap();
        let mut img = Vec::new();
        img.extend_from_slice(&1u64.to_le_bytes());
        img.extend_from_slice(&3i32.to_le_bytes());
        for v in [1.0_f64, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0] {
            img.extend_from_slice(&v.to_le_bytes());
        }
        img.extend_from_slice(&1i32.to_le_bytes());
        img.extend_from_slice(b"a.png\0");
        img.extend_from_slice(&0u64.to_le_bytes());
        std::fs::write(dir.join("images.bin"), img).unwrap();
        let ds = load_dataset(&dir, &TrainConfig::default()).unwrap();
        assert_eq!(ds.frames[0].name, "a.png");
        assert!((ds.frames[0].fx - 10.0).abs() < 1e-9);
        assert!((ds.frames[0].c2w_gl[11] + 1.0).abs() < 1e-6);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nerfstudio_opencv_frame() {
        let dir = std::env::temp_dir().join(format!("spirula-ns-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("images")).unwrap();
        std::fs::write(
            dir.join("transforms.json"),
            r#"{
                "camera_model": "OPENCV",
                "fl_x": 100, "fl_y": 90, "cx": 50, "cy": 40, "w": 100, "h": 80,
                "k1": -0.1, "k2": 0.01, "p1": 0.0, "p2": 0.0,
                "frames": [{
                    "file_path": "images/f.png",
                    "transform_matrix": [
                        [1, 0, 0, 0],
                        [0, 1, 0, 0],
                        [0, 0, 1, 2],
                        [0, 0, 0, 1]
                    ]
                }]
            }"#,
        )
        .unwrap();
        let ds = load_dataset(&dir, &TrainConfig::default()).unwrap();
        assert_eq!(ds.format, "nerfstudio");
        assert_eq!(ds.frames[0].model, CameraModel::Pinhole);
        assert_eq!(ds.frames[0].distortion, Distortion::OpenCv);
        assert!((ds.frames[0].dist[0] + 0.1).abs() < 1e-6);
        assert!((ds.frames[0].c2w_gl[11] - 2.0).abs() < 1e-9);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fov_cameras_are_refused() {
        let dir = std::env::temp_dir().join(format!("spirula-fov-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("cameras.txt"), "1 FOV 64 48 40 32 24 0.5\n").unwrap();
        std::fs::write(dir.join("images.txt"), "1 1 0 0 0 0 0 0 1 a.png\n\n").unwrap();
        let err = load_dataset(&dir, &TrainConfig::default()).unwrap_err();
        assert!(err.message.contains("no exact distortion tier"), "{}", err.message);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nerfstudio_seed_pointcloud_replaces_an_empty_cloud() {
        let dir = std::env::temp_dir().join(format!("spirula-seed-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("transforms.json"),
            r#"{
                "camera_model": "PINHOLE",
                "fl_x": 10, "fl_y": 10, "cx": 4, "cy": 3, "w": 8, "h": 6,
                "frames": [{"file_path": "a.png", "transform_matrix": [
                    [1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]}]
            }"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("seed.ply"),
            b"ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n4 5 6 1 2 3\n",
        )
        .unwrap();
        let mut cfg = TrainConfig::default();
        cfg.seed_pointcloud = "seed.ply".into();
        let ds = load_dataset(&dir, &cfg).unwrap();
        assert_eq!(ds.points.xyz, vec![[4.0, 5.0, 6.0]]);
        assert!(ds.notes.iter().any(|n| n.contains("seed points")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
