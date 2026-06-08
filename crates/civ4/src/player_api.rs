use crate::client::{BridgeClient, Result};
use crate::state::{
    PlayerEconomyState, PlayerGoldPerTurnState, PlayerGoldResult, PlayerOptions, PlayerState,
    PlayersResult,
};
use crate::types::{CommerceType, InfoType, PlayerId};
use serde_json::json;

impl BridgeClient {
    pub fn get_player_gold<P: Into<PlayerId>>(&mut self, player: P) -> Result<i32> {
        let player = player.into();
        let result: PlayerGoldResult =
            self.query("get_player_gold", json!({ "player": player.0 }))?;
        Ok(result.gold)
    }

    pub fn get_player_state<P: Into<PlayerId>>(&mut self, player: P) -> Result<PlayerState> {
        let player = player.into();
        self.query("get_player_state", json!({ "player": player.0 }))
    }

    pub fn list_players(&mut self) -> Result<Vec<PlayerState>> {
        let result: PlayersResult = self.query("list_players", json!({}))?;
        Ok(result.players)
    }

    pub fn list_alive_players(&mut self) -> Result<Vec<PlayerState>> {
        Ok(self
            .list_players()?
            .into_iter()
            .filter(|player| player.alive)
            .collect())
    }

    pub fn get_player_options<P: Into<PlayerId>>(&mut self, player: P) -> Result<PlayerOptions> {
        let player = player.into();
        self.query("get_player_options", json!({ "player": player.0 }))
    }

    pub fn get_player_economy_state<P: Into<PlayerId>>(
        &mut self,
        player: P,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.query("get_player_economy_state", json!({ "player": player.0 }))
    }

    pub fn get_player_gold_per_turn_state<P, O>(
        &mut self,
        player: P,
        other_player: O,
    ) -> Result<PlayerGoldPerTurnState>
    where
        P: Into<PlayerId>,
        O: Into<PlayerId>,
    {
        let player = player.into();
        let other_player = other_player.into();
        self.query(
            "get_player_gold_per_turn_state",
            json!({ "player": player.0, "other_player": other_player.0 }),
        )
    }

    pub fn set_player_gold<P: Into<PlayerId>>(&mut self, player: P, value: i32) -> Result<i32> {
        let player = player.into();
        let result: PlayerGoldResult = self.command(
            "set_player_gold",
            json!({ "player": player.0, "value": value }),
        )?;
        Ok(result.gold)
    }

    pub fn change_player_gold<P: Into<PlayerId>>(
        &mut self,
        player: P,
        change: i32,
    ) -> Result<PlayerState> {
        let player = player.into();
        self.command(
            "change_player_gold",
            json!({ "player": player.0, "change": change }),
        )
    }

    pub fn set_player_alive<P: Into<PlayerId>>(
        &mut self,
        player: P,
        alive: bool,
    ) -> Result<PlayerState> {
        let player = player.into();
        self.command(
            "set_player_alive",
            json!({ "player": player.0, "alive": alive }),
        )
    }

    pub fn set_player_playable<P: Into<PlayerId>>(
        &mut self,
        player: P,
        playable: bool,
    ) -> Result<PlayerState> {
        let player = player.into();
        self.command(
            "set_player_playable",
            json!({ "player": player.0, "playable": playable }),
        )
    }

    pub fn set_player_current_era<P, E>(&mut self, player: P, era: E) -> Result<PlayerState>
    where
        P: Into<PlayerId>,
        E: Into<InfoType>,
    {
        let player = player.into();
        self.command(
            "set_player_current_era",
            json!({ "player": player.0, "era": era.into() }),
        )
    }

    pub fn set_player_personality<P, L>(&mut self, player: P, leader: L) -> Result<PlayerState>
    where
        P: Into<PlayerId>,
        L: Into<InfoType>,
    {
        let player = player.into();
        self.command(
            "set_player_personality",
            json!({ "player": player.0, "leader": leader.into() }),
        )
    }

    pub fn set_player_parent<P: Into<PlayerId>>(
        &mut self,
        player: P,
        parent: Option<PlayerId>,
    ) -> Result<PlayerState> {
        let player = player.into();
        self.command(
            "set_player_parent",
            json!({ "player": player.0, "parent": parent.map_or(-1, |player| player.0) }),
        )
    }

    pub fn set_player_advanced_start_points<P: Into<PlayerId>>(
        &mut self,
        player: P,
        value: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "set_player_advanced_start_points",
            json!({ "player": player.0, "value": value }),
        )
    }

    pub fn change_player_advanced_start_points<P: Into<PlayerId>>(
        &mut self,
        player: P,
        change: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "change_player_advanced_start_points",
            json!({ "player": player.0, "change": change }),
        )
    }

    pub fn change_player_golden_age_turns<P: Into<PlayerId>>(
        &mut self,
        player: P,
        change: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "change_player_golden_age_turns",
            json!({ "player": player.0, "change": change }),
        )
    }

    pub fn change_player_num_unit_golden_ages<P: Into<PlayerId>>(
        &mut self,
        player: P,
        change: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "change_player_num_unit_golden_ages",
            json!({ "player": player.0, "change": change }),
        )
    }

    pub fn change_player_anarchy_turns<P: Into<PlayerId>>(
        &mut self,
        player: P,
        change: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "change_player_anarchy_turns",
            json!({ "player": player.0, "change": change }),
        )
    }

    pub fn change_player_strike_turns<P: Into<PlayerId>>(
        &mut self,
        player: P,
        change: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "change_player_strike_turns",
            json!({ "player": player.0, "change": change }),
        )
    }

    pub fn set_player_combat_experience<P: Into<PlayerId>>(
        &mut self,
        player: P,
        value: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "set_player_combat_experience",
            json!({ "player": player.0, "value": value }),
        )
    }

    pub fn change_player_combat_experience<P: Into<PlayerId>>(
        &mut self,
        player: P,
        change: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "change_player_combat_experience",
            json!({ "player": player.0, "change": change }),
        )
    }

    pub fn set_player_commerce_percent<P: Into<PlayerId>>(
        &mut self,
        player: P,
        commerce: CommerceType,
        value: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "set_player_commerce_percent",
            json!({ "player": player.0, "commerce": commerce, "value": value }),
        )
    }

    pub fn change_player_commerce_percent<P: Into<PlayerId>>(
        &mut self,
        player: P,
        commerce: CommerceType,
        change: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "change_player_commerce_percent",
            json!({ "player": player.0, "commerce": commerce, "change": change }),
        )
    }

    pub fn change_player_commerce_rate_modifier<P: Into<PlayerId>>(
        &mut self,
        player: P,
        commerce: CommerceType,
        change: i32,
    ) -> Result<PlayerEconomyState> {
        let player = player.into();
        self.command(
            "change_player_commerce_rate_modifier",
            json!({ "player": player.0, "commerce": commerce, "change": change }),
        )
    }

    pub fn set_player_gold_per_turn_by_player<P, O>(
        &mut self,
        player: P,
        other_player: O,
        value: i32,
    ) -> Result<PlayerGoldPerTurnState>
    where
        P: Into<PlayerId>,
        O: Into<PlayerId>,
    {
        let player = player.into();
        let other_player = other_player.into();
        self.command(
            "set_player_gold_per_turn_by_player",
            json!({ "player": player.0, "other_player": other_player.0, "value": value }),
        )
    }

    pub fn change_player_gold_per_turn_by_player<P, O>(
        &mut self,
        player: P,
        other_player: O,
        change: i32,
    ) -> Result<PlayerGoldPerTurnState>
    where
        P: Into<PlayerId>,
        O: Into<PlayerId>,
    {
        let player = player.into();
        let other_player = other_player.into();
        self.command(
            "change_player_gold_per_turn_by_player",
            json!({ "player": player.0, "other_player": other_player.0, "change": change }),
        )
    }

    pub fn set_player_civic<P, C>(&mut self, player: P, civic: C) -> Result<PlayerOptions>
    where
        P: Into<PlayerId>,
        C: Into<InfoType>,
    {
        let player = player.into();
        self.command(
            "set_player_civic",
            json!({ "player": player.0, "civic": civic.into() }),
        )
    }

    pub fn set_player_civic_for_option<P, C>(
        &mut self,
        player: P,
        civic_option: i32,
        civic: C,
    ) -> Result<PlayerOptions>
    where
        P: Into<PlayerId>,
        C: Into<InfoType>,
    {
        let player = player.into();
        self.command(
            "set_player_civic",
            json!({ "player": player.0, "civic_option": civic_option, "civic": civic.into() }),
        )
    }

    pub fn set_player_state_religion<P, R>(
        &mut self,
        player: P,
        religion: R,
    ) -> Result<PlayerOptions>
    where
        P: Into<PlayerId>,
        R: Into<InfoType>,
    {
        let player = player.into();
        self.command(
            "set_player_state_religion",
            json!({ "player": player.0, "religion": religion.into() }),
        )
    }

    pub fn clear_player_state_religion<P: Into<PlayerId>>(
        &mut self,
        player: P,
    ) -> Result<PlayerOptions> {
        let player = player.into();
        self.command(
            "set_player_state_religion",
            json!({ "player": player.0, "religion": -1 }),
        )
    }

    pub fn set_player_research<P, T>(&mut self, player: P, tech: T) -> Result<PlayerOptions>
    where
        P: Into<PlayerId>,
        T: Into<InfoType>,
    {
        let player = player.into();
        self.command(
            "set_player_research",
            json!({ "player": player.0, "tech": tech.into() }),
        )
    }
}
