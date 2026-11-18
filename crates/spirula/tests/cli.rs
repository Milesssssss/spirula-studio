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
