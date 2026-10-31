use super::*;

#[test]
fn known_factions_have_configured_colors() {
    assert_eq!(faction_color("Ionia"), "#39e911");
    assert_eq!(faction_color("Noxus"), "#C84B55");
    assert_eq!(faction_color("Bandle City"), "#0ff2eb");
    assert_eq!(faction_color("The Void"), "#8C79C8");
}

#[test]
fn unknown_factions_use_the_accent_color() {
    assert_eq!(faction_color("Unknown"), ACCENT);
}
