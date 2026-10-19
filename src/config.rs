pub const CANVAS_WIDTH: u32 = 1920;
pub const CANVAS_HEIGHT: u32 = 1080;
pub const BACKGROUND: &str = "#07111D";
pub const PANEL: &str = "#0D1B2A";
pub const TEXT: &str = "#FFFFFF";
pub const MUTED_TEXT: &str = "#AAB8C5";
pub const ACCENT: &str = "#E8B85A";
pub const MARGIN_X: f32 = 90.0;
pub const GRID_TOP: f32 = 235.0;
pub const GRID_BOTTOM: f32 = 1010.0;
pub const GAP: f32 = 14.0;
pub const MIN_TILE_WIDTH: f32 = 220.0;
pub const MAX_TILE_WIDTH: f32 = 600.0;
pub const BORDER_RADIUS: f32 = 18.0;
pub const BORDER_WIDTH: f32 = 3.0;

pub fn faction_color(faction: &str) -> &'static str {
    match faction {
        "Ionia" => "#39e911",
        "Noxus" => "#C84B55",
        "Piltover" => "#D8A84E",
        "Demacia" => "#8BAFD1",
        "Shadow Isles" => "#657A91",
        "Shurima" => "#D18A45",
        "Freljord" => "#77AFC6",
        "Mount Targon" => "#0f3a65",
        "Zaun" => "#45371d",
        "Demon" => "#211253",
        "Bilgewater" => "#dc0aef",
        "The Void" => "#8C79C8",
        "Iztal" => "#ef2610",
        "Bandle City" => "#0ff2eb",
        _ => ACCENT,
    }
}
