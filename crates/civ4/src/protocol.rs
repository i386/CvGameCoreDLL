use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const BRIDGE_PROTOCOL_VERSION: u32 = 1;

pub const BRIDGED_QUERY_NAMES: &[&str] = &[
    "can_unit_group_do_command",
    "can_unit_group_start_mission",
    "can_unit_join_group",
    "get_city_building_class_change",
    "get_city_building_state",
    "get_city_corporation_state",
    "get_city_detail_state",
    "get_city_identity_state",
    "get_city_production_options",
    "get_city_religion_state",
    "get_city_state",
    "get_force_control_state",
    "get_game_option_state",
    "get_game_state",
    "get_game_turn",
    "get_info_count",
    "get_info_type",
    "get_map_state",
    "get_mod_state",
    "get_multiplayer_option_state",
    "get_player_economy_state",
    "get_player_gold",
    "get_player_gold_per_turn_state",
    "get_player_identity",
    "get_player_options",
    "get_player_state",
    "get_plot_culture_state",
    "get_plot_state",
    "get_plot_visibility_state",
    "get_selection_group_state",
    "get_team_relation_state",
    "get_team_state",
    "get_team_tech_state",
    "get_unit_detail_state",
    "get_unit_group_state",
    "get_unit_promotion_state",
    "get_unit_state",
    "list_info_types",
    "list_player_cities",
    "list_player_selection_groups",
    "list_player_units",
    "list_players",
    "list_teams",
];

pub const BRIDGED_COMMAND_NAMES: &[&str] = &[
    "change_city_hurry_anger_timer",
    "change_city_occupation_timer",
    "change_city_population",
    "change_city_production",
    "change_game_ai_auto_play",
    "change_game_max_turns",
    "change_game_nukes_exploded",
    "change_player_advanced_start_points",
    "change_player_anarchy_turns",
    "change_player_combat_experience",
    "change_player_commerce_percent",
    "change_player_commerce_rate_modifier",
    "change_player_gold",
    "change_player_gold_per_turn_by_player",
    "change_player_golden_age_turns",
    "change_player_num_unit_golden_ages",
    "change_player_strike_turns",
    "change_plot_culture",
    "change_team_research_progress",
    "change_team_stolen_visibility_timer",
    "change_team_war_weariness",
    "change_unit_damage",
    "change_unit_experience",
    "change_unit_fortify_turns",
    "change_unit_immobile_timer",
    "change_unit_level",
    "change_unit_moves",
    "clear_city_order_queue",
    "clear_unit_group_mission_queue",
    "declare_war",
    "do_unit_group_command",
    "finish_unit_moves",
    "join_unit_group",
    "kill_unit",
    "make_peace",
    "meet_team",
    "pop_city_order",
    "pop_unit_group_mission",
    "push_city_order",
    "push_unit_group_mission",
    "set_city_building_happiness_change",
    "set_city_building_health_change",
    "set_city_building_production",
    "set_city_corporation",
    "set_city_culture",
    "set_city_free_building",
    "set_city_name",
    "set_city_occupation_timer",
    "set_city_population",
    "set_city_production",
    "set_city_project_production",
    "set_city_real_building",
    "set_city_religion",
    "set_city_script_data",
    "set_city_unit_production",
    "set_force_control",
    "set_game_advanced_start_points",
    "set_game_ai_auto_play",
    "set_game_estimate_end_turn",
    "set_game_max_city_elimination",
    "set_game_max_turns",
    "set_game_option",
    "set_game_pause_player",
    "set_game_start_turn",
    "set_game_start_year",
    "set_game_state",
    "set_game_target_score",
    "set_game_turn",
    "set_game_winner",
    "set_mod_state",
    "set_multiplayer_option",
    "set_player_advanced_start_points",
    "set_player_alive",
    "set_player_civic",
    "set_player_combat_experience",
    "set_player_commerce_percent",
    "set_player_current_era",
    "set_player_gold",
    "set_player_gold_per_turn_by_player",
    "set_player_parent",
    "set_player_personality",
    "set_player_playable",
    "set_player_research",
    "set_player_state_religion",
    "set_plot_bonus",
    "set_plot_culture",
    "set_plot_feature",
    "set_plot_improvement",
    "set_plot_owner",
    "set_plot_revealed",
    "set_plot_route",
    "set_plot_terrain",
    "set_team_defensive_pact",
    "set_team_force_peace",
    "set_team_has_tech",
    "set_team_open_borders",
    "set_team_permanent_war_peace",
    "set_team_stolen_visibility_timer",
    "set_team_vassal",
    "set_team_war_weariness",
    "set_unit_base_combat",
    "set_unit_damage",
    "set_unit_experience",
    "set_unit_fortify_turns",
    "set_unit_immobile_timer",
    "set_unit_level",
    "set_unit_made_attack",
    "set_unit_moves",
    "set_unit_promotion",
    "set_unit_xy",
    "spawn_unit",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BridgeCapability {
    Events,
    Queries,
    Commands,
    Callbacks,
    CallbackRequests,
    ModState,
}

impl BridgeCapability {
    pub const FULL_GAMEPLAY: &'static [Self] = &[
        Self::Events,
        Self::Queries,
        Self::Commands,
        Self::Callbacks,
        Self::CallbackRequests,
        Self::ModState,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Events => "events",
            Self::Queries => "queries",
            Self::Commands => "commands",
            Self::Callbacks => "callbacks",
            Self::CallbackRequests => "callback_requests",
            Self::ModState => "mod_state",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "events" => Self::Events,
            "queries" => Self::Queries,
            "commands" => Self::Commands,
            "callbacks" => Self::Callbacks,
            "callback_requests" => Self::CallbackRequests,
            "mod_state" => Self::ModState,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Hello {
        protocol: u32,
        side: String,
        capabilities: Vec<String>,
    },
    Event {
        seq: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    CallbackMirror {
        seq: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    CallbackRequest {
        id: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    Query {
        id: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    Command {
        id: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    Reply {
        id: u64,
        ok: bool,
        #[serde(default)]
        result: Option<Value>,
        #[serde(default)]
        error: Option<BridgeErrorBody>,
    },
    Log {
        level: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeHello {
    pub protocol: u32,
    pub side: String,
    pub capabilities: Vec<String>,
}

impl BridgeHello {
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|available| available == capability)
    }

    pub fn missing_capabilities<'a>(&self, required: &'a [&'a str]) -> Vec<&'a str> {
        required
            .iter()
            .copied()
            .filter(|capability| !self.has_capability(capability))
            .collect()
    }

    pub fn has_bridge_capability(&self, capability: BridgeCapability) -> bool {
        self.has_capability(capability.name())
    }

    pub fn missing_bridge_capabilities(
        &self,
        required: &[BridgeCapability],
    ) -> Vec<BridgeCapability> {
        required
            .iter()
            .copied()
            .filter(|capability| !self.has_bridge_capability(*capability))
            .collect()
    }

    pub fn bridge_capabilities(&self) -> Vec<BridgeCapability> {
        self.capabilities
            .iter()
            .filter_map(|capability| BridgeCapability::from_name(capability))
            .collect()
    }

    pub fn into_message(self) -> Message {
        Message::Hello {
            protocol: self.protocol,
            side: self.side,
            capabilities: self.capabilities,
        }
    }
}

impl Message {
    pub fn into_hello(self) -> Option<BridgeHello> {
        match self {
            Self::Hello {
                protocol,
                side,
                capabilities,
            } => Some(BridgeHello {
                protocol,
                side,
                capabilities,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeReply {
    pub id: u64,
    pub ok: bool,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<BridgeErrorBody>,
}

impl BridgeReply {
    pub fn success(id: u64, result: Value) -> Self {
        Self {
            id,
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: u64, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id,
            ok: false,
            result: None,
            error: Some(BridgeErrorBody {
                code: code.into(),
                message: message.into(),
            }),
        }
    }

    pub fn into_message(self) -> Message {
        Message::Reply {
            id: self.id,
            ok: self.ok,
            result: self.result,
            error: self.error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeErrorBody {
    pub code: String,
    pub message: String,
}

pub fn encode_jsonl(message: &Message) -> serde_json::Result<String> {
    let mut line = serde_json::to_string(message)?;
    line.push('\n');
    Ok(line)
}

pub fn decode_jsonl(line: &str) -> serde_json::Result<Message> {
    serde_json::from_str(line.trim_end_matches(['\r', '\n']))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_event() {
        let msg = Message::Event {
            seq: 1,
            name: "begin_game_turn".to_string(),
            args: serde_json::json!({ "turn": 42 }),
        };
        let line = encode_jsonl(&msg).unwrap();
        let decoded = decode_jsonl(&line).unwrap();
        assert!(matches!(decoded, Message::Event { seq: 1, .. }));
    }

    #[test]
    fn decodes_reply() {
        let decoded =
            decode_jsonl(r#"{"type":"reply","id":7,"ok":true,"result":{"gold":500}}"#).unwrap();
        match decoded {
            Message::Reply { id, ok, .. } => {
                assert_eq!(id, 7);
                assert!(ok);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn decodes_callback_request() {
        let decoded = decode_jsonl(
            r#"{"type":"callback_request","id":9,"name":"kbd_event","args":{"key":65}}"#,
        )
        .unwrap();
        match decoded {
            Message::CallbackRequest { id, name, args } => {
                assert_eq!(id, 9);
                assert_eq!(name, "kbd_event");
                assert_eq!(args["key"], 65);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn hello_reports_capabilities() {
        let hello = decode_jsonl(
            r#"{"type":"hello","protocol":1,"side":"dll","capabilities":["events","queries","callback_requests"]}"#,
        )
        .unwrap()
        .into_hello()
        .unwrap();

        assert_eq!(hello.protocol, BRIDGE_PROTOCOL_VERSION);
        assert_eq!(hello.side, "dll");
        assert!(hello.has_capability("queries"));
        assert!(hello.has_bridge_capability(BridgeCapability::Queries));
        assert_eq!(
            hello.missing_capabilities(&["queries", "commands", "callback_requests"]),
            vec!["commands"]
        );
        assert_eq!(
            hello.missing_bridge_capabilities(&[
                BridgeCapability::Queries,
                BridgeCapability::Commands,
                BridgeCapability::CallbackRequests
            ]),
            vec![BridgeCapability::Commands]
        );
        assert_eq!(
            hello.bridge_capabilities(),
            vec![
                BridgeCapability::Events,
                BridgeCapability::Queries,
                BridgeCapability::CallbackRequests
            ]
        );
        assert_eq!(
            BridgeCapability::FULL_GAMEPLAY
                .iter()
                .map(|capability| capability.name())
                .collect::<Vec<_>>(),
            vec![
                "events",
                "queries",
                "commands",
                "callbacks",
                "callback_requests",
                "mod_state"
            ]
        );
    }

    #[test]
    fn bridged_query_and_command_catalogs_are_stable() {
        assert_eq!(BRIDGED_QUERY_NAMES.len(), 43);
        assert_eq!(BRIDGED_COMMAND_NAMES.len(), 111);
        assert_eq!(BRIDGED_QUERY_NAMES.len() + BRIDGED_COMMAND_NAMES.len(), 154);

        assert_sorted_unique(BRIDGED_QUERY_NAMES);
        assert_sorted_unique(BRIDGED_COMMAND_NAMES);

        for query in BRIDGED_QUERY_NAMES {
            assert!(
                !BRIDGED_COMMAND_NAMES.contains(query),
                "{query} should not be both a query and command"
            );
        }
    }

    fn assert_sorted_unique(names: &[&str]) {
        for window in names.windows(2) {
            assert!(
                window[0] < window[1],
                "{} should sort before {}",
                window[0],
                window[1]
            );
        }
    }
}
