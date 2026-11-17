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
    assert!(dir.join("rust-port.json").is_file());
    let _ = std::fs::remove_dir_all(&dir);
}
