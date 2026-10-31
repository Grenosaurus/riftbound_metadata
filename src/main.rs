mod artwork;
mod config;
mod layout;
mod models;
mod renderer;

use crate::layout::build_layout;
use crate::layout::LegendCount;
use crate::models::{Legend, TournamentFile};
use crate::renderer::render_svg;
use anyhow::{Context, Result};
use clap::Parser;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[command(
    name = "riftbound_meta",
    version,
    about = "Generate a Riftbound Legend meta breakdown PNG."
)]
struct Args {
    #[arg(default_value = "data/tournament.json")]
    tournament: PathBuf,
    #[arg(long, default_value = "data/legends.json")]
    legends: PathBuf,
    #[arg(long, default_value = "output/meta.png")]
    output: PathBuf,
    #[arg(long)]
    svg: Option<PathBuf>,
    #[arg(long, default_value = "../DataFiles/RiftboundAssets/legends")]
    assets: PathBuf,
    /// Do not contact Piltover Archive; render using cached/local artwork only.
    #[arg(long)]
    skip_artwork: bool,
    /// Re-download artwork even when a cached file already exists.
    #[arg(long)]
    refresh_artwork: bool,
}

fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("Could not parse JSON in {}", path.display()))
}

fn main() -> Result<()> {
    run(Args::parse())
}

fn run(args: Args) -> Result<()> {
    let tournament: TournamentFile = load_json(&args.tournament)?;
    let legend_db: HashMap<String, Legend> = load_json(&args.legends)?;

    sync_artwork_if_enabled(&args, &tournament, &legend_db);
    let legends = prepare_legends(&tournament, &legend_db);
    write_outputs(&args, &tournament, &legend_db, &legends)
}

fn prepare_legends(
    tournament: &TournamentFile,
    legend_db: &HashMap<String, Legend>,
) -> Vec<LegendCount> {
    let mut legends = Vec::with_capacity(tournament.legends.len());

    for (key, count) in &tournament.legends {
        if *count == 0 {
            continue;
        }

        if !legend_db.contains_key(key) {
            eprintln!("Warning: '{}' is missing from legends.json; skipping.", key);
            continue;
        }

        legends.push(layout::LegendCount {
            key: key.clone(),
            count: *count,
        });
    }

    legends.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.key.cmp(&b.key)));
    legends
}

fn sync_artwork_if_enabled(
    args: &Args,
    tournament: &TournamentFile,
    legend_db: &HashMap<String, Legend>,
) {
    if args.skip_artwork {
        return;
    }

    let tournament_keys: Vec<String> = tournament
        .legends
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(key, _)| key.clone())
        .collect();

    if let Err(err) = artwork::sync_artwork(
        legend_db,
        &tournament_keys,
        &args.assets,
        args.refresh_artwork,
    ) {
        eprintln!("Warning: artwork sync failed: {err:#}");
        eprintln!(
            "Continuing with any artwork already cached in {}.",
            args.assets.display()
        );
    }
}

fn write_outputs(
    args: &Args,
    tournament: &TournamentFile,
    legend_db: &HashMap<String, Legend>,
    legends: &[LegendCount],
) -> Result<()> {
    let tiles = build_layout(legends);
    let svg = render_svg(&tournament.tournament, legend_db, &tiles, &args.assets)?;

    let svg_path = args
        .svg
        .clone()
        .unwrap_or_else(|| args.output.with_extension("svg"));

    ensure_parent_dir(&args.output)?;
    ensure_parent_dir(&svg_path)?;

    fs::write(&svg_path, &svg)
        .with_context(|| format!("Could not write {}", svg_path.display()))?;

    render_png(&svg, &args.output)?;

    println!("Generated: {}", args.output.display());
    println!("SVG:       {}", svg_path.display());

    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Could not create directory {}", parent.display()))?;
    }

    Ok(())
}

fn render_png(svg: &str, output: &Path) -> Result<()> {
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

#[cfg(test)]
#[path = "../tests/unit/entrypoint.rs"]
mod tests;
