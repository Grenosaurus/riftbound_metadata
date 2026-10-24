use crate::config::{CANVAS_WIDTH, GAP, GRID_BOTTOM, GRID_TOP, MARGIN_X};

#[derive(Debug, Clone)]
pub struct LegendCount {
    pub key: String,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub key: String,
    pub count: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn build_layout(legends: &[LegendCount]) -> Vec<Tile> {
    if legends.is_empty() {
        return Vec::new();
    }

    let available_width = CANVAS_WIDTH as f32 - 2.0 * MARGIN_X;
    let available_height = GRID_BOTTOM - GRID_TOP;
    let (columns, rows, tile_size) = (1..=legends.len())
        .map(|columns| {
            let rows = legends.len().div_ceil(columns);
            let width_per_tile =
                (available_width - GAP * columns.saturating_sub(1) as f32) / columns as f32;
            let height_per_tile =
                (available_height - GAP * rows.saturating_sub(1) as f32) / rows as f32;
            (columns, rows, width_per_tile.min(height_per_tile))
        })
        .max_by(|a, b| a.2.total_cmp(&b.2))
        .expect("at least one column candidate exists");

    let grid_height = rows as f32 * tile_size + GAP * rows.saturating_sub(1) as f32;
    let mut tiles = Vec::with_capacity(legends.len());

    for (index, legend) in legends.iter().enumerate() {
        let row = index / columns;
        let column = index % columns;
        let items_in_row = (legends.len() - row * columns).min(columns);
        let row_width =
            items_in_row as f32 * tile_size + GAP * items_in_row.saturating_sub(1) as f32;
        let row_x = (CANVAS_WIDTH as f32 - row_width) / 2.0;

        tiles.push(Tile {
            key: legend.key.clone(),
            count: legend.count,
            x: row_x + column as f32 * (tile_size + GAP),
            y: GRID_TOP + (available_height - grid_height) / 2.0 + row as f32 * (tile_size + GAP),
            width: tile_size,
            height: tile_size,
        });
    }

    tiles
}
