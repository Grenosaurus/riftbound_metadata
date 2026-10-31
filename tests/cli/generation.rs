use crate::common::{
    assets_path, output_path, run_cli, run_cli_with_derived_svg, write_inputs, write_inputs_named,
    TestDir,
};
use image::GenericImageView;
use std::fs;

#[test]
fn generates_svg_and_png_from_tournament_data() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(
        dir.path(),
        r#"{
            "tournament": {
                "name": "Cup & <Final>",
                "date": "2026-10-31",
                "players": 16
            },
            "legends": {
                "ahri": 4,
                "zero-count": 0,
                "unknown": 9
            }
        }"#,
        r#"{
            "ahri": {
                "name": "Ahri & Friends",
                "title": "The <Fox>",
                "image": "ahri.png",
                "faction": "Ionia"
            },
            "zero-count": {
                "name": "Not Used",
                "title": "Zero",
                "image": "missing.png",
                "faction": "Noxus"
            }
        }"#,
    );
    let assets = assets_path(dir.path());
    fs::create_dir_all(&assets).expect("create assets directory");
    image::RgbaImage::from_pixel(100, 100, image::Rgba([20, 40, 60, 255]))
        .save(assets.join("ahri.png"))
        .expect("write artwork fixture");

    let output_file = output_path(dir.path(), "meta.png");
    let svg_file = output_path(dir.path(), "meta.svg");
    let result = run_cli(&tournament, &legends, &output_file, &svg_file, &assets);

    assert!(
        result.status.success(),
        "CLI failed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let png = fs::read(&output_file).expect("read generated PNG");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    let decoded_png = image::load_from_memory(&png).expect("decode generated PNG");
    assert_eq!(decoded_png.dimensions(), (1920, 1080));

    let svg = fs::read_to_string(&svg_file).expect("read generated SVG");
    assert!(svg.contains("Cup &amp; &lt;Final&gt; (16 players)"));
    assert!(svg.contains("2026-10-31"));
    assert!(svg.contains("AHRI &amp; FRIENDS"));
    assert!(svg.contains("The &lt;Fox&gt;"));
    assert!(svg.contains(">4</text>"));
    assert!(svg.contains("data:image/png;base64,"));
    assert!(!svg.contains("Not Used"));
    assert!(!svg.contains("zero-count"));
    assert!(!svg.contains("unknown"));
    assert!(String::from_utf8_lossy(&result.stderr).contains("missing from legends.json"));
}

#[test]
fn missing_artwork_still_generates_output() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(
        dir.path(),
        r#"{
            "tournament": {"name": "Offline event"},
            "legends": {"ahri": 1}
        }"#,
        r#"{
            "ahri": {
                "name": "Ahri",
                "title": "The Fox",
                "image": "not-cached.png",
                "faction": "Ionia"
            }
        }"#,
    );
    let output_file = output_path(dir.path(), "meta.png");
    let svg_file = output_path(dir.path(), "meta.svg");
    let result = run_cli(
        &tournament,
        &legends,
        &output_file,
        &svg_file,
        &dir.path().join("empty-assets"),
    );

    assert!(
        result.status.success(),
        "CLI failed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let svg = fs::read_to_string(svg_file).expect("read generated SVG");
    assert!(svg.contains("Offline event (0 players)"));
    assert!(svg.contains("AHRI"));
    assert!(!svg.contains("data:image/png;base64,"));
    assert!(output_file.exists());
}

#[test]
fn legends_are_sorted_by_count_then_key() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(
        dir.path(),
        r#"{
            "tournament": {"name": "Ordering", "players": 3},
            "legends": {"zulu": 2, "beta": 5, "alpha": 5}
        }"#,
        r#"{
            "alpha": {"name":"Alpha","title":"A","image":"a.png","faction":"Ionia"},
            "beta": {"name":"Beta","title":"B","image":"b.png","faction":"Noxus"},
            "zulu": {"name":"Zulu","title":"Z","image":"z.png","faction":"Demacia"}
        }"#,
    );
    let output_file = output_path(dir.path(), "meta.png");
    let svg_file = output_path(dir.path(), "meta.svg");
    let result = run_cli(
        &tournament,
        &legends,
        &output_file,
        &svg_file,
        &dir.path().join("assets"),
    );

    assert!(
        result.status.success(),
        "CLI failed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let svg = fs::read_to_string(svg_file).expect("read SVG");
    let alpha = svg.find(">ALPHA</text>").unwrap();
    let beta = svg.find(">BETA</text>").unwrap();
    let zulu = svg.find(">ZULU</text>").unwrap();
    assert!(alpha < beta && beta < zulu);
}

#[test]
fn zero_legends_still_produces_a_valid_empty_breakdown() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(
        dir.path(),
        r#"{
            "tournament": {"name": "No entries"},
            "legends": {"zero": 0}
        }"#,
        r#"{
            "zero": {"name":"Zero","title":"Unused","image":"zero.png","faction":"Ionia"}
        }"#,
    );
    let output_file = output_path(dir.path(), "meta.png");
    let svg_file = output_path(dir.path(), "meta.svg");
    let result = run_cli(
        &tournament,
        &legends,
        &output_file,
        &svg_file,
        &dir.path().join("assets"),
    );

    assert!(result.status.success());
    let svg = fs::read_to_string(svg_file).expect("read SVG");
    assert!(svg.contains("No entries (0 players)"));
    assert!(!svg.contains(">ZERO</text>"));
    assert!(output_file.exists());
}

#[test]
fn svg_path_defaults_to_the_png_path_with_svg_extension() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs(
        dir.path(),
        r#"{"tournament":{"name":"Derived path"},"legends":{}}"#,
        "{}",
    );
    let output_file = output_path(dir.path(), "result.png");
    let svg_file = output_file.with_extension("svg");
    let result = run_cli_with_derived_svg(
        &tournament,
        &legends,
        &output_file,
        &dir.path().join("assets"),
    );

    assert!(
        result.status.success(),
        "CLI failed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output_file.is_file());
    assert!(svg_file.is_file());
}

#[test]
fn custom_tournament_and_output_filenames_stay_in_their_default_folders() {
    let dir = TestDir::new();
    let (tournament, legends) = write_inputs_named(
        dir.path(),
        "weekly-event.json",
        r#"{"tournament":{"name":"Weekly Event"},"legends":{}}"#,
        "{}",
    );
    let output_file = output_path(dir.path(), "weekly-meta.png");
    let svg_file = output_path(dir.path(), "event-breakdown.svg");
    let result = run_cli(
        &tournament,
        &legends,
        &output_file,
        &svg_file,
        &assets_path(dir.path()),
    );

    assert!(
        result.status.success(),
        "CLI failed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output_file.is_file());
    assert!(svg_file.is_file());
    assert!(fs::read_to_string(svg_file)
        .unwrap()
        .contains("Weekly Event (0 players)"));
}
