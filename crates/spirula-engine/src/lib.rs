// SPDX-License-Identifier: GPL-3.0-or-later
// A pinhole training step for the Rust port: project, composite, L1/L2,
// Adam on means, opacity and DC. Everything else is listed in `port_gaps`
// and is not applied. Copyright (C) 2026 Spirula Studio contributors.

#![forbid(unsafe_code)]

mod adam;
mod densify;
mod gaps;
mod metrics;
mod model;
mod raster;
mod train;

pub use densify::DensifyMode;
pub use gaps::{port_gaps, Gap};
pub use metrics::{image_l1, image_psnr, image_ssim};
pub use model::{seed_from_points, GaussianModel};
pub use raster::render;
pub use train::{gaps_for, synthetic_scene, train, views_from_dataset, TrainOptions, TrainOutput, View};
