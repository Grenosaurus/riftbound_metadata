mod artwork;
mod cli;
mod config;
mod layout;
mod models;
mod output;
mod renderer;

use crate::cli::{resolve_filename, Args, OUTPUT_DIR, TOURNAMENT_DIR};
use crate::layout::LegendCount;
use crate::models::{Legend, TournamentFile};
use crate::output::write_outputs;
#[cfg(test)]
use crate::output::{ensure_parent_dir, render_png};
use anyhow::{Context, Result};
use clap::Parser;
use std::{collections::HashMap, fs, path::Path};

const LEGENDS_FILE: &str = "data/legends.json";
const ASSETS_DIR: &str = "../DataFiles/RiftboundAssets/legends";

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

    if let Err(err) = artwork::sync_artwork(
        legend_db,
        &tournament.legends,
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

#[cfg(test)]
#[path = "../tests/unit/test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "../tests/unit/entrypoint.rs"]
mod tests;
