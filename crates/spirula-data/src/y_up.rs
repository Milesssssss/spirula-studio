//! SuperSplat Y-up PLY turn from src/checkpoint/SplatYUp.h.
//! The stored map is (x, y, z) -> (-x, -z, -y). It is an involution; the
//! quaternion sign is what separates the two directions. SH uses the
//! Ivanic-Ruedenberg recursion in src/core/ShRotation.cpp, conjugated for
//! the Condon-Shortley phase in shaders/harmonics.slang.

use spirula_core::color::{lobe_axis, ColorBasis};

pub const Y_UP_COMMENT: &str = "comment spirula_coordinate_system supersplat_y_up";

const Y_UP_R: [f64; 9] = [-1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -1.0, 0.0];

struct Term {
    column: usize,
    value: f32,
}

pub struct SupersplatYUp {
    rows: Vec<Vec<Term>>,
}

impl SupersplatYUp {
    pub fn new() -> Self {
        let sh = ShRotation::new(&Y_UP_R, 4);
        let mut rows = (0..24).map(|_| Vec::new()).collect::<Vec<_>>();
        for l in 1..=4 {
            let w = 2 * l + 1;
            let at = l * l - 1;
            let band = &sh.bands[l as usize];
            for a in 0..w {
                for b in 0..w {
                    let value = band[(a * w + b) as usize];
                    if value.abs() > 1e-12 {
                        rows[(at + a) as usize].push(Term {
                            column: (at + b) as usize,
                            value: value as f32,
                        });
                    }
                }
            }
        }
        Self { rows }
    }

    /// `rest` is coefficient-major SH, then six floats per lobe. `to_native`
    /// is the decode direction.
    pub fn apply(
        &self,
        mean: &mut [f32; 3],
        quat: &mut [f32; 4],
        rest: Option<&mut [f32]>,
        basis: ColorBasis,
        to_native: bool,
    ) {
        let y = mean[1];
        mean[0] = -mean[0];
        mean[1] = -mean[2];
        mean[2] = -y;
        // h and -h are one rotation. The sign is what a round trip undoes.
        let h = if to_native {
            -0.7071067811865475244f32
        } else {
            0.7071067811865475244f32
        };
        let (w, x, qy, z) = (quat[0], quat[1], quat[2], quat[3]);
        quat[0] = h * (z - qy);
        quat[1] = h * (z + qy);
        quat[2] = h * (w - x);
        quat[3] = -h * (w + x);
        let Some(rest) = rest else {
            return;
        };
        let n = basis.sh_coeffs() as usize;
        if n > 0 {
            let mut rotated = vec![0.0f32; n * 3];
            for a in 0..n.min(self.rows.len()) {
                for term in &self.rows[a] {
                    if 3 * term.column + 2 >= rest.len() {
                        continue;
                    }
                    for ch in 0..3 {
                        rotated[3 * a + ch] += term.value * rest[3 * term.column + ch];
                    }
                }
            }
            let n3 = (n * 3).min(rest.len());
            rest[..n3].copy_from_slice(&rotated[..n3]);
        }
        for lobe in 0..basis.sb_lobes.max(0) as usize {
            let base = 3 * n + 6 * lobe;
            if base + 4 >= rest.len() {
                break;
            }
            let axis = lobe_axis(rest[base + 3] as f64, rest[base + 4] as f64);
            let z = (-axis[1]).clamp(-1.0, 1.0);
            rest[base + 3] = z.acos() as f32;
            rest[base + 4] = (-axis[2]).atan2(-axis[0]) as f32;
        }
    }
}

impl Default for SupersplatYUp {
    fn default() -> Self {
        Self::new()
    }
}

struct ShRotation {
    bands: Vec<Vec<f64>>,
}

impl ShRotation {
    fn new(r: &[f64; 9], degree: i32) -> Self {
        let degree = degree.clamp(0, 4);
        let mut bands = vec![Vec::new(); 5];
        bands[0] = vec![1.0];
        if degree < 1 {
            return Self { bands };
        }
        // Band 1 of the phase-free basis is (y, z, x), so M_1 is R permuted.
        let p = [1usize, 2, 0];
        let mut r1 = vec![0.0; 9];
        for i in 0..3 {
            for j in 0..3 {
                r1[i * 3 + j] = r[p[i] * 3 + p[j]];
            }
        }
        bands[1] = r1;
        for l in 2..=degree {
            let prev_l = l - 1;
            let w = 2 * l + 1;
            let mut m = vec![0.0; (w * w) as usize];
            let prev = bands[prev_l as usize].clone();
            let b1 = bands[1].clone();
            let at = |band: &[f64], ll: i32, a: i32, b: i32| -> f64 {
                let ww = 2 * ll + 1;
                band[((a + ll) * ww + (b + ll)) as usize]
            };
            let pfun = |i: i32, a: i32, b: i32| -> f64 {
                if b == l {
                    return at(&b1, 1, i, 1) * at(&prev, prev_l, a, l - 1)
                        - at(&b1, 1, i, -1) * at(&prev, prev_l, a, -l + 1);
                }
                if b == -l {
                    return at(&b1, 1, i, 1) * at(&prev, prev_l, a, -l + 1)
                        + at(&b1, 1, i, -1) * at(&prev, prev_l, a, l - 1);
                }
                at(&b1, 1, i, 0) * at(&prev, prev_l, a, b)
            };
            for mm in -l..=l {
                for n in -l..=l {
                    let am = mm.abs();
                    let d = if n.abs() == l {
                        (2 * l) as f64 * (2 * l - 1) as f64
                    } else {
                        (l + n) as f64 * (l - n) as f64
                    };
                    let d0 = if mm == 0 { 1.0 } else { 0.0 };
                    let u = (((l + mm) as f64 * (l - mm) as f64) / d).sqrt();
                    let v = 0.5
                        * (1.0 - 2.0 * d0)
                        * (((1.0 + d0) * (l + am - 1) as f64 * (l + am) as f64) / d).sqrt();
                    let ww =
                        -0.5 * (1.0 - d0) * (((l - am - 1) as f64 * (l - am) as f64) / d).sqrt();
                    let mut acc = 0.0;
                    if u != 0.0 {
                        acc += u * pfun(0, mm, n);
                    }
                    if v != 0.0 {
                        let vv = if mm == 0 {
                            pfun(1, 1, n) + pfun(-1, -1, n)
                        } else if mm > 0 {
                            if mm == 1 {
                                2.0f64.sqrt() * pfun(1, 0, n)
                            } else {
                                pfun(1, mm - 1, n) - pfun(-1, -mm + 1, n)
                            }
                        } else if mm == -1 {
                            2.0f64.sqrt() * pfun(-1, 0, n)
                        } else {
                            pfun(1, mm + 1, n) + pfun(-1, -mm - 1, n)
                        };
                        acc += v * vv;
                    }
                    if ww != 0.0 {
                        let extra = if mm > 0 {
                            pfun(1, mm + 1, n) + pfun(-1, -mm - 1, n)
                        } else {
                            pfun(1, mm - 1, n) - pfun(-1, -mm + 1, n)
                        };
                        acc += ww * extra;
                    }
                    m[((mm + l) * w + (n + l)) as usize] = acc;
                }
            }
            bands[l as usize] = m;
        }
        for l in 1..=degree {
            let w = 2 * l + 1;
            let m = &mut bands[l as usize];
            for a in -l..=l {
                for b in -l..=l {
                    if (a + b) & 1 != 0 {
                        let i = ((a + l) * w + (b + l)) as usize;
                        m[i] = -m[i];
                    }
                }
            }
        }
        Self { bands }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nonzeros(l: i32) -> Vec<(i32, i32, f64)> {
        let sh = ShRotation::new(&Y_UP_R, 4);
        let w = 2 * l + 1;
        let band = &sh.bands[l as usize];
        let mut out = Vec::new();
        for a in 0..w {
            for b in 0..w {
                let v = band[(a * w + b) as usize];
                if v.abs() > 1e-12 {
                    out.push((a, b, v));
                }
            }
        }
        out
    }

    #[test]
    fn bands_match_sh_rotation_for_the_supersplat_turn() {
        let expect = [
            (1, vec![(0, 1, 1.0), (1, 0, 1.0), (2, 2, -1.0)]),
            (
                2,
                vec![
                    (0, 3, -1.0),
                    (1, 1, 1.0000000000000002),
                    (2, 2, -0.5),
                    (2, 4, -0.8660254037844386),
                    (3, 0, -1.0),
                    (4, 2, -0.8660254037844386),
                    (4, 4, 0.5),
                ],
            ),
            (
                3,
                vec![
                    (0, 3, -0.79056941504209488),
                    (0, 5, 0.61237243569579447),
                    (1, 1, -1.0),
                    (2, 3, -0.61237243569579447),
                    (2, 5, -0.79056941504209488),
                    (3, 0, -0.79056941504209477),
                    (3, 2, -0.61237243569579458),
                    (4, 4, 0.25),
                    (4, 6, 0.96824583655185414),
                    (5, 0, 0.61237243569579447),
                    (5, 2, -0.7905694150420951),
                    (6, 4, 0.96824583655185426),
                    (6, 6, -0.25),
                ],
            ),
            (
                4,
                vec![
                    (0, 5, 0.93541434669348533),
                    (0, 7, -0.35355339059327379),
                    (1, 1, 0.74999999999999989),
                    (1, 3, -0.6614378277661479),
                    (2, 5, 0.35355339059327373),
                    (2, 7, 0.93541434669348533),
                    (3, 1, -0.66143782776614779),
                    (3, 3, -0.75000000000000022),
                    (4, 4, 0.37499999999999994),
                    (4, 6, 0.55901699437494745),
                    (4, 8, 0.73950997288745191),
                    (5, 0, 0.93541434669348533),
                    (5, 2, 0.35355339059327379),
                    (6, 4, 0.55901699437494745),
                    (6, 6, 0.50000000000000011),
                    (6, 8, -0.66143782776614757),
                    (7, 0, -0.35355339059327373),
                    (7, 2, 0.93541434669348533),
                    (8, 4, 0.73950997288745202),
                    (8, 6, -0.66143782776614768),
                    (8, 8, 0.125),
                ],
            ),
        ];
        for (l, rows) in expect {
            let got = nonzeros(l);
            assert_eq!(got.len(), rows.len(), "band {l}");
            for (g, e) in got.iter().zip(rows.iter()) {
                assert_eq!((g.0, g.1), (e.0, e.1), "band {l}");
                assert!(
                    (g.2 - e.2).abs() < 1e-12,
                    "band {l} {} {} {} vs {}",
                    g.0,
                    g.1,
                    g.2,
                    e.2
                );
            }
        }
    }

    #[test]
    fn export_matches_splat_y_up_and_round_trips() {
        let yup = SupersplatYUp::new();
        let basis = ColorBasis {
            sh_degree: 2,
            sb_lobes: 1,
        };
        let mut mean = [1.0f32, 2.0, 3.0];
        let mut quat = [1.0f32, 0.0, 0.0, 0.0];
        let mut rest = vec![0.0f32; 3 * basis.sh_coeffs() as usize + 6];
        rest[0] = 1.0;
        rest[4] = -0.5;
        rest[3 * basis.sh_coeffs() as usize + 3] = 0.4;
        rest[3 * basis.sh_coeffs() as usize + 4] = -1.2;
        rest[3 * basis.sh_coeffs() as usize + 5] = 0.25;
        let original = rest.clone();
        yup.apply(&mut mean, &mut quat, Some(&mut rest), basis, false);
        assert_eq!(mean.map(f32::to_bits), [3212836864, 3225419776, 3221225472]);
        assert_eq!(quat.map(f32::to_bits), [0, 0, 1060439283, 3207922931]);
        let mut expect = vec![0.0f32; rest.len()];
        expect[1] = f32::from_bits(3204448256);
        expect[3] = f32::from_bits(1065353216);
        expect[27] = f32::from_bits(1067025578);
        expect[28] = f32::from_bits(3218900290);
        expect[29] = f32::from_bits(1048576000);
        for (i, (g, e)) in rest.iter().zip(expect.iter()).enumerate() {
            assert_eq!(g.to_bits(), e.to_bits(), "rest {i}");
        }
        yup.apply(&mut mean, &mut quat, Some(&mut rest), basis, true);
        assert_eq!(mean, [1.0, 2.0, 3.0]);
        for (a, b) in quat.iter().zip([1.0, 0.0, 0.0, 0.0]) {
            assert!((a - b).abs() < 2e-6, "{quat:?}");
        }
        for i in 0..3 * basis.sh_coeffs() as usize {
            assert!((rest[i] - original[i]).abs() < 2e-6, "sh {i}");
        }
        let back = lobe_axis(rest[27] as f64, rest[28] as f64);
        let start = lobe_axis(0.4, -1.2);
        for k in 0..3 {
            assert!((back[k] - start[k]).abs() < 2e-6);
        }
        assert!((rest[29] - 0.25).abs() < 1e-6);
    }
}
