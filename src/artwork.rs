use crate::models::Legend;
use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path, time::Duration};

const API_BASE: &str = "https://piltoverarchive.com/api/external/v1";
const REQUEST_TIMEOUT_SECS: u64 = 30;
const DOWNLOAD_DELAY_MS: u64 = 120;
const MAX_PAGES: u32 = 100;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(default)]
    data: Vec<ApiVariant>,
    #[serde(default)]
    pagination: Option<Pagination>,
}

#[derive(Debug, Deserialize)]
struct Pagination {
    #[serde(default, rename = "totalPages")]
    total_pages: Option<u32>,
    #[serde(default, rename = "hasNext")]
    has_next: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ApiVariant {
    #[serde(default, rename = "imageUrl")]
    image_url: Option<String>,
    #[serde(default, rename = "variantNumber")]
    variant_number: Option<String>,
    #[serde(default)]
    card: Option<ApiCard>,
}

#[derive(Debug, Deserialize)]
struct ApiCard {
    #[serde(default)]
    name: Option<String>,
}

fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn preferred_set(image_name: &str) -> Option<&'static str> {
    let stem = Path::new(image_name).file_stem()?.to_str()?;
    let suffix = stem.rsplit('_').next()?.to_ascii_lowercase();

    match suffix.as_str() {
        "ogn" => Some("OGN"),
        "ogs" => Some("OGS"),
        "sfd" => Some("SFD"),
        "rad" => Some("RAD"),
        "ven" => Some("VEN"),
        "unl" => Some("UNL"),
        _ => None,
    }
}

fn fetch_all_variants(client: &Client) -> Result<Vec<ApiVariant>> {
    let mut all = Vec::new();
    let mut page = 1_u32;

    loop {
        let response = client
            .get(format!("{API_BASE}/cards"))
            .query(&[
                ("types", "legend"),
                ("rarities", "rare"),
                ("artworks", "all"),
                ("limit", "100"),
            ])
            .query(&[("page", page.to_string())])
            .send()
            .with_context(|| format!("Request to Piltover Archive failed (page {page})"))?
            .error_for_status()
            .with_context(|| format!("Piltover Archive returned an error (page {page})"))?;

        let body: ApiResponse = response
            .json()
            .context("Could not parse Piltover Archive JSON response")?;

        let count = body.data.len();
        let pagination = body.pagination;

        all.extend(body.data);

        let has_next = pagination
            .as_ref()
            .and_then(|p| p.has_next)
            .or_else(|| {
                pagination
                    .as_ref()
                    .and_then(|p| p.total_pages)
                    .map(|total| page < total)
            })
            .unwrap_or(count == 100);

        if !has_next || count == 0 {
            break;
        }

        if page >= MAX_PAGES {
            return Err(anyhow!(
                "Safety stop: Piltover Archive returned too many pages"
            ));
        }

        page += 1;
    }

    Ok(all)
}

fn has_http_image(variant: &ApiVariant) -> bool {
    variant
        .image_url
        .as_deref()
        .is_some_and(|url| url.starts_with("https://") || url.starts_with("http://"))
}

fn matches_legend_name(variant: &ApiVariant, full_name: &str, name: &str) -> bool {
    variant
        .card
        .as_ref()
        .and_then(|card| card.name.as_deref())
        .map(normalize)
        .is_some_and(|variant_name| variant_name == full_name || variant_name == name)
}

fn is_preferred_set(variant: &ApiVariant, set: &str) -> bool {
    variant
        .variant_number
        .as_deref()
        .is_some_and(|number| number.to_ascii_uppercase().starts_with(set))
}

fn is_special_variant(variant: &ApiVariant) -> bool {
    variant
        .variant_number
        .as_deref()
        .map(str::to_ascii_uppercase)
        .is_some_and(|number| {
            number.contains("FOIL") || number.contains("NEXUS") || number.contains("SHOWCASE")
        })
}

fn choose_variant<'a>(variants: &'a [ApiVariant], legend: &Legend) -> Option<&'a ApiVariant> {
    let full_name = format!("{}, {}", legend.name, legend.title);
    let full_normalized = normalize(&full_name);
    let name_normalized = normalize(&legend.name);

    let mut matches: Vec<&ApiVariant> = variants
        .iter()
        .filter(|variant| {
            has_http_image(variant)
                && matches_legend_name(variant, &full_normalized, &name_normalized)
        })
        .collect();

    if let Some(set) = preferred_set(&legend.image) {
        if let Some(found) = matches
            .iter()
            .copied()
            .find(|variant| is_preferred_set(variant, set))
        {
            return Some(found);
        }
    }

    matches.sort_by_key(|variant| is_special_variant(variant));

    matches.into_iter().next()
}

pub fn sync_artwork(
    legend_db: &HashMap<String, Legend>,
    tournament_keys: &[String],
    assets_dir: &Path,
    refresh: bool,
) -> Result<()> {
    fs::create_dir_all(assets_dir)
        .with_context(|| format!("Could not create {}", assets_dir.display()))?;

    let needed: Vec<(&String, &Legend)> = tournament_keys
        .iter()
        .filter_map(|key| legend_db.get(key).map(|legend| (key, legend)))
        .filter(|(_, legend)| refresh || !assets_dir.join(&legend.image).is_file())
        .collect();

    if needed.is_empty() {
        println!("Artwork cache is complete for all Legends in this tournament.");
        return Ok(());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .user_agent("RiftboundMetaGenerator/0.2 (artwork cache)")
        .build()
        .context("Could not create HTTP client")?;

    println!("Fetching Legend card records from Piltover Archive...");

    let variants = fetch_all_variants(&client)?;
    let mut downloaded = 0_usize;
    let mut unmatched = 0_usize;

    for (key, legend) in needed {
        let target = assets_dir.join(&legend.image);

        let Some(variant) = choose_variant(&variants, legend) else {
            eprintln!(
                "Warning: no matching Piltover Archive artwork found for {} ({key}).",
                legend.name
            );
            unmatched += 1;
            continue;
        };

        let url = variant
            .image_url
            .as_deref()
            .ok_or_else(|| anyhow!("Matched variant has no image URL"))?;

        let response = client
            .get(url)
            .send()
            .with_context(|| format!("Failed downloading artwork for {}", legend.name))?
            .error_for_status()
            .with_context(|| format!("Artwork server returned an error for {}", legend.name))?;

        let bytes = response
            .bytes()
            .with_context(|| format!("Could not read artwork bytes for {}", legend.name))?;

        if bytes.is_empty() {
            eprintln!("Warning: empty artwork for {}; skipping.", legend.name);
            unmatched += 1;
            continue;
        }

        let tmp = target.with_extension("download.tmp");

        fs::write(&tmp, &bytes).with_context(|| format!("Could not write {}", tmp.display()))?;

        fs::rename(&tmp, &target)
            .with_context(|| format!("Could not cache artwork at {}", target.display()))?;

        downloaded += 1;

        println!("Downloaded {} -> {}", legend.name, target.display());

        std::thread::sleep(Duration::from_millis(DOWNLOAD_DELAY_MS));
    }

    println!("Artwork sync complete: {downloaded} image(s) downloaded");
    println!("{unmatched} unmatched or empty image(s).\n");

    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/artwork.rs"]
mod tests;
