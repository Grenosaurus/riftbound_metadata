use std::process::Command;

#[test]
fn help_lists_cli_options_without_needing_input_files() {
    let result = Command::new(env!("CARGO_BIN_EXE_riftbound_meta"))
        .arg("--help")
        .output()
        .expect("run help");

    assert!(result.status.success());
    let help = String::from_utf8_lossy(&result.stdout);
    for option in ["--output", "--svg", "--skip-artwork", "--refresh-artwork"] {
        assert!(help.contains(option), "help is missing {option}");
    }
    assert!(help.contains("tournament.json"));
    assert!(help.contains("meta.png"));
    assert!(!help.contains("--legends"));
    assert!(!help.contains("--assets"));
}

#[test]
fn tournament_and_output_arguments_reject_directory_paths() {
    let dir = crate::common::TestDir::new();
    let project_dir = dir.path().join("riftbound_metadata");
    std::fs::create_dir_all(&project_dir).expect("create project fixture");

    for (tournament, output) in [
        ("nested/tournament.json", "meta.png"),
        ("tournament.json", "nested/meta.png"),
    ] {
        let result = crate::common::run_cli_with_names(&project_dir, tournament, output);
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("filename only"),
            "unexpected error: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    let result = crate::common::run_cli_with_filenames(
        &project_dir,
        "tournament.json",
        "meta.png",
        "nested/meta.svg",
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("filename only"));
}
