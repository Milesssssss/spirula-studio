use crate::model::{longest_axis, GaussianModel};
use spirula_core::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DensifyMode {
    Off,
    /// Prune faint splats and split the largest scales. Not MCMC.
    SplitPrune,
    Mcmc,
}

impl DensifyMode {
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "off" => Ok(Self::Off),
            "split-prune" => Ok(Self::SplitPrune),
            "mcmc" => Ok(Self::Mcmc),
            other => Err(Error::new(format!(
                "unknown densify mode '{other}' (off, split-prune, mcmc)"
            ))),
        }
    }
}

#[derive(Debug)]
pub struct DensifyStats {
    pub pruned: usize,
    pub split: usize,
}

pub fn optimizer_reset(stats: &DensifyStats) -> bool {
    stats.pruned > 0 || stats.split > 0
}

pub fn densify(
    model: &mut GaussianModel,
    mode: DensifyMode,
    min_opacity: f32,
    max_split_fraction: f32,
    cap_max: usize,
) -> Result<DensifyStats> {
    match mode {
        DensifyMode::Off => Ok(DensifyStats { pruned: 0, split: 0 }),
        DensifyMode::Mcmc => Err(Error::new(
            "MCMC densification is not ported; it remains in the C++ trainer. Use --densify split-prune or off",
        )),
        DensifyMode::SplitPrune => Ok(split_prune(model, min_opacity, max_split_fraction, cap_max)),
    }
}

fn split_prune(
    model: &mut GaussianModel,
    min_opacity: f32,
    fraction: f32,
    cap: usize,
) -> DensifyStats {
    let keep: Vec<usize> = (0..model.len())
        .filter(|&i| sigmoid(model.opacity_logit[i]) >= min_opacity)
        .collect();
    let mut pruned = model.len() - keep.len();
    retain(model, &keep);
    if model.len() > cap {
        let mut faint: Vec<usize> = (0..model.len()).collect();
        faint.sort_by(|&a, &b| {
            sigmoid(model.opacity_logit[a])
                .partial_cmp(&sigmoid(model.opacity_logit[b]))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let drop_n = model.len() - cap;
        let mut gone = faint[..drop_n].to_vec();
        gone.sort_unstable();
        let stay: Vec<usize> = (0..model.len())
            .filter(|i| gone.binary_search(i).is_err())
            .collect();
        pruned += drop_n;
        retain(model, &stay);
    }
    let n = model.len();
    if n == 0 || fraction <= 0.0 {
        return DensifyStats { pruned, split: 0 };
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        let sa = model.log_scales[a].into_iter().fold(f32::MIN, f32::max);
        let sb = model.log_scales[b].into_iter().fold(f32::MIN, f32::max);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut n_split = ((n as f32) * fraction).ceil().max(1.0) as usize;
    n_split = n_split.min(n).min(cap.saturating_sub(n));
    let chosen = order[..n_split].to_vec();
    let shrink = 1.6_f32.ln();
    for i in chosen {
        let axis = longest_axis(model.quats[i], model.log_scales[i]);
        let mut child_mean = model.means[i];
        for k in 0..3 {
            model.means[i][k] -= 0.5 * axis[k];
            child_mean[k] += 0.5 * axis[k];
            model.log_scales[i][k] -= shrink;
        }
        let quat = model.quats[i];
        let scale = model.log_scales[i];
        let opac = model.opacity_logit[i];
        let dc = model.dc[i];
        let row = if model.row_stride() > 0 {
            model.sh_row(i).to_vec()
        } else {
            Vec::new()
        };
        model.means.push(child_mean);
        model.quats.push(quat);
        model.log_scales.push(scale);
        model.opacity_logit.push(opac);
        model.dc.push(dc);
        model.features_sh.extend_from_slice(&row);
    }
    DensifyStats { pruned, split: n_split }
}

fn retain(model: &mut GaussianModel, keep: &[usize]) {
    if keep.len() == model.len() {
        return;
    }
    model.means = keep.iter().map(|&i| model.means[i]).collect();
    model.quats = keep.iter().map(|&i| model.quats[i]).collect();
    model.log_scales = keep.iter().map(|&i| model.log_scales[i]).collect();
    model.opacity_logit = keep.iter().map(|&i| model.opacity_logit[i]).collect();
    model.dc = keep.iter().map(|&i| model.dc[i]).collect();
    if model.row_stride() > 0 {
        let stride = model.row_stride();
        let mut next = Vec::with_capacity(keep.len() * stride);
        for &i in keep {
            next.extend_from_slice(model.sh_row(i));
        }
        model.features_sh = next;
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcmc_is_an_error_and_split_prune_changes_the_count() {
        let err = densify(&mut empty(), DensifyMode::Mcmc, 0.005, 0.1, 8).unwrap_err();
        assert!(err.message.contains("MCMC"));
        let mut model = GaussianModel {
            means: vec![[0.0; 3], [1.0, 0.0, 0.0]],
            quats: vec![[1.0, 0.0, 0.0, 0.0]; 2],
            log_scales: vec![[0.0, -4.0, -4.0], [-6.0, -6.0, -6.0]],
            opacity_logit: vec![2.0, -8.0],
            dc: vec![[0.0; 3]; 2],
            sh_degree: 0,
            sb_lobes: 0,
            features_sh: Vec::new(),
        };
        let stats = densify(&mut model, DensifyMode::SplitPrune, 0.05, 1.0, 8).unwrap();
        assert_eq!(stats.pruned, 1);
        assert_eq!(stats.split, 1);
        assert_eq!(model.len(), 2);
    }

    #[test]
    fn cap_max_stops_the_split() {
        let mut model = GaussianModel {
            means: vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]],
            quats: vec![[1.0, 0.0, 0.0, 0.0]; 4],
            log_scales: vec![[0.0; 3]; 4],
            opacity_logit: vec![2.0; 4],
            dc: vec![[0.0; 3]; 4],
            sh_degree: 0,
            sb_lobes: 0,
            features_sh: Vec::new(),
        };
        let stats = densify(&mut model, DensifyMode::SplitPrune, 0.05, 1.0, 2).unwrap();
        assert!(stats.pruned >= 2);
        assert_eq!(stats.split, 0);
        assert_eq!(model.len(), 2);
    }

    fn empty() -> GaussianModel {
        GaussianModel {
            means: Vec::new(),
            quats: Vec::new(),
            log_scales: Vec::new(),
            opacity_logit: Vec::new(),
            dc: Vec::new(),
            sh_degree: 0,
            sb_lobes: 0,
            features_sh: Vec::new(),
        }
    }
}
