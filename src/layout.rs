use crate::config::{CANVAS_WIDTH, GAP, GRID_BOTTOM, GRID_TOP, MARGIN_X, MAX_TILE_HEIGHT, MAX_TILE_WIDTH, MIN_TILE_HEIGHT, MIN_TILE_WIDTH};


#[derive(Debug, Clone)]
pub struct LegendCount { pub key: String, pub count: u32 }
#[derive(Debug, Clone)]
pub struct Tile { pub key: String, pub count: u32, pub percentage: f32, pub x: f32, pub y: f32, pub width: f32, pub height: f32 }


fn calculate_base_width(count: u32, maximum: u32) -> f32
{
    if maximum == 0
    {
        return MIN_TILE_WIDTH;
    }

    MIN_TILE_WIDTH + (count as f32 / maximum as f32).sqrt() * (MAX_TILE_WIDTH - MIN_TILE_WIDTH)
}


pub fn build_layout(legends: &[LegendCount]) -> Vec<Tile>
{
    if legends.is_empty()
    {
        return Vec::new();
    }

    let max_count = legends.iter().map(|x| x.count).max().unwrap_or(0);
    let total: u32 = legends.iter().map(|x| x.count).sum();
    let desired: Vec<(&LegendCount, f32, f32)> = legends.iter().map(|item| {
        let width = calculate_base_width(item.count, max_count);
        let height = (MIN_TILE_HEIGHT + (width - MIN_TILE_WIDTH) * 0.45).min(MAX_TILE_HEIGHT);
        (item, width, height)
    }).collect();

    let available_width = CANVAS_WIDTH as f32 - 2.0 * MARGIN_X;
    let available_height = GRID_BOTTOM - GRID_TOP;
    let mut rows: Vec<Vec<(&LegendCount, f32, f32)>> = Vec::new();
    let mut current = Vec::new();
    let mut current_width = 0.0;

    for item in desired
    {
        let proposed = if current.is_empty() { item.1 } else { current_width + GAP + item.1 };
        if !current.is_empty() && proposed > available_width { rows.push(current); current = Vec::new(); current_width = 0.0; }
        current_width = if current.is_empty() { item.1 } else { current_width + GAP + item.1 };
        current.push(item);
    }

    if !current.is_empty()
    {
        rows.push(current);
    }

    let row_heights: Vec<f32> = rows.iter().map(|r| r.iter().map(|x| x.2).fold(0.0, f32::max)).collect();
    let total_height = row_heights.iter().sum::<f32>() + GAP * row_heights.len().saturating_sub(1) as f32;
    let scale = if total_height > 0.0 { (available_height / total_height).min(1.0) } else { 1.0 };
    let mut tiles = Vec::new();
    let mut y = GRID_TOP;

    for (row_index, row) in rows.iter().enumerate()
    {
        let row_height = row_heights[row_index] * scale;
        let widths: Vec<f32> = row.iter().map(|x| x.1 * scale).collect();
        let row_width = widths.iter().sum::<f32>() + GAP * widths.len().saturating_sub(1) as f32;
        let mut x = MARGIN_X + (available_width - row_width) / 2.0;

        for ((item, _, _), width) in row.iter().zip(widths.iter())
        {
            tiles.push(Tile { key: item.key.clone(), count: item.count, percentage: if total > 0 { item.count as f32 / total as f32 * 100.0 } else { 0.0 }, x, y, width: *width, height: row_height });
            x += *width + GAP;
        }

        y += row_height + GAP;
    }

    tiles
}

