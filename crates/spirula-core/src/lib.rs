// SPDX-License-Identifier: GPL-3.0-or-later
// Cameras, splat colour and the training-config subset for the Rust port.
// Host math follows shaders/projection_utils.slang and src/data/CameraMath.cpp.
// Copyright (C) 2026 Spirula Studio contributors. Derived from Spirula Studio
// by harry7557558, https://github.com/harry7557558/spirula-studio

#![forbid(unsafe_code)]

pub mod camera;
pub mod color;
pub mod config;
pub mod error;
pub mod math;
pub mod project;
pub mod sh;

pub use camera::{
    colmap_to_c2w_gl, c2w_gl_to_w2c_cv, CameraModel, Distortion,
};
pub use color::ColorBasis;
pub use sh::SH_C0;
pub use config::TrainConfig;
pub use error::{Error, Result};
pub use project::{project_pinhole, ProjectedSplat, ProjectionBatch};
