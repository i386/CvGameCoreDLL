use crate::client::{BridgeClient, Result};
use crate::state::{KilledUnit, UnitPromotionState, UnitState};
use crate::types::{InfoType, PlayerId, Plot, UnitRef};
use serde_json::json;

impl BridgeClient {
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
}
