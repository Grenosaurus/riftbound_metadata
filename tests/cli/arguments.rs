use std::process::Command;

#[test]
fn help_lists_cli_options_without_needing_input_files() {
    let result = Command::new(env!("CARGO_BIN_EXE_riftbound_meta"))
        .arg("--help")
        .output()
        .expect("run help");

    assert!(result.status.success());
    let help = String::from_utf8_lossy(&result.stdout);
    for option in [
        "--legends",
        "--output",
        "--svg",
        "--assets",
        "--skip-artwork",
        "--refresh-artwork",
    ] {
        assert!(help.contains(option), "help is missing {option}");
    }
}
