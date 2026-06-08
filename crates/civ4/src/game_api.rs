use crate::client::{BridgeClient, Result};
use crate::state::{
    ForceControlState, GameOptionState, GameState, GameTurnResult, MultiplayerOptionState,
};
use crate::types::{GameStatus, InfoType, PlayerId, TeamId};
use serde_json::json;

impl BridgeClient {
    pub fn get_game_turn(&mut self) -> Result<i32> {
        let result: GameTurnResult = self.query("get_game_turn", json!({}))?;
        Ok(result.turn)
    }

    pub fn get_game_state(&mut self) -> Result<GameState> {
        self.query("get_game_state", json!({}))
    }

    pub fn get_game_option_state<O>(&mut self, option: O) -> Result<GameOptionState>
    where
        O: Into<InfoType>,
    {
        self.query("get_game_option_state", json!({ "option": option.into() }))
    }

    pub fn get_multiplayer_option_state<O>(&mut self, option: O) -> Result<MultiplayerOptionState>
    where
        O: Into<InfoType>,
    {
        self.query(
            "get_multiplayer_option_state",
            json!({ "option": option.into() }),
        )
    }

    pub fn get_force_control_state<C>(&mut self, control: C) -> Result<ForceControlState>
    where
        C: Into<InfoType>,
    {
        self.query(
            "get_force_control_state",
            json!({ "control": control.into() }),
        )
    }

    pub fn set_game_turn(&mut self, value: i32) -> Result<GameState> {
        self.command("set_game_turn", json!({ "value": value }))
    }

    pub fn set_game_max_turns(&mut self, value: i32) -> Result<GameState> {
        self.command("set_game_max_turns", json!({ "value": value }))
    }

    pub fn change_game_max_turns(&mut self, change: i32) -> Result<GameState> {
        self.command("change_game_max_turns", json!({ "change": change }))
    }

    pub fn set_game_start_turn(&mut self, value: i32) -> Result<GameState> {
        self.command("set_game_start_turn", json!({ "value": value }))
    }

    pub fn set_game_start_year(&mut self, value: i32) -> Result<GameState> {
        self.command("set_game_start_year", json!({ "value": value }))
    }

    pub fn set_game_estimate_end_turn(&mut self, value: i32) -> Result<GameState> {
        self.command("set_game_estimate_end_turn", json!({ "value": value }))
    }

    pub fn set_game_target_score(&mut self, value: i32) -> Result<GameState> {
        self.command("set_game_target_score", json!({ "value": value }))
    }

    pub fn set_game_max_city_elimination(&mut self, value: i32) -> Result<GameState> {
        self.command("set_game_max_city_elimination", json!({ "value": value }))
    }

    pub fn set_game_advanced_start_points(&mut self, value: i32) -> Result<GameState> {
        self.command("set_game_advanced_start_points", json!({ "value": value }))
    }

    pub fn set_game_ai_auto_play(&mut self, value: i32) -> Result<GameState> {
        self.command("set_game_ai_auto_play", json!({ "value": value }))
    }

    pub fn change_game_ai_auto_play(&mut self, change: i32) -> Result<GameState> {
        self.command("change_game_ai_auto_play", json!({ "change": change }))
    }

    pub fn change_game_nukes_exploded(&mut self, change: i32) -> Result<GameState> {
        self.command("change_game_nukes_exploded", json!({ "change": change }))
    }

    pub fn set_game_pause_player(&mut self, player: Option<PlayerId>) -> Result<GameState> {
        self.command(
            "set_game_pause_player",
            json!({ "player": player.map_or(-1, |player| player.0) }),
        )
    }

    pub fn pause_game_for<P: Into<PlayerId>>(&mut self, player: P) -> Result<GameState> {
        self.set_game_pause_player(Some(player.into()))
    }

    pub fn clear_game_pause(&mut self) -> Result<GameState> {
        self.set_game_pause_player(None)
    }

    pub fn set_game_winner<V>(&mut self, team: TeamId, victory: V) -> Result<GameState>
    where
        V: Into<InfoType>,
    {
        self.command(
            "set_game_winner",
            json!({ "team": team.0, "victory": victory.into() }),
        )
    }

    pub fn clear_game_winner(&mut self) -> Result<GameState> {
        self.command("set_game_winner", json!({ "team": -1, "victory": -1 }))
    }

    pub fn set_game_status(&mut self, value: GameStatus) -> Result<GameState> {
        self.command("set_game_state", json!({ "value": value }))
    }

    pub fn set_game_option<O>(&mut self, option: O, enabled: bool) -> Result<GameOptionState>
    where
        O: Into<InfoType>,
    {
        self.command(
            "set_game_option",
            json!({
                "option": option.into(),
                "enabled": if enabled { 1 } else { 0 }
            }),
        )
    }

    pub fn set_multiplayer_option<O>(
        &mut self,
        option: O,
        enabled: bool,
    ) -> Result<MultiplayerOptionState>
    where
        O: Into<InfoType>,
    {
        self.command(
            "set_multiplayer_option",
            json!({
                "option": option.into(),
                "enabled": if enabled { 1 } else { 0 }
            }),
        )
    }

    pub fn set_force_control<C>(&mut self, control: C, enabled: bool) -> Result<ForceControlState>
    where
        C: Into<InfoType>,
    {
        self.command(
            "set_force_control",
            json!({
                "control": control.into(),
                "enabled": if enabled { 1 } else { 0 }
            }),
        )
    }
}
