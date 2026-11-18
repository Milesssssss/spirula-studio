//! 3DGS splat PLY and ordinary xyz/rgb point clouds.
//! File order matches src/checkpoint/SplatPly.cpp: channel-major `f_rest`,
//! field-major `sb_params`. In memory, SH is coefficient-major and each lobe
//! is six contiguous floats.

use spirula_core::color::ColorBasis;
use spirula_core::sh::sh_degree_from_rest_coeffs;
use spirula_core::{Error, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SplatCloud {
    pub sh_degree: i32,
    pub sb_lobes: i32,
    pub primitive: String,
    pub means: Vec<[f32; 3]>,
    pub quats: Vec<[f32; 4]>,
    pub scales: Vec<[f32; 3]>,
    pub opacities: Vec<f32>,
    pub features_dc: Vec<[f32; 3]>,
    /// Per splat: `sh_coeffs * 3` then `sb_lobes * 6` floats. Empty when both are zero.
    pub features_sh: Vec<f32>,
}

impl SplatCloud {
    pub fn len(&self) -> usize {
        self.means.len()
    }
    pub fn is_empty(&self) -> bool {
        self.means.is_empty()
    }
    pub fn basis(&self) -> ColorBasis {
        ColorBasis {
            sh_degree: self.sh_degree,
            sb_lobes: self.sb_lobes,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PointCloud {
    pub xyz: Vec<[f64; 3]>,
    pub rgb: Vec<[u8; 3]>,
}

struct Header {
    format: PlyFormat,
    count: usize,
    props: Vec<Prop>,
    vertex_bytes: Option<usize>,
    body: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlyFormat {
    Ascii,
    BinaryLe,
}

struct Prop {
    name: String,
    kind: PropKind,
}

#[derive(Clone, Copy)]
enum PropKind {
    F32,
    F64,
    U8,
    I32,
    U32,
}

impl PropKind {
    fn width(self) -> usize {
        match self {
            Self::F32 | Self::I32 | Self::U32 => 4,
            Self::F64 => 8,
            Self::U8 => 1,
        }
    }
}

pub fn read_splat_ply(path: &Path) -> Result<SplatCloud> {
    let bytes = read_bytes(path)?;
    let header = parse_header(&bytes, path)?;
    let index = |name: &str| -> Result<usize> {
        header
            .props
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| Error::new(format!("{}: missing property {name}", path.display())))
    };
    let ix = index("x")?;
    let iy = index("y")?;
    let iz = index("z")?;
    let iop = index("opacity")?;
    let isc = [index("scale_0")?, index("scale_1")?, index("scale_2")?];
    let iq = [index("rot_0")?, index("rot_1")?, index("rot_2")?, index("rot_3")?];
    let idc = [index("f_dc_0")?, index("f_dc_1")?, index("f_dc_2")?];
    let mut rest_idx = Vec::new();
    for (i, p) in header.props.iter().enumerate() {
        if let Some(n) = p.name.strip_prefix("f_rest_") {
            if n.chars().all(|c| c.is_ascii_digit()) {
                rest_idx.push((n.parse::<usize>().unwrap_or(usize::MAX), i));
            }
        }
    }
    rest_idx.sort_by_key(|p| p.0);
    if rest_idx.len() % 3 != 0 {
        return Err(Error::new(format!(
            "{}: f_rest count {} is not a multiple of 3",
            path.display(),
            rest_idx.len()
        )));
    }
    let sh_degree = sh_degree_from_rest_coeffs(rest_idx.len() / 3).ok_or_else(|| {
        Error::new(format!("{}: f_rest does not match an SH degree", path.display()))
    })?;
    let mut sb_idx = Vec::new();
    for (i, p) in header.props.iter().enumerate() {
        if let Some(n) = p.name.strip_prefix("sb_params_") {
            if n.chars().all(|c| c.is_ascii_digit()) {
                sb_idx.push((n.parse::<usize>().unwrap_or(usize::MAX), i));
            }
        }
    }
    sb_idx.sort_by_key(|p| p.0);
    if sb_idx.len() % 6 != 0 {
        return Err(Error::new(format!(
            "{}: sb_params count is not a multiple of 6",
            path.display()
        )));
    }
    let sb_lobes = (sb_idx.len() / 6) as i32;
    if sb_lobes > ColorBasis::MAX_SB_LOBES {
        return Err(Error::new(format!(
            "{}: {sb_lobes} spherical-beta lobes; at most {} are supported",
            path.display(),
            ColorBasis::MAX_SB_LOBES
        )));
    }
    let rows = read_rows(&bytes, &header, path)?;
    let n = rows.len();
    let k = ColorBasis::sh_coeffs_of(sh_degree) as usize;
    let mut cloud = SplatCloud {
        sh_degree,
        sb_lobes,
        primitive: primitive_from_header(&bytes[..header.body]),
        means: Vec::with_capacity(n),
        quats: Vec::with_capacity(n),
        scales: Vec::with_capacity(n),
        opacities: Vec::with_capacity(n),
        features_dc: Vec::with_capacity(n),
        features_sh: vec![0.0; n * (k * 3 + sb_lobes as usize * 6)],
    };
    let stride = k * 3 + sb_lobes as usize * 6;
    for (row_i, row) in rows.iter().enumerate() {
        cloud.means.push([row[ix] as f32, row[iy] as f32, row[iz] as f32]);
        cloud.opacities.push(row[iop] as f32);
        cloud.scales.push([row[isc[0]] as f32, row[isc[1]] as f32, row[isc[2]] as f32]);
        cloud.quats.push([
            row[iq[0]] as f32,
            row[iq[1]] as f32,
            row[iq[2]] as f32,
            row[iq[3]] as f32,
        ]);
        cloud.features_dc.push([row[idc[0]] as f32, row[idc[1]] as f32, row[idc[2]] as f32]);
        let base = row_i * stride;
        for (file_j, (_, prop)) in rest_idx.iter().enumerate() {
            let ch = file_j / k;
            let coeff = file_j % k;
            cloud.features_sh[base + coeff * 3 + ch] = row[*prop] as f32;
        }
        let lobe_base = base + k * 3;
        let lobes = sb_lobes as usize;
        for (file_j, (_, prop)) in sb_idx.iter().enumerate() {
            let field = file_j / lobes.max(1);
            let lobe = file_j % lobes.max(1);
            if lobes > 0 {
                cloud.features_sh[lobe_base + lobe * 6 + field] = row[*prop] as f32;
            }
        }
    }
    Ok(cloud)
}

pub fn write_splat_ply(cloud: &SplatCloud, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::new(format!("creating {}: {e}", parent.display())))?;
        }
    }
    let basis = cloud.basis();
    let mut f = File::create(path)
        .map_err(|e| Error::new(format!("creating {}: {e}", path.display())))?;
    let mut header = String::new();
    header.push_str("ply\nformat binary_little_endian 1.0\n");
    if cloud.primitive == "mip" {
        header.push_str("comment SplatRenderMode: mip\n");
    }
    header.push_str(&format!("element vertex {}\n", cloud.len()));
    for name in ["x", "y", "z", "nx", "ny", "nz", "f_dc_0", "f_dc_1", "f_dc_2"] {
        header.push_str(&format!("property float {name}\n"));
    }
    let k = basis.sh_coeffs() as usize;
    for j in 0..3 * k {
        header.push_str(&format!("property float f_rest_{j}\n"));
    }
    let lobes = basis.sb_lobes as usize;
    for j in 0..6 * lobes {
        header.push_str(&format!("property float sb_params_{j}\n"));
    }
    header.push_str("property float opacity\n");
    header.push_str("property float scale_0\nproperty float scale_1\nproperty float scale_2\n");
    header.push_str("property float rot_0\nproperty float rot_1\nproperty float rot_2\nproperty float rot_3\n");
    header.push_str("end_header\n");
    f.write_all(header.as_bytes())
        .map_err(|e| Error::new(format!("writing {}: {e}", path.display())))?;
    let stride = k * 3 + lobes * 6;
    for i in 0..cloud.len() {
        let mut row = Vec::with_capacity(3 + 3 + 3 + stride + 1 + 3 + 4);
        row.extend_from_slice(&cloud.means[i]);
        row.extend_from_slice(&[0.0, 0.0, 0.0]);
        row.extend_from_slice(&cloud.features_dc[i]);
        let sh = if cloud.features_sh.is_empty() {
            &[][..]
        } else {
            &cloud.features_sh[i * stride..(i + 1) * stride]
        };
        for ch in 0..3 {
            for j in 0..k {
                row.push(*sh.get(j * 3 + ch).unwrap_or(&0.0));
            }
        }
        let lobes_mem = &sh[k * 3..];
        for field in 0..6 {
            for lobe in 0..lobes {
                row.push(*lobes_mem.get(lobe * 6 + field).unwrap_or(&0.0));
            }
        }
        row.push(cloud.opacities[i]);
        row.extend_from_slice(&cloud.scales[i]);
        row.extend_from_slice(&cloud.quats[i]);
        for v in row {
            f.write_all(&v.to_le_bytes())
                .map_err(|e| Error::new(format!("writing {}: {e}", path.display())))?;
        }
    }
    Ok(())
}

pub fn read_point_ply(path: &Path) -> Result<PointCloud> {
    let bytes = read_bytes(path)?;
    let header = parse_header(&bytes, path)?;
    let find = |name: &str| header.props.iter().position(|p| p.name == name);
    let (Some(ix), Some(iy), Some(iz)) = (find("x"), find("y"), find("z")) else {
        return Err(Error::new(format!("{}: point cloud PLY needs x y z", path.display())));
    };
    let rows = read_rows(&bytes, &header, path)?;
    let mut cloud = PointCloud {
        xyz: Vec::with_capacity(rows.len()),
        rgb: Vec::with_capacity(rows.len()),
    };
    let ir = find("red");
    let ig = find("green");
    let ib = find("blue");
    for row in &rows {
        cloud.xyz.push([row[ix], row[iy], row[iz]]);
        if let (Some(r), Some(g), Some(b)) = (ir, ig, ib) {
            cloud.rgb.push([to_u8(row[r]), to_u8(row[g]), to_u8(row[b])]);
        } else {
            cloud.rgb.push([255, 255, 255]);
        }
    }
    Ok(cloud)
}

fn to_u8(v: f64) -> u8 {
    if v <= 1.0 && v >= 0.0 && v.fract() != 0.0 {
        (v * 255.0).round().clamp(0.0, 255.0) as u8
    } else {
        v.round().clamp(0.0, 255.0) as u8
    }
}

pub fn find_splat_ply(path: &Path) -> Result<(PathBuf, PathBuf)> {
    if path.is_file() {
        let cloud = read_splat_ply(path)?;
        if cloud.is_empty() && !path.exists() {
            return Err(Error::new(format!("{} is not a splat PLY", path.display())));
        }
        let run = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        return Ok((path.to_path_buf(), run));
    }
    if !path.is_dir() {
        return Err(Error::new(format!("{} is not a PLY or a run directory", path.display())));
    }
    let canonical = path.join("splat.ply");
    if canonical.is_file() {
        return Ok((canonical, path.to_path_buf()));
    }
    let mut best: Option<(i32, PathBuf, PathBuf)> = None;
    if let Ok(rd) = std::fs::read_dir(path) {
        for ent in rd.flatten() {
            let name = ent.file_name();
            let name = name.to_string_lossy();
            if let Some(step) = step_of_ckpt(&name) {
                let ply = ent.path().join("splat.ply");
                if ply.is_file() && best.as_ref().map(|b| step > b.0).unwrap_or(true) {
                    best = Some((step, ply, ent.path()));
                }
            }
        }
    }
    if let Some((_, ply, run)) = best {
        return Ok((ply, run));
    }
    Err(Error::new(format!(
        "no splat.ply under {}",
        path.display()
    )))
}

fn step_of_ckpt(name: &str) -> Option<i32> {
    let rest = name.strip_prefix("step-")?.strip_suffix(".ckpt")?;
    rest.parse().ok()
}

fn primitive_from_header(header: &[u8]) -> String {
    let text = String::from_utf8_lossy(header);
    if text.contains("SplatRenderMode: mip") {
        "mip".into()
    } else {
        String::new()
    }
}

fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    let mut f = File::open(path).map_err(|e| Error::new(format!("cannot open {}: {e}", path.display())))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)
        .map_err(|e| Error::new(format!("reading {}: {e}", path.display())))?;
    Ok(buf)
}

fn parse_header(bytes: &[u8], path: &Path) -> Result<Header> {
    let text_end = bytes
        .windows(11)
        .position(|w| w == b"end_header\n")
        .or_else(|| bytes.windows(12).position(|w| w == b"end_header\r\n"))
        .ok_or_else(|| Error::new(format!("{}: PLY header has no end_header", path.display())))?;
    let nl = if bytes.get(text_end + 10) == Some(&b'\n') { 11 } else { 12 };
    let header_txt = String::from_utf8_lossy(&bytes[..text_end]);
    if !header_txt.starts_with("ply") {
        return Err(Error::new(format!("{}: not a PLY", path.display())));
    }
    let mut format = None;
    let mut count = None;
    let mut props = Vec::new();
    let mut in_vertex = false;
    for line in header_txt.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("format ") {
            format = Some(if rest.starts_with("ascii") {
                PlyFormat::Ascii
            } else if rest.starts_with("binary_little_endian") {
                PlyFormat::BinaryLe
            } else {
                return Err(Error::new(format!("{}: unsupported PLY format '{rest}'", path.display())));
            });
        } else if let Some(rest) = line.strip_prefix("element ") {
            let mut it = rest.split_whitespace();
            let name = it.next().unwrap_or("");
            let n: usize = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            in_vertex = name == "vertex";
            if in_vertex {
                count = Some(n);
            }
        } else if let Some(rest) = line.strip_prefix("property ") {
            if !in_vertex {
                continue;
            }
            if rest.starts_with("list ") {
                return Err(Error::new(format!(
                    "{}: list properties are a mesh, not a splat or point cloud",
                    path.display()
                )));
            }
            let mut it = rest.split_whitespace();
            let ty = it.next().unwrap_or("");
            let name = it.next().unwrap_or("").to_string();
            let kind = match ty {
                "float" | "float32" => PropKind::F32,
                "double" | "float64" => PropKind::F64,
                "uchar" | "uint8" => PropKind::U8,
                "int" | "int32" => PropKind::I32,
                "uint" | "uint32" => PropKind::U32,
                other => {
                    return Err(Error::new(format!(
                        "{}: unsupported PLY type {other}",
                        path.display()
                    )))
                }
            };
            props.push(Prop { name, kind });
        }
    }
    let format = format.ok_or_else(|| Error::new(format!("{}: PLY has no format", path.display())))?;
    let count = count.ok_or_else(|| Error::new(format!("{}: PLY has no vertex element", path.display())))?;
    let vertex_bytes = if format == PlyFormat::BinaryLe {
        Some(props.iter().map(|p| p.kind.width()).sum())
    } else {
        None
    };
    Ok(Header { format, count, props, vertex_bytes, body: text_end + nl })
}

fn read_rows(bytes: &[u8], header: &Header, path: &Path) -> Result<Vec<Vec<f64>>> {
    match header.format {
        PlyFormat::Ascii => {
            let body = String::from_utf8_lossy(&bytes[header.body..]);
            let mut rows = Vec::with_capacity(header.count);
            for line in body.lines().map(str::trim).filter(|l| !l.is_empty()) {
                if rows.len() == header.count {
                    break;
                }
                let mut row = Vec::with_capacity(header.props.len());
                for tok in line.split_whitespace() {
                    let v: f64 = tok.parse().map_err(|_| {
                        Error::new(format!("{}: bad PLY value '{tok}'", path.display()))
                    })?;
                    row.push(v);
                }
                if row.len() < header.props.len() {
                    return Err(Error::new(format!("{}: short ASCII PLY row", path.display())));
                }
                rows.push(row);
            }
            if rows.len() != header.count {
                return Err(Error::new(format!(
                    "{}: expected {} vertices, read {}",
                    path.display(),
                    header.count,
                    rows.len()
                )));
            }
            Ok(rows)
        }
        PlyFormat::BinaryLe => {
            let width = header.vertex_bytes.unwrap();
            let need = header.count.saturating_mul(width);
            if bytes.len() < header.body + need {
                return Err(Error::new(format!("{}: truncated binary PLY", path.display())));
            }
            let mut rows = Vec::with_capacity(header.count);
            let mut o = header.body;
            for _ in 0..header.count {
                let mut row = Vec::with_capacity(header.props.len());
                for p in &header.props {
                    let v = match p.kind {
                        PropKind::F32 => f32::from_le_bytes(bytes[o..o + 4].try_into().unwrap()) as f64,
                        PropKind::F64 => f64::from_le_bytes(bytes[o..o + 8].try_into().unwrap()),
                        PropKind::U8 => bytes[o] as f64,
                        PropKind::I32 => i32::from_le_bytes(bytes[o..o + 4].try_into().unwrap()) as f64,
                        PropKind::U32 => u32::from_le_bytes(bytes[o..o + 4].try_into().unwrap()) as f64,
                    };
                    o += p.kind.width();
                    row.push(v);
                }
                rows.push(row);
            }
            Ok(rows)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splat_ply_roundtrip_keeps_sh_and_a_lobe() {
        let dir = std::env::temp_dir().join(format!("spirula-ply-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("splat.ply");
        let mut sh = vec![0.0; 9 + 6];
        sh[0] = 0.1;
        sh[1] = 0.2;
        sh[2] = 0.3;
        sh[3] = 1.0;
        sh[5] = 0.4;
        sh[9] = 0.25;
        sh[14] = -0.5;
        let cloud = SplatCloud {
            sh_degree: 1,
            sb_lobes: 1,
            primitive: String::new(),
            means: vec![[1.0, 2.0, 3.0]],
            quats: vec![[1.0, 0.0, 0.0, 0.0]],
            scales: vec![[-1.0, -1.5, -2.0]],
            opacities: vec![0.25],
            features_dc: vec![[0.5, -0.25, 0.0]],
            features_sh: sh.clone(),
        };
        write_splat_ply(&cloud, &path).unwrap();
        let back = read_splat_ply(&path).unwrap();
        assert_eq!(back.sh_degree, 1);
        assert_eq!(back.sb_lobes, 1);
        assert_eq!(back.means[0], [1.0, 2.0, 3.0]);
        assert_eq!(back.opacities[0], 0.25);
        assert_eq!(back.features_dc[0], [0.5, -0.25, 0.0]);
        for (a, b) in back.features_sh.iter().zip(sh.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ascii_point_cloud() {
        let dir = std::env::temp_dir().join(format!("spirula-pts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("points.ply");
        std::fs::write(
            &path,
            b"ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n1 2 3 10 20 30\n",
        )
        .unwrap();
        let pts = read_point_ply(&path).unwrap();
        assert_eq!(pts.xyz[0], [1.0, 2.0, 3.0]);
        assert_eq!(pts.rgb[0], [10, 20, 30]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
