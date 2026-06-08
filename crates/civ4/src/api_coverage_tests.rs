use crate::protocol::{BRIDGED_COMMAND_NAMES, BRIDGED_QUERY_NAMES};
use std::collections::BTreeSet;

const API_SOURCES: &[&str] = &[
    include_str!("city_api.rs"),
    include_str!("game_api.rs"),
    include_str!("info_api.rs"),
    include_str!("mod_state_api.rs"),
    include_str!("player_api.rs"),
    include_str!("plot_api.rs"),
    include_str!("selection_group_api.rs"),
    include_str!("team_api.rs"),
    include_str!("unit_api.rs"),
];

#[test]
fn typed_query_api_covers_every_bridged_query() {
    assert_catalog_coverage(
        "query",
        BRIDGED_QUERY_NAMES,
        &collect_api_call_names(&["query"]),
    );
}

#[test]
fn typed_command_api_covers_every_bridged_command() {
    assert_catalog_coverage(
        "command",
        BRIDGED_COMMAND_NAMES,
        &collect_api_call_names(&["command", "command_plot_state"]),
    );
}

fn assert_catalog_coverage(kind: &str, catalog: &[&str], api_names: &BTreeSet<String>) {
    let catalog_names: BTreeSet<String> = catalog.iter().map(|name| name.to_string()).collect();
    let missing: Vec<&String> = catalog_names.difference(api_names).collect();
    let unexpected: Vec<&String> = api_names.difference(&catalog_names).collect();

    assert!(
        missing.is_empty(),
        "bridged {kind}s missing typed Rust API wrappers: {missing:?}"
    );
    assert!(
        unexpected.is_empty(),
        "typed Rust API references unknown bridged {kind}s: {unexpected:?}"
    );
}

fn collect_api_call_names(call_names: &[&str]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();

    for source in API_SOURCES {
        for call_name in call_names {
            collect_source_call_names(source, call_name, &mut names);
        }
    }

    names
}

fn collect_source_call_names(source: &str, call_name: &str, names: &mut BTreeSet<String>) {
    let marker = format!(".{call_name}(");
    let mut offset = 0;

    while let Some(relative_start) = source[offset..].find(&marker) {
        let args_start = offset + relative_start + marker.len();
        offset = args_start;

        let literal_start = source[args_start..]
            .char_indices()
            .find(|(_, character)| !character.is_whitespace());
        let Some((literal_start, '"')) = literal_start else {
            continue;
        };
        let name_start = args_start + literal_start + 1;
        let Some(name_end) = source[name_start..].find('"') else {
            continue;
        };

        names.insert(source[name_start..name_start + name_end].to_string());
    }
}
