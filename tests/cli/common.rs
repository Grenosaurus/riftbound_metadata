use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

pub struct TestDir(PathBuf);

impl TestDir {
    pub fn new() -> Self {
        let id = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("riftbound_meta_test_{}_{}", std::process::id(), id));
        fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn write_inputs(dir: &Path, tournament: &str, legends: &str) -> (PathBuf, PathBuf) {
    let tournament_path = dir.join("tournament.json");
    let legends_path = dir.join("legends.json");
    fs::write(&tournament_path, tournament).expect("write tournament fixture");
    fs::write(&legends_path, legends).expect("write legend fixture");
    (tournament_path, legends_path)
}

pub fn run_cli(
    tournament: &Path,
    legends: &Path,
    output: &Path,
    svg: &Path,
    assets: &Path,
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_riftbound_meta"))
        .arg(tournament)
        .arg("--legends")
        .arg(legends)
        .arg("--output")
        .arg(output)
        .arg("--svg")
        .arg(svg)
        .arg("--assets")
        .arg(assets)
        .arg("--skip-artwork")
        .output()
        .expect("run riftbound_meta")
}

pub fn run_cli_with_derived_svg(
    tournament: &Path,
    legends: &Path,
    output: &Path,
    assets: &Path,
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_riftbound_meta"))
        .arg(tournament)
        .arg("--legends")
        .arg(legends)
        .arg("--output")
        .arg(output)
        .arg("--assets")
        .arg(assets)
        .arg("--skip-artwork")
        .output()
        .expect("run riftbound_meta")
}
