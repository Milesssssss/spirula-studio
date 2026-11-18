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
