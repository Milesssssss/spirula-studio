//! RGB images as linear or display-encoded floats in 0..1.

use spirula_core::{Error, Result};

#[derive(Clone, Debug)]
pub struct RgbImage {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<f32>,
}

pub fn decode_rgb(bytes: &[u8], linear: bool) -> Result<RgbImage> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| Error::new(format!("image decode: {e}")))?;
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();
    let mut out = Vec::with_capacity((width * height * 3) as usize);
    for p in rgb.pixels() {
        for c in p.0 {
            let u = c as f32 / 255.0;
            out.push(if linear { u } else { srgb_to_linear(u) });
        }
    }
    Ok(RgbImage { width, height, rgb: out })
}

pub fn resize_max_edge(img: &RgbImage, max_edge: u32) -> RgbImage {
    if max_edge == 0 || img.width.max(img.height) <= max_edge {
        return img.clone();
    }
    let scale = max_edge as f32 / img.width.max(img.height) as f32;
    let nw = ((img.width as f32 * scale).round() as u32).max(1);
    let nh = ((img.height as f32 * scale).round() as u32).max(1);
    let mut rgb = vec![0.0; (nw * nh * 3) as usize];
    for y in 0..nh {
        let y0 = (y as f32 / nh as f32 * img.height as f32) as u32;
        let y1 = (((y + 1) as f32 / nh as f32 * img.height as f32) as u32).max(y0 + 1).min(img.height);
        for x in 0..nw {
            let x0 = (x as f32 / nw as f32 * img.width as f32) as u32;
            let x1 = (((x + 1) as f32 / nw as f32 * img.width as f32) as u32).max(x0 + 1).min(img.width);
            let mut acc = [0.0; 3];
            let mut n = 0.0;
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let i = ((yy * img.width + xx) * 3) as usize;
                    for c in 0..3 {
                        acc[c] += img.rgb[i + c];
                    }
                    n += 1.0;
                }
            }
            let o = ((y * nw + x) * 3) as usize;
            for c in 0..3 {
                rgb[o + c] = acc[c] / n;
            }
        }
    }
    RgbImage { width: nw, height: nh, rgb }
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_black_and_white_stay_put() {
        let png = {
            let mut img = image::RgbImage::new(2, 2);
            img.put_pixel(0, 0, image::Rgb([0, 0, 0]));
            img.put_pixel(1, 0, image::Rgb([255, 255, 255]));
            img.put_pixel(0, 1, image::Rgb([128, 0, 0]));
            img.put_pixel(1, 1, image::Rgb([0, 0, 0]));
            let mut buf = std::io::Cursor::new(Vec::new());
            image::ImageEncoder::write_image(
                image::codecs::png::PngEncoder::new(&mut buf),
                img.as_raw(),
                2,
                2,
                image::ExtendedColorType::Rgb8,
            )
            .unwrap();
            buf.into_inner()
        };
        let decoded = decode_rgb(&png, false).unwrap();
        assert_eq!(decoded.rgb[0], 0.0);
        assert!((decoded.rgb[3] - 1.0).abs() < 1e-6);
        assert!(decoded.rgb[6] > 0.2 && decoded.rgb[6] < 0.3);
        let small = resize_max_edge(&decoded, 1);
        assert_eq!(small.width.max(small.height), 1);
    }
}
