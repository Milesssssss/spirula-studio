use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spirula"))
}

#[test]
fn help_lists_train_and_names_the_cpp_trainer() {
    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("train"));
    assert!(text.contains("partial") || text.contains("C++"));
}

#[test]
fn sfm_is_refused() {
    let out = bin().arg("sfm").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("not in the Rust port"));
}

#[test]
fn synthetic_train_writes_a_ply() {
    let dir = std::env::temp_dir().join(format!("spirula-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let out = bin()
        .args([
            "train",
            "--synthetic",
            "--num-iterations",
            "3",
            "--backend",
            "cpu",
            "--output",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stdout {}\nstderr {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(dir.join("splat.ply").is_file());
    assert!(dir.join("config.json").is_file());
    let cfg = std::fs::read_to_string(dir.join("config.json")).unwrap();
    assert!(cfg.contains("\"splat_color_is_linear\": false"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mcmc_exits_before_writing_a_run() {
    let dir = std::env::temp_dir().join(format!("spirula-mcmc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let out = bin()
        .args([
            "train",
            "--synthetic",
            "--densify",
            "mcmc",
            "--num-iterations",
            "2",
            "--output",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("MCMC"));
    assert!(!dir.join("splat.ply").exists());
}

#[test]
fn require_ported_rejects_an_unknown_key() {
    let dir = std::env::temp_dir().join(format!("spirula-strict-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = dir.join("cfg.json");
    std::fs::write(&cfg, b"{\"not_a_real_flag\": 1}\n").unwrap();
    let out_dir = dir.join("out");
    let out = bin()
        .args([
            "train",
            "--synthetic",
            "--require-ported",
            "--num-iterations",
            "1",
            "--config",
        ])
        .arg(&cfg)
        .arg("--output")
        .arg(&out_dir)
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
    assert!(!out_dir.join("splat.ply").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn require_ported_rejects_random_background_and_resume() {
    for (name, body) in [
        ("background_mode", r#"{"background_mode":"random"}"#),
        ("resume", r#"{"resume":"missing-ckpt"}"#),
    ] {
        let dir = std::env::temp_dir().join(format!("spirula-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = dir.join("cfg.json");
        std::fs::write(&cfg, body).unwrap();
        let out_dir = dir.join("out");
        let out = bin()
            .args([
                "train",
                "--synthetic",
                "--require-ported",
                "--num-iterations",
                "1",
                "--config",
            ])
            .arg(&cfg)
            .arg("--output")
            .arg(&out_dir)
            .output()
            .unwrap();
        assert_ne!(out.status.code(), Some(0), "{name}");
        assert!(!out_dir.join("splat.ply").exists(), "{name}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn cap_max_applies_before_the_first_logged_step() {
    let dir = std::env::temp_dir().join(format!("spirula-cap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    const PNG: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 2,
        0, 0, 0, 144, 119, 83, 222, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 0, 0,
        3, 1, 1, 0, 201, 254, 146, 239, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];
    std::fs::write(dir.join("a.png"), PNG).unwrap();
    std::fs::write(
        dir.join("seed.ply"),
        b"ply\nformat ascii 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n0 0 2 9 9 9\n0.1 0 2 9 9 9\n0 0.1 2 9 9 9\n0.1 0.1 2 9 9 9\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("transforms.json"),
        r#"{"camera_model":"PINHOLE","fl_x":8,"fl_y":8,"cx":0,"cy":0,"w":1,"h":1,
            "frames":[{"file_path":"a.png","transform_matrix":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]}]}"#,
    )
    .unwrap();
    let cfg = dir.join("cfg.json");
    std::fs::write(
        &cfg,
        r#"{"cap_max":2,"seed_pointcloud":"seed.ply","num_iterations":3}"#,
    )
    .unwrap();
    let out_dir = dir.join("out");
    let out = bin()
        .arg("train")
        .arg(&dir)
        .arg("--config")
        .arg(&cfg)
        .arg("--densify")
        .arg("split-prune")
        .arg("--num-iterations")
        .arg("3")
        .arg("--output")
        .arg(&out_dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{} {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out_dir.join("rust-port.json")).unwrap())
            .unwrap();
    let counts: Vec<u64> = report["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["splats"].as_u64().unwrap())
        .collect();
    assert_eq!(counts, vec![2, 2, 2], "{counts:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

fn pinhole_dataset(dir: &std::path::Path, points: &str) {
    const PNG: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 2,
        0, 0, 0, 144, 119, 83, 222, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 0, 0,
        3, 1, 1, 0, 201, 254, 146, 239, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("a.png"), PNG).unwrap();
    if !points.is_empty() {
        std::fs::write(dir.join("seed.ply"), points).unwrap();
    }
    let ply = if points.is_empty() {
        ""
    } else {
        "\"ply_file_path\": \"seed.ply\","
    };
    std::fs::write(
        dir.join("transforms.json"),
        format!(
            r#"{{"camera_model":"PINHOLE","fl_x":8,"fl_y":8,"cx":0,"cy":0,"w":1,"h":1,{ply}
            "frames":[{{"file_path":"a.png","transform_matrix":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]}}]}}"#
        ),
    )
    .unwrap();
}

fn strict_base() -> &'static str {
    r#""use_bilateral_grid":false,"use_ppisp":false,"ssim_lambda":0,"sh_degree":0,"load_masks":false,"normal_supervision_weight":0,"orientation_method":"none","background_mode":"color""#
}

#[test]
fn random_init_never_refuses_an_empty_cloud() {
    let dir = std::env::temp_dir().join(format!("spirula-never-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    pinhole_dataset(&dir, "");
    let cfg = dir.join("cfg.json");
    std::fs::write(
        &cfg,
        format!(r#"{{{base},"random_init":"never"}}"#, base = strict_base()),
    )
    .unwrap();
    let out_dir = dir.join("out");
    let out = bin()
        .arg("train")
        .arg(&dir)
        .arg("--config")
        .arg(&cfg)
        .arg("--require-ported")
        .arg("--num-iterations")
        .arg("1")
        .arg("--output")
        .arg(&out_dir)
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("random_init is never"), "{err}");
    assert!(!out_dir.join("splat.ply").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn random_init_always_replaces_a_seed_point() {
    let dir = std::env::temp_dir().join(format!("spirula-always-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    pinhole_dataset(
        &dir,
        "ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n0 0 2 9 9 9\n",
    );
    let cfg = dir.join("cfg.json");
    std::fs::write(
        &cfg,
        format!(r#"{{{base},"random_init":"always"}}"#, base = strict_base()),
    )
    .unwrap();
    let out_dir = dir.join("out");
    let out = bin()
        .arg("train")
        .arg(&dir)
        .arg("--config")
        .arg(&cfg)
        .arg("--require-ported")
        .arg("--num-iterations")
        .arg("1")
        .arg("--output")
        .arg(&out_dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{} {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("random_init always"));
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out_dir.join("rust-port.json")).unwrap())
            .unwrap();
    assert_eq!(report["steps"][0]["splats"], 32);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn max_edge_zero_keeps_a_positive_max_resolution() {
    // 4x4 RGB PNG. The long edge is what the caps are measured against.
    const PNG: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 4, 0, 0, 0, 4, 8, 2,
        0, 0, 0, 38, 147, 9, 41, 0, 0, 0, 16, 73, 68, 65, 84, 120, 218, 99, 56, 193, 197, 5, 71, 12,
        196, 113, 0, 113, 163, 13, 193, 21, 69, 146, 196, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96,
        130,
    ];
    const POINT: &str = "ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n0 0 2 9 9 9\n";
    const SPLAT: &str = "\
ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property float opacity
property float scale_0
property float scale_1
property float scale_2
property float rot_0
property float rot_1
property float rot_2
property float rot_3
property float f_dc_0
property float f_dc_1
property float f_dc_2
end_header
0 0 2 0 -3 -3 -3 1 0 0 0 0 0 0
";
    let root = std::env::temp_dir().join(format!("spirula-edge-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let cases = [
        ("points", 64u32, 2i32, 2u32),
        ("points", 0, 2, 2),
        ("points", 2, 0, 2),
        ("points", 0, 0, 4),
        ("init-ply", 64, 2, 2),
        ("init-ply", 0, 2, 2),
        ("init-ply", 2, 0, 2),
        ("init-ply", 0, 0, 4),
    ];
    for (kind, cli_edge, config_edge, expect) in cases {
        let label = format!("{kind} max-edge={cli_edge} max_resolution={config_edge}");
        let dir = root.join(format!("{kind}-{cli_edge}-{config_edge}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.png"), PNG).unwrap();
        // A quote and a backslash here are what an unescaped Windows temp path does to JSON.
        let ply = if kind == "init-ply" {
            let ply_dir = if cfg!(windows) {
                dir.clone()
            } else {
                let awkward = dir.join("q\"ote\\slash");
                std::fs::create_dir_all(&awkward).unwrap();
                awkward
            };
            let ply = ply_dir.join("init.ply");
            std::fs::write(&ply, SPLAT).unwrap();
            Some(ply)
        } else {
            std::fs::write(dir.join("seed.ply"), POINT).unwrap();
            None
        };
        let ply_key = if kind == "points" {
            "\"ply_file_path\": \"seed.ply\","
        } else {
            ""
        };
        std::fs::write(
            dir.join("transforms.json"),
            format!(
                r#"{{"camera_model":"PINHOLE","fl_x":8,"fl_y":8,"cx":2,"cy":2,"w":4,"h":4,{ply_key}
                "frames":[{{"file_path":"a.png","transform_matrix":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]}}]}}"#
            ),
        )
        .unwrap();
        let mut cfg_json = serde_json::json!({
            "use_bilateral_grid": false,
            "use_ppisp": false,
            "ssim_lambda": 0,
            "sh_degree": 0,
            "load_masks": false,
            "normal_supervision_weight": 0,
            "orientation_method": "none",
            "background_mode": "color",
            "max_resolution": config_edge,
            "num_iterations": 1
        });
        if let Some(ply) = &ply {
            let path = ply.to_str().expect("ply path is utf-8");
            cfg_json["init_ply"] = serde_json::Value::String(path.to_string());
            if !cfg!(windows) {
                assert!(path.contains('\\') && path.contains('"'), "{label}: {path}");
            }
        }
        let cfg = dir.join("cfg.json");
        let cfg_body = serde_json::to_string(&cfg_json).unwrap();
        if let Some(ply) = &ply {
            let path = ply.to_str().unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&cfg_body).unwrap();
            assert_eq!(parsed["init_ply"], path, "{label}");
            if path.contains('\\') {
                assert!(cfg_body.contains(r"\\"), "{label}: {cfg_body}");
            }
            if path.contains('"') {
                assert!(cfg_body.contains(r#"\""#), "{label}: {cfg_body}");
            }
        }
        std::fs::write(&cfg, cfg_body).unwrap();
        let out_dir = dir.join("out");
        let out = bin()
            .arg("train")
            .arg(&dir)
            .arg("--config")
            .arg(&cfg)
            .arg("--require-ported")
            .arg("--max-edge")
            .arg(cli_edge.to_string())
            .arg("--num-iterations")
            .arg("1")
            .arg("--output")
            .arg(&out_dir)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{label}\n{} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let ppm = std::fs::read(out_dir.join("preview.ppm")).unwrap();
        let header_end = ppm
            .iter()
            .enumerate()
            .filter(|(_, b)| **b == b'\n')
            .nth(1)
            .unwrap()
            .0;
        let header = std::str::from_utf8(&ppm[..=header_end]).unwrap();
        let mut lines = header.lines();
        assert_eq!(lines.next().unwrap(), "P6", "{label}");
        let dims: Vec<u32> = lines
            .next()
            .unwrap()
            .split_whitespace()
            .map(|n| n.parse().unwrap())
            .collect();
        assert_eq!(dims, vec![expect, expect], "{label}");
        let report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out_dir.join("rust-port.json")).unwrap())
                .unwrap();
        assert_eq!(report["steps"][0]["splats"], 1, "{label}");
    }
    let _ = std::fs::remove_dir_all(&root);
}
