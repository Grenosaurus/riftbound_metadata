# Riftbound Meta Generator (Rust) — Piltover Archive artwork sync

This program generates a 1920×1080 PNG and matching SVG from local tournament data. Before rendering, it can download missing Legend artwork from Piltover Archive and cache the files in `assets/legends/`.

## Requirements

- Rust and Cargo: https://www.rust-lang.org/tools/install
- Internet access on the first run to fetch missing artwork (optional after the cache is populated)

## Run

From the project directory:

```bash
cargo run
```

The program reads `data/tournament.json` and `data/legends.json`, downloads missing art for Legends in the tournament, and writes `output/meta.png` and `output/meta.svg`.

## Options

```bash
# Choose input files
cargo run -- data/tournament.json --legends data/legends.json

# Choose output paths
cargo run -- --output output/my_meta.png --svg output/my_meta.svg

# Render offline, using only artwork already in assets/legends
cargo run -- --skip-artwork

# Force re-download of artwork used in this tournament
cargo run -- --refresh-artwork

# Custom asset folder
cargo run -- --assets assets/legends
```

## How artwork sync works

1. The app reads the tournament's Legend keys.
2. It requests pages from `https://piltoverarchive.com/api/external/v1/cards`, filtering for Legend cards and artwork.
3. It matches the returned card name to the full local name (`name` + `title`). If the local image filename ends with a set suffix such as `_ogn`, `_ven`, `_sfd`, `_rad`, `_unl`, or `_ogs`, it prefers a printing from that set.
4. It downloads the chosen `imageUrl` to the filename already defined in `data/legends.json`.
5. Cached images are reused on later runs. `--refresh-artwork` forces re-downloads.

The API is publicly accessible according to community reverse-engineered documentation, but it is not an officially published/stable API and may change. The program uses no API key. Requests are paginated (limit 100), the API response is cached indirectly through local image files, and a small delay is used between image downloads. Check Piltover Archive's terms and Riot Games' applicable policies before redistributing card art.

If the API is unavailable, the program warns and continues with any local artwork already present. Missing images render as blank tile backgrounds. If a card is not matched automatically, check its spelling/title and the archive's card record.

## Data files

- `data/tournament.json`: tournament title and counts keyed by the IDs in `data/legends.json`.
- `data/legends.json`: local display metadata, target image filenames, and faction names/colors.

Counts are entered by you; they are **not** fetched from Piltover Archive. The archive integration is for artwork only.

## Project structure

```text
riftbound_meta_rust/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── models.rs
│   ├── layout.rs
│   ├── renderer.rs
│   └── artwork.rs
├── data/
│   ├── tournament.json
│   └── legends.json
├── assets/
│   ├── legends/
│   └── logos/
└── output/
```
