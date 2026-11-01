pub const CANVAS_WIDTH: u32 = 1920;
pub const CANVAS_HEIGHT: u32 = 1080;
pub const BACKGROUND: &str = "#07111D";
pub const PANEL: &str = "#0D1B2A";
pub const TEXT: &str = "#FFFFFF";
pub const MUTED_TEXT: &str = "#AAB8C5";
pub const ACCENT: &str = "#AAB4C0";
pub const MARGIN_X: f32 = 90.0;
pub const GRID_TOP: f32 = 235.0;
pub const GRID_BOTTOM: f32 = 1010.0;
pub const GAP: f32 = 14.0;
pub const BORDER_RADIUS: f32 = 18.0;
pub const BORDER_WIDTH: f32 = 3.0;

pub fn faction_color(faction: &str) -> &'static str {
    match faction {
        "Bandle City" => "#B78BFA",
        "Buhru" => "#36C9B0",
        "Blessed Isles" => "#E9E4C9",
        "Bilgewater" => "#287D89",
        "Demacia" => "#4D8FE8",
        "Freljord" => "#91DDF5",
        "Ionia" => "#E88EBB",
        "Iztal" => "#63B84F",
        "Noxus" => "#D64045",
        "Piltover" => "#E8B85A",
        "Shadow Isles" => "#5EBD8A",
        "Shurima" => "#D6A447",
        "Targon" => "#8D9EFF",
        "Void" => "#B45BDF",
        "Zaun" => "#A2D94F",
        _ => ACCENT,
    }
}

#[cfg(test)]
#[path = "../tests/unit/config.rs"]
mod tests;
