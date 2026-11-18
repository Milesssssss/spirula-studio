//! Small host vectors. Row-major 3x3 matches the COLMAP `qvec2rotmat` layout.

pub const PI: f64 = 3.14159265358979323846;

#[inline]
pub fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
pub fn norm3(v: [f64; 3]) -> [f64; 3] {
    let l = dot3(v, v).sqrt();
    let l = if l < 1e-12 { 1.0 } else { l };
    [v[0] / l, v[1] / l, v[2] / l]
}

#[inline]
pub fn hypot2(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

/// Unit quaternion `(w, x, y, z)` to the COLMAP / Hamilton rotation, row-major.
pub fn quat_to_rotmat(q: [f64; 4]) -> [f64; 9] {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    let (w, x, y, z) = if n < 1e-12 {
        (1.0, 0.0, 0.0, 0.0)
    } else {
        (q[0] / n, q[1] / n, q[2] / n, q[3] / n)
    };
    let (x2, y2, z2) = (x * x, y * y, z * z);
    let (xy, xz, yz) = (x * y, x * z, y * z);
    let (wx, wy, wz) = (w * x, w * y, w * z);
    [
        1.0 - 2.0 * (y2 + z2),
        2.0 * (xy - wz),
        2.0 * (xz + wy),
        2.0 * (xy + wz),
        1.0 - 2.0 * (x2 + z2),
        2.0 * (yz - wx),
        2.0 * (xz - wy),
        2.0 * (yz + wx),
        1.0 - 2.0 * (x2 + y2),
    ]
}

pub fn mul_mat3_vec(m: &[f64; 9], v: [f64; 3]) -> [f64; 3] {
    [
        m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
        m[3] * v[0] + m[4] * v[1] + m[5] * v[2],
        m[6] * v[0] + m[7] * v[1] + m[8] * v[2],
    ]
}

/// f32 twin of [`quat_to_rotmat`], used by the pinhole projector.
pub fn quat_to_rotmat_f32(q: [f32; 4]) -> [[f32; 3]; 3] {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    let (w, x, y, z) = if n == 0.0 {
        (1.0, 0.0, 0.0, 0.0)
    } else {
        (q[0] / n, q[1] / n, q[2] / n, q[3] / n)
    };
    let (x2, y2, z2) = (x * x, y * y, z * z);
    let (xy, xz, yz) = (x * y, x * z, y * z);
    let (wx, wy, wz) = (w * x, w * y, w * z);
    [
        [
            1.0 - 2.0 * (y2 + z2),
            2.0 * (xy - wz),
            2.0 * (xz + wy),
        ],
        [
            2.0 * (xy + wz),
            1.0 - 2.0 * (x2 + z2),
            2.0 * (yz - wx),
        ],
        [
            2.0 * (xz - wy),
            2.0 * (yz + wx),
            1.0 - 2.0 * (x2 + y2),
        ],
    ]
}
