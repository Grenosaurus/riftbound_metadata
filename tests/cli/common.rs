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
    write_inputs_named(dir, "tournament.json", tournament, legends)
}

pub fn write_inputs_named(
    dir: &Path,
    tournament_filename: &str,
    tournament: &str,
    legends: &str,
) -> (PathBuf, PathBuf) {
    let project_dir = dir.join("riftbound_metadata");
    let tournament_dir = dir.join("DataFiles").join("RiftboundTournaments");
    let legends_dir = project_dir.join("data");
    fs::create_dir_all(&tournament_dir).expect("create tournament fixture directory");
    fs::create_dir_all(&legends_dir).expect("create project data directory");
    let tournament_path = tournament_dir.join(tournament_filename);
    let legends_path = legends_dir.join("legends.json");
    fs::write(&tournament_path, tournament).expect("write tournament fixture");
    fs::write(&legends_path, legends).expect("write legend fixture");
    (tournament_path, legends_path)
}

pub fn run_cli_with_names(project_dir: &Path, tournament: &str, output: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_riftbound_meta"))
        .current_dir(project_dir)
        .arg(tournament)
        .arg("--output")
        .arg(output)
        .arg("--skip-artwork")
        .output()
        .expect("run riftbound_meta")
}

pub fn run_cli_with_filenames(
    project_dir: &Path,
    tournament: &str,
    output: &str,
    svg: &str,
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_riftbound_meta"))
        .current_dir(project_dir)
        .arg(tournament)
        .arg("--output")
        .arg(output)
        .arg("--svg")
        .arg(svg)
        .arg("--skip-artwork")
        .output()
        .expect("run riftbound_meta")
}

pub fn output_path(dir: &Path, filename: &str) -> PathBuf {
    dir.join("DataFiles")
        .join("RiftboundTournaments")
        .join("TournamentOutput")
        .join(filename)
}

pub fn assets_path(dir: &Path) -> PathBuf {
    dir.join("DataFiles")
        .join("RiftboundAssets")
        .join("legends")
}

pub fn run_cli(
    tournament: &Path,
    legends: &Path,
    output: &Path,
    svg: &Path,
    assets: &Path,
) -> Output {
    let project_dir = legends
        .parent()
        .and_then(Path::parent)
        .expect("fixture project directory");
    prepare_assets(project_dir, assets);
    Command::new(env!("CARGO_BIN_EXE_riftbound_meta"))
        .current_dir(project_dir)
        .arg(tournament.file_name().expect("tournament filename"))
        .arg("--output")
        .arg(output.file_name().expect("PNG filename"))
        .arg("--svg")
        .arg(svg.file_name().expect("SVG filename"))
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
    let project_dir = legends
        .parent()
        .and_then(Path::parent)
        .expect("fixture project directory");
    prepare_assets(project_dir, assets);
    Command::new(env!("CARGO_BIN_EXE_riftbound_meta"))
        .current_dir(project_dir)
        .arg(tournament.file_name().expect("tournament filename"))
        .arg("--output")
        .arg(output.file_name().expect("PNG filename"))
        .arg("--skip-artwork")
        .output()
        .expect("run riftbound_meta")
}

fn prepare_assets(project_dir: &Path, source: &Path) {
    let root = project_dir.parent().expect("fixture root directory");
    let target = root
        .join("DataFiles")
        .join("RiftboundAssets")
        .join("legends");
    fs::create_dir_all(&target).expect("create default artwork cache");

    if source.is_dir() && source != target {
        for entry in fs::read_dir(source).expect("read artwork fixture directory") {
            let entry = entry.expect("read artwork fixture");
            if entry
                .file_type()
                .expect("inspect artwork fixture")
                .is_file()
            {
                fs::copy(entry.path(), target.join(entry.file_name()))
                    .expect("copy artwork fixture");
            }
        }
    }
}
