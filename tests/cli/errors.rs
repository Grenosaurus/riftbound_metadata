use crate::common::{run_cli, write_inputs, TestDir};
use std::fs;

#[test]
fn invalid_tournament_json_returns_an_error_without_outputs() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(dir.path(), "{ invalid", "{}");
    let output_path = dir.path().join("meta.png");
    let svg_path = dir.path().join("meta.svg");

    let result = run_cli(
        &tournament,
        &legends,
        &output_path,
        &svg_path,
        &dir.path().join("assets"),
    );

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Could not parse JSON"));
    assert!(!output_path.exists());
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
    let output_path = dir.path().join("meta.png");
    let svg_path = dir.path().join("meta.svg");

    let result = run_cli(
        &tournament,
        &legends,
        &output_path,
        &svg_path,
        &dir.path().join("assets"),
    );

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Could not parse JSON"));
    assert!(!output_path.exists());
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
    let assets = dir.path().join("assets");
    fs::create_dir_all(&assets).expect("create assets");
    fs::write(assets.join("ahri.png"), b"invalid image").expect("write invalid artwork");
    let output_path = dir.path().join("meta.png");
    let svg_path = dir.path().join("meta.svg");
    let result = run_cli(&tournament, &legends, &output_path, &svg_path, &assets);

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Could not decode artwork"));
    assert!(!output_path.exists());
    assert!(!svg_path.exists());
}

#[test]
fn missing_input_file_reports_which_file_could_not_be_read() {
    let dir = TestDir::new();
    let output_path = dir.path().join("meta.png");
    let svg_path = dir.path().join("meta.svg");
    let result = run_cli(
        &dir.path().join("missing-tournament.json"),
        &dir.path().join("missing-legends.json"),
        &output_path,
        &svg_path,
        &dir.path().join("assets"),
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
    let output_path = dir.path().join("output-directory");
    fs::create_dir_all(&output_path).expect("create output path as a directory");
    let svg_path = dir.path().join("result.svg");
    let result = run_cli(
        &tournament,
        &legends,
        &output_path,
        &svg_path,
        &dir.path().join("assets"),
    );

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Could not write"));
    assert!(svg_path.is_file());
}
