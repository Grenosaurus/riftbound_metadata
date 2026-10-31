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
    s.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><defs><linearGradient id="bg" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="{}"/><stop offset="100%" stop-color="#10283B"/></linearGradient><filter id="shadow" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="8" stdDeviation="10" flood-opacity="0.35"/></filter></defs><rect width="100%" height="100%" fill="url(#bg)"/>"##,
        CANVAS_WIDTH, CANVAS_HEIGHT, CANVAS_WIDTH, CANVAS_HEIGHT, BACKGROUND
    ));

    s.push_str(&format!(r##"<text x="90" y="72" fill="{}" font-family="Arial,sans-serif" font-size="30" font-weight="700" letter-spacing="4">RIFTBOUND</text>"##,
                        ACCENT));

    if let Some(date) = t
        .date
        .as_deref()
        .map(str::trim)
        .filter(|date| !date.is_empty())
    {
        s.push_str(&format!(
            r##"<text x="1830" y="72" text-anchor="end" fill="{}" font-family="Arial,sans-serif" font-size="18">{}</text>"##,
            MUTED_TEXT,
            esc(date)
        ));
    }

    s.push_str(&format!(r##"<text x="90" y="126" fill="{}" font-family="Arial,sans-serif" font-size="48" font-weight="800">LEGEND META BREAKDOWN</text>"##,
                        TEXT));

    let tournament_label = if t.players == 1 {
        format!("{} (1 player)", t.name)
    } else {
        format!("{} ({} players)", t.name, t.players)
    };
    s.push_str(&format!(r##"<text x="90" y="165" fill="{}" font-family="Arial,sans-serif" font-size="22">{}</text><line x1="90" y1="195" x2="1830" y2="195" stroke="{}" stroke-opacity="0.45" stroke-width="2"/>"##,
                        MUTED_TEXT,
                        esc(&tournament_label),
                        ACCENT));

    for tile in tiles {
        let Some(d) = legends.get(&tile.key) else {
            continue;
        };

        let border = faction_color(&d.faction);
        let (x, y, w, h) = (tile.x, tile.y, tile.width, tile.height);

        s.push_str(&format!(r##"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}" fill="{panel}" stroke="{border}" stroke-width="{bw}" filter="url(#shadow)"/>"##,
                            x = x,
                            y = y,
                            w = w,
                            h = h,
                            r = BORDER_RADIUS,
                            panel = PANEL,
                            border = border,
                            bw = BORDER_WIDTH));

        let image_path = assets.join(&d.image);
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

            s.push_str(&format!(r##"<clipPath id="{clip}"><rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}"/></clipPath><image href="{image}" x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" preserveAspectRatio="xMidYMid meet" opacity="0.78" clip-path="url(#{clip})"/><rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}" fill="#000000" fill-opacity="0.28"/>"##,
                                clip = clip,
                                x = x,
                                y = y,
                                w = w,
                                h = h,
                                r = BORDER_RADIUS,
                                image = image));
        }

        let bh = 30.0_f32.min(h * 0.12);
        let by = y + 12.0;
        let bw = 54.0_f32.min(w * 0.25);
        let bx = x + 12.0;

        s.push_str(&format!(r##"<rect x="{bx:.1}" y="{by:.1}" width="{bw:.1}" height="{bh:.1}" rx="9" fill="#000000" fill-opacity="0.62"/><text x="{cx:.1}" y="{cy:.1}" text-anchor="middle" fill="{text}" font-family="Arial,sans-serif" font-size="{fs:.1}" font-weight="700">{count}</text>"##,
                            bx = bx,
                            by = by,
                            bw = bw,
                            bh = bh,
                            cx = bx + bw / 2.0,
                            cy = by + bh * 0.70,
                            text = TEXT,
                            fs = (bh * 0.58).clamp(12.0, 16.0),
                            count = tile.count));

        let ts = (w / 16.0).clamp(14.0, 25.0);
        let ss = (w / 24.0).clamp(10.0, 15.0);
        let legend_name = d.name.to_uppercase();
        let name_x = x + w / 2.0;
        let name_y = y + h - 43.0;
        let max_name_width = (w - 24.0).max(1.0);
        let estimated_name_width = legend_name
            .chars()
            .map(|character| match character {
                'I' | '!' | '.' | ',' | '\'' => 0.35,
                'J' | 'L' | 'T' => 0.55,
                'M' | 'W' => 0.9,
                _ => 0.68,
            })
            .sum::<f32>()
            * ts;
        let name_font_size = if estimated_name_width > max_name_width {
            ts * max_name_width / estimated_name_width
        } else {
            ts
        };
        s.push_str(&format!(
            r##"<text x="{name_x:.1}" y="{name_y:.1}" text-anchor="middle" fill="{text}" font-family="Arial,sans-serif" font-size="{font_size:.1}" font-weight="800">{name}</text>"##,
            name_x = name_x,
            name_y = name_y,
            text = TEXT,
            font_size = name_font_size,
            name = esc(&legend_name),
        ));

        s.push_str(&format!(r##"<text x="{x2:.1}" y="{y2:.1}" text-anchor="middle" fill="{muted}" font-family="Arial,sans-serif" font-size="{ss:.1}" font-weight="600">{title}</text>"##,
                            x2 = x + w / 2.0,
                            y2 = y + h - 19.0,
                            muted = MUTED_TEXT,
                            ss = ss,
                            title = esc(&d.title)));
    }

    s.push_str("</svg>");

    Ok(s)
}

#[cfg(test)]
#[path = "../tests/unit/renderer.rs"]
mod tests;
