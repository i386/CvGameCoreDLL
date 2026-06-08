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

Rust clients should perform the hello handshake before registering gameplay behavior:

```rust
use civ4::{BridgeClient, Result};

fn connect() -> Result<BridgeClient> {
    let (client, hello) = BridgeClient::connect_default_with_handshake()?;
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
get_player_gold {"player":0} -> {"gold":500}
get_player_state {"player":0} -> {"player":0,"team":0,"alive":true,"human":true,"gold":500,"cities":3,"units":8,"population":12}
list_players -> {"players":[player state, ...]}
get_player_options {"player":0} -> {"player":0,"team":0,"state_religion":-1,"current_research":3,"civics":[1,2,3,4,5]}
get_team_state {"team":0} -> {"team":0,"alive":true,"ever_alive":true,"human":true,"barbarian":false,"minor":false,"leader":0,"secretary":0,"members":1,"cities":3,"population":12,"land":40,"assets":500,"power":120,"defensive_power":100,"at_war_count":0,"has_met_count":2,"defensive_pact_count":0,"vassal_count":0,"vassal":false,"nuke_interception":0,"map_trading":true,"tech_trading":true,"gold_trading":true,"open_borders_trading":true,"defensive_pact_trading":false,"permanent_alliance_trading":false,"vassal_trading":false}
list_teams -> {"teams":[team state, ...]}
get_team_tech_state {"team":0,"tech":"TECH_BRONZE_WORKING"} -> {"team":0,"tech":7,"has":true,"progress":0}
get_team_relation_state {"team":0,"other_team":1} -> {"team":0,"other_team":1,"has_met":true,"at_war":false,"can_declare_war":true,"can_change_war_peace":true,"permanent_war_peace":false,"open_borders":true,"defensive_pact":false,"force_peace":false,"vassal":false,"master":false,"war_weariness":0,"stolen_visibility_timer":0,"war_plan":-1}
get_map_state -> {"width":84,"height":52,"plots":4368,"land_plots":1472}
get_plot_state {"x":10,"y":12} -> {"x":10,"y":12,"owner":0,"terrain":1,"feature":-1,"bonus":-1,"improvement":2,"route":1,"water":false,"peak":false,"units":1,"city_player":0,"city":3}
get_city_state {"player":0,"city":3} -> {"player":0,"city":3,"x":10,"y":12,"population":5,"culture":42,"production":10,"production_needed":35,"production_unit":0,"production_unit_ai":2,"production_building":-1,"production_project":-1,"production_process":-1,"order_queue_length":1,"occupation_timer":0,"hurry_anger_timer":0}
get_city_building_state {"player":0,"city":3,"building":"BUILDING_GRANARY"} -> {"player":0,"city":3,"building":12,"real":1,"free":0,"active":true}
get_city_religion_state {"player":0,"city":3,"religion":"RELIGION_BUDDHISM"} -> {"player":0,"city":3,"religion":0,"has":true}
get_city_corporation_state {"player":0,"city":3,"corporation":"CORPORATION_SID_SUSHI"} -> {"player":0,"city":3,"corporation":0,"has":false}
get_city_building_class_change {"player":0,"city":3,"building_class":"BUILDINGCLASS_GRANARY"} -> {"player":0,"city":3,"building_class":12,"happiness":0,"health":0}
list_player_cities {"player":0} -> {"player":0,"cities":[city state, ...]}
get_unit_state {"player":0,"unit":123} -> {"player":0,"unit":123,"unit_type":0,"unit_ai":2,"domain":0,"x":10,"y":12,"damage":0,"experience":2,"level":1,"moves":0,"max_moves":2,"base_combat":3,"cargo":0,"fortify_turns":0,"immobile_timer":0,"made_attack":false,"promotions":[1,4]}
get_unit_promotion_state {"player":0,"unit":123,"promotion":"PROMOTION_COMBAT1"} -> {"player":0,"unit":123,"promotion":1,"has":true}
list_player_units {"player":0} -> {"player":0,"units":[unit state, ...]}
get_mod_state -> {"json":"{\"schema_version\":1}"}
```

## Commands

Commands are rejected in multiplayer in this first version.

```text
set_player_gold {"player":0,"value":500} -> {"gold":500}
change_player_gold {"player":0,"change":50} -> player state
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
and `corporation` may be either numeric Civ4 info IDs or XML type names. `unit_ai` for
`spawn_unit` may also be numeric or an XML type name.
If `culture_player` is omitted from `set_city_culture`, the DLL uses the city owner.
If `civic_option` is omitted from `set_player_civic`, the DLL derives it from the civic.
Use religion `-1` with `set_player_state_religion` to clear a player's state religion.
`set_city_religion` and `set_city_corporation` accept optional integer flags `announce` and
`arrows`; `announce` defaults to `0` so external mod state changes do not emit UI messages unless
requested.
`war_plan` accepts `none`, `attacked_recent`, `attacked`, `preparing_limited`,
`preparing_total`, `limited`, `total`, `dogpile`, the matching Civ4 enum names, or numeric
`WarPlanTypes` values. The relation commands `set_team_open_borders`,
`set_team_defensive_pact`, `set_team_force_peace`, and `set_team_permanent_war_peace` accept an
optional `reciprocal` flag that defaults to `1`.
Use `-1` with `set_plot_feature`, `set_plot_bonus`, `set_plot_improvement`, `set_plot_route`, or
`set_plot_owner` to clear that plot value.
`push_city_order` accepts `order` as `train`, `construct`, `create`, `maintain`, or the matching
Civ4 enum name. `data1` is interpreted as a unit, building, project, or process according to the
order. Optional flags `save`, `pop`, `append`, and `force` are integers where `0` is false and
non-zero is true.

`set_mod_state` stores an opaque UTF-8 JSON string owned by the external Rust client:

```json
{"type":"command","id":10,"name":"set_mod_state","args":{"json":"{\"schema_version\":1}"}}
```

The string is saved and loaded with `CvGame`.

## Callbacks

The bridge mirrors selected Python event callbacks to the callback pipe as `callback_mirror`
messages before the normal in-process Python event call runs.

Keyboard and mouse input callbacks are sent as blocking `callback_request` messages before Python.
If the external process replies before the timeout, the DLL uses `result.consume` as the callback
return value and skips Python. If there is no reply, the callback pipe is disconnected, or the reply
times out, the DLL falls back to the normal Python callback. Set `CVGAME_BRIDGE_CALLBACK_TIMEOUT_MS`
to override the default 50ms timeout.

```json
{"type":"callback_request","id":200,"name":"kbd_event","args":{"evt":6,"key":65,"cursor_x":100,"cursor_y":120,"x":10,"y":12}}
{"type":"reply","id":200,"ok":true,"result":{"consume":false}}
```

Input callback request payloads:

```text
kbd_event {"evt":6,"key":65,"cursor_x":100,"cursor_y":120,"x":10,"y":12}
mouse_event {"evt":1,"cursor_x":100,"cursor_y":120,"x":10,"y":12,"interface_consumed":false}
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

let mut client = BridgeClient::connect_default()?;
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

- `connect_default_with_handshake`, `connect_with_prefix_and_handshake`, `handshake`, and `BridgeHello`
- `get_game_turn`, `get_player_gold`, `set_player_gold`
- `get_player_state`, `get_player_options`, `list_players`, `list_alive_players`, `change_player_gold`
- `set_player_civic`, `set_player_civic_for_option`, `set_player_state_religion`, `clear_player_state_religion`, `set_player_research`
- `get_team_tech_state`, `set_team_has_tech`, `grant_team_tech`, `change_team_research_progress`
- `get_team_state`, `list_teams`, `get_team_relation_state`, `TeamState`, and `TeamRelationState`
- `meet_team`, `declare_war`, `make_peace`, `WarPlan`
- `set_team_open_borders`, `set_team_defensive_pact`, `set_team_force_peace`, `set_team_permanent_war_peace`
- `set_team_vassal`, `set_team_war_weariness`, `change_team_war_weariness`
- `set_team_stolen_visibility_timer`, `change_team_stolen_visibility_timer`
- `get_map_state`, `get_plot_state`
- `get_city_state`, `list_player_cities`, `list_all_cities`, `set_city_population`, `change_city_population`, `set_city_culture`, `set_owner_city_culture`
- `set_city_production`, `change_city_production`, `set_city_unit_production`, `set_city_building_production`, `set_city_project_production`
- `push_city_order`, `clear_city_order_queue`, `pop_city_order`, `CityOrder`, and `CityOrderType`
- `get_city_building_state`, `set_city_real_building`, `set_city_free_building`, and `CityBuildingState`
- `get_city_religion_state`, `set_city_religion`, `add_city_religion`, `remove_city_religion`, and `CityReligionState`
- `get_city_corporation_state`, `set_city_corporation`, `add_city_corporation`, `remove_city_corporation`, and `CityCorporationState`
- `get_city_building_class_change`, `set_city_building_happiness_change`, `set_city_building_health_change`, and `CityBuildingClassChange`
- `set_city_occupation_timer`, `change_city_occupation_timer`, `change_city_hurry_anger_timer`
- `set_plot_owner`, `clear_plot_owner`, `set_plot_terrain`, `set_plot_feature`, `clear_plot_feature`, `set_plot_bonus`, `clear_plot_bonus`
- `set_plot_improvement`, `clear_plot_improvement`, `set_plot_route`, `clear_plot_route`, `set_plot_culture`, `change_plot_culture`, `set_plot_revealed`
- `get_unit_state`, `get_unit_promotion_state`, `list_player_units`, `list_all_units`, `set_unit_damage`, `change_unit_damage`, `set_unit_experience`, `change_unit_experience`
- `set_unit_xy`, `set_unit_moves`, `change_unit_moves`, `finish_unit_moves`, `set_unit_level`, `change_unit_level`
- `set_unit_fortify_turns`, `change_unit_fortify_turns`, `set_unit_made_attack`, `set_unit_base_combat`
- `set_unit_immobile_timer`, `change_unit_immobile_timer`, `set_unit_promotion`, `grant_unit_promotion`, `remove_unit_promotion`, `kill_unit`
- `spawn_unit`, `KilledUnit`, and `UnitPromotionState`
- `get_mod_state`, `set_mod_state`, `load_mod_state<T>`, `save_mod_state<T>`
- `next_bridge_event`, `next_callback_event`, `next_callback_message`, and `next_callback_request`
- `CallbackDispatcher`, `CallbackControl`, and `CallbackDispatch`
