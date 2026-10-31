use crate::config::{
    faction_color, ACCENT, BACKGROUND, BORDER_RADIUS, BORDER_WIDTH, CANVAS_HEIGHT, CANVAS_WIDTH,
    MUTED_TEXT, PANEL, TEXT,
};
use crate::layout::Tile;
use crate::models::{Legend, Tournament};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{imageops, GenericImageView, ImageFormat, ImageReader};
use std::{
    collections::{hash_map::Entry, HashMap},
    io::Cursor,
    path::{Path, PathBuf},
};

const ART_CROP_LEFT: f32 = 0.06;
const ART_CROP_RIGHT: f32 = 0.94;
const ART_CROP_TOP: f32 = 0.02;
const ART_CROP_BOTTOM: f32 = 0.65;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn image_href(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let source = ImageReader::open(path)
        .with_context(|| format!("Could not read artwork {}", path.display()))?
        .with_guessed_format()
        .with_context(|| format!("Could not inspect artwork {}", path.display()))?
        .decode()
        .with_context(|| format!("Could not decode artwork {}", path.display()))?;
    let (width, height) = source.dimensions();
    let crop_x = (width as f32 * ART_CROP_LEFT).round() as u32;
    let crop_right = (width as f32 * ART_CROP_RIGHT).round() as u32;
    let crop_y = (height as f32 * ART_CROP_TOP).round() as u32;
    let crop_bottom = (height as f32 * ART_CROP_BOTTOM).round() as u32;
    let crop_width = crop_right.saturating_sub(crop_x).max(1);
    let crop_height = crop_bottom.saturating_sub(crop_y).max(1);
    let display_crop =
        imageops::crop_imm(&source, crop_x, crop_y, crop_width, crop_height).to_image();

    let mut png = Cursor::new(Vec::new());
    display_crop
        .write_to(&mut png, ImageFormat::Png)
        .with_context(|| format!("Could not encode display crop for {}", path.display()))?;
    let encoded = STANDARD.encode(png.into_inner());

    Ok(Some(format!("data:image/png;base64,{}", encoded)))
}

pub fn render_svg(
    t: &Tournament,
    legends: &HashMap<String, Legend>,
    tiles: &[Tile],
    assets: &Path,
) -> Result<String> {
    let mut image_cache: HashMap<PathBuf, Option<String>> = HashMap::with_capacity(tiles.len());
    let mut s = String::with_capacity(4096);
    append_svg_start(&mut s);
    append_tournament_header(&mut s, t);

    for tile in tiles {
        if let Some(legend) = legends.get(&tile.key) {
            append_tile(&mut s, tile, legend, assets, &mut image_cache)?;
        }
    }

    s.push_str("</svg>");
    Ok(s)
}

fn append_svg_start(svg: &mut String) {
    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><defs><linearGradient id="bg" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="{}"/><stop offset="100%" stop-color="#10283B"/></linearGradient><filter id="shadow" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="8" stdDeviation="10" flood-opacity="0.35"/></filter></defs><rect width="100%" height="100%" fill="url(#bg)"/>"##,
        CANVAS_WIDTH, CANVAS_HEIGHT, CANVAS_WIDTH, CANVAS_HEIGHT, BACKGROUND
    ));
}

fn append_tournament_header(svg: &mut String, tournament: &Tournament) {
    svg.push_str(&format!(r##"<text x="90" y="72" fill="{}" font-family="Arial,sans-serif" font-size="30" font-weight="700" letter-spacing="4">RIFTBOUND</text>"##,
                        ACCENT));

    if let Some(date) = tournament
        .date
        .as_deref()
        .map(str::trim)
        .filter(|date| !date.is_empty())
    {
        svg.push_str(&format!(
            r##"<text x="1830" y="72" text-anchor="end" fill="{}" font-family="Arial,sans-serif" font-size="18">{}</text>"##,
            MUTED_TEXT,
            esc(date)
        ));
    }

    svg.push_str(&format!(r##"<text x="90" y="126" fill="{}" font-family="Arial,sans-serif" font-size="48" font-weight="800">LEGEND META BREAKDOWN</text>"##,
                        TEXT));

    let player_label = if tournament.players == 1 {
        format!("{} (1 player)", tournament.name)
    } else {
        format!("{} ({} players)", tournament.name, tournament.players)
    };
    svg.push_str(&format!(r##"<text x="90" y="165" fill="{}" font-family="Arial,sans-serif" font-size="22">{}</text><line x1="90" y1="195" x2="1830" y2="195" stroke="{}" stroke-opacity="0.45" stroke-width="2"/>"##,
                        MUTED_TEXT,
                        esc(&player_label),
                        ACCENT));
}

fn append_tile(
    svg: &mut String,
    tile: &Tile,
    legend: &Legend,
    assets: &Path,
    image_cache: &mut HashMap<PathBuf, Option<String>>,
) -> Result<()> {
    let border = faction_color(&legend.faction);
    let (x, y, width, height) = (tile.x, tile.y, tile.width, tile.height);

    svg.push_str(&format!(r##"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}" fill="{panel}" stroke="{border}" stroke-width="{bw}" filter="url(#shadow)"/>"##,
                        x = x,
                        y = y,
                        w = width,
                        h = height,
                        r = BORDER_RADIUS,
                        panel = PANEL,
                        border = border,
                        bw = BORDER_WIDTH));

    append_tile_artwork(svg, tile, legend, assets, image_cache)?;
    append_count_badge(svg, tile, x, y, width, height);
    append_legend_labels(svg, legend, x, y, width, height);
    Ok(())
}

fn append_tile_artwork(
    svg: &mut String,
    tile: &Tile,
    legend: &Legend,
    assets: &Path,
    image_cache: &mut HashMap<PathBuf, Option<String>>,
) -> Result<()> {
    let image_path = assets.join(&legend.image);
    let cached_image = match image_cache.entry(image_path) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => {
            let image = image_href(entry.key())?;
            entry.insert(image)
        }
    };

    if let Some(image) = cached_image {
        let clip = format!(
            "clip_{}",
            tile.key.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
        );

        svg.push_str(&format!(r##"<clipPath id="{clip}"><rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}"/></clipPath><image href="{image}" x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" preserveAspectRatio="xMidYMid meet" opacity="0.78" clip-path="url(#{clip})"/><rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}" fill="#000000" fill-opacity="0.28"/>"##,
                            clip = clip,
                            x = tile.x,
                            y = tile.y,
                            w = tile.width,
                            h = tile.height,
                            r = BORDER_RADIUS,
                            image = image));
    }

    Ok(())
}

fn append_count_badge(svg: &mut String, tile: &Tile, x: f32, y: f32, width: f32, height: f32) {
    let badge_height = 30.0_f32.min(height * 0.12);
    let badge_y = y + 12.0;
    let badge_width = 54.0_f32.min(width * 0.25);
    let badge_x = x + 12.0;

    svg.push_str(&format!(r##"<rect x="{bx:.1}" y="{by:.1}" width="{bw:.1}" height="{bh:.1}" rx="9" fill="#000000" fill-opacity="0.62"/><text x="{cx:.1}" y="{cy:.1}" text-anchor="middle" fill="{text}" font-family="Arial,sans-serif" font-size="{fs:.1}" font-weight="700">{count}</text>"##,
                        bx = badge_x,
                        by = badge_y,
                        bw = badge_width,
                        bh = badge_height,
                        cx = badge_x + badge_width / 2.0,
                        cy = badge_y + badge_height * 0.70,
                        text = TEXT,
                        fs = (badge_height * 0.58).clamp(12.0, 16.0),
                        count = tile.count));
}

fn append_legend_labels(
    svg: &mut String,
    legend: &Legend,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let title_font_size = (width / 16.0).clamp(14.0, 25.0);
    let subtitle_font_size = (width / 24.0).clamp(10.0, 15.0);
    let name = legend.name.to_uppercase();
    let name_x = x + width / 2.0;
    let name_y = y + height - 43.0;
    let max_name_width = (width - 24.0).max(1.0);
    let estimated_name_width = name
        .chars()
        .map(|character| match character {
            'I' | '!' | '.' | ',' | '\'' => 0.35,
            'J' | 'L' | 'T' => 0.55,
            'M' | 'W' => 0.9,
            _ => 0.68,
        })
        .sum::<f32>()
        * title_font_size;
    let fitted_name_font_size = if estimated_name_width > max_name_width {
        title_font_size * max_name_width / estimated_name_width
    } else {
        title_font_size
    };

    svg.push_str(&format!(
        r##"<text x="{name_x:.1}" y="{name_y:.1}" text-anchor="middle" fill="{text}" font-family="Arial,sans-serif" font-size="{font_size:.1}" font-weight="800">{name}</text>"##,
        name_x = name_x,
        name_y = name_y,
        text = TEXT,
        font_size = fitted_name_font_size,
        name = esc(&name),
    ));

    svg.push_str(&format!(r##"<text x="{x2:.1}" y="{y2:.1}" text-anchor="middle" fill="{muted}" font-family="Arial,sans-serif" font-size="{ss:.1}" font-weight="600">{title}</text>"##,
                        x2 = x + width / 2.0,
                        y2 = y + height - 19.0,
                        muted = MUTED_TEXT,
                        ss = subtitle_font_size,
                        title = esc(&legend.title)));
}

#[cfg(test)]
#[path = "../tests/unit/renderer.rs"]
mod tests;
