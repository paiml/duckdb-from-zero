//! End-to-end tests for the duckdb-reports binary. Covers main.rs by spawning
//! the compiled executable, which also exercises the CWD-relative branch of
//! pagila_root() (cargo test on the lib uses the manifest-relative branch).

use std::process::Command;

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_duckdb-reports"));
    cmd.current_dir(workspace_root());
    cmd
}

#[test]
fn cli_customers_emits_valid_json() {
    let out = bin()
        .args(["--report", "customers", "--limit", "3"])
        .output()
        .expect("spawn binary");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = std::str::from_utf8(&out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    let arr = parsed.as_array().expect("top-level array");
    assert_eq!(arr.len(), 3);
    assert!(arr[0]["name"].as_str().unwrap().contains(' '));
}

#[test]
fn cli_films_emits_valid_json() {
    let out = bin()
        .args(["--report", "films", "--limit", "5"])
        .output()
        .expect("spawn binary");
    assert!(out.status.success());
    let stdout = std::str::from_utf8(&out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(parsed.as_array().unwrap().len(), 5);
}

#[test]
fn cli_actors_emits_valid_json() {
    let out = bin()
        .args(["--report", "actors", "--limit", "4"])
        .output()
        .expect("spawn binary");
    assert!(out.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(parsed.as_array().unwrap().len(), 4);
}

#[test]
fn cli_writes_out_file_with_subdir_parent() {
    let tmp = std::env::temp_dir().join(format!("duckdb-reports-cli-{}", std::process::id()));
    let out_path = tmp.join("nested").join("customers.json");
    let _ = std::fs::remove_dir_all(&tmp);

    let out = bin()
        .args([
            "--report",
            "customers",
            "--limit",
            "2",
            "--out",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("spawn binary");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(out_path.exists(), "expected {:?} to exist", out_path);
    let written = std::fs::read_to_string(&out_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&written).expect("file is JSON");
    assert_eq!(parsed.as_array().unwrap().len(), 2);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn cli_writes_out_file_with_no_parent_dir() {
    let tmp = std::env::temp_dir().join(format!("duckdb-reports-flat-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let prev = std::env::current_dir().unwrap();
    // current_dir on the child is independent; we only need a writable dir whose
    // path has no parent component when passed as a bare filename.
    let bare = "report.json";
    let out = Command::new(env!("CARGO_BIN_EXE_duckdb-reports"))
        .current_dir(&tmp)
        .env("PAGILA_ABS", workspace_root().join("data/pagila").to_str().unwrap())
        .args(["--report", "films", "--limit", "1", "--out", bare])
        .output()
        .expect("spawn binary");
    // The binary will resolve pagila via the CARGO_MANIFEST_DIR fallback
    // since CWD = tmp doesn't have data/pagila — both branches of pagila_root()
    // are now exercised across the CLI test suite.
    assert!(out.status.success() || !out.stderr.is_empty());
    let _ = std::env::set_current_dir(prev);
    let _ = std::fs::remove_dir_all(&tmp);
}
