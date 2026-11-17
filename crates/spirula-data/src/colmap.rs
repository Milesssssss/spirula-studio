//! COLMAP sparse model, text and binary. Camera baking covers the models
//! that map onto a distortion tier exactly. A model the C++ reader fits
//! (FOV, division, EUCM, a rational FULL_OPENCV) is an error here.

use spirula_core::camera::{self, CameraModel, Distortion, DISTORTION_PARAMS};
use spirula_core::{Error, Result};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct RawCamera {
    pub id: i32,
    pub model: String,
    pub width: u64,
    pub height: u64,
    pub params: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct RawImage {
    #[allow(dead_code)]
    pub id: i32,
    pub qvec: [f64; 4],
    pub tvec: [f64; 3],
    pub camera_id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct RawPoints {
    pub xyz: Vec<f64>,
    pub rgb: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct BakedCamera {
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64,
    pub model: CameraModel,
    pub distortion: Distortion,
    pub dist: [f32; DISTORTION_PARAMS],
}

struct ModelInfo {
    name: &'static str,
    nparams: usize,
}

fn model_table(id: i32) -> Option<ModelInfo> {
    Some(match id {
        0 => ModelInfo { name: "SIMPLE_PINHOLE", nparams: 3 },
        1 => ModelInfo { name: "PINHOLE", nparams: 4 },
        2 => ModelInfo { name: "SIMPLE_RADIAL", nparams: 4 },
        3 => ModelInfo { name: "RADIAL", nparams: 5 },
        4 => ModelInfo { name: "OPENCV", nparams: 8 },
        5 => ModelInfo { name: "OPENCV_FISHEYE", nparams: 8 },
        6 => ModelInfo { name: "FULL_OPENCV", nparams: 12 },
        7 => ModelInfo { name: "FOV", nparams: 5 },
        8 => ModelInfo { name: "SIMPLE_RADIAL_FISHEYE", nparams: 4 },
        9 => ModelInfo { name: "RADIAL_FISHEYE", nparams: 5 },
        10 => ModelInfo { name: "THIN_PRISM_FISHEYE", nparams: 12 },
        11 => ModelInfo { name: "RAD_TAN_THIN_PRISM_FISHEYE", nparams: 16 },
        12 => ModelInfo { name: "SIMPLE_DIVISION", nparams: 4 },
        13 => ModelInfo { name: "DIVISION", nparams: 5 },
        14 => ModelInfo { name: "SIMPLE_FISHEYE", nparams: 3 },
        15 => ModelInfo { name: "FISHEYE", nparams: 4 },
        16 => ModelInfo { name: "EUCM", nparams: 6 },
        17 => ModelInfo { name: "EQUIRECTANGULAR", nparams: 2 },
        _ => return None,
    })
}

pub fn find_recon_dir(root: &Path, configured: &str) -> Result<PathBuf> {
    if !configured.is_empty() {
        let p = root.join(configured);
        if !p.is_dir() {
            return Err(Error::new(format!(
                "colmap_recon_dir '{}' is not a directory",
                p.display()
            )));
        }
        return Ok(p);
    }
    for rel in ["sparse/0", "colmap/sparse/0", "sparse", "colmap", ""] {
        let p = if rel.is_empty() { root.to_path_buf() } else { root.join(rel) };
        if p.join("cameras.bin").is_file() || p.join("cameras.txt").is_file() {
            return Ok(p);
        }
    }
    Err(Error::new(format!(
        "no COLMAP cameras.bin or cameras.txt under {}",
        root.display()
    )))
}

pub fn read_model(dir: &Path) -> Result<(BTreeMap<i32, RawCamera>, BTreeMap<i32, RawImage>, RawPoints)> {
    let binary = dir.join("cameras.bin").is_file();
    let text = dir.join("cameras.txt").is_file();
    if binary {
        Ok((
            read_cameras_bin(&dir.join("cameras.bin"))?,
            read_images_bin(&dir.join("images.bin"))?,
            read_points_bin(&dir.join("points3D.bin")).unwrap_or_default(),
        ))
    } else if text {
        Ok((
            read_cameras_txt(&dir.join("cameras.txt"))?,
            read_images_txt(&dir.join("images.txt"))?,
            read_points_txt(&dir.join("points3D.txt")).unwrap_or_default(),
        ))
    } else {
        Err(Error::new(format!(
            "{} has neither cameras.bin nor cameras.txt",
            dir.display()
        )))
    }
}

pub fn bake_camera(cam: &RawCamera) -> Result<BakedCamera> {
    let p = &cam.params;
    let need = |n: usize| -> Result<()> {
        if p.len() < n {
            Err(Error::new(format!(
                "COLMAP camera {} ({}) needs {n} parameters, got {}",
                cam.id,
                cam.model,
                p.len()
            )))
        } else {
            Ok(())
        }
    };
    let pf = |i: usize| p[i] as f32;
    let mut out = BakedCamera {
        fx: 0.0,
        fy: 0.0,
        cx: 0.0,
        cy: 0.0,
        model: CameraModel::Pinhole,
        distortion: Distortion::None,
        dist: [0.0; DISTORTION_PARAMS],
    };
    match cam.model.as_str() {
        "SIMPLE_PINHOLE" => {
            need(3)?;
            out.fx = p[0];
            out.fy = p[0];
            out.cx = p[1];
            out.cy = p[2];
        }
        "PINHOLE" => {
            need(4)?;
            out.fx = p[0];
            out.fy = p[1];
            out.cx = p[2];
            out.cy = p[3];
        }
        "SIMPLE_FISHEYE" => {
            need(3)?;
            out.fx = p[0];
            out.fy = p[0];
            out.cx = p[1];
            out.cy = p[2];
            out.model = CameraModel::Fisheye;
        }
        "FISHEYE" => {
            need(4)?;
            out.fx = p[0];
            out.fy = p[1];
            out.cx = p[2];
            out.cy = p[3];
            out.model = CameraModel::Fisheye;
        }
        "SIMPLE_RADIAL" | "RADIAL" | "SIMPLE_RADIAL_FISHEYE" | "RADIAL_FISHEYE" => {
            let two = cam.model == "RADIAL" || cam.model == "RADIAL_FISHEYE";
            need(if two { 5 } else { 4 })?;
            out.fx = p[0];
            out.fy = p[0];
            out.cx = p[1];
            out.cy = p[2];
            out.distortion = Distortion::OpenCv;
            out.dist[0] = pf(3);
            if two {
                out.dist[1] = pf(4);
            }
            if cam.model.ends_with("FISHEYE") {
                out.model = CameraModel::Fisheye;
            }
        }
        "OPENCV" => {
            need(8)?;
            out.fx = p[0];
            out.fy = p[1];
            out.cx = p[2];
            out.cy = p[3];
            out.distortion = Distortion::OpenCv;
            out.dist[0] = pf(4);
            out.dist[1] = pf(5);
            out.dist[2] = pf(6);
            out.dist[3] = pf(7);
        }
        "FULL_OPENCV" if p.len() >= 12 && p[9] == 0.0 && p[10] == 0.0 && p[11] == 0.0 => {
            out.fx = p[0];
            out.fy = p[1];
            out.cx = p[2];
            out.cy = p[3];
            out.distortion = Distortion::ThinPrism;
            out.dist[0] = pf(4);
            out.dist[1] = pf(5);
            out.dist[2] = pf(8);
            out.dist[4] = pf(6);
            out.dist[5] = pf(7);
        }
        "OPENCV_FISHEYE" => {
            need(8)?;
            out.fx = p[0];
            out.fy = p[1];
            out.cx = p[2];
            out.cy = p[3];
            out.model = CameraModel::Fisheye;
            out.distortion = Distortion::ThinPrism;
            out.dist[0] = pf(4);
            out.dist[1] = pf(5);
            out.dist[2] = pf(6);
            out.dist[3] = pf(7);
        }
        "THIN_PRISM_FISHEYE" => {
            need(12)?;
            out.fx = p[0];
            out.fy = p[1];
            out.cx = p[2];
            out.cy = p[3];
            out.model = CameraModel::Fisheye;
            out.distortion = Distortion::ThinPrism;
            out.dist[0] = pf(4);
            out.dist[1] = pf(5);
            out.dist[2] = pf(8);
            out.dist[3] = pf(9);
            out.dist[4] = pf(6);
            out.dist[5] = pf(7);
            out.dist[6] = pf(10);
            out.dist[7] = pf(11);
        }
        "EQUIRECTANGULAR" => {
            need(2)?;
            let pi = std::f64::consts::PI;
            out.fx = p[0] / (2.0 * pi);
            out.fy = p[1] / pi;
            out.cx = p[0] / 2.0;
            out.cy = p[1] / 2.0;
            out.model = CameraModel::Equirectangular;
        }
        other => {
            return Err(Error::new(format!(
                "COLMAP camera model {other} has no exact distortion tier in the Rust port; the C++ reader fits it"
            )));
        }
    }
    out.distortion = camera::demote_distortion(out.distortion, &mut out.dist);
    Ok(out)
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    let mut f = File::open(path).map_err(|e| Error::new(format!("cannot open {}: {e}", path.display())))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)
        .map_err(|e| Error::new(format!("reading {}: {e}", path.display())))?;
    Ok(buf)
}

struct Bin<'a> {
    b: &'a [u8],
    o: usize,
    path: &'a Path,
}

impl<'a> Bin<'a> {
    fn rest(&self) -> usize {
        self.b.len().saturating_sub(self.o)
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.rest() < n {
            return Err(Error::new(format!(
                "{} ended early (need {n} bytes, {} left)",
                self.path.display(),
                self.rest()
            )));
        }
        let s = &self.b[self.o..self.o + n];
        self.o += n;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32> {
        Ok(self.u32()? as i32)
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn cstr(&mut self) -> Result<String> {
        let start = self.o;
        while self.o < self.b.len() && self.b[self.o] != 0 {
            self.o += 1;
        }
        if self.o >= self.b.len() {
            return Err(Error::new(format!("{} has an unterminated string", self.path.display())));
        }
        let s = String::from_utf8_lossy(&self.b[start..self.o]).into_owned();
        self.o += 1;
        Ok(s)
    }
    fn skip(&mut self, n: usize) -> Result<()> {
        self.take(n).map(|_| ())
    }
}

fn read_cameras_bin(path: &Path) -> Result<BTreeMap<i32, RawCamera>> {
    let bytes = read_file(path)?;
    let mut r = Bin { b: &bytes, o: 0, path };
    let n = r.u64()?;
    let mut cameras = BTreeMap::new();
    for _ in 0..n {
        let id = r.i32()?;
        let model_id = r.i32()?;
        let width = r.u64()?;
        let height = r.u64()?;
        let info = model_table(model_id).ok_or_else(|| {
            Error::new(format!("COLMAP unknown camera model id {model_id} in {}", path.display()))
        })?;
        let mut params = Vec::with_capacity(info.nparams);
        for _ in 0..info.nparams {
            params.push(r.f64()?);
        }
        cameras.insert(
            id,
            RawCamera { id, model: info.name.into(), width, height, params },
        );
    }
    Ok(cameras)
}

fn read_images_bin(path: &Path) -> Result<BTreeMap<i32, RawImage>> {
    let bytes = read_file(path)?;
    let mut r = Bin { b: &bytes, o: 0, path };
    let n = r.u64()?;
    let mut images = BTreeMap::new();
    for _ in 0..n {
        let id = r.i32()?;
        let mut qvec = [0.0; 4];
        let mut tvec = [0.0; 3];
        for q in &mut qvec {
            *q = r.f64()?;
        }
        for t in &mut tvec {
            *t = r.f64()?;
        }
        let camera_id = r.i32()?;
        let name = r.cstr()?;
        let n2 = r.u64()?;
        r.skip((n2 as usize).saturating_mul(8 + 8 + 8))?;
        images.insert(
            id,
            RawImage { id, qvec, tvec, camera_id, name },
        );
    }
    Ok(images)
}

fn read_points_bin(path: &Path) -> Result<RawPoints> {
    if !path.is_file() {
        return Ok(RawPoints::default());
    }
    let bytes = read_file(path)?;
    let mut r = Bin { b: &bytes, o: 0, path };
    let n = r.u64()?;
    let mut pts = RawPoints { xyz: Vec::with_capacity(n as usize * 3), rgb: Vec::with_capacity(n as usize * 3) };
    for _ in 0..n {
        r.skip(8)?;
        for _ in 0..3 {
            pts.xyz.push(r.f64()?);
        }
        for _ in 0..3 {
            pts.rgb.push(r.u8()?);
        }
        r.skip(8)?;
        let track = r.u64()?;
        r.skip((track as usize).saturating_mul(8))?;
    }
    Ok(pts)
}

struct Text<'a> {
    data: &'a str,
    pos: usize,
}

impl<'a> Text<'a> {
    fn next_content(&mut self) -> Option<&'a str> {
        while self.pos < self.data.len() {
            let rest = &self.data[self.pos..];
            let (line, next) = match rest.find('\n') {
                Some(i) => (&rest[..i], self.pos + i + 1),
                None => (rest, self.data.len()),
            };
            self.pos = next;
            let line = line.trim().trim_end_matches('\r').trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            return Some(line);
        }
        None
    }
    fn skip_raw(&mut self) {
        if self.pos >= self.data.len() {
            return;
        }
        if let Some(i) = self.data[self.pos..].find('\n') {
            self.pos += i + 1;
        } else {
            self.pos = self.data.len();
        }
    }
}

fn read_cameras_txt(path: &Path) -> Result<BTreeMap<i32, RawCamera>> {
    let bytes = read_file(path)?;
    let text = String::from_utf8_lossy(&bytes);
    let mut r = Text { data: &text, pos: 0 };
    let mut cameras = BTreeMap::new();
    while let Some(line) = r.next_content() {
        let mut it = line.split_whitespace();
        let id: i32 = it.next().and_then(|s| s.parse().ok()).ok_or_else(|| {
            Error::new(format!("malformed cameras.txt line in {}", path.display()))
        })?;
        let model = it.next().unwrap_or("").to_string();
        let width: u64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let height: u64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let mut params = Vec::new();
        for tok in it {
            let v: f64 = tok.parse().map_err(|_| {
                Error::new(format!("bad camera parameter '{tok}' in {}", path.display()))
            })?;
            params.push(v);
        }
        if model.is_empty() {
            return Err(Error::new(format!("malformed cameras.txt line in {}", path.display())));
        }
        cameras.insert(id, RawCamera { id, model, width, height, params });
    }
    Ok(cameras)
}

fn read_images_txt(path: &Path) -> Result<BTreeMap<i32, RawImage>> {
    let bytes = read_file(path)?;
    let text = String::from_utf8_lossy(&bytes);
    let mut r = Text { data: &text, pos: 0 };
    let mut images = BTreeMap::new();
    while let Some(line) = r.next_content() {
        let mut it = line.split_whitespace();
        let id: i32 = parse_tok(it.next(), path, "image id")?;
        let mut qvec = [0.0; 4];
        let mut tvec = [0.0; 3];
        for q in &mut qvec {
            *q = parse_tok(it.next(), path, "qvec")?;
        }
        for t in &mut tvec {
            *t = parse_tok(it.next(), path, "tvec")?;
        }
        let camera_id: i32 = parse_tok(it.next(), path, "camera id")?;
        let name = it.next().unwrap_or("").to_string();
        if name.is_empty() {
            return Err(Error::new(format!("image {} has no name in {}", id, path.display())));
        }
        r.skip_raw();
        images.insert(id, RawImage { id, qvec, tvec, camera_id, name });
    }
    Ok(images)
}

fn read_points_txt(path: &Path) -> Result<RawPoints> {
    if !path.is_file() {
        return Ok(RawPoints::default());
    }
    let bytes = read_file(path)?;
    let text = String::from_utf8_lossy(&bytes);
    let mut r = Text { data: &text, pos: 0 };
    let mut pts = RawPoints::default();
    while let Some(line) = r.next_content() {
        let mut it = line.split_whitespace();
        let _id: i64 = parse_tok(it.next(), path, "point id")?;
        for _ in 0..3 {
            pts.xyz.push(parse_tok(it.next(), path, "xyz")?);
        }
        for _ in 0..3 {
            let c: i64 = parse_tok(it.next(), path, "rgb")?;
            pts.rgb.push(c.clamp(0, 255) as u8);
        }
    }
    Ok(pts)
}

fn parse_tok<T: std::str::FromStr>(tok: Option<&str>, path: &Path, what: &str) -> Result<T> {
    tok.and_then(|s| s.parse().ok()).ok_or_else(|| {
        Error::new(format!("malformed {what} in {}", path.display()))
    })
}
