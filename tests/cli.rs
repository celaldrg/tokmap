use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmap"))
}

fn fixture(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("tokmap-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("node_modules/pkg")).unwrap();
    fs::write(
        dir.join("src/main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();
    fs::write(
        dir.join("package-lock.json"),
        "{\"lockfileVersion\": 3}\n".repeat(50),
    )
    .unwrap();
    fs::write(
        dir.join("node_modules/pkg/index.js"),
        "module.exports = 1;\n",
    )
    .unwrap();
    fs::write(dir.join("logo.png"), [0x89u8, b'P', b'N', b'G', 0, 0, 1]).unwrap();
    dir
}

#[test]
fn text_report_lists_groups_and_hints() {
    let dir = fixture("text");
    let out = bin().arg(&dir).arg("--no-git").output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("in 2 files"), "{s}");
    assert!(s.contains("src/"));
    assert!(s.contains("[lockfile]"));
    assert!(s.contains("1 binary files skipped"));
    assert!(!s.contains("node_modules"));
}

#[test]
fn budget_sets_exit_code() {
    let dir = fixture("budget");
    let over = bin()
        .arg(&dir)
        .args(["--no-git", "--budget", "10"])
        .output()
        .unwrap();
    assert_eq!(over.status.code(), Some(1));
    let under = bin()
        .arg(&dir)
        .args(["--no-git", "--budget", "1m"])
        .output()
        .unwrap();
    assert_eq!(under.status.code(), Some(0));
}

#[test]
fn json_output_is_parseable_shape() {
    let dir = fixture("json");
    let out = bin()
        .arg(&dir)
        .args(["--no-git", "--json"])
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.starts_with("{\"root\":"));
    assert!(s.contains("\"hint\":\"lockfile\""));
    assert!(s.contains("\"path\":\"src/main.rs\""));
}

#[test]
fn bad_option_exits_2() {
    let out = bin().arg("--nope").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}
