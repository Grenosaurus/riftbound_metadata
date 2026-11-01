# Riftbound Meta Generator

This Rust command-line tool turns a Riftbound tournament JSON file into a polished 1920×1080 tournament meta card. It reads the tournament and Legend metadata, filters out zero-count entries, optionally syncs missing artwork from Piltover Archive, and writes both a PNG and an SVG output.

The final image includes the tournament name, shows the player count when it is at least two, and prints the tournament date when one is provided. Legends with counts greater than zero are the only entries included in the output or considered for artwork downloads.

## Quick start

1. Make sure the project has access to the expected input and cache directories:
   - Tournament JSON: `../DataFiles/RiftboundTournaments`
   - Legend database: `data/legends.json`
   - Artwork cache: `../DataFiles/RiftboundAssets/legends`
2. Place your tournament JSON file in the tournament directory (for example `tournament.json`).
3. Run the tool from the project directory:

```powershell
cargo run
```

By default, the tool writes:

- PNG: `../DataFiles/RiftboundTournaments/TournamentOutput/meta.png`
- SVG: `../DataFiles/RiftboundTournaments/TournamentOutput/meta.svg`

## Requirements

- Rust and Cargo: https://www.rust-lang.org/tools/install
- Internet access to download artwork that is not already cached (unless using `--skip-artwork`)

## Run

From the project directory:

```powershell
cargo run
```

## Tests

Run all unit and command-line integration tests with:

```powershell
cargo test
```

All tests use local fixtures and run offline. They cover CLI help and input
errors; SVG/PNG generation and dimensions; output path handling; filtering and
ordering; tournament and Legend text escaping; layout boundaries and overlap;
faction colors; artwork cropping, variant selection, cache use, and error
handling. They do not call Piltover Archive, so API availability and remote
downloads are not validated by `cargo test`.

Optional formatting and lint checks:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

The program uses fixed data folders and accepts filenames rather than paths.
By default, it reads `tournament.json` from
`../DataFiles/RiftboundTournaments` and `data/legends.json` from the project,
then writes:

- PNG: `../DataFiles/RiftboundTournaments/TournamentOutput/meta.png`
- SVG: `../DataFiles/RiftboundTournaments/TournamentOutput/meta.svg`

All relative paths are resolved from the current working directory.

## Command-line options

Run `cargo run -- --help` to see the CLI help. The available arguments are:

| Argument | Default | Purpose |
|---|---|---|
| `tournament` | `tournament.json` | Tournament JSON filename, read from `../DataFiles/RiftboundTournaments` |
| `--output` | `meta.png` | PNG filename, written to `../DataFiles/RiftboundTournaments/TournamentOutput` |
| `--svg` | PNG filename with `.svg` extension | Optional SVG filename, written to the same output folder |
| `--skip-artwork` | Off | Do not contact Piltover Archive; render using local artwork only |
| `--refresh-artwork` | Off | Re-download artwork for positive-count tournament Legends, even if cached |

Tournament and output arguments must be filenames only; directory paths are
rejected. The tournament database and artwork cache locations are fixed at
`data/legends.json` and `../DataFiles/RiftboundAssets/legends`.

Examples:

```powershell
# Use the default tournament.json and meta.png/meta.svg names
cargo run

# Use a different tournament filename and output filenames
cargo run -- weekly-event.json --output weekly-meta.png --svg weekly-meta.svg

# Render offline from the configured artwork cache
cargo run -- --skip-artwork

# Force artwork downloads for Legends with counts greater than zero
cargo run -- --refresh-artwork
```

## Input data

The selected tournament file (by default
`../DataFiles/RiftboundTournaments/tournament.json`) contains a `tournament`
object with:

- `name`: tournament name
- `date`: optional date string; an empty string omits the date from the output
- `players`: player count

It also contains a `legends` object mapping Legend IDs to tournament counts. Counts of zero are excluded from both the rendered output and artwork synchronization.

`data/legends.json` maps those IDs to Legend metadata: `name`, `title`, `image`,
and `faction`. Counts are supplied in the tournament file; they are not fetched
from Piltover Archive. Faction names must exactly match the configured names
(for example, use `Void`, not `The Void`); unrecognized names use the default
accent color.

Example tournament file:

```json
{
  "tournament": {
    "name": "Local Skirmish",
    "date": "31/10/2026",
    "players": 16
  },
  "legends": {
    "ahri_ogn": 4,
    "darius_ogn": 2
  }
}
```

Example Legend metadata:

```json
{
  "ahri_ogn": {
    "name": "Ahri",
    "title": "The Fox",
    "image": "ahri_ogn.png",
    "faction": "Ionia"
  }
}
```

`date` and `players` may be omitted; they default to no date and zero players.
Legends with zero counts are ignored. Positive-count IDs missing from
`legends.json` are skipped with a warning.

## Artwork behavior

By default, artwork is synchronized only for tournament Legends whose count is greater than zero and whose image is not already present in the configured cache. Cached images are reused on later runs. `--refresh-artwork` forces a new download for the positive-count Legends, and `--skip-artwork` disables synchronization entirely.

The program requests paginated card data from `https://piltoverarchive.com/api/external/v1/cards`, matches card names to the local Legend metadata, and prefers the set indicated by the image filename when possible. It downloads the selected image unchanged into the cache; display cropping is applied during rendering, not to the cached image.

If artwork synchronization fails, the program prints a warning and continues
with any cached artwork. A tile whose image is missing renders without that
image; an existing but unreadable or corrupt image causes rendering to fail.
The external API is community-documented, not an official stable contract, and
may change. Check Piltover Archive's terms and applicable Riot Games policies
before redistributing card artwork.

## Project structure

```text
riftbound_metadata/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── models.rs
│   ├── layout.rs
│   ├── renderer.rs
│   ├── output.rs
│   └── artwork.rs
├── tests/
│   ├── cli.rs
│   ├── cli/
│   │   ├── arguments.rs
│   │   ├── common.rs
│   │   ├── errors.rs
│   │   └── generation.rs
│   └── unit/
│       ├── artwork.rs
│       ├── config.rs
│       ├── entrypoint.rs
│       ├── layout.rs
│       ├── renderer.rs
│       └── test_support.rs
├── data/
│   └── legends.json
```

Tournament JSON files are read from
`../DataFiles/RiftboundTournaments`; only the filename is supplied on the
command line. The artwork cache is at
`../DataFiles/RiftboundAssets/legends`. Generated PNG and SVG files are written
to `../DataFiles/RiftboundTournaments/TournamentOutput`; only their filenames
are supplied on the command line.

## Program architecture

`cli.rs` defines command-line arguments and validates filename-only paths.
`main.rs` coordinates the workflow: it loads the JSON inputs, optionally
synchronizes missing artwork, and filters and sorts Legends. `layout.rs` builds
tile positions, `renderer.rs` creates the SVG, and `output.rs` writes the SVG
and rasterizes it to PNG. Rendering and artwork-sync failures are reported with
context; artwork-sync failures are warnings so rendering can continue with
cached files.

Unit tests are stored under `tests/unit/` and included by their corresponding
source modules when testing. Shared unit-test utilities are in
`tests/unit/test_support.rs`. End-to-end CLI tests are grouped under `tests/cli/`.

```mermaid
sequenceDiagram
    actor User
    participant CLI as main.rs
    participant Tournament as tournament.json
    participant LegendData as legends.json
    participant Artwork as artwork.rs
    participant Archive as Piltover Archive API
    participant Cache as Artwork cache
    participant Layout as layout.rs
    participant Renderer as renderer.rs
    participant Output as output files

    User->>CLI: Run with CLI arguments
    CLI->>Tournament: Read tournament metadata and counts
    Tournament-->>CLI: Tournament data
    CLI->>LegendData: Read Legend metadata
    LegendData-->>CLI: Legend definitions

    opt Artwork sync enabled (default)
        CLI->>Artwork: Sync positive-count Legends
        Artwork->>Cache: Check for missing image files
        alt All required artwork is cached
            Cache-->>Artwork: No downloads needed
        else Artwork is missing or refresh requested
            Artwork->>Archive: Fetch paginated Legend card variants
            Archive-->>Artwork: Card names, variants, and image URLs
            loop Each required uncached or refreshed Legend
                Artwork->>Archive: Download selected image
                Archive-->>Artwork: Original image bytes
                Artwork->>Cache: Save image unchanged
            end
        end
        Artwork-->>CLI: Sync complete or warning on failure
    end

    CLI->>CLI: Exclude zero-count and unknown Legends, then sort by count
    CLI->>Layout: Build tile positions and sizes
    Layout-->>CLI: Legend tiles
    CLI->>Renderer: Render tournament, tiles, and artwork
    Renderer->>Cache: Read available local artwork
    Cache-->>Renderer: Image files
    Renderer->>Renderer: Crop for display and generate SVG
    Renderer-->>CLI: SVG content
    CLI->>Output: Write SVG
    CLI->>CLI: Parse SVG and rasterize to PNG
    CLI->>Output: Write PNG
    CLI-->>User: Report generated file paths
```
