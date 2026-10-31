mod artwork;
mod config;
mod layout;
mod models;
mod renderer;

use crate::layout::{build_layout, LegendCount};
use crate::models::{Legend, TournamentFile};
use crate::renderer::render_svg;
use anyhow::{bail, Context, Result};
use clap::Parser;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

const TOURNAMENT_DIR: &str = "../DataFiles/RiftboundTournaments";
const OUTPUT_DIR: &str = "../DataFiles/RiftboundTournaments/TournamentOutput";
const LEGENDS_FILE: &str = "data/legends.json";
const ASSETS_DIR: &str = "../DataFiles/RiftboundAssets/legends";

#[derive(Parser, Debug)]
#[command(
    name = "riftbound_meta",
    version,
    about = "Generate a Riftbound Legend meta breakdown PNG."
)]
struct Args {
    #[arg(
        default_value = "tournament.json",
        help = "Tournament JSON filename in ../DataFiles/RiftboundTournaments."
    )]
    tournament: PathBuf,
    #[arg(
        long,
        default_value = "meta.png",
        help = "PNG output filename in ../DataFiles/RiftboundTournaments/TournamentOutput."
    )]
    output: PathBuf,
    #[arg(
        long,
        help = "SVG output filename. Defaults to the PNG filename with .svg."
    )]
    svg: Option<PathBuf>,
    /// Do not contact Piltover Archive; render using cached/local artwork only.
    #[arg(
        long,
        help = "Do not contact Piltover Archive; render using cached/local artwork only."
    )]
    skip_artwork: bool,
    #[arg(
        long,
        help = "Re-download artwork even when a cached file already exists."
    )]
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
    let tournament_path = resolve_filename(TOURNAMENT_DIR, &args.tournament, "Tournament")?;
    let output_path = resolve_filename(OUTPUT_DIR, &args.output, "PNG output")?;
    let svg_path = match &args.svg {
        Some(path) => resolve_filename(OUTPUT_DIR, path, "SVG output")?,
        None => output_path.with_extension("svg"),
    };
    let assets_dir = Path::new(ASSETS_DIR);

    let tournament: TournamentFile = load_json(&tournament_path)?;
    let legend_db: HashMap<String, Legend> = load_json(Path::new(LEGENDS_FILE))?;

    sync_artwork_if_enabled(&args, &tournament, &legend_db, assets_dir);
    let legends = prepare_legends(&tournament, &legend_db);
    write_outputs(
        &tournament,
        &legend_db,
        &legends,
        assets_dir,
        &output_path,
        &svg_path,
    )
}

fn resolve_filename(directory: &str, filename: &Path, label: &str) -> Result<PathBuf> {
    let mut components = filename.components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        bail!("{label} must be a filename only; directory paths are not accepted");
    }

    Ok(Path::new(directory).join(filename))
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
    assets_dir: &Path,
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
        assets_dir,
        args.refresh_artwork,
    ) {
        eprintln!("Warning: artwork sync failed: {err:#}");
        eprintln!(
            "Continuing with any artwork already cached in {}.",
            assets_dir.display()
        );
    }
}

fn write_outputs(
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
