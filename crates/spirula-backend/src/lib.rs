// SPDX-License-Identifier: GPL-3.0-or-later
// Backend seam for the Rust port. The Vulkan path runs the committed SPIR-V
// of shaders/project_pinhole.comp, the pinhole subset of
// shaders/primitive_3dgs.slang. Copyright (C) 2026 Spirula Studio contributors.

mod cpu;
mod vulkan;

pub use cpu::CpuBackend;
pub use spirula_core::project::{ProjectedSplat, ProjectionBatch};
pub use spirula_core::{Error, Result};
pub use vulkan::{list_devices, VulkanBackend, VulkanDevice};

pub trait Backend {
    fn name(&self) -> &str;
    fn project(&mut self, batch: &ProjectionBatch<'_>) -> Result<Vec<ProjectedSplat>>;
}
