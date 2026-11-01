use anyhow::{ensure, Result};
use clap::Parser;
use std::path::{Path, PathBuf};

pub(crate) const TOURNAMENT_DIR: &str = "../DataFiles/RiftboundTournaments";
pub(crate) const OUTPUT_DIR: &str = "../DataFiles/RiftboundTournaments/TournamentOutput";

#[derive(Parser, Debug)]
#[command(version, about = "Generate a Riftbound Legend meta breakdown PNG.")]
pub(crate) struct Args {
    #[arg(
        default_value = "tournament.json",
        help = "Tournament JSON filename in ../DataFiles/RiftboundTournaments."
    )]
    pub(crate) tournament: PathBuf,
    #[arg(
        long,
        default_value = "meta.png",
        help = "PNG output filename in ../DataFiles/RiftboundTournaments/TournamentOutput."
    )]
    pub(crate) output: PathBuf,
    #[arg(
        long,
        help = "SVG output filename. Defaults to the PNG filename with .svg."
    )]
    pub(crate) svg: Option<PathBuf>,
    #[arg(
        long,
        help = "Do not contact Piltover Archive; render using cached/local artwork only."
    )]
    pub(crate) skip_artwork: bool,
    #[arg(
        long,
        help = "Re-download artwork even when a cached file already exists."
    )]
    pub(crate) refresh_artwork: bool,
}

pub(crate) fn resolve_filename(directory: &str, filename: &Path, label: &str) -> Result<PathBuf> {
    let mut components = filename.components();
    ensure!(
        matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none(),
        "{label} must be a filename only; directory paths are not accepted"
    );

    Ok(Path::new(directory).join(filename))
}
