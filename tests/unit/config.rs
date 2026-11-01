use super::*;

#[test]
fn known_factions_have_configured_colors() {
    assert_eq!(faction_color("Ionia"), "#E88EBB");
    assert_eq!(faction_color("Noxus"), "#D64045");
    assert_eq!(faction_color("Bandle City"), "#B78BFA");
    assert_eq!(faction_color("Void"), "#B45BDF");
}

#[test]
fn unknown_factions_use_the_accent_color() {
    assert_eq!(faction_color("Unknown"), ACCENT);
}
