use super::*;
use crate::layout::Tile;
use crate::test_support::TestDir;
use std::fs;

fn legend(name: &str, title: &str, image: &str, faction: &str) -> Legend {
    Legend {
        name: name.to_owned(),
        title: title.to_owned(),
        image: image.to_owned(),
        faction: faction.to_owned(),
    }
}

fn tile(key: &str, count: u32) -> Tile {
    Tile {
        key: key.to_owned(),
        count,
        x: 100.0,
        y: 250.0,
        width: 300.0,
        height: 300.0,
    }
}

fn tournament(name: &str, date: Option<&str>, players: u32) -> Tournament {
    Tournament {
        name: name.to_owned(),
        date: date.map(str::to_owned),
        players,
    }
}

#[test]
fn svg_contains_canvas_and_escaped_tournament_metadata() {
    let svg = render_svg(
        &tournament("Cup & <Final>", Some("  2026-10-31  "), 1),
        &HashMap::new(),
        &[],
        Path::new("unused"),
    )
    .unwrap();

    assert!(svg.starts_with("<svg "));
    assert!(svg.contains(r#"width="1920" height="1080" viewBox="0 0 1920 1080""#));
    assert!(svg.contains("Cup &amp; &lt;Final&gt; (1 player)"));
    assert!(svg.contains(">2026-10-31</text>"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn svg_omits_blank_dates_and_uses_plural_player_label() {
    let svg = render_svg(
        &tournament("Event", Some("  "), 0),
        &HashMap::new(),
        &[],
        Path::new("unused"),
    )
    .unwrap();

    assert!(svg.contains("Event (0 players)"));
    assert!(!svg.contains("text-anchor=\"end\""));
}

#[test]
fn svg_escapes_legend_fields_and_uses_faction_color() {
    let legends = HashMap::from([(
        "ahri".to_owned(),
        legend("Ahri & Friends", "The <Fox>", "missing.png", "Noxus"),
    )]);
    let svg = render_svg(
        &tournament("Event", None, 2),
        &legends,
        &[tile("ahri", 7)],
        Path::new("unused"),
    )
    .unwrap();

    assert!(svg.contains("AHRI &amp; FRIENDS"));
    assert!(svg.contains("The &lt;Fox&gt;"));
    assert!(svg.contains("stroke=\"#C84B55\""));
    assert!(svg.contains(">7</text>"));
    assert!(!svg.contains("data:image/png;base64,"));
}

#[test]
fn tiles_without_legend_metadata_are_skipped() {
    let svg = render_svg(
        &tournament("Event", None, 2),
        &HashMap::new(),
        &[tile("missing", 99)],
        Path::new("unused"),
    )
    .unwrap();

    assert!(!svg.contains(">99</text>"));
}

#[test]
fn artwork_is_cropped_encoded_and_cached_for_shared_paths() {
    let dir = TestDir::new();
    let artwork_path = dir.0.join("art.png");
    image::RgbaImage::from_pixel(100, 100, image::Rgba([1, 2, 3, 255]))
        .save(&artwork_path)
        .unwrap();
    let first = image_href(&artwork_path).unwrap().unwrap();
    let second = image_href(&artwork_path).unwrap().unwrap();
    assert_eq!(first, second);

    let encoded = first.strip_prefix("data:image/png;base64,").unwrap();
    let bytes = STANDARD.decode(encoded).unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap();
    assert_eq!(decoded.dimensions(), (88, 63));

    let shared_image_legends = HashMap::from([
        (
            "first".to_owned(),
            legend("First", "One", "art.png", "Ionia"),
        ),
        (
            "second".to_owned(),
            legend("Second", "Two", "art.png", "Noxus"),
        ),
    ]);
    let svg = render_svg(
        &tournament("Event", None, 2),
        &shared_image_legends,
        &[tile("first", 1), tile("second", 1)],
        &dir.0,
    )
    .unwrap();
    assert_eq!(svg.matches("data:image/png;base64,").count(), 2);
}

#[test]
fn absent_artwork_is_ignored_but_corrupt_artwork_returns_an_error() {
    let dir = TestDir::new();
    assert!(image_href(&dir.0.join("missing.png")).unwrap().is_none());

    let corrupt_path = dir.0.join("corrupt.png");
    fs::write(&corrupt_path, b"not an image").unwrap();
    let error = image_href(&corrupt_path).unwrap_err();
    assert!(error.to_string().contains("Could not decode artwork"));
}
