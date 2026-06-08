# Civ4 Bridge Protocol

The bridge is disabled by default. Set `CVGAME_BRIDGE=1` before launching the game to enable it.

Default pipes:

```text
\\.\pipe\CvGameCoreDLL-Control
\\.\pipe\CvGameCoreDLL-Callbacks
```

Set `CVGAME_BRIDGE_PIPE_PREFIX=Name` to use:

```text
\\.\pipe\Name-Control
\\.\pipe\Name-Callbacks
```

Each message is one JSON object followed by `\n`.

## Companion Autolaunch

Set `CVGAME_BRIDGE_AUTOLAUNCH=1` with `CVGAME_BRIDGE=1` to let the DLL launch a
Rust companion process after it creates the bridge pipes. This ports the old
companion launch hook onto the new bridge direction: the DLL owns the named pipe
server, launches the companion, and the companion connects back as a bridge
client.

Executable discovery order:

```text
%CVGAME_BRIDGE_COMPANION_EXE%
CvGameCoreDLL.dll directory\CvGameBridgeCompanion.exe
CvGameCoreDLL.dll directory\AgesBeyondCompanion.exe
CvGameCoreDLL.dll directory\..\Companion\CvGameBridgeCompanion.exe
CvGameCoreDLL.dll directory\..\Companion\AgesBeyondCompanion.exe
CvGameCoreDLL.dll directory\..\CvGameBridgeCompanion.exe
CvGameCoreDLL.dll directory\..\AgesBeyondCompanion.exe
```

Optional knobs:

```text
CVGAME_BRIDGE_COMPANION_EXE=C:\path\to\Companion.exe
CVGAME_BRIDGE_COMPANION_ARGS=--some --companion --flags
```

The launched process inherits the game environment. Rust companions should use
`BridgeClient::connect_from_env_with_handshake()` so they respect
`CVGAME_BRIDGE_PIPE_PREFIX`, `CVGAME_BRIDGE_CONTROL_PIPE`, and
`CVGAME_BRIDGE_CALLBACK_PIPE`.

Rust clients should perform the hello handshake before registering gameplay behavior:

```rust
use civ4::{BridgeClient, Result};

fn connect() -> Result<BridgeClient> {
    let (client, hello) = BridgeClient::connect_from_env_with_handshake()?;
    let missing = hello.missing_capabilities(&[
        "events",
        "queries",
        "commands",
        "callbacks",
        "callback_requests",
        "mod_state",
    ]);
    if !missing.is_empty() {
        return Err(civ4::BridgeError::Protocol(format!(
            "bridge is missing capabilities: {}",
            missing.join(", ")
        )));
    }
    Ok(client)
}
```

## Messages

```json
{"type":"hello","protocol":1,"side":"dll","capabilities":["events","queries","commands","callbacks","callback_requests","mod_state"]}
{"type":"event","seq":1,"name":"begin_game_turn","args":{"turn":42}}
{"type":"callback_mirror","seq":2,"name":"city_built","args":{"player":0,"city":3,"x":10,"y":12}}
{"type":"callback_request","id":200,"name":"kbd_event","args":{"evt":6,"key":65,"cursor_x":100,"cursor_y":120,"x":10,"y":12}}
{"type":"query","id":100,"name":"get_player_gold","args":{"player":0}}
{"type":"command","id":101,"name":"set_player_gold","args":{"player":0,"value":500}}
{"type":"reply","id":101,"ok":true,"result":{"gold":500}}
{"type":"reply","id":101,"ok":false,"error":{"code":"bad_player","message":"player is missing or out of range"}}
```

## Queries

```text
get_game_turn -> {"turn":42}
get_game_state -> {"turn":42,"year":1000,"elapsed_turns":40,"start_turn":0,"start_year":-4000,"estimate_end_turn":500,"max_turns":460,"max_city_elimination":0,"advanced_start_points":0,"target_score":0,"active_player":0,"active_team":0,"pause_player":-1,"paused":false,"winner":-1,"victory":-1,"game_state":0,"start_era":0,"current_era":1,"calendar":0,"game_speed":2,"handicap":3,"num_cities":12,"num_civ_cities":11,"total_population":42,"num_human_players":1,"num_deals":2,"nukes_exploded":0,"ai_auto_play":0,"network_multiplayer":false,"game_multiplayer":false,"team_game":false,"debug_mode":false,"final_initialized":true}
get_info_count {"kind":"unit"} -> {"kind":"unit","count":128}
get_info_type {"kind":"unit","value":"UNIT_WARRIOR"} -> {"kind":"unit","id":1,"type":"UNIT_WARRIOR"}
list_info_types {"kind":"tech"} -> {"kind":"tech","types":[{"id":0,"type":"TECH_AGRICULTURE"},{"id":1,"type":"TECH_MINING"}]}
get_game_option_state {"option":"GAMEOPTION_NO_BARBARIANS"} -> {"option":0,"enabled":false}
get_multiplayer_option_state {"option":"MPOPTION_SIMULTANEOUS_TURNS"} -> {"option":0,"enabled":false}
get_force_control_state {"control":"FORCECONTROL_SPEED"} -> {"control":0,"enabled":false}
get_player_gold {"player":0} -> {"gold":500}
get_player_state {"player":0} -> {"player":0,"team":0,"alive":true,"ever_alive":true,"human":true,"barbarian":false,"minor":false,"playable":true,"founded_first_city":true,"extended_game":false,"turn_active":true,"turn_done":false,"end_turn":false,"auto_moves":false,"strike":false,"handicap":3,"civilization":1,"leader":2,"personality":2,"current_era":1,"parent":-1,"player_color":4,"gold":500,"cities":3,"units":8,"population":12}
list_players -> {"players":[player state, ...]}
get_player_options {"player":0} -> {"player":0,"team":0,"state_religion":-1,"current_research":3,"civics":[1,2,3,4,5]}
get_player_economy_state {"player":0} -> {"player":0,"gold":500,"gold_per_turn":10,"advanced_start_points":-1,"golden_age_turns":0,"golden_age_length":8,"golden_age":false,"num_unit_golden_ages":0,"units_required_for_golden_age":2,"units_golden_age_ready":1,"anarchy_turns":0,"anarchy":false,"strike_turns":0,"strike":false,"combat_experience":4,"gold_per_unit":1,"gold_per_military_unit":1,"total_culture":100,"commerce_percent":[0,80,20,0],"commerce_rate":[10,40,5,0],"commerce_rate_modifier":[0,25,0,0]}
get_player_gold_per_turn_state {"player":0,"other_player":1} -> {"player":0,"other_player":1,"value":-3}
get_team_state {"team":0} -> {"team":0,"alive":true,"ever_alive":true,"human":true,"barbarian":false,"minor":false,"leader":0,"secretary":0,"members":1,"cities":3,"population":12,"land":40,"assets":500,"power":120,"defensive_power":100,"at_war_count":0,"has_met_count":2,"defensive_pact_count":0,"vassal_count":0,"vassal":false,"nuke_interception":0,"map_trading":true,"tech_trading":true,"gold_trading":true,"open_borders_trading":true,"defensive_pact_trading":false,"permanent_alliance_trading":false,"vassal_trading":false}
list_teams -> {"teams":[team state, ...]}
get_team_tech_state {"team":0,"tech":"TECH_BRONZE_WORKING"} -> {"team":0,"tech":7,"has":true,"progress":0}
get_team_relation_state {"team":0,"other_team":1} -> {"team":0,"other_team":1,"has_met":true,"at_war":false,"can_declare_war":true,"can_change_war_peace":true,"permanent_war_peace":false,"open_borders":true,"defensive_pact":false,"force_peace":false,"vassal":false,"master":false,"war_weariness":0,"stolen_visibility_timer":0,"war_plan":-1}
get_map_state -> {"width":84,"height":52,"plots":4368,"land_plots":1472}
get_plot_state {"x":10,"y":12} -> {"x":10,"y":12,"owner":0,"terrain":1,"feature":-1,"bonus":-1,"improvement":2,"route":1,"water":false,"peak":false,"units":1,"city_player":0,"city":3}
get_plot_culture_state {"x":10,"y":12,"player":0} -> {"x":10,"y":12,"player":0,"culture":42,"total_culture":50,"culture_percent":84}
get_plot_visibility_state {"x":10,"y":12,"team":0} -> {"x":10,"y":12,"team":0,"debug":false,"visible":true,"revealed":true,"revealed_owner":0,"revealed_team":0,"revealed_improvement":2,"revealed_route":1}
get_city_state {"player":0,"city":3} -> {"player":0,"city":3,"x":10,"y":12,"population":5,"culture":42,"production":10,"production_needed":35,"production_unit":0,"production_unit_ai":2,"production_building":-1,"production_project":-1,"production_process":-1,"order_queue_length":1,"occupation_timer":0,"hurry_anger_timer":0}
get_city_detail_state {"player":0,"city":3} -> {"player":0,"city":3,"x":10,"y":12,"production":true,"food_production":false,"disorder":false,"occupation":false,"we_love_the_king_day":false,"food":12,"food_kept":4,"growth_threshold":26,"food_consumption":8,"food_difference":3,"happy_level":7,"unhappy_level":5,"angry_population":0,"good_health":6,"bad_health":4,"health_rate":0,"unhealthy_population":0,"maintenance":3,"distance_maintenance":1,"num_cities_maintenance":2,"colony_maintenance":0,"corporation_maintenance":0,"production_left":12,"current_production_difference":5,"defense_damage":0,"total_defense":40,"defense_modifier":40,"yield_rate":[11,8,12],"commerce_rate":[6,14,2,0],"commerce_rate_times100":[600,1400,200,0]}
get_city_production_options {"player":0,"city":3,"continue_current":0,"test_visible":0,"ignore_cost":0,"ignore_upgrades":0} -> {"player":0,"city":3,"continue_current":false,"test_visible":false,"ignore_cost":false,"ignore_upgrades":false,"units":[0,1],"buildings":[12],"projects":[],"processes":[2]}
get_city_building_state {"player":0,"city":3,"building":"BUILDING_GRANARY"} -> {"player":0,"city":3,"building":12,"real":1,"free":0,"active":true}
get_city_religion_state {"player":0,"city":3,"religion":"RELIGION_BUDDHISM"} -> {"player":0,"city":3,"religion":0,"has":true}
get_city_corporation_state {"player":0,"city":3,"corporation":"CORPORATION_SID_SUSHI"} -> {"player":0,"city":3,"corporation":0,"has":false}
get_city_building_class_change {"player":0,"city":3,"building_class":"BUILDINGCLASS_GRANARY"} -> {"player":0,"city":3,"building_class":12,"happiness":0,"health":0}
list_player_cities {"player":0} -> {"player":0,"cities":[city state, ...]}
get_unit_state {"player":0,"unit":123} -> {"player":0,"unit":123,"unit_type":0,"unit_ai":2,"domain":0,"x":10,"y":12,"damage":0,"experience":2,"level":1,"moves":0,"max_moves":2,"base_combat":3,"cargo":0,"fortify_turns":0,"immobile_timer":0,"made_attack":false,"promotions":[1,4]}
get_unit_detail_state {"player":0,"unit":123} -> {"player":0,"unit":123,"unit_type":0,"unit_ai":2,"domain":0,"unit_combat":1,"special_unit":-1,"x":10,"y":12,"area":5,"group":9,"in_group":true,"group_head":true,"base_moves":2,"max_moves":4,"moves_left":2,"can_move":true,"has_moved":false,"visibility_range":1,"air_range":0,"nuke_range":-1,"can_build_route":false,"build_type":-1,"work_rate":0,"max_work_rate":0,"can_fight":true,"can_attack":true,"can_defend":true,"fighting":false,"attacking":false,"defending":false,"combat":false,"hurt":false,"dead":false,"max_hit_points":100,"curr_hit_points":100,"base_combat":3,"curr_combat":300,"combat_limit":100,"air_combat_limit":100,"fortify_modifier":0,"experience_needed":2,"attack_xp_value":4,"defense_xp_value":2,"max_xp_value":10,"special_cargo":-1,"domain_cargo":-1,"cargo":0,"cargo_space":0,"cargo_space_available":0,"has_cargo":false,"full":false,"cargo_can_move":true,"automated":false,"waiting":false,"fortifyable":true,"made_interception":false,"promotion_ready":false,"animal":false,"only_defensive":false,"rival_territory":false,"military_happiness":true,"spy":false,"found":false,"golden_age":false,"last_move_turn":41,"game_turn_created":1,"experience_percent":0}
get_unit_group_state {"player":0,"unit":123} -> {"player":0,"group":9,"team":0,"x":10,"y":12,"area":5,"domain":0,"head_player":0,"head_unit":123,"head_unit_type":0,"activity":0,"automate":-1,"automated":false,"mission_timer":0,"units":1,"cargo":0,"base_moves":1,"can_all_move":true,"can_any_move":true,"has_moved":false,"waiting":false,"full":false,"has_cargo":false,"can_fight":true,"can_defend":true,"has_worker":false,"ready_to_select":true,"ready_to_move":true,"ready_to_auto":false,"mission_queue_length":1,"missions":[{"mission":1,"data1":11,"data2":12}]}
get_selection_group_state {"player":0,"group":9} -> selection group state
list_player_selection_groups {"player":0} -> {"player":0,"groups":[selection group state, ...]}
can_unit_group_start_mission {"player":0,"unit":123,"mission":"MISSION_MOVE_TO","data1":11,"data2":12,"x":11,"y":12} -> {"player":0,"group":9,"mission":1,"data1":11,"data2":12,"x":11,"y":12,"test_visible":false,"use_cache":false,"can_start":true}
can_unit_group_do_command {"player":0,"unit":123,"command":"load_unit","data1":0,"data2":456} -> {"player":0,"group":9,"command":10,"data1":0,"data2":456,"test_visible":false,"use_cache":false,"can_do":true}
can_unit_join_group {"player":0,"unit":123,"head_player":0,"head_unit":456} -> {"player":0,"unit":123,"group":9,"head_player":0,"head_unit":456,"target_group":10,"split":false,"can_join":true}
get_unit_promotion_state {"player":0,"unit":123,"promotion":"PROMOTION_COMBAT1"} -> {"player":0,"unit":123,"promotion":1,"has":true}
list_player_units {"player":0} -> {"player":0,"units":[unit state, ...]}
get_mod_state -> {"json":"{\"schema_version\":1}"}
```

## Commands

Commands are rejected in multiplayer in this first version.

```text
set_game_turn {"value":42} -> game state
set_game_max_turns {"value":500} -> game state
change_game_max_turns {"change":10} -> game state
set_game_start_turn {"value":0} -> game state
set_game_start_year {"value":-4000} -> game state
set_game_estimate_end_turn {"value":500} -> game state
set_game_target_score {"value":0} -> game state
set_game_max_city_elimination {"value":0} -> game state
set_game_advanced_start_points {"value":0} -> game state
set_game_ai_auto_play {"value":0} -> game state
change_game_ai_auto_play {"change":-1} -> game state
change_game_nukes_exploded {"change":1} -> game state
set_game_pause_player {"player":0} -> game state
set_game_winner {"team":0,"victory":"VICTORY_CONQUEST"} -> game state
set_game_state {"value":"extended"} -> game state
set_game_option {"option":"GAMEOPTION_NO_BARBARIANS","enabled":1} -> game option state
set_multiplayer_option {"option":"MPOPTION_SIMULTANEOUS_TURNS","enabled":0} -> multiplayer option state
set_force_control {"control":"FORCECONTROL_SPEED","enabled":1} -> force control state
set_player_gold {"player":0,"value":500} -> {"gold":500}
change_player_gold {"player":0,"change":50} -> player state
set_player_alive {"player":0,"alive":true} -> player state
set_player_playable {"player":0,"playable":true} -> player state
set_player_current_era {"player":0,"era":"ERA_CLASSICAL"} -> player state
set_player_personality {"player":0,"leader":"LEADER_GANDHI"} -> player state
set_player_parent {"player":0,"parent":-1} -> player state
set_player_advanced_start_points {"player":0,"value":100} -> player economy state
change_player_advanced_start_points {"player":0,"change":-10} -> player economy state
change_player_golden_age_turns {"player":0,"change":8} -> player economy state
change_player_num_unit_golden_ages {"player":0,"change":1} -> player economy state
change_player_anarchy_turns {"player":0,"change":1} -> player economy state
change_player_strike_turns {"player":0,"change":1} -> player economy state
set_player_combat_experience {"player":0,"value":5} -> player economy state
change_player_combat_experience {"player":0,"change":1} -> player economy state
set_player_commerce_percent {"player":0,"commerce":"research","value":80} -> player economy state
change_player_commerce_percent {"player":0,"commerce":"culture","change":10} -> player economy state
change_player_commerce_rate_modifier {"player":0,"commerce":"research","change":25} -> player economy state
set_player_gold_per_turn_by_player {"player":0,"other_player":1,"value":-3} -> player gold-per-turn state
change_player_gold_per_turn_by_player {"player":0,"other_player":1,"change":1} -> player gold-per-turn state
set_city_population {"player":0,"city":3,"value":6} -> city state
change_city_population {"player":0,"city":3,"change":1} -> city state
set_city_culture {"player":0,"city":3,"culture_player":0,"value":100} -> city state
set_city_production {"player":0,"city":3,"value":20} -> city state
change_city_production {"player":0,"city":3,"change":5} -> city state
set_city_unit_production {"player":0,"city":3,"unit_type":"UNIT_WARRIOR","value":10} -> city state
set_city_building_production {"player":0,"city":3,"building_type":"BUILDING_GRANARY","value":20} -> city state
set_city_project_production {"player":0,"city":3,"project_type":"PROJECT_APOLLO_PROGRAM","value":100} -> city state
push_city_order {"player":0,"city":3,"order":"train","data1":"UNIT_WARRIOR","append":1} -> city state
clear_city_order_queue {"player":0,"city":3} -> city state
pop_city_order {"player":0,"city":3,"index":0} -> city state
set_city_occupation_timer {"player":0,"city":3,"value":2} -> city state
change_city_occupation_timer {"player":0,"city":3,"change":-1} -> city state
change_city_hurry_anger_timer {"player":0,"city":3,"change":5} -> city state
set_city_real_building {"player":0,"city":3,"building":"BUILDING_GRANARY","value":1} -> city building state
set_city_free_building {"player":0,"city":3,"building":"BUILDING_GRANARY","value":1} -> city building state
set_city_religion {"player":0,"city":3,"religion":"RELIGION_BUDDHISM","has":1} -> city religion state
set_city_corporation {"player":0,"city":3,"corporation":"CORPORATION_SID_SUSHI","has":1} -> city corporation state
set_city_building_happiness_change {"player":0,"city":3,"building_class":"BUILDINGCLASS_GRANARY","value":1} -> city building class change
set_city_building_health_change {"player":0,"city":3,"building_class":"BUILDINGCLASS_GRANARY","value":1} -> city building class change
set_plot_owner {"x":10,"y":12,"owner":0} -> plot state
set_plot_terrain {"x":10,"y":12,"terrain":"TERRAIN_GRASS"} -> plot state
set_plot_feature {"x":10,"y":12,"feature":"FEATURE_FOREST"} -> plot state
set_plot_bonus {"x":10,"y":12,"bonus":"BONUS_COPPER"} -> plot state
set_plot_improvement {"x":10,"y":12,"improvement":"IMPROVEMENT_MINE"} -> plot state
set_plot_route {"x":10,"y":12,"route":"ROUTE_ROAD"} -> plot state
set_plot_culture {"x":10,"y":12,"player":0,"value":20} -> plot state
change_plot_culture {"x":10,"y":12,"player":0,"change":5} -> plot state
set_plot_revealed {"x":10,"y":12,"team":0,"revealed":1} -> plot state
set_unit_damage {"player":0,"unit":123,"value":25} -> unit state
change_unit_damage {"player":0,"unit":123,"change":-10} -> unit state
set_unit_experience {"player":0,"unit":123,"value":5} -> unit state
change_unit_experience {"player":0,"unit":123,"change":1} -> unit state
set_unit_xy {"player":0,"unit":123,"x":11,"y":12} -> unit state
set_unit_moves {"player":0,"unit":123,"value":0} -> unit state
change_unit_moves {"player":0,"unit":123,"change":60} -> unit state
finish_unit_moves {"player":0,"unit":123} -> unit state
set_unit_level {"player":0,"unit":123,"value":2} -> unit state
change_unit_level {"player":0,"unit":123,"change":1} -> unit state
set_unit_fortify_turns {"player":0,"unit":123,"value":3} -> unit state
change_unit_fortify_turns {"player":0,"unit":123,"change":1} -> unit state
set_unit_made_attack {"player":0,"unit":123,"value":1} -> unit state
set_unit_base_combat {"player":0,"unit":123,"value":4} -> unit state
set_unit_immobile_timer {"player":0,"unit":123,"value":2} -> unit state
change_unit_immobile_timer {"player":0,"unit":123,"change":-1} -> unit state
set_unit_promotion {"player":0,"unit":123,"promotion":"PROMOTION_COMBAT1","has":1} -> unit state
push_unit_group_mission {"player":0,"unit":123,"mission":"MISSION_MOVE_TO","data1":11,"data2":12,"append":0,"manual":0} -> selection group state
pop_unit_group_mission {"player":0,"unit":123} -> selection group state
clear_unit_group_mission_queue {"player":0,"unit":123} -> selection group state
do_unit_group_command {"player":0,"unit":123,"command":"load_unit","data1":0,"data2":456} -> {"player":0,"group":9,"command":10,"data1":0,"data2":456,"executed":true,"executing_player":0,"executing_unit":123,"unit_exists":true,"x":10,"y":12,"current_group":9}
join_unit_group {"player":0,"unit":123,"head_player":0,"head_unit":456} -> selection group state
join_unit_group {"player":0,"unit":123} -> selection group state
kill_unit {"player":0,"unit":123,"killer":1} -> {"player":0,"unit":123,"killed":true}
spawn_unit {"player":0,"unit_type":"UNIT_WARRIOR","x":10,"y":12} -> {"player":0,"unit":123,"x":10,"y":12}
set_mod_state {"json":"{\"schema_version\":1}"} -> {"bytes":20}
set_player_civic {"player":0,"civic":"CIVIC_SLAVERY"} -> player options
set_player_state_religion {"player":0,"religion":"RELIGION_BUDDHISM"} -> player options
set_player_research {"player":0,"tech":"TECH_BRONZE_WORKING"} -> player options
set_team_has_tech {"team":0,"tech":"TECH_BRONZE_WORKING","has":1,"player":0} -> team tech state
change_team_research_progress {"team":0,"tech":"TECH_BRONZE_WORKING","change":50,"player":0} -> team tech state
meet_team {"team":0,"other_team":1} -> team relation state
declare_war {"team":0,"other_team":1,"war_plan":"total"} -> team relation state
make_peace {"team":0,"other_team":1,"bump_units":1} -> team relation state
set_team_open_borders {"team":0,"other_team":1,"open":1} -> team relation state
set_team_defensive_pact {"team":0,"other_team":1,"pact":1} -> team relation state
set_team_force_peace {"team":0,"other_team":1,"peace":1} -> team relation state
set_team_permanent_war_peace {"team":0,"other_team":1,"permanent":1} -> team relation state
set_team_vassal {"team":1,"other_team":0,"vassal":1,"capitulated":0} -> team relation state
set_team_war_weariness {"team":0,"other_team":1,"value":100} -> team relation state
change_team_war_weariness {"team":0,"other_team":1,"change":-10} -> team relation state
set_team_stolen_visibility_timer {"team":0,"other_team":1,"value":2} -> team relation state
change_team_stolen_visibility_timer {"team":0,"other_team":1,"change":-1} -> team relation state
```

`unit_type`, `building_type`, `building`, `building_class`, `project_type`, `process_type`,
`terrain`, `feature`, `bonus`, `improvement`, `route`, `promotion`, `tech`, `civic`, `religion`,
`corporation`, `victory`, `era`, `leader`, game `option`, multiplayer `option`, and force
`control` may be either numeric Civ4 info IDs or XML type names. `mission` for
`push_unit_group_mission` and `unit_ai` for
`spawn_unit` may also be numeric or an XML type name.
Info metadata queries accept `kind` values: `unit`, `unit_ai`, `building`, `building_class`,
`project`, `process`, `terrain`, `feature`, `bonus`, `improvement`, `route`, `promotion`, `tech`,
`civic`, `civic_option`, `religion`, `corporation`, `victory`, `game_option`,
`multiplayer_option`, `force_control`, `era`, `leader`, `civilization`, `handicap`, `game_speed`,
`hurry`, `build`, `goody`, `mission`, `espionage_mission`, `specialist`, `unit_class`,
`unit_combat`, `player_option`, `commerce`, and `yield`.
`commerce` accepts `gold`, `research`, `culture`, `espionage`, the matching Civ4 enum names, or
numeric `CommerceTypes` values. Boolean command arguments may be sent as JSON booleans or `0`/`1`.
If `culture_player` is omitted from `set_city_culture`, the DLL uses the city owner.
If `civic_option` is omitted from `set_player_civic`, the DLL derives it from the civic.
Use religion `-1` with `set_player_state_religion` to clear a player's state religion.
Use parent `-1` with `set_player_parent` to clear the parent player.
`set_city_religion` and `set_city_corporation` accept optional integer flags `announce` and
`arrows`; `announce` defaults to `0` so external mod state changes do not emit UI messages unless
requested.
`get_city_production_options` uses Civ4's `canTrain`, `canConstruct`, `canCreate`, and
`canMaintain` checks. Optional boolean flags default to `false`; `ignore_upgrades` only affects
unit training checks.
`war_plan` accepts `none`, `attacked_recent`, `attacked`, `preparing_limited`,
`preparing_total`, `limited`, `total`, `dogpile`, the matching Civ4 enum names, or numeric
`WarPlanTypes` values. The relation commands `set_team_open_borders`,
`set_team_defensive_pact`, `set_team_force_peace`, and `set_team_permanent_war_peace` accept an
optional `reciprocal` flag that defaults to `1`.
`set_game_state` accepts `on`, `over`, `extended`, the matching Civ4 enum names, or numeric
`GameStateTypes` values. Use `{"team":-1,"victory":-1}` with `set_game_winner` to clear the winner.
Use `{"player":-1}` with `set_game_pause_player` to unpause.
Use `-1` with `set_plot_feature`, `set_plot_bonus`, `set_plot_improvement`, `set_plot_route`, or
`set_plot_owner` to clear that plot value.
Plot commands accept the same optional update flags as the Civ4 DLL methods: `set_plot_owner`
accepts `check_units` and `update_plot_group`; `set_plot_terrain` accepts `recalculate` and
`rebuild_graphics`; `set_plot_feature` accepts `variety`; `set_plot_route` accepts
`update_plot_group`; `set_plot_culture` accepts `update` and `update_plot_groups`;
`change_plot_culture` accepts `update`; `set_plot_revealed` accepts `terrain_only`, `from_team`,
and `update_plot_group`.
`push_city_order` accepts `order` as `train`, `construct`, `create`, `maintain`, or the matching
Civ4 enum name. `data1` is interpreted as a unit, building, project, or process according to the
order. Optional flags `save`, `pop`, `append`, and `force` are integers where `0` is false and
non-zero is true.
`push_unit_group_mission` accepts `mission` as a numeric Civ4 mission ID or XML type name.
`data1` and `data2` are mission-specific integer payloads, such as destination `x` and `y` for
`MISSION_MOVE_TO`. Optional flags `flags`, `append`, and `manual` default to `0`, `0`, and `0`.
`can_unit_group_start_mission` accepts the same `mission`, `data1`, and `data2` fields, plus
optional `x`/`y`, `test_visible`, and `use_cache` fields matching Civ4's `canStartMission`.
`can_unit_group_do_command` accepts `command` as a numeric Civ4 `CommandTypes` value or one of
`promotion`, `upgrade`, `automate`, `wake`, `cancel`, `cancel_all`, `stop_automation`, `delete`,
`gift`, `load`, `load_unit`, `unload`, `unload_all`, or `hotkey`, plus optional `data1`, `data2`,
`test_visible`, and `use_cache` fields matching Civ4's `canDoCommand`.
`do_unit_group_command` accepts the same `command`, `data1`, and `data2` fields. It executes the
command on the first unit in the selected unit's group that can actually perform it. Some commands
can delete, gift, upgrade, load, or unload units; the reply reports the executing unit's original
ID and whether that same ID still exists after the command.
`can_unit_join_group` and `join_unit_group` accept optional `head_player` and `head_unit` fields.
When omitted, the unit is checked or moved into a fresh one-unit selection group. When provided,
the unit joins the head unit's current selection group if Civ4's `canJoinGroup` allows it.

`set_mod_state` stores an opaque UTF-8 JSON string owned by the external Rust client:

```json
{"type":"command","id":10,"name":"set_mod_state","args":{"json":"{\"schema_version\":1}"}}
```

The string is saved and loaded with `CvGame`.

## Callbacks

The bridge mirrors the main Python event callbacks to the callback pipe as `callback_mirror`
messages before the normal in-process Python event call runs.

Keyboard, mouse input, and selected Python game-rule hooks are sent as blocking
`callback_request` messages before Python. For input callbacks, if the external process replies
before the timeout, the DLL uses `result.consume` as the callback return value and skips Python.
For game-rule hooks, it uses `result.value` as the hook return value and skips Python. If there is
no reply, the callback pipe is disconnected, or the reply times out, the DLL falls back to the
normal Python callback. Set `CVGAME_BRIDGE_CALLBACK_TIMEOUT_MS` to override the default 50ms
timeout.

```json
{"type":"callback_request","id":200,"name":"kbd_event","args":{"evt":6,"key":65,"cursor_x":100,"cursor_y":120,"x":10,"y":12}}
{"type":"reply","id":200,"ok":true,"result":{"consume":false}}
```

Input callback request payloads:

```text
kbd_event {"evt":6,"key":65,"cursor_x":100,"cursor_y":120,"x":10,"y":12}
mouse_event {"evt":1,"cursor_x":100,"cursor_y":120,"x":10,"y":12,"interface_consumed":false}
```

City production rule callback request payloads:

```text
can_train {"player":0,"city":3,"x":10,"y":12,"unit":1,"continue_current":false,"test_visible":false,"ignore_cost":false,"ignore_upgrades":false}
cannot_train {"player":0,"city":3,"x":10,"y":12,"unit":1,"continue_current":false,"test_visible":false,"ignore_cost":false,"ignore_upgrades":false}
can_construct {"player":0,"city":3,"x":10,"y":12,"building":12,"continue_current":false,"test_visible":false,"ignore_cost":false,"ignore_upgrades":false}
cannot_construct {"player":0,"city":3,"x":10,"y":12,"building":12,"continue_current":false,"test_visible":false,"ignore_cost":false,"ignore_upgrades":false}
can_create {"player":0,"city":3,"x":10,"y":12,"project":1,"continue_current":false,"test_visible":false,"ignore_cost":false,"ignore_upgrades":false}
cannot_create {"player":0,"city":3,"x":10,"y":12,"project":1,"continue_current":false,"test_visible":false,"ignore_cost":false,"ignore_upgrades":false}
can_maintain {"player":0,"city":3,"x":10,"y":12,"process":2,"continue_current":false,"test_visible":false,"ignore_cost":false,"ignore_upgrades":false}
cannot_maintain {"player":0,"city":3,"x":10,"y":12,"process":2,"continue_current":false,"test_visible":false,"ignore_cost":false,"ignore_upgrades":false}
```

Reply with `{"value":true}` to make the corresponding hook return true. Reply with
`{"value":false}` to make it return false. For example, `can_train` true allows the unit before
normal Civ4 checks, while `cannot_train` true vetoes it after normal Civ4 checks.

Callback mirror payloads cover most of `CvEventReporter`. Object references are serialized as
stable game IDs and coordinates:

```text
first_contact {"team":0,"other_team":1}
combat_result {"winner_player":0,"winner_unit":12,"winner_unit_type":3,"winner_x":10,"winner_y":11,"loser_player":1,"loser_unit":9,"loser_unit_type":4,"loser_x":10,"loser_y":11}
improvement_built {"improvement":2,"x":10,"y":12}
improvement_destroyed {"improvement":2,"player":0,"x":10,"y":12}
route_built {"route":1,"x":10,"y":12}
plot_revealed {"x":10,"y":12,"team":0}
plot_feature_removed {"x":10,"y":12,"feature":1,"city_player":0,"city":3}
plot_picked {"x":10,"y":12}
nuke_explosion {"x":10,"y":12,"player":0,"unit":123,"unit_type":7}
goto_plot_set {"x":10,"y":12,"player":0}
culture_expansion {"player":0,"city":3,"x":10,"y":12}
city_do_turn {"player":0,"city":3,"x":10,"y":12}
city_building_unit {"player":0,"city":3,"unit_type":1}
city_building_building {"player":0,"city":3,"building":12}
city_rename {"player":0,"city":3,"x":10,"y":12}
city_hurry {"player":0,"city":3,"hurry":1}
selection_group_push_mission {"player":0,"group":42,"mission":1}
unit_set_xy {"player":0,"unit":123,"unit_type":1,"x":10,"y":12}
unit_promoted {"player":0,"unit":123,"unit_type":1,"promotion":4,"x":10,"y":12}
unit_selected {"player":0,"unit":123,"unit_type":1,"x":10,"y":12}
unit_rename {"player":0,"unit":123,"unit_type":1,"x":10,"y":12}
unit_pillage {"player":0,"unit":123,"unit_type":1,"improvement":2,"route":1,"pillage_player":1,"x":10,"y":12}
unit_spread_religion_attempt {"player":0,"unit":123,"unit_type":1,"religion":0,"success":1,"x":10,"y":12}
unit_gifted {"player":1,"unit":123,"unit_type":1,"gifting_player":0,"x":10,"y":12}
unit_build_improvement {"player":0,"unit":123,"unit_type":1,"build":5,"finished":1,"x":10,"y":12}
goody_received {"player":0,"x":10,"y":12,"unit_player":0,"unit":123,"unit_type":1,"goody":2}
great_person_born {"player":0,"city_player":0,"city":3,"unit":123,"unit_type":8,"x":10,"y":12}
project_built {"player":0,"city":3,"project":1}
tech_selected {"player":0,"tech":7}
religion_spread {"player":0,"city":3,"religion":0}
religion_remove {"player":0,"city":3,"religion":0}
corporation_founded {"player":0,"corporation":0}
corporation_spread {"player":0,"city":3,"corporation":0}
corporation_remove {"player":0,"city":3,"corporation":0}
set_player_alive {"player":0,"alive":1}
player_change_state_religion {"player":0,"new_religion":1,"old_religion":0}
vassal_state {"master":0,"vassal":1,"is_vassal":1}
```

Use `callback_mirror` for fire-and-forget events. Use `callback_request` when the DLL must wait for
the external process to decide a return value. Callback request replies use the same `reply` shape
as control-pipe query and command replies.

The Rust `CallbackDispatcher` runs handlers over typed callback messages. Handlers receive
`&mut BridgeClient`, so they can query and command game state while reacting to callbacks. When a
blocking `callback_request` is dispatched, the dispatcher writes a success reply automatically; a
handler can return `CallbackControl::Respond(value)` or `RespondAndStop(value)` to set the reply
result.

```rust
use civ4::{BridgeClient, BridgeEvent, CallbackControl, CallbackDispatcher};
use serde_json::json;

let (mut client, _hello) = BridgeClient::connect_from_env_with_handshake()?;
let mut callbacks = CallbackDispatcher::new();

callbacks.on_name("begin_player_turn", |client, event| {
    if let BridgeEvent::BeginPlayerTurn { player, .. } = event.event() {
        let state = client.get_player_state(*player)?;
        if state.gold < 100 {
            client.set_player_gold(*player, 100)?;
        }
    }
    Ok(CallbackControl::Continue)
});

callbacks.on_name("kbd_event", |_client, _event| {
    Ok(CallbackControl::Respond(json!({ "consume": false })))
});

callbacks.on_name("pre_save", |_client, _event| Ok(CallbackControl::Stop));
callbacks.run_until_stopped(&mut client)?;
```

The Rust `civ4` crate exposes typed helpers for the current operation set:

- `connect_from_env_with_handshake`, `connect_default_with_handshake`,
  `connect_with_prefix_and_handshake`, `handshake`, and `BridgeHello`
- `get_game_turn`, `get_game_state`, `set_game_turn`, `set_game_max_turns`, `change_game_max_turns`
- `get_info_count`, `get_info_type`, `resolve_info_id`, `list_info_types`, `InfoKind`, `InfoTypeState`, and `InfoTypeEntry`
- `set_game_start_turn`, `set_game_start_year`, `set_game_estimate_end_turn`, `set_game_target_score`
- `set_game_max_city_elimination`, `set_game_advanced_start_points`, `set_game_ai_auto_play`, `change_game_ai_auto_play`
- `change_game_nukes_exploded`, `set_game_pause_player`, `pause_game_for`, `clear_game_pause`, `set_game_winner`, `clear_game_winner`, `set_game_status`
- `get_game_option_state`, `set_game_option`, `GameOptionState`, `get_multiplayer_option_state`, `set_multiplayer_option`, `MultiplayerOptionState`
- `get_force_control_state`, `set_force_control`, `ForceControlState`, `GameState`, and `GameStatus`
- `get_player_gold`, `set_player_gold`, `change_player_gold`
- `get_player_state`, `get_player_options`, `get_player_economy_state`, `get_player_gold_per_turn_state`, `list_players`, `list_alive_players`
- `set_player_alive`, `set_player_playable`, `set_player_current_era`, `set_player_personality`, `set_player_parent`
- `set_player_advanced_start_points`, `change_player_advanced_start_points`, `change_player_golden_age_turns`, `change_player_num_unit_golden_ages`
- `change_player_anarchy_turns`, `change_player_strike_turns`, `set_player_combat_experience`, `change_player_combat_experience`
- `set_player_commerce_percent`, `change_player_commerce_percent`, `change_player_commerce_rate_modifier`, `CommerceType`
- `set_player_gold_per_turn_by_player`, `change_player_gold_per_turn_by_player`, `PlayerEconomyState`, and `PlayerGoldPerTurnState`
- `set_player_civic`, `set_player_civic_for_option`, `set_player_state_religion`, `clear_player_state_religion`, `set_player_research`
- `get_team_tech_state`, `set_team_has_tech`, `grant_team_tech`, `change_team_research_progress`
- `get_team_state`, `list_teams`, `get_team_relation_state`, `TeamState`, and `TeamRelationState`
- `meet_team`, `declare_war`, `make_peace`, `WarPlan`
- `set_team_open_borders`, `set_team_defensive_pact`, `set_team_force_peace`, `set_team_permanent_war_peace`
- `set_team_vassal`, `set_team_war_weariness`, `change_team_war_weariness`
- `set_team_stolen_visibility_timer`, `change_team_stolen_visibility_timer`
- `get_map_state`, `get_plot_state`, `get_plot_culture_state`, `get_plot_visibility_state`, `get_plot_visibility_state_with_debug`,
  `PlotCultureState`, and `PlotVisibilityState`
- `get_city_state`, `get_city_detail_state`, `get_city_production_options`, `CityDetailState`, `CityProductionOptions`, `CityProductionOptionsQuery`, `list_player_cities`, `list_all_cities`, `set_city_population`, `change_city_population`, `set_city_culture`, `set_owner_city_culture`
- `set_city_production`, `change_city_production`, `set_city_unit_production`, `set_city_building_production`, `set_city_project_production`
- `push_city_order`, `clear_city_order_queue`, `pop_city_order`, `CityOrder`, and `CityOrderType`
- `get_city_building_state`, `set_city_real_building`, `set_city_free_building`, and `CityBuildingState`
- `get_city_religion_state`, `set_city_religion`, `add_city_religion`, `remove_city_religion`, and `CityReligionState`
- `get_city_corporation_state`, `set_city_corporation`, `add_city_corporation`, `remove_city_corporation`, and `CityCorporationState`
- `get_city_building_class_change`, `set_city_building_happiness_change`, `set_city_building_health_change`, and `CityBuildingClassChange`
- `set_city_occupation_timer`, `change_city_occupation_timer`, `change_city_hurry_anger_timer`
- `set_plot_owner`, `clear_plot_owner`, `set_plot_terrain`, `set_plot_feature`, `clear_plot_feature`, `set_plot_bonus`, `clear_plot_bonus`
- `set_plot_improvement`, `clear_plot_improvement`, `set_plot_route`, `clear_plot_route`, `set_plot_culture`, `change_plot_culture`, `set_plot_revealed`
- `set_plot_owner_with_options`, `clear_plot_owner_with_options`, `set_plot_terrain_with_options`, `set_plot_feature_with_options`, `set_plot_route_with_options`,
  `set_plot_culture_with_options`, `change_plot_culture_with_options`, `set_plot_revealed_with_options`, and the matching `Plot*Options` structs
- `get_unit_state`, `get_unit_detail_state`, `UnitDetailState`, `get_unit_promotion_state`, `list_player_units`, `list_all_units`, `set_unit_damage`, `change_unit_damage`, `set_unit_experience`, `change_unit_experience`
- `set_unit_xy`, `set_unit_moves`, `change_unit_moves`, `finish_unit_moves`, `set_unit_level`, `change_unit_level`
- `set_unit_fortify_turns`, `change_unit_fortify_turns`, `set_unit_made_attack`, `set_unit_base_combat`
- `set_unit_immobile_timer`, `change_unit_immobile_timer`, `set_unit_promotion`, `grant_unit_promotion`, `remove_unit_promotion`, `kill_unit`
- `get_selection_group_state`, `list_player_selection_groups`, `list_all_selection_groups`, `SelectionGroupRef`
- `get_unit_group_state`, `can_unit_group_start_mission`, `can_unit_group_do_command`, `can_unit_join_group`, `push_unit_group_mission`, `pop_unit_group_mission`, `clear_unit_group_mission_queue`, `do_unit_group_command`, `join_unit_group`, `split_unit_group`, `UnitGroupMission`, `UnitGroupCommand`, `UnitGroupJoin`, `UnitCommandName`, `UnitCommandType`, `SelectionGroupState`, `SelectionGroupMissionState`, `SelectionGroupMissionCheck`, `SelectionGroupCommandCheck`, `UnitCommandResult`, and `UnitGroupJoinCheck`
- `spawn_unit`, `KilledUnit`, and `UnitPromotionState`
- `get_mod_state`, `set_mod_state`, `load_mod_state<T>`, `save_mod_state<T>`
- `BridgeEvent` typed variants for mirrored `CvEventReporter` payloads and city production rule callback requests, plus `CityProductionRule`
- `next_bridge_event`, `next_callback_event`, `next_callback_message`, and `next_callback_request`
- `CallbackDispatcher`, `CallbackControl`, and `CallbackDispatch`
