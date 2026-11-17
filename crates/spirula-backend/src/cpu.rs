use crate::{Backend, ProjectedSplat, ProjectionBatch, Result};
use spirula_core::project::project_pinhole;

pub struct CpuBackend;

impl Backend for CpuBackend {
    fn name(&self) -> &str {
        "cpu"
    }

    fn project(&mut self, batch: &ProjectionBatch<'_>) -> Result<Vec<ProjectedSplat>> {
        Ok(project_pinhole(batch))
    }
}
