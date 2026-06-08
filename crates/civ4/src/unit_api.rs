use crate::client::{BridgeClient, Result};
use crate::commands::{SpawnUnitRequest, SpawnUnitResult, SpawnedUnit};
use crate::state::{KilledUnit, PlayerUnitsResult, UnitPromotionState, UnitState};
use crate::types::{InfoType, PlayerId, Plot, UnitRef};
use serde_json::json;

impl BridgeClient {
    pub fn get_unit_state(&mut self, unit: UnitRef) -> Result<UnitState> {
        self.query(
            "get_unit_state",
            json!({ "player": unit.player, "unit": unit.id }),
        )
    }

    pub fn list_player_units<P: Into<PlayerId>>(&mut self, player: P) -> Result<Vec<UnitState>> {
        let player = player.into();
        let result: PlayerUnitsResult =
            self.query("list_player_units", json!({ "player": player.0 }))?;
        Ok(result.units)
    }

    pub fn list_all_units(&mut self) -> Result<Vec<UnitState>> {
        let mut units = Vec::new();
        for player in self.list_alive_players()? {
            units.extend(self.list_player_units(player.player_id())?);
        }
        Ok(units)
    }

    pub fn get_unit_promotion_state<P>(
        &mut self,
        unit: UnitRef,
        promotion: P,
    ) -> Result<UnitPromotionState>
    where
        P: Into<InfoType>,
    {
        self.query(
            "get_unit_promotion_state",
            json!({
                "player": unit.player,
                "unit": unit.id,
                "promotion": promotion.into()
            }),
        )
    }

    pub fn set_unit_xy(&mut self, unit: UnitRef, plot: Plot) -> Result<UnitState> {
        self.command(
            "set_unit_xy",
            json!({ "player": unit.player, "unit": unit.id, "x": plot.x, "y": plot.y }),
        )
    }

    pub fn set_unit_damage(&mut self, unit: UnitRef, value: i32) -> Result<UnitState> {
        self.command(
            "set_unit_damage",
            json!({ "player": unit.player, "unit": unit.id, "value": value }),
        )
    }

    pub fn change_unit_damage(&mut self, unit: UnitRef, change: i32) -> Result<UnitState> {
        self.command(
            "change_unit_damage",
            json!({ "player": unit.player, "unit": unit.id, "change": change }),
        )
    }

    pub fn set_unit_experience(&mut self, unit: UnitRef, value: i32) -> Result<UnitState> {
        self.command(
            "set_unit_experience",
            json!({ "player": unit.player, "unit": unit.id, "value": value }),
        )
    }

    pub fn change_unit_experience(&mut self, unit: UnitRef, change: i32) -> Result<UnitState> {
        self.command(
            "change_unit_experience",
            json!({ "player": unit.player, "unit": unit.id, "change": change }),
        )
    }

    pub fn set_unit_moves(&mut self, unit: UnitRef, value: i32) -> Result<UnitState> {
        self.command(
            "set_unit_moves",
            json!({ "player": unit.player, "unit": unit.id, "value": value }),
        )
    }

    pub fn change_unit_moves(&mut self, unit: UnitRef, change: i32) -> Result<UnitState> {
        self.command(
            "change_unit_moves",
            json!({ "player": unit.player, "unit": unit.id, "change": change }),
        )
    }

    pub fn finish_unit_moves(&mut self, unit: UnitRef) -> Result<UnitState> {
        self.command(
            "finish_unit_moves",
            json!({ "player": unit.player, "unit": unit.id }),
        )
    }

    pub fn set_unit_level(&mut self, unit: UnitRef, value: i32) -> Result<UnitState> {
        self.command(
            "set_unit_level",
            json!({ "player": unit.player, "unit": unit.id, "value": value }),
        )
    }

    pub fn change_unit_level(&mut self, unit: UnitRef, change: i32) -> Result<UnitState> {
        self.command(
            "change_unit_level",
            json!({ "player": unit.player, "unit": unit.id, "change": change }),
        )
    }

    pub fn set_unit_fortify_turns(&mut self, unit: UnitRef, value: i32) -> Result<UnitState> {
        self.command(
            "set_unit_fortify_turns",
            json!({ "player": unit.player, "unit": unit.id, "value": value }),
        )
    }

    pub fn change_unit_fortify_turns(&mut self, unit: UnitRef, change: i32) -> Result<UnitState> {
        self.command(
            "change_unit_fortify_turns",
            json!({ "player": unit.player, "unit": unit.id, "change": change }),
        )
    }

    pub fn set_unit_made_attack(&mut self, unit: UnitRef, made_attack: bool) -> Result<UnitState> {
        self.command(
            "set_unit_made_attack",
            json!({
                "player": unit.player,
                "unit": unit.id,
                "value": if made_attack { 1 } else { 0 }
            }),
        )
    }

    pub fn set_unit_base_combat(&mut self, unit: UnitRef, value: i32) -> Result<UnitState> {
        self.command(
            "set_unit_base_combat",
            json!({ "player": unit.player, "unit": unit.id, "value": value }),
        )
    }

    pub fn set_unit_immobile_timer(&mut self, unit: UnitRef, value: i32) -> Result<UnitState> {
        self.command(
            "set_unit_immobile_timer",
            json!({ "player": unit.player, "unit": unit.id, "value": value }),
        )
    }

    pub fn change_unit_immobile_timer(&mut self, unit: UnitRef, change: i32) -> Result<UnitState> {
        self.command(
            "change_unit_immobile_timer",
            json!({ "player": unit.player, "unit": unit.id, "change": change }),
        )
    }

    pub fn set_unit_promotion<P>(
        &mut self,
        unit: UnitRef,
        promotion: P,
        has: bool,
    ) -> Result<UnitState>
    where
        P: Into<InfoType>,
    {
        self.command(
            "set_unit_promotion",
            json!({
                "player": unit.player,
                "unit": unit.id,
                "promotion": promotion.into(),
                "has": if has { 1 } else { 0 }
            }),
        )
    }

    pub fn grant_unit_promotion<P>(&mut self, unit: UnitRef, promotion: P) -> Result<UnitState>
    where
        P: Into<InfoType>,
    {
        self.set_unit_promotion(unit, promotion, true)
    }

    pub fn remove_unit_promotion<P>(&mut self, unit: UnitRef, promotion: P) -> Result<UnitState>
    where
        P: Into<InfoType>,
    {
        self.set_unit_promotion(unit, promotion, false)
    }

    pub fn kill_unit(&mut self, unit: UnitRef, killer: Option<PlayerId>) -> Result<KilledUnit> {
        let mut args = json!({
            "player": unit.player,
            "unit": unit.id,
        });
        if let Some(killer) = killer {
            args["killer"] = json!(killer.0);
        }
        self.command("kill_unit", args)
    }

    pub fn spawn_unit(&mut self, request: SpawnUnitRequest) -> Result<SpawnedUnit> {
        let args = request.into_args();
        let result: SpawnUnitResult = self.command("spawn_unit", args)?;
        Ok(SpawnedUnit {
            unit: UnitRef {
                player: result.player,
                id: result.unit,
            },
            plot: Plot::new(result.x, result.y),
        })
    }
}
