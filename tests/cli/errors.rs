use crate::common::{assets_path, output_path, run_cli, write_inputs, TestDir};
use std::fs;

#[test]
fn invalid_tournament_json_returns_an_error_without_outputs() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(dir.path(), "{ invalid", "{}");
    let png_path = output_path(dir.path(), "meta.png");
    let svg_path = output_path(dir.path(), "meta.svg");

    let result = run_cli(
        &tournament,
        &legends,
        &png_path,
        &svg_path,
        &assets_path(dir.path()),
    );

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Could not parse JSON"));
    assert!(!png_path.exists());
    assert!(!svg_path.exists());
}

#[test]
fn malformed_legend_json_returns_an_error_without_outputs() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(
        dir.path(),
        r#"{"tournament":{"name":"Event"},"legends":{}}"#,
        "{ invalid",
    );
    let png_path = output_path(dir.path(), "meta.png");
    let svg_path = output_path(dir.path(), "meta.svg");

    let result = run_cli(
        &tournament,
        &legends,
        &png_path,
        &svg_path,
        &assets_path(dir.path()),
    );

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Could not parse JSON"));
    assert!(!png_path.exists());
    assert!(!svg_path.exists());
}

#[test]
fn unreadable_artwork_returns_an_error_instead_of_silently_rendering() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(
        dir.path(),
        r#"{
            "tournament": {"name": "Bad artwork"},
            "legends": {"ahri": 1}
        }"#,
        r#"{
            "ahri": {
                "name":"Ahri",
                "title":"The Fox",
                "image":"ahri.png",
                "faction":"Ionia"
            }
        }"#,
    );
    let assets = assets_path(dir.path());
    fs::create_dir_all(&assets).expect("create assets");
    fs::write(assets.join("ahri.png"), b"invalid image").expect("write invalid artwork");
    let png_path = output_path(dir.path(), "meta.png");
    let svg_path = output_path(dir.path(), "meta.svg");
    let result = run_cli(&tournament, &legends, &png_path, &svg_path, &assets);

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Could not decode artwork"));
    assert!(!png_path.exists());
    assert!(!svg_path.exists());
}

#[test]
fn missing_input_file_reports_which_file_could_not_be_read() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(dir.path(), "{}", "{}");
    let png_path = output_path(dir.path(), "meta.png");
    let svg_path = output_path(dir.path(), "meta.svg");
    let result = run_cli(
        &tournament.with_file_name("missing-tournament.json"),
        &legends,
        &png_path,
        &svg_path,
        &assets_path(dir.path()),
    );

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("Could not read"));
    assert!(stderr.contains("missing-tournament.json"));
}

#[test]
fn png_write_failure_is_reported_after_svg_is_written() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(
        dir.path(),
        r#"{"tournament":{"name":"Write failure"},"legends":{}}"#,
        "{}",
    );
    let output_dir = output_path(dir.path(), "output-directory.png");
    fs::create_dir_all(&output_dir).expect("create output path as a directory");
    let svg_path = output_path(dir.path(), "result.svg");
    let result = run_cli(
        &tournament,
        &legends,
        &output_dir,
        &svg_path,
        &assets_path(dir.path()),
    );

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Could not write"));
    assert!(svg_path.is_file());
}
