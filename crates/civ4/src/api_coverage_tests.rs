use crate::event_kind::BridgeEventKind;
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

const BRIDGE_EVENT_CPP_SOURCES: &[&str] = &[
    include_str!("../../../CvGameCoreDLL/CvCity.cpp"),
    include_str!("../../../CvGameCoreDLL/CvEventReporter.cpp"),
    include_str!("../../../CvGameCoreDLL/CyGame.cpp"),
    include_str!("../../../CvGameCoreDLL/CvPlayer.cpp"),
    include_str!("../../../CvGameCoreDLL/CvPlayerAI.cpp"),
    include_str!("../../../CvGameCoreDLL/CvPlot.cpp"),
    include_str!("../../../CvGameCoreDLL/CvTeam.cpp"),
    include_str!("../../../CvGameCoreDLL/CvUnit.cpp"),
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

#[test]
fn bridged_event_catalog_covers_cpp_emitters_and_callbacks() {
    let catalog_names: BTreeSet<String> = BridgeEventKind::BRIDGED_NAMES
        .iter()
        .map(|name| name.to_string())
        .collect();
    let cpp_names = collect_cpp_bridge_event_names();

    let missing: Vec<&String> = cpp_names.difference(&catalog_names).collect();
    let stale: Vec<&String> = catalog_names.difference(&cpp_names).collect();

    assert!(
        missing.is_empty(),
        "C++ bridge event/callback names missing from Rust catalog: {missing:?}"
    );
    assert!(
        stale.is_empty(),
        "Rust event catalog names not emitted or requested by C++ bridge: {stale:?}"
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

fn collect_cpp_bridge_event_names() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let call_markers = [
        "bridgePayload(",
        "bridgeSignal(",
        "sendEvent(",
        "sendCallbackMirror(",
        "requestCallbackBool(",
        "requestCallbackConsume(",
        "requestCallbackConsumeTimeout(",
        "requestCallbackInt(",
        "requestCallbackText(",
        "requestCallbackTextTimeout(",
    ];

    for source in BRIDGE_EVENT_CPP_SOURCES {
        for marker in call_markers {
            collect_call_names(source, marker, &mut names);
        }
    }

    names
}

fn collect_source_call_names(source: &str, call_name: &str, names: &mut BTreeSet<String>) {
    let marker = format!(".{call_name}(");
    collect_call_names(source, &marker, names);
}

fn collect_call_names(source: &str, marker: &str, names: &mut BTreeSet<String>) {
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
