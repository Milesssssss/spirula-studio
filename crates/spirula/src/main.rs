// SPDX-License-Identifier: GPL-3.0-or-later
// `spirula` for the Rust port. The C++ binary of the same name is still the
// full trainer. Copyright (C) 2026 Spirula Studio contributors, derived from
// Spirula Studio by harry7557558.

use clap::{Parser, Subcommand};
use spirula_backend::{Backend, CpuBackend, VulkanBackend};
use spirula_core::config::TrainConfig;
use spirula_core::Error;
use spirula_data::{find_splat_ply, load_dataset, read_splat_ply, write_splat_ply};
use spirula_engine::{
    gaps_for, seed_from_points, synthetic_scene, train, views_from_dataset, DensifyMode,
    GaussianModel, TrainOptions,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "spirula",
    version,
    about = "Rust port of Spirula Studio (partial). The C++ tree in this repo is the full trainer.",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Train a pinhole subset on the CPU, or project splats with Vulkan.
    Train(TrainArgs),
    /// List Vulkan devices the loader can see.
    Devices,
    /// Not ported. See docs/rust.md. Exits 2.
    #[command(name = "sfm")]
    Sfm,
    /// Not ported. See docs/rust.md. Exits 2.
    #[command(name = "sam")]
    Sam,
    /// Not ported. See docs/rust.md. Exits 2.
    #[command(name = "mesh")]
    Mesh,
    /// Not ported. See docs/rust.md. Exits 2.
    #[command(name = "geometry")]
    Geometry,
    /// Not ported. See docs/rust.md. Exits 2.
    #[command(name = "partition")]
    Partition,
}

#[derive(Parser)]
struct TrainArgs {
    /// Dataset directory (COLMAP or Nerfstudio). Omit with --synthetic.
    data: Option<PathBuf>,
    /// Train on a built-in two-splat scene instead of a dataset.
    #[arg(long)]
    synthetic: bool,
    /// Flat config.json. Unknown keys are listed, not applied.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Steps. 40 with --synthetic, else 30, unless --config sets num_iterations.
    #[arg(long)]
    num_iterations: Option<i32>,
    /// cpu or vulkan. Vulkan projects splats; the composite stays on the CPU.
    #[arg(long, default_value = "cpu")]
    backend: String,
    /// Longest training edge. 0 leaves this cap unset.
    #[arg(long, default_value_t = 64)]
    max_edge: u32,
    /// Seed splats kept from the point cloud.
    #[arg(long, default_value_t = 1024)]
    max_splats: usize,
    /// off, split-prune, or mcmc (mcmc exits with an error).
    #[arg(long, default_value = "off")]
    densify: String,
    /// Run directory. Default outputs/rust-train.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Serve a one-route status page on the config's viewer port.
    #[arg(long)]
    viewer: bool,
    /// Exit before training if the config enables a feature this step skips.
    #[arg(long)]
    require_ported: bool,
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        None => {
            println!("{}", short_help());
            0
        }
        Some(Command::Train(args)) => match run_train(args) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("error: {}", e.message);
                1
            }
        },
        Some(Command::Devices) => match spirula_backend::list_devices() {
            Ok(devices) => {
                if devices.is_empty() {
                    println!("no Vulkan devices");
                }
                for d in devices {
                    println!("{}\t{}", d.kind, d.name);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {}", e.message);
                1
            }
        },
        Some(other) => {
            let name = match other {
                Command::Sfm => "sfm",
                Command::Sam => "sam",
                Command::Mesh => "mesh",
                Command::Geometry => "geometry",
                Command::Partition => "partition",
                Command::Train(_) | Command::Devices => unreachable!(),
            };
            eprintln!("{name} is not in the Rust port. See docs/rust.md.");
            eprintln!("The C++ spirula binary still implements it.");
            2
        }
    };
    std::process::exit(code);
}

fn short_help() -> &'static str {
    "\
spirula (Rust port, partial)

  spirula train --help
  spirula train --synthetic
  spirula train <dataset>
  spirula devices

Build: cargo build --release
Tests: cargo test --workspace
Full trainer: the C++ spirula binary. See docs/rust.md.
"
}

fn run_train(args: TrainArgs) -> Result<(), Error> {
    let densify = DensifyMode::parse(&args.densify)?;
    if densify == DensifyMode::Mcmc {
        return Err(Error::new(
            "MCMC densification is not ported; it remains in the C++ trainer",
        ));
    }
    if args.synthetic && args.data.is_some() {
        return Err(Error::new("pass either a dataset or --synthetic"));
    }
    let mut cfg = TrainConfig::default();
    let mut unknown = Vec::new();
    if let Some(path) = &args.config {
        let text = std::fs::read_to_string(path)
            .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
        let (loaded, extra) = TrainConfig::from_json_str(&text)?;
        cfg = loaded;
        unknown = extra;
    }
    if args.synthetic {
        cfg.use_bilateral_grid = false;
        cfg.use_ppisp = false;
        cfg.ssim_lambda = 0.0;
        cfg.sh_degree = 0;
        cfg.sb_lobes = 0;
        cfg.primitive = "3dgs".into();
        cfg.means_lr = 0.05;
        cfg.means_lr_final = 0.01;
        cfg.features_dc_lr = 0.05;
        cfg.opacities_lr = 0.05;
        cfg.load_masks = false;
        cfg.apply_loss_for_mask = None;
        cfg.depth_supervision_weight = 0.0;
        cfg.normal_supervision_weight = 0.0;
        cfg.orientation_method = "none".into();
    }
    let iterations = args
        .num_iterations
        .or(if args.config.is_some() { Some(cfg.num_iterations) } else { None })
        .unwrap_or(if args.synthetic { 40 } else { 30 });
    if iterations <= 0 {
        return Err(Error::new("num_iterations must be positive"));
    }
    if args.require_ported && !unknown.is_empty() {
        return Err(Error::new(format!(
            "refusing to train because --require-ported is set and the config has unknown keys: {}",
            unknown.join(", ")
        )));
    }
    let gaps = gaps_for(&cfg);
    let blocking: Vec<_> = gaps.iter().filter(|g| g.affects_result).cloned().collect();
    if args.require_ported && !blocking.is_empty() {
        for g in &blocking {
            eprintln!("not ported: {}", g.detail);
        }
        return Err(Error::new(
            "refusing to train because --require-ported is set and the config enables an unported feature",
        ));
    }

    let (model, views, mut notes) = if args.synthetic {
        let (model, views, _) = synthetic_scene();
        (model, views, vec!["synthetic scene: two splats, one pinhole".into()])
    } else {
        let data = args.data.as_ref().ok_or_else(|| {
            Error::new("spirula train needs a dataset directory or --synthetic")
        })?;
        cfg.data = data.display().to_string();
        prepare_dataset(&cfg, data, args.max_edge, args.max_splats)?
    };
    notes.extend(unknown.iter().map(|k| format!("ignored config key {k}")));
    for g in &gaps {
        notes.push(format!("{}: {}", g.name, g.detail));
    }

    let run_name = if cfg.output_dir_name.is_empty() {
        "rust-train"
    } else {
        cfg.output_dir_name.as_str()
    };
    let out_dir = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(&cfg.output_dir_prefix).join(run_name));
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| Error::new(format!("creating {}: {e}", out_dir.display())))?;

    let status = std::sync::Arc::new(std::sync::Mutex::new(String::from("{\"step\":-1}")));
    let viewer = if args.viewer {
        Some(spawn_status(cfg.viewer_port, status.clone())?)
    } else {
        None
    };

    let opts = TrainOptions::from_config(&cfg, iterations, densify);
    println!(
        "backend {} | {} splats | {} views | {} steps | densify {}",
        args.backend,
        model.len(),
        views.len(),
        iterations,
        args.densify
    );
    for note in &notes {
        println!("note: {note}");
    }
    let mut backend = open_backend(&args.backend)?;
    let trained = train(model, &views, &opts, backend.as_mut(), gaps, notes)?;
    if let Some(last) = trained.steps.last() {
        println!(
            "step {} loss {:.6} l1 {:.6} psnr {} splats {}",
            last.step,
            last.loss,
            last.l1,
            fmt_psnr(last.psnr),
            last.splats
        );
    }
    let ply_path = out_dir.join("splat.ply");
    write_splat_ply(&trained.model.to_cloud(), &ply_path)?;
    let image_linear = cfg.image_color_is_linear.unwrap_or(false);
    write_ppm(
        &out_dir.join("preview.ppm"),
        trained.preview_width,
        trained.preview_height,
        &trained.preview,
        image_linear,
    )?;
    let config_body = format!(
        "{{\n  \"image_color_is_linear\": {image_linear},\n  \"splat_color_is_linear\": {image_linear}\n}}\n"
    );
    std::fs::write(out_dir.join("config.json"), config_body)
        .map_err(|e| Error::new(format!("writing config.json: {e}")))?;
    let report = serde_json::json!({
        "backend": args.backend,
        "steps": trained.steps.iter().map(|s| serde_json::json!({
            "step": s.step, "loss": s.loss, "l1": s.l1, "psnr": s.psnr, "splats": s.splats, "view": s.view
        })).collect::<Vec<_>>(),
        "notes": trained.notes,
        "gaps": trained.gaps.iter().map(|g| serde_json::json!({
            "name": g.name, "detail": g.detail, "affects_result": g.affects_result
        })).collect::<Vec<_>>(),
        "preview": "preview.ppm is the last render. config.json records whether those values are linear",
    });
    std::fs::write(out_dir.join("rust-port.json"), serde_json::to_string_pretty(&report).unwrap())
        .map_err(|e| Error::new(format!("writing report: {e}")))?;
    println!("wrote {}", ply_path.display());
    if let Ok(mut slot) = status.lock() {
        *slot = serde_json::to_string(&report).unwrap_or_else(|_| "{}".into());
    }
    if let Some(flag) = viewer {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(())
}

fn prepare_dataset(
    cfg: &TrainConfig,
    data: &PathBuf,
    max_edge: u32,
    max_splats: usize,
) -> Result<(GaussianModel, Vec<spirula_engine::View>, Vec<String>), Error> {
    let mut notes = Vec::new();
    let model = if !cfg.init_ply.is_empty() {
        let (ply, _) = find_splat_ply(PathBuf::from(&cfg.init_ply).as_path())?;
        cloud_to_model(&read_splat_ply(&ply)?, max_splats, &mut notes)
    } else {
        let ds = load_dataset(data, cfg)?;
        notes.extend(ds.notes.clone());
        let (views, view_notes) = views_from_dataset(
            &ds,
            merged_max_edge(max_edge, cfg.max_resolution),
            cfg.image_color_is_linear.unwrap_or(false),
        )?;
        notes.extend(view_notes);
        let model = match cfg.random_init.as_str() {
            "never" => {
                if ds.points.xyz.is_empty() {
                    return Err(Error::new(
                        "random_init is never and the dataset has no points",
                    ));
                }
                seed_from_cloud(&ds, cfg, max_splats, &mut notes)
            }
            "always" => {
                notes.push("random_init always replaced the seed cloud".into());
                seed_random(&views, random_count(max_splats))
            }
            "auto" => {
                if ds.points.xyz.is_empty() {
                    notes.push(
                        "dataset has no points; seeding splats inside the camera bbox".into(),
                    );
                    seed_random(&views, random_count(max_splats))
                } else {
                    seed_from_cloud(&ds, cfg, max_splats, &mut notes)
                }
            }
            other => {
                return Err(Error::new(format!(
                    "unknown random_init '{other}' (never, auto, always)"
                )))
            }
        };
        return Ok((model, views, notes));
    };
    let ds = load_dataset(data, cfg)?;
    let (views, view_notes) = views_from_dataset(
        &ds,
        merged_max_edge(max_edge, cfg.max_resolution),
        cfg.image_color_is_linear.unwrap_or(false),
    )?;
    notes.extend(ds.notes);
    notes.extend(view_notes);
    Ok((model, views, notes))
}

// 0 means that cap is unset. `resize_max_edge` leaves the image alone when handed 0.
fn merged_max_edge(cli: u32, config: i32) -> u32 {
    let config = if config > 0 { config as u32 } else { 0 };
    match (cli, config) {
        (0, 0) => 0,
        (0, limit) | (limit, 0) => limit,
        (cli, config) => cli.min(config),
    }
}

fn cloud_to_model(cloud: &spirula_data::SplatCloud, cap: usize, notes: &mut Vec<String>) -> GaussianModel {
    let n = cloud.len().min(cap);
    if cloud.len() > n {
        notes.push(format!("init ply truncated from {} splats to {n}", cloud.len()));
    }
    let stride = cloud.sh_degree.max(0) as usize * (cloud.sh_degree as usize + 2) * 3
        + cloud.sb_lobes as usize * 6;
    let features_sh = if stride > 0 && cloud.features_sh.len() >= n * stride {
        cloud.features_sh[..n * stride].to_vec()
    } else {
        Vec::new()
    };
    GaussianModel {
        means: cloud.means[..n].to_vec(),
        quats: cloud.quats[..n].to_vec(),
        log_scales: cloud.scales[..n].to_vec(),
        opacity_logit: cloud.opacities[..n].to_vec(),
        dc: cloud.features_dc[..n].to_vec(),
        sh_degree: cloud.sh_degree,
        sb_lobes: cloud.sb_lobes,
        features_sh,
    }
}

fn random_count(max_splats: usize) -> usize {
    32.min(max_splats).max(1)
}

fn seed_from_cloud(
    ds: &spirula_data::Dataset,
    cfg: &TrainConfig,
    max_splats: usize,
    notes: &mut Vec<String>,
) -> GaussianModel {
    let n = ds.points.xyz.len().min(max_splats);
    if ds.points.xyz.len() > n {
        notes.push(format!(
            "seed cloud truncated from {} points to {n}",
            ds.points.xyz.len()
        ));
    }
    seed_from_points(&ds.points, cfg.scale_init, cfg.opacity_init, n)
}

fn seed_random(views: &[spirula_engine::View], n: usize) -> GaussianModel {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    if views.is_empty() {
        min = [-1.0; 3];
        max = [1.0; 3];
    }
    for view in views {
        let center = camera_center(&view.w2c);
        for k in 0..3 {
            min[k] = min[k].min(center[k]);
            max[k] = max[k].max(center[k]);
        }
    }
    for k in 0..3 {
        if max[k] - min[k] < 1e-3 {
            min[k] -= 0.5;
            max[k] += 0.5;
        }
    }
    let mut state = 0x1234_5678_u32;
    let mut rnd = || {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        (state >> 8) as f32 / 16_777_216.0
    };
    let mut means = Vec::with_capacity(n);
    for _ in 0..n {
        means.push([
            min[0] + (max[0] - min[0]) * rnd(),
            min[1] + (max[1] - min[1]) * rnd(),
            min[2] + (max[2] - min[2]) * rnd(),
        ]);
    }
    let points = spirula_data::PointCloud {
        xyz: means.iter().map(|m| [m[0] as f64, m[1] as f64, m[2] as f64]).collect(),
        rgb: vec![[180, 180, 180]; n],
    };
    seed_from_points(&points, 0.2, 0.1, n)
}

fn camera_center(w2c: &[f32; 12]) -> [f32; 3] {
    let t = [w2c[3], w2c[7], w2c[11]];
    [
        -(w2c[0] * t[0] + w2c[4] * t[1] + w2c[8] * t[2]),
        -(w2c[1] * t[0] + w2c[5] * t[1] + w2c[9] * t[2]),
        -(w2c[2] * t[0] + w2c[6] * t[1] + w2c[10] * t[2]),
    ]
}

fn open_backend(name: &str) -> Result<Box<dyn Backend>, Error> {
    match name {
        "cpu" => Ok(Box::new(CpuBackend)),
        "vulkan" => Ok(Box::new(VulkanBackend::open()?)),
        other => Err(Error::new(format!(
            "unknown backend '{other}' (cpu or vulkan). CUDA is not in this port"
        ))),
    }
}

fn fmt_psnr(v: f32) -> String {
    if v.is_infinite() {
        "inf".into()
    } else {
        format!("{v:.3}")
    }
}

fn write_ppm(path: &std::path::Path, w: u32, h: u32, rgb: &[f32], encode_srgb: bool) -> Result<(), Error> {
    if w == 0 || h == 0 || rgb.len() < (w * h * 3) as usize {
        return Ok(());
    }
    let mut bytes = format!("P6\n{w} {h}\n255\n").into_bytes();
    for c in rgb.iter().take((w * h * 3) as usize) {
        let v = (*c).clamp(0.0, 1.0);
        let s = if encode_srgb { linear_to_srgb(v) } else { v };
        bytes.push((s * 255.0).round().clamp(0.0, 255.0) as u8);
    }
    std::fs::write(path, bytes).map_err(|e| Error::new(format!("writing {}: {e}", path.display())))
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn spawn_status(port: i32, body: std::sync::Arc<std::sync::Mutex<String>>) -> Result<std::sync::Arc<std::sync::atomic::AtomicBool>, Error> {
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = stop.clone();
    let addr = format!("127.0.0.1:{port}");
    let listener = std::net::TcpListener::bind(&addr)
        .map_err(|e| Error::new(format!("viewer port {port}: {e}")))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| Error::new(format!("viewer port {port}: {e}")))?;
    println!("status http://{addr}/  (not the training viewer)");
    std::thread::spawn(move || {
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    use std::io::{Read, Write};
                    let mut buf = [0u8; 512];
                    let _ = stream.read(&mut buf);
                    let json = body.lock().map(|s| s.clone()).unwrap_or_else(|_| "{}".into());
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{json}",
                        json.len()
                    );
                    let _ = stream.write_all(resp.as_bytes());
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    });
    Ok(flag)
}

#[cfg(test)]
mod tests {
    use super::merged_max_edge;

    #[test]
    fn zero_edge_cap_stays_unset() {
        assert_eq!(merged_max_edge(64, 2), 2);
        assert_eq!(merged_max_edge(0, 2), 2);
        assert_eq!(merged_max_edge(2, 0), 2);
        assert_eq!(merged_max_edge(0, 0), 0);
        assert_eq!(merged_max_edge(8, 3), 3);
        assert_eq!(merged_max_edge(3, -1), 3);
    }
}
