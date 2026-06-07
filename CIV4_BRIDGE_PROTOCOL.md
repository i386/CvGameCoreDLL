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

## Messages

```json
{"type":"hello","protocol":1,"side":"dll","capabilities":["events","queries","commands","callbacks","mod_state"]}
{"type":"event","seq":1,"name":"begin_game_turn","args":{"turn":42}}
{"type":"callback_mirror","seq":2,"name":"city_built","args":{"player":0,"city":3,"x":10,"y":12}}
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
get_map_state -> {"width":84,"height":52,"plots":4368,"land_plots":1472}
get_plot_state {"x":10,"y":12} -> {"x":10,"y":12,"owner":0,"terrain":1,"feature":-1,"bonus":-1,"improvement":2,"water":false,"peak":false,"units":1,"city_player":0,"city":3}
get_city_state {"player":0,"city":3} -> {"player":0,"city":3,"x":10,"y":12,"population":5,"culture":42}
get_unit_state {"player":0,"unit":123} -> {"player":0,"unit":123,"unit_type":0,"x":10,"y":12,"damage":0,"experience":2,"level":1}
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
set_unit_damage {"player":0,"unit":123,"value":25} -> unit state
change_unit_damage {"player":0,"unit":123,"change":-10} -> unit state
set_unit_experience {"player":0,"unit":123,"value":5} -> unit state
spawn_unit {"player":0,"unit_type":"UNIT_WARRIOR","x":10,"y":12} -> {"player":0,"unit":123,"x":10,"y":12}
set_mod_state {"json":"{\"schema_version\":1}"} -> {"bytes":20}
```

`unit_type` and `unit_ai` for `spawn_unit` may be either numeric Civ4 info IDs or XML type names.
If `culture_player` is omitted from `set_city_culture`, the DLL uses the city owner.

`set_mod_state` stores an opaque UTF-8 JSON string owned by the external Rust client:

```json
{"type":"command","id":10,"name":"set_mod_state","args":{"json":"{\"schema_version\":1}"}}
```

The string is saved and loaded with `CvGame`.

## Callbacks

This first bridge version mirrors selected Python event callbacks to the callback pipe as
`callback_mirror` messages before the normal in-process Python event call runs. Blocking
callback replacement is intentionally not enabled yet.

The Rust `civ4` crate exposes typed helpers for the current operation set:

- `get_game_turn`, `get_player_gold`, `set_player_gold`
- `get_player_state`, `change_player_gold`
- `get_map_state`, `get_plot_state`
- `get_city_state`, `set_city_population`, `change_city_population`, `set_city_culture`, `set_owner_city_culture`
- `get_unit_state`, `set_unit_damage`, `change_unit_damage`, `set_unit_experience`
- `spawn_unit`
- `get_mod_state`, `set_mod_state`, `load_mod_state<T>`, `save_mod_state<T>`
- `next_bridge_event` and `next_callback_event`
