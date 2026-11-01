use crate::config;
use crate::layout::{build_layout, LegendCount};
use crate::models::{Legend, TournamentFile};
use crate::renderer::render_svg;
use anyhow::{Context, Result};
use std::{collections::HashMap, fs, path::Path};

pub(crate) fn write_outputs(
    tournament: &TournamentFile,
    legend_db: &HashMap<String, Legend>,
    legends: &[LegendCount],
    assets_dir: &Path,
    output_path: &Path,
    svg_path: &Path,
) -> Result<()> {
    let tiles = build_layout(legends);
    let svg = render_svg(&tournament.tournament, legend_db, &tiles, assets_dir)?;

    ensure_parent_dir(output_path)?;
    ensure_parent_dir(svg_path)?;

    fs::write(svg_path, &svg).with_context(|| format!("Could not write {}", svg_path.display()))?;

    render_png(&svg, output_path)?;

    println!("Generated: {}", output_path.display());
    println!("SVG:       {}", svg_path.display());

    Ok(())
}

pub(crate) fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Could not create directory {}", parent.display()))?;
    }

    Ok(())
}

pub(crate) fn render_png(svg: &str, output: &Path) -> Result<()> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg, &opt).context("Could not parse generated SVG")?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(config::CANVAS_WIDTH, config::CANVAS_HEIGHT)
        .context("Could not allocate output image")?;

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    pixmap
        .save_png(output)
        .with_context(|| format!("Could not write {}", output.display()))
}
