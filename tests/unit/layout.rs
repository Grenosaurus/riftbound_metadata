use super::*;

fn legends(count: usize) -> Vec<LegendCount> {
    (0..count)
        .map(|index| LegendCount {
            key: format!("legend-{index}"),
            count: index as u32 + 1,
        })
        .collect()
}

#[test]
fn empty_input_produces_no_tiles() {
    assert!(build_layout(&[]).is_empty());
}

#[test]
fn one_legend_is_centered_and_preserves_its_data() {
    let input = legends(1);
    let tiles = build_layout(&input);

    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].key, input[0].key);
    assert_eq!(tiles[0].count, input[0].count);
    assert!((tiles[0].x + tiles[0].width / 2.0 - CANVAS_WIDTH as f32 / 2.0).abs() < 0.1);
    assert!((tiles[0].y + tiles[0].height / 2.0 - (GRID_TOP + GRID_BOTTOM) / 2.0).abs() < 0.1);
    assert!((tiles[0].width - tiles[0].height).abs() < f32::EPSILON);
}

#[test]
fn multiple_legends_fit_canvas_without_overlap() {
    let input = legends(57);
    let tiles = build_layout(&input);
    let left_bound = MARGIN_X;
    let right_bound = CANVAS_WIDTH as f32 - MARGIN_X;

    assert_eq!(tiles.len(), input.len());

    for (index, tile) in tiles.iter().enumerate() {
        assert_eq!(tile.key, input[index].key);
        assert_eq!(tile.count, input[index].count);
        assert!(tile.width.is_finite() && tile.width > 0.0);
        assert!((tile.width - tile.height).abs() < f32::EPSILON);
        assert!(tile.x >= left_bound - 0.1);
        assert!(tile.x + tile.width <= right_bound + 0.1);
        assert!(tile.y >= GRID_TOP - 0.1);
        assert!(tile.y + tile.height <= GRID_BOTTOM + 0.1);

        for other in tiles.iter().skip(index + 1) {
            let separated = tile.x + tile.width <= other.x + 0.1
                || other.x + other.width <= tile.x + 0.1
                || tile.y + tile.height <= other.y + 0.1
                || other.y + other.height <= tile.y + 0.1;
            assert!(separated, "{} overlaps {}", tile.key, other.key);
        }
    }
}
