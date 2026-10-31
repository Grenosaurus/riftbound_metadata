use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let id = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("riftbound_main_test_{}_{}", std::process::id(), id));
        fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn legend(name: &str) -> Legend {
    Legend {
        name: name.to_owned(),
        title: "Test title".to_owned(),
        image: format!("{name}.png"),
        faction: "Ionia".to_owned(),
    }
}

#[test]
fn prepare_legends_excludes_zero_and_unknown_keys_and_sorts_deterministically() {
    let tournament = TournamentFile {
        tournament: crate::models::Tournament {
            name: "Test".to_owned(),
            date: None,
            players: 0,
        },
        legends: HashMap::from([
            ("zulu".to_owned(), 3),
            ("zero".to_owned(), 0),
            ("beta".to_owned(), 5),
            ("alpha".to_owned(), 5),
            ("unknown".to_owned(), 9),
        ]),
    };
    let legend_db = HashMap::from([
        ("zulu".to_owned(), legend("Zulu")),
        ("beta".to_owned(), legend("Beta")),
        ("alpha".to_owned(), legend("Alpha")),
        ("zero".to_owned(), legend("Zero")),
    ]);

    let prepared = prepare_legends(&tournament, &legend_db);

    assert_eq!(
        prepared
            .iter()
            .map(|entry| (entry.key.as_str(), entry.count))
            .collect::<Vec<_>>(),
        [("alpha", 5), ("beta", 5), ("zulu", 3)]
    );
}

#[test]
fn ensure_parent_dir_creates_nested_parent() {
    let dir = TestDir::new();
    let output_path = dir.0.join("nested").join("result.png");

    ensure_parent_dir(&output_path).unwrap();

    assert!(dir.0.join("nested").is_dir());
}

#[test]
fn ensure_parent_dir_accepts_a_filename_without_a_parent() {
    ensure_parent_dir(Path::new("result.png")).unwrap();
}

#[test]
fn render_png_rejects_invalid_svg() {
    let dir = TestDir::new();
    let error = render_png("<not-svg", &dir.0.join("output.png")).unwrap_err();

    assert!(error.to_string().contains("Could not parse generated SVG"));
}
