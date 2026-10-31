use super::*;
use crate::test_support::TestDir;

fn legend(image: &str) -> Legend {
    Legend {
        name: "Ahri".to_owned(),
        title: "The Fox".to_owned(),
        image: image.to_owned(),
        faction: "Ionia".to_owned(),
    }
}

fn variant(name: Option<&str>, number: Option<&str>, url: Option<&str>) -> ApiVariant {
    ApiVariant {
        image_url: url.map(str::to_owned),
        variant_number: number.map(str::to_owned),
        card: name.map(|name| ApiCard {
            name: Some(name.to_owned()),
        }),
    }
}

fn select_variant<'a>(variants: &'a [ApiVariant], legend: &Legend) -> Option<&'a ApiVariant> {
    choose_variant(&index_variants(variants), legend)
}

#[test]
fn normalize_ignores_punctuation_and_case() {
    assert_eq!(normalize("Ahri, The-Fox!"), "ahrithefox");
    assert_eq!(normalize("Ékko"), "ékko");
}

#[test]
fn preferred_set_recognizes_supported_codes_case_insensitively() {
    for (code, expected) in [
        ("ogn", "OGN"),
        ("ogs", "OGS"),
        ("sfd", "SFD"),
        ("rad", "RAD"),
        ("ven", "VEN"),
        ("unl", "UNL"),
    ] {
        assert_eq!(preferred_set(&format!("legend_{code}.png")), Some(expected));
    }
    assert_eq!(preferred_set("legend_unknown.png"), None);
    assert_eq!(preferred_set("legend.png"), None);
}

#[test]
fn choose_variant_prefers_the_requested_set() {
    let variants = [
        variant(Some("Ahri, The Fox"), Some("OGS-001"), Some("https://ogs")),
        variant(Some("Ahri, The Fox"), Some("OGN-001"), Some("https://ogn")),
    ];

    let selected = select_variant(&variants, &legend("ahri_ogn.png")).unwrap();
    assert_eq!(selected.image_url.as_deref(), Some("https://ogn"));
}

#[test]
fn choose_variant_accepts_name_only_and_prefers_standard_artwork() {
    let variants = [
        variant(Some("Ahri"), Some("OGN-FOIL"), Some("https://foil")),
        variant(Some("Ahri"), Some("OGN-001"), Some("https://standard")),
    ];

    let selected = select_variant(&variants, &legend("ahri_other.png")).unwrap();
    assert_eq!(selected.image_url.as_deref(), Some("https://standard"));
}

#[test]
fn choose_variant_preserves_api_order_across_full_and_name_only_matches() {
    let variants = [
        variant(
            Some("Ahri"),
            Some("OGN-FOIL"),
            Some("https://name-only-special"),
        ),
        variant(
            Some("Ahri, The Fox"),
            Some("OGN-001"),
            Some("https://full-name-standard"),
        ),
    ];

    let selected = select_variant(&variants, &legend("ahri_other.png")).unwrap();
    assert_eq!(
        selected.image_url.as_deref(),
        Some("https://full-name-standard")
    );
}

#[test]
fn choose_variant_rejects_nonmatching_names_and_non_http_urls() {
    let variants = [
        variant(
            Some("Different Legend"),
            Some("OGN-001"),
            Some("https://image"),
        ),
        variant(Some("Ahri"), Some("OGN-001"), Some("file:///image")),
        variant(Some("Ahri"), Some("OGN-001"), None),
    ];

    assert!(select_variant(&variants, &legend("ahri_ogn.png")).is_none());
}

#[test]
fn choose_variant_prefers_standard_over_nexus_and_showcase() {
    for special in ["OGN-NEXUS", "OGN-SHOWCASE"] {
        let variants = [
            variant(Some("Ahri"), Some(special), Some("https://special")),
            variant(Some("Ahri"), Some("OGN-001"), Some("https://standard")),
        ];

        let selected = select_variant(&variants, &legend("ahri_other.png")).unwrap();
        assert_eq!(selected.image_url.as_deref(), Some("https://standard"));
    }
}

#[test]
fn matching_set_is_preferred_even_when_it_is_a_special_variant() {
    let variants = [
        variant(Some("Ahri"), Some("OGS-001"), Some("https://standard")),
        variant(Some("Ahri"), Some("OGN-FOIL"), Some("https://preferred")),
    ];

    let selected = select_variant(&variants, &legend("ahri_ogn.png")).unwrap();
    assert_eq!(selected.image_url.as_deref(), Some("https://preferred"));
}

#[test]
fn api_response_deserializes_pagination_and_optional_variant_fields() {
    let response: ApiResponse = serde_json::from_str(
        r#"{"data":[{"imageUrl":"https://image","variantNumber":"OGN-1","card":{"name":"Ahri"}},{"card":{}}],"pagination":{"totalPages":2,"hasNext":true}}"#,
    )
    .unwrap();

    assert_eq!(response.data.len(), 2);
    assert_eq!(response.data[0].image_url.as_deref(), Some("https://image"));
    assert_eq!(
        response
            .pagination
            .as_ref()
            .and_then(|page| page.total_pages),
        Some(2)
    );
    assert_eq!(
        response.pagination.as_ref().and_then(|page| page.has_next),
        Some(true)
    );
    assert!(response.data[1].image_url.is_none());
}

#[test]
fn api_response_defaults_missing_fields() {
    let response: ApiResponse = serde_json::from_str("{}").unwrap();
    assert!(response.data.is_empty());
    assert!(response.pagination.is_none());
}

#[test]
fn artwork_sync_skips_network_when_cache_is_complete() {
    let dir = TestDir::new();
    let assets = dir.0.join("assets");
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("ahri.png"), b"cached").unwrap();
    let mut legends = HashMap::new();
    legends.insert("ahri".to_owned(), legend("ahri.png"));

    sync_artwork(
        &legends,
        &HashMap::from([("ahri".to_owned(), 1)]),
        &assets,
        false,
    )
    .unwrap();
    assert_eq!(fs::read(assets.join("ahri.png")).unwrap(), b"cached");
}

#[test]
fn artwork_sync_ignores_zero_count_legends() {
    let dir = TestDir::new();
    let assets = dir.0.join("assets");
    let mut legends = HashMap::new();
    legends.insert("ahri".to_owned(), legend("ahri.png"));

    sync_artwork(
        &legends,
        &HashMap::from([("ahri".to_owned(), 0)]),
        &assets,
        false,
    )
    .unwrap();
    assert!(assets.is_dir());
    assert!(!assets.join("ahri.png").exists());
}

#[test]
fn artwork_sync_reports_directory_creation_errors() {
    let dir = TestDir::new();
    let not_a_directory = dir.0.join("file");
    fs::write(&not_a_directory, b"file").unwrap();

    let error =
        sync_artwork(&HashMap::new(), &HashMap::new(), &not_a_directory, false).unwrap_err();
    assert!(error.to_string().contains("Could not create"));
}
