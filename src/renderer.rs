use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{collections::HashMap, fs, path::Path};
use crate::config::{faction_color, ACCENT, BACKGROUND, BORDER_RADIUS, BORDER_WIDTH, CANVAS_HEIGHT, CANVAS_WIDTH, MUTED_TEXT, PANEL, TEXT};
use crate::layout::Tile;
use crate::models::{Legend, Tournament};


fn esc(s: &str) -> String
{
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}


fn image_href(path: &Path) -> Result<Option<String>>
{
    if !path.exists()
    {
        return Ok(None);
    }
    let bytes = fs::read(path)?;
    let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n")
    {
        "image/png"
    }
    else if bytes.starts_with(b"\xff\xd8\xff")
    {
        "image/jpeg"
    }
    else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
    {
        "image/webp"
    }
    else
    {
        match path.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase().as_str()
        {
            "jpg" | "jpeg" => "image/jpeg", "webp" => "image/webp", _ => "image/png"
        }
    };

    let encoded = STANDARD.encode(bytes);

    Ok(Some(format!("data:{};base64,{}", mime, encoded)))
}


pub fn render_svg(t: &Tournament, legends: &HashMap<String, Legend>, tiles: &[Tile], assets: &Path) -> Result<String>
{
    let mut s = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><defs><linearGradient id="bg" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="{}"/><stop offset="100%" stop-color="#10283B"/></linearGradient><filter id="shadow" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="8" stdDeviation="10" flood-opacity="0.35"/></filter></defs><rect width="100%" height="100%" fill="url(#bg)"/>"#, CANVAS_WIDTH, CANVAS_HEIGHT, CANVAS_WIDTH, CANVAS_HEIGHT, BACKGROUND);
    s.push_str(&format!(r#"<text x="90" y="72" fill="{}" font-family="Arial,sans-serif" font-size="30" font-weight="700" letter-spacing="4">RIFTBOUND</text>"#, ACCENT));
    s.push_str(&format!(r#"<text x="90" y="126" fill="{}" font-family="Arial,sans-serif" font-size="48" font-weight="800">LEGEND META BREAKDOWN</text>"#, TEXT));
    // Per the requested layout, show only the tournament name: no date or player count.
    s.push_str(&format!(r#"<text x="90" y="165" fill="{}" font-family="Arial,sans-serif" font-size="22">{}</text><line x1="90" y1="195" x2="1830" y2="195" stroke="{}" stroke-opacity="0.45" stroke-width="2"/>"#, MUTED_TEXT, esc(&t.name), ACCENT));

    for tile in tiles
    {
        let Some(d) = legends.get(&tile.key) else
        {
            continue;
        };

        let border = faction_color(&d.faction); let (x,y,w,h)=(tile.x,tile.y,tile.width,tile.height);
        s.push_str(&format!(r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}" fill="{panel}" stroke="{border}" stroke-width="{bw}" filter="url(#shadow)"/>"#, x=x,y=y,w=w,h=h,r=BORDER_RADIUS,panel=PANEL,border=border,bw=BORDER_WIDTH));

        if let Some(image) = image_href(&assets.join(&d.image))?
        {
            let clip = format!("clip_{}", tile.key.replace(|c: char| !c.is_ascii_alphanumeric(), "_"));
            s.push_str(&format!(r#"<clipPath id="{clip}"><rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}"/></clipPath><image href="{image}" x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" preserveAspectRatio="xMidYMid slice" opacity="0.78" clip-path="url(#{clip})"/><rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}" fill="#000000" fill-opacity="0.28"/>"#, clip=clip,x=x,y=y,w=w,h=h,r=BORDER_RADIUS,image=image));
        }

        let bw = 110.0_f32.min(w*0.25); let bh = 58.0_f32.min(h*0.23); let bx=x+w-bw-18.0; let by=y+18.0;
        s.push_str(&format!(r#"<rect x="{bx:.1}" y="{by:.1}" width="{bw:.1}" height="{bh:.1}" rx="12" fill="#000000" fill-opacity="0.68"/><text x="{cx:.1}" y="{cy:.1}" text-anchor="middle" fill="{text}" font-family="Arial,sans-serif " font-size="{fs:.1}" font-weight="800">{count}</text>"#,bx=bx,by=by,bw=bw,bh=bh,cx=bx+bw/2.0,cy=by+bh*0.68,text=TEXT,fs=(bh*0.48).max(24.0),count=tile.count));

        let ts=(w/12.0).clamp(18.0,34.0); let ss=(w/19.0).clamp(12.0,19.0);
        s.push_str(&format!(r#"<text x="{x:.1}" y="{y:.1}" fill="{text}" font-family="Arial,sans-serif" font-size="{ts:.1}" font-weight="800">{name}</text><text x="{x2:.1}" y="{y2:.1}" fill="{muted}" font-family="Arial,sans-serif" font-size="{ss:.1}" font-weight="600">{title}</text><text x="{px:.1}" y="{py:.1}" text-anchor="end" fill="{accent}" font-family="Arial,sans-serif" font-size="{ss:.1}" font-weight="700">{pct:.1}%</text>"#,x=x+22.0,y=y+h-52.0,text=TEXT,ts=ts,name=esc(&d.name.to_uppercase()),x2=x+22.0,y2=y+h-27.0,muted=MUTED_TEXT,ss=ss,title=esc(&d.title),px=x+w-20.0,py=y+h-27.0,accent=ACCENT,pct=tile.percentage));
    }

    s.push_str("</svg>"); Ok(s)
}

