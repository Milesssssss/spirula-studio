//! l1 / psnr / SSIM, the no-mask path of src/app/EvalMetrics.cpp.
//! SSIM is torchmetrics' 11x11 gaussian, data range 1, reflect padding.

use spirula_core::{Error, Result};

pub fn image_l1(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut s = 0.0_f64;
    for i in 0..n {
        s += (a[i] as f64 - b[i] as f64).abs();
    }
    (s / n as f64) as f32
}

pub fn image_psnr(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    if n == 0 {
        return f32::INFINITY;
    }
    let mut s = 0.0_f64;
    for i in 0..n {
        let d = a[i] as f64 - b[i] as f64;
        s += d * d;
    }
    let mse = s / n as f64;
    if mse <= 0.0 {
        f32::INFINITY
    } else {
        (-10.0 * mse.log10()) as f32
    }
}

pub fn image_ssim(a: &[f32], b: &[f32], width: u32, height: u32) -> Result<f32> {
    if height < 11 || width < 11 {
        return Err(Error::new("image_ssim: image smaller than the 11x11 window"));
    }
    let (h, w) = (height as usize, width as usize);
    let np = h * w;
    if a.len() < np * 3 || b.len() < np * 3 {
        return Err(Error::new("image_ssim: buffer shorter than H*W*3"));
    }
    let g = gaussian_window();
    let c1 = 0.01_f64 * 0.01;
    let c2 = 0.03_f64 * 0.03;
    let mut total = 0.0;
    for c in 0..3 {
        let mut x = vec![0.0; np];
        let mut y = vec![0.0; np];
        let mut xx = vec![0.0; np];
        let mut yy = vec![0.0; np];
        let mut xy = vec![0.0; np];
        for p in 0..np {
            let va = a[p * 3 + c] as f64;
            let vb = b[p * 3 + c] as f64;
            x[p] = va;
            y[p] = vb;
            xx[p] = va * va;
            yy[p] = vb * vb;
            xy[p] = va * vb;
        }
        let mx = conv(&x, h, w, &g);
        let my = conv(&y, h, w, &g);
        let mxx = conv(&xx, h, w, &g);
        let myy = conv(&yy, h, w, &g);
        let mxy = conv(&xy, h, w, &g);
        let mut acc = 0.0;
        for i in 0..np {
            let ux = mx[i];
            let uy = my[i];
            let sxx = (mxx[i] - ux * ux).max(0.0);
            let syy = (myy[i] - uy * uy).max(0.0);
            let sxy = mxy[i] - ux * uy;
            let num = (2.0 * ux * uy + c1) * (2.0 * sxy + c2);
            let den = (ux * ux + uy * uy + c1) * (sxx + syy + c2);
            acc += num / den;
        }
        total += acc / np as f64;
    }
    Ok((total / 3.0) as f32)
}

fn gaussian_window() -> [f64; 11] {
    let mut g = [0.0; 11];
    let mut sum = 0.0;
    for i in 0..11 {
        let x = i as f64 - 5.0;
        g[i] = (-(x * x) / (2.0 * 1.5 * 1.5)).exp();
        sum += g[i];
    }
    for v in &mut g {
        *v /= sum;
    }
    g
}

fn reflect(i: isize, n: isize) -> usize {
    let v = if i < 0 {
        -i
    } else if i >= n {
        2 * (n - 1) - i
    } else {
        i
    };
    v as usize
}

fn conv(src: &[f64], h: usize, w: usize, g: &[f64; 11]) -> Vec<f64> {
    let mut tmp = vec![0.0; h * w];
    let mut dst = vec![0.0; h * w];
    let p = 5isize;
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0;
            for k in 0..11 {
                let xx = reflect(x as isize + k as isize - p, w as isize);
                acc += src[y * w + xx] * g[k];
            }
            tmp[y * w + x] = acc;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0;
            for k in 0..11 {
                let yy = reflect(y as isize + k as isize - p, h as isize);
                acc += tmp[yy * w + x] * g[k];
            }
            dst[y * w + x] = acc;
        }
    }
    dst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images_score_perfectly() {
        let mut img = vec![0.0; 16 * 16 * 3];
        for (i, v) in img.iter_mut().enumerate() {
            *v = ((i * 17) % 64) as f32 / 64.0;
        }
        assert_eq!(image_l1(&img, &img), 0.0);
        assert!(image_psnr(&img, &img).is_infinite());
        let ssim = image_ssim(&img, &img, 16, 16).unwrap();
        assert!((ssim - 1.0).abs() < 1e-5, "{ssim}");
    }

    #[test]
    fn l1_of_black_and_white_is_one() {
        let a = vec![0.0; 3];
        let b = vec![1.0; 3];
        assert!((image_l1(&a, &b) - 1.0).abs() < 1e-6);
    }
}
