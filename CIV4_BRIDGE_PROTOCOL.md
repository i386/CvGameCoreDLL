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
get_mod_state -> {"json":"{\"schema_version\":1}"}
```

## Commands

Commands are rejected in multiplayer in this first version.

```text
set_player_gold {"player":0,"value":500} -> {"gold":500}
spawn_unit {"player":0,"unit_type":"UNIT_WARRIOR","x":10,"y":12} -> {"player":0,"unit":123,"x":10,"y":12}
set_mod_state {"json":"{\"schema_version\":1}"} -> {"bytes":20}
```

`unit_type` and `unit_ai` for `spawn_unit` may be either numeric Civ4 info IDs or XML type names.

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
- `spawn_unit`
- `get_mod_state`, `set_mod_state`, `load_mod_state<T>`, `save_mod_state<T>`
- `next_bridge_event` and `next_callback_event`
