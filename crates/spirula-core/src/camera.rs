//! Camera models and the host lens math.
//!
//! Values match `CameraModelType` in src/core/CameraModel.h. Coordinates are
//! normalized image coordinates, rays are OpenCV camera space (+Z forward,
//! +Y down).

use crate::math::{self, PI};

pub const DISTORTION_PARAMS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CameraModel {
    Pinhole = 0,
    Fisheye = 1,
    Equisolid = 2,
    Equirectangular = 3,
    Orthographic = 4,
}

impl CameraModel {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "PINHOLE" | "SIMPLE_PINHOLE" | "SIMPLE_RADIAL" | "RADIAL" | "OPENCV" => Self::Pinhole,
            "FISHEYE" | "SIMPLE_FISHEYE" | "SIMPLE_RADIAL_FISHEYE" | "RADIAL_FISHEYE"
            | "OPENCV_FISHEYE" | "THIN_PRISM_FISHEYE" => Self::Fisheye,
            "ORTHOGRAPHIC" => Self::Orthographic,
            "EQUISOLID" => Self::Equisolid,
            "EQUIRECTANGULAR" => Self::Equirectangular,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pinhole => "PINHOLE",
            Self::Fisheye => "FISHEYE",
            Self::Equisolid => "EQUISOLID",
            Self::Equirectangular => "EQUIRECTANGULAR",
            Self::Orthographic => "ORTHOGRAPHIC",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Distortion {
    None = 0,
    OpenCv = 1,
    ThinPrism = 2,
}

impl Distortion {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "NONE" => Self::None,
            "OPENCV" => Self::OpenCv,
            "THIN_PRISM" => Self::ThinPrism,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::OpenCv => "OPENCV",
            Self::ThinPrism => "THIN_PRISM",
        }
    }

    pub fn num_params(self) -> usize {
        match self {
            Self::None => 0,
            Self::OpenCv => 4,
            Self::ThinPrism => 8,
        }
    }
}

pub fn demote_distortion(tier: Distortion, coeffs: &mut [f32; DISTORTION_PARAMS]) -> Distortion {
    let z = |v: f32| v == 0.0;
    if tier == Distortion::ThinPrism
        && z(coeffs[2])
        && z(coeffs[3])
        && z(coeffs[6])
        && z(coeffs[7])
    {
        let (k1, k2, p1, p2) = (coeffs[0], coeffs[1], coeffs[4], coeffs[5]);
        *coeffs = [0.0; DISTORTION_PARAMS];
        coeffs[0] = k1;
        coeffs[1] = k2;
        coeffs[2] = p1;
        coeffs[3] = p2;
        return demote_distortion(Distortion::OpenCv, coeffs);
    }
    if tier == Distortion::OpenCv && coeffs[..4].iter().all(|v| *v == 0.0) {
        *coeffs = [0.0; DISTORTION_PARAMS];
        return Distortion::None;
    }
    if tier == Distortion::None {
        *coeffs = [0.0; DISTORTION_PARAMS];
    }
    tier
}

pub fn promote_distortion(
    from: Distortion,
    to: Distortion,
    coeffs: &[f32; DISTORTION_PARAMS],
) -> Option<[f32; DISTORTION_PARAMS]> {
    if from == to {
        return Some(*coeffs);
    }
    let mut out = [0.0; DISTORTION_PARAMS];
    if from == Distortion::None {
        return Some(out);
    }
    if from == Distortion::OpenCv && to == Distortion::ThinPrism {
        out[0] = coeffs[0];
        out[1] = coeffs[1];
        out[4] = coeffs[2];
        out[5] = coeffs[3];
        return Some(out);
    }
    None
}

pub fn distortion_compiled(model: CameraModel, tier: Distortion) -> bool {
    match model {
        CameraModel::Pinhole | CameraModel::Fisheye | CameraModel::Equisolid => true,
        CameraModel::Equirectangular => tier == Distortion::None,
        CameraModel::Orthographic => tier == Distortion::None,
    }
}

pub fn has_distortion(tier: Distortion, d: &[f32; DISTORTION_PARAMS]) -> bool {
    tier != Distortion::None && d.iter().any(|v| *v != 0.0)
}

pub fn distort_lens(u: f64, v: f64, tier: Distortion, d: &[f32; DISTORTION_PARAMS]) -> [f64; 2] {
    let r2 = u * u + v * v;
    match tier {
        Distortion::None => [u, v],
        Distortion::OpenCv => {
            let radial = 1.0 + r2 * (d[0] as f64 + r2 * d[1] as f64);
            [
                u * radial + 2.0 * d[2] as f64 * u * v + d[3] as f64 * (r2 + 2.0 * u * u),
                v * radial + 2.0 * d[3] as f64 * u * v + d[2] as f64 * (r2 + 2.0 * v * v),
            ]
        }
        Distortion::ThinPrism => {
            let radial = 1.0
                + r2 * (d[0] as f64
                    + r2 * (d[1] as f64 + r2 * (d[2] as f64 + r2 * d[3] as f64)));
            [
                u * radial
                    + 2.0 * d[4] as f64 * u * v
                    + d[5] as f64 * (r2 + 2.0 * u * u)
                    + d[6] as f64 * r2,
                v * radial
                    + 2.0 * d[5] as f64 * u * v
                    + d[4] as f64 * (r2 + 2.0 * v * v)
                    + d[7] as f64 * r2,
            ]
        }
    }
}

fn distort_jac(u: f64, v: f64, tier: Distortion, d: &[f32; DISTORTION_PARAMS]) -> [f64; 4] {
    let e = 1e-5;
    let f = distort_lens(u, v, tier, d);
    let fx = distort_lens(u + e, v, tier, d);
    let fy = distort_lens(u, v + e, tier, d);
    [
        (fx[0] - f[0]) / e,
        (fx[1] - f[1]) / e,
        (fy[0] - f[0]) / e,
        (fy[1] - f[1]) / e,
    ]
}

pub fn undistort_lens(
    u: f64,
    v: f64,
    tier: Distortion,
    d: &[f32; DISTORTION_PARAMS],
) -> Option<[f64; 2]> {
    if tier == Distortion::None {
        return Some([u, v]);
    }
    let (mut qx, mut qy) = (u, v);
    for _ in 0..8 {
        let f = distort_lens(qx, qy, tier, d);
        let j = distort_jac(qx, qy, tier, d);
        let det = j[0] * j[3] - j[2] * j[1];
        if det.abs() < 1e-12 {
            break;
        }
        let (rx, ry, inv) = (f[0] - u, f[1] - v, 1.0 / det);
        qx -= (j[3] * rx - j[2] * ry) * inv;
        qy -= (-j[1] * rx + j[0] * ry) * inv;
    }
    if !qx.is_finite() || !qy.is_finite() {
        return None;
    }
    let j = distort_jac(qx, qy, tier, d);
    let det = j[0] * j[3] - j[2] * j[1];
    if det.min(j[0]).min(j[3]) <= 0.0 {
        return None;
    }
    let f = distort_lens(qx, qy, tier, d);
    if math::hypot2(f[0] - u, f[1] - v) >= 0.01 {
        return None;
    }
    Some([qx, qy])
}

pub fn valid_distortion(u: f64, v: f64, tier: Distortion, d: &[f32; DISTORTION_PARAMS]) -> bool {
    if tier == Distortion::None {
        return true;
    }
    let j = distort_jac(u, v, tier, d);
    let det = j[0] * j[3] - j[2] * j[1];
    let jd = det.min(j[0]).min(j[3]);
    let f = distort_lens(u, v, tier, d);
    jd > 0.25 && jd < 4.0 && (u * f[0] + v * f[1]) >= 0.0
}

pub fn distort_point(
    mut u: f64,
    mut v: f64,
    model: CameraModel,
    tier: Distortion,
    d: &[f32; DISTORTION_PARAMS],
) -> [f64; 2] {
    if model == CameraModel::Equirectangular {
        return [u, v];
    }
    if model == CameraModel::Fisheye || model == CameraModel::Equisolid {
        let r = math::hypot2(u, v);
        let theta = r.atan();
        let k = if model == CameraModel::Fisheye {
            if r < 1e-3 {
                1.0 - theta * theta / 6.0
            } else {
                theta / r
            }
        } else if r < 1e-3 {
            1.0 - theta * theta / 24.0
        } else {
            2.0 * (0.5 * theta).sin() / r
        };
        u *= k;
        v *= k;
    }
    distort_lens(u, v, tier, d)
}

pub fn generate_ray(
    u: f64,
    v: f64,
    model: CameraModel,
    tier: Distortion,
    d: &[f32; DISTORTION_PARAMS],
) -> Option<[f64; 3]> {
    if model == CameraModel::Orthographic {
        return Some([0.0, 0.0, 1.0]);
    }
    if model == CameraModel::Equirectangular {
        if u.abs() > PI || v.abs() > PI / 2.0 {
            return None;
        }
        let cl = v.cos();
        return Some([cl * u.sin(), v.sin(), cl * u.cos()]);
    }
    let (uu, vv) = if has_distortion(tier, d) {
        let q = undistort_lens(u, v, tier, d)?;
        (q[0], q[1])
    } else {
        (u, v)
    };
    let r = math::hypot2(uu, vv);
    let dir = if model == CameraModel::Fisheye {
        if r >= PI {
            return None;
        }
        let s = if r < 1e-3 {
            1.0 - r * r / 6.0
        } else {
            r.sin() / r
        };
        [uu * s, vv * s, r.cos()]
    } else if model == CameraModel::Equisolid {
        if r >= 2.0 {
            return None;
        }
        let k = (1.0 - 0.25 * r * r).max(0.0).sqrt();
        [uu * k, vv * k, 1.0 - 0.5 * r * r]
    } else {
        [uu, vv, 1.0]
    };
    Some(math::norm3(dir))
}

pub fn project_ray(
    ray: [f64; 3],
    model: CameraModel,
    tier: Distortion,
    d: &[f32; DISTORTION_PARAMS],
) -> Option<[f64; 2]> {
    if model == CameraModel::Equirectangular {
        return Some([
            ray[0].atan2(ray[2]),
            ray[1].atan2(math::hypot2(ray[0], ray[2])),
        ]);
    }
    if model == CameraModel::Orthographic {
        return Some([ray[0], ray[1]]);
    }
    if model == CameraModel::Pinhole {
        if ray[2] <= 1e-12 {
            return None;
        }
        let u = ray[0] / ray[2];
        let v = ray[1] / ray[2];
        if !valid_distortion(u, v, tier, d) {
            return None;
        }
        return Some(distort_lens(u, v, tier, d));
    }
    let rxy = math::hypot2(ray[0], ray[1]);
    let theta = rxy.atan2(ray[2]);
    if theta >= PI {
        return None;
    }
    let r_ang = if model == CameraModel::Fisheye {
        theta
    } else {
        2.0 * (0.5 * theta).sin()
    };
    let s = if rxy < 1e-12 { 0.0 } else { r_ang / rxy };
    let (u, v) = (ray[0] * s, ray[1] * s);
    if !valid_distortion(u, v, tier, d) {
        return None;
    }
    Some(distort_lens(u, v, tier, d))
}

pub fn pinhole_coverage(model: CameraModel, width: i32, height: i32, fx: f64, fy: f64) -> f64 {
    if model == CameraModel::Equirectangular {
        return 0.0;
    }
    if model != CameraModel::Fisheye && model != CameraModel::Equisolid {
        return 1.0;
    }
    if !(fx > 0.0) || !(fy > 0.0) || width <= 0 || height <= 0 {
        return 1.0;
    }
    let u = 0.5 * width as f64 / fx;
    let v = 0.5 * height as f64 / fy;
    let kept = |phi: f64| {
        let (c, s) = (phi.cos(), phi.sin());
        let rect = (u / c.max(1e-300)).min(v / s.max(1e-300));
        let theta = rect.atan();
        let r = if model == CameraModel::Fisheye {
            theta
        } else {
            2.0 * (0.5 * theta).sin()
        };
        r * r
    };
    let corner = v.atan2(u);
    let mut area = 0.0;
    for half in 0..2 {
        let (a, b) = if half == 0 {
            (0.0, corner)
        } else {
            (corner, 0.5 * PI)
        };
        let n = 128;
        let h = (b - a) / n as f64;
        let mut s = kept(a) + kept(b);
        for i in 1..n {
            s += kept(a + i as f64 * h) * if i & 1 == 1 { 4.0 } else { 2.0 };
        }
        area += s * h / 3.0;
    }
    (area / (2.0 * u * v)).min(1.0)
}

pub fn splits_to_pinhole_faces(
    model: CameraModel,
    width: i32,
    height: i32,
    fx: f64,
    fy: f64,
) -> bool {
    pinhole_coverage(model, width, height, fx, fy) <= 0.75
}

/// COLMAP `(qw, qx, qy, qz)` and `t` (world to camera, OpenCV) to a
/// Nerfstudio/OpenGL camera-to-world 3x4, row-major, matching `colmap_to_c2w`.
pub fn colmap_to_c2w_gl(qvec: [f64; 4], tvec: [f64; 3]) -> [f64; 12] {
    let r = math::quat_to_rotmat(qvec);
    let flip = [1.0, -1.0, -1.0];
    let mut out = [0.0; 12];
    for row in 0..3 {
        for col in 0..3 {
            out[row * 4 + col] = r[col * 3 + row] * flip[col];
        }
        let mut t = 0.0;
        for col in 0..3 {
            t -= r[col * 3 + row] * tvec[col];
        }
        out[row * 4 + 3] = t;
    }
    out
}

/// OpenGL c2w 3x4 to the engine world-to-camera 3x4 (OpenCV axes).
pub fn c2w_gl_to_w2c_cv(c2w: &[f64; 12]) -> [f64; 12] {
    let d = [1.0, -1.0, -1.0];
    let mut vm = [0.0; 12];
    for r in 0..3 {
        let mut ti = 0.0;
        for c in 0..3 {
            let rf = c2w[c * 4 + r] * d[r];
            vm[r * 4 + c] = rf;
            ti -= rf * c2w[c * 4 + 3];
        }
        vm[r * 4 + 3] = ti;
    }
    vm
}

pub fn pixel_to_normalized(px: f64, py: f64, fx: f64, fy: f64, cx: f64, cy: f64) -> [f64; 2] {
    [(px - cx) / fx, (py - cy) / fy]
}

pub fn normalized_to_pixel(u: f64, v: f64, fx: f64, fy: f64, cx: f64, cy: f64) -> [f64; 2] {
    [fx * u + cx, fy * v + cy]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_the_cpp_enum() {
        assert_eq!(CameraModel::from_name("OPENCV_FISHEYE"), Some(CameraModel::Fisheye));
        assert_eq!(CameraModel::from_name("SIMPLE_PINHOLE").unwrap() as i32, 0);
        assert_eq!(CameraModel::Equirectangular as i32, 3);
        assert_eq!(CameraModel::Orthographic.as_str(), "ORTHOGRAPHIC");
        assert!(CameraModel::from_name("FOV").is_none());
    }

    #[test]
    fn opencv_distortion_roundtrip() {
        let mut d = [0.0; 8];
        d[0] = -0.12;
        d[1] = 0.01;
        d[2] = 0.001;
        d[3] = -0.002;
        let uv = distort_lens(0.2, -0.15, Distortion::OpenCv, &d);
        let back = undistort_lens(uv[0], uv[1], Distortion::OpenCv, &d).unwrap();
        assert!((back[0] - 0.2).abs() < 1e-6);
        assert!((back[1] + 0.15).abs() < 1e-6);
    }

    #[test]
    fn thin_prism_demotes_when_the_tail_is_zero() {
        let mut d = [0.0; 8];
        d[0] = 0.1;
        d[1] = -0.02;
        d[4] = 0.001;
        d[5] = -0.001;
        let tier = demote_distortion(Distortion::ThinPrism, &mut d);
        assert_eq!(tier, Distortion::OpenCv);
        assert!((d[2] - 0.001).abs() < 1e-8);
        assert_eq!(d[4], 0.0);
    }

    #[test]
    fn pinhole_ray_roundtrip() {
        let d = [0.0; 8];
        let ray = math::norm3([0.2, -0.4, 1.0]);
        let uv = project_ray(ray, CameraModel::Pinhole, Distortion::None, &d).unwrap();
        let back = generate_ray(uv[0], uv[1], CameraModel::Pinhole, Distortion::None, &d).unwrap();
        for i in 0..3 {
            assert!((ray[i] - back[i]).abs() < 1e-9, "{ray:?} vs {back:?}");
        }
    }

    #[test]
    fn fisheye_and_equirect_roundtrip() {
        let d = [0.0; 8];
        let ray = math::norm3([0.4, 0.2, 0.7]);
        for model in [CameraModel::Fisheye, CameraModel::Equisolid, CameraModel::Equirectangular] {
            let uv = project_ray(ray, model, Distortion::None, &d).unwrap();
            let back = generate_ray(uv[0], uv[1], model, Distortion::None, &d).unwrap();
            for i in 0..3 {
                assert!(
                    (ray[i] - back[i]).abs() < 1e-6,
                    "{model:?} {ray:?} vs {back:?}"
                );
            }
        }
    }

    #[test]
    fn colmap_identity_pose_flips_opengl_axes() {
        let c2w = colmap_to_c2w_gl([1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((c2w[0] - 1.0).abs() < 1e-12);
        assert!((c2w[5] + 1.0).abs() < 1e-12);
        assert!((c2w[10] + 1.0).abs() < 1e-12);
        let w2c = c2w_gl_to_w2c_cv(&c2w);
        assert!((w2c[0] - 1.0).abs() < 1e-12 && (w2c[5] - 1.0).abs() < 1e-12);
        assert!((w2c[10] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn wide_fisheye_splits_equirect_always_does() {
        assert!(splits_to_pinhole_faces(
            CameraModel::Equirectangular,
            1024,
            512,
            1024.0 / (2.0 * PI),
            512.0 / PI
        ));
        assert!(!splits_to_pinhole_faces(CameraModel::Pinhole, 640, 480, 500.0, 500.0));
        assert!(splits_to_pinhole_faces(CameraModel::Fisheye, 800, 800, 200.0, 200.0));
        assert_eq!(
            pinhole_coverage(CameraModel::Equirectangular, 10, 10, 1.0, 1.0),
            0.0
        );
    }
}
