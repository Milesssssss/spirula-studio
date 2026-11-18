// SPDX-License-Identifier: GPL-3.0-or-later
// Dataset readers and splat PLY I/O for the Rust port of Spirula Studio.
// COLMAP binary layout follows src/data/parsers/ColmapParser.cpp. Metashape
// is not implemented. Copyright (C) 2026 Spirula Studio contributors.

#![forbid(unsafe_code)]

mod colmap;
mod dataset;
mod image;
mod nerfstudio;
mod ply;

pub use dataset::{load_dataset, Dataset, Frame};
pub use ply::PointCloud;
pub use image::{decode_rgb, resize_max_edge, RgbImage};
pub use ply::{read_point_ply, read_splat_ply, write_splat_ply, SplatCloud};
pub use spirula_core::Error;

pub fn find_splat_ply(path: &std::path::Path) -> Result<(std::path::PathBuf, std::path::PathBuf), Error> {
    ply::find_splat_ply(path)
}
