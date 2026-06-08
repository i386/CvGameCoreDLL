use crate::types::{Plot, UnitRef};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnitState {
    pub player: i32,
    pub unit: i32,
    pub unit_type: i32,
    pub unit_ai: i32,
    pub domain: i32,
    pub x: i32,
    pub y: i32,
    pub damage: i32,
    pub experience: i32,
    pub level: i32,
    pub moves: i32,
    pub max_moves: i32,
    pub base_combat: i32,
    pub cargo: i32,
    pub fortify_turns: i32,
    pub immobile_timer: i32,
    pub made_attack: bool,
    pub promotions: Vec<i32>,
}

impl UnitState {
    pub fn unit_ref(&self) -> UnitRef {
        UnitRef::new(self.player, self.unit)
    }

    pub fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnitDetailState {
    pub player: i32,
    pub unit: i32,
    pub unit_type: i32,
    pub unit_ai: i32,
    pub domain: i32,
    pub unit_combat: i32,
    pub special_unit: i32,
    pub x: i32,
    pub y: i32,
    pub area: i32,
    pub group: i32,
    pub in_group: bool,
    pub group_head: bool,
    pub base_moves: i32,
    pub max_moves: i32,
    pub moves_left: i32,
    pub can_move: bool,
    pub has_moved: bool,
    pub visibility_range: i32,
    pub air_range: i32,
    pub nuke_range: i32,
    pub can_build_route: bool,
    pub build_type: i32,
    pub work_rate: i32,
    pub max_work_rate: i32,
    pub can_fight: bool,
    pub can_attack: bool,
    pub can_defend: bool,
    pub fighting: bool,
    pub attacking: bool,
    pub defending: bool,
    pub combat: bool,
    pub hurt: bool,
    pub dead: bool,
    pub max_hit_points: i32,
    pub curr_hit_points: i32,
    pub base_combat: i32,
    pub curr_combat: i32,
    pub combat_limit: i32,
    pub air_combat_limit: i32,
    pub fortify_modifier: i32,
    pub experience_needed: i32,
    pub attack_xp_value: i32,
    pub defense_xp_value: i32,
    pub max_xp_value: i32,
    pub special_cargo: i32,
    pub domain_cargo: i32,
    pub cargo: i32,
    pub cargo_space: i32,
    pub cargo_space_available: i32,
    pub has_cargo: bool,
    pub full: bool,
    pub cargo_can_move: bool,
    pub automated: bool,
    pub waiting: bool,
    pub fortifyable: bool,
    pub made_interception: bool,
    pub promotion_ready: bool,
    pub animal: bool,
    pub only_defensive: bool,
    pub rival_territory: bool,
    pub military_happiness: bool,
    pub spy: bool,
    pub found: bool,
    pub golden_age: bool,
    pub last_move_turn: i32,
    pub game_turn_created: i32,
    pub experience_percent: i32,
}

impl UnitDetailState {
    pub fn unit_ref(&self) -> UnitRef {
        UnitRef::new(self.player, self.unit)
    }

    pub fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnitPromotionState {
    pub player: i32,
    pub unit: i32,
    pub promotion: i32,
    pub has: bool,
}

impl UnitPromotionState {
    pub fn unit_ref(&self) -> UnitRef {
        UnitRef::new(self.player, self.unit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct KilledUnit {
    pub player: i32,
    pub unit: i32,
    pub killed: bool,
}

impl KilledUnit {
    pub fn unit_ref(&self) -> UnitRef {
        UnitRef::new(self.player, self.unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_unit_state() {
        let unit: UnitState = serde_json::from_value(json!({
            "player": 0,
            "unit": 42,
            "unit_type": 1,
            "unit_ai": 2,
            "domain": 0,
            "x": 10,
            "y": 11,
            "damage": 0,
            "experience": 2,
            "level": 1,
            "moves": 0,
            "max_moves": 2,
            "base_combat": 3,
            "cargo": 0,
            "fortify_turns": 0,
            "immobile_timer": 0,
            "made_attack": false,
            "promotions": [1, 4]
        }))
        .unwrap();

        assert_eq!(unit.unit_ref(), UnitRef::new(0, 42));
        assert_eq!(unit.plot(), Plot::new(10, 11));
        assert_eq!(unit.promotions, vec![1, 4]);
    }

    #[test]
    fn decodes_unit_detail_state() {
        let detail: UnitDetailState = serde_json::from_str(
            r#"{
                "player":0,
                "unit":42,
                "unit_type":1,
                "unit_ai":2,
                "domain":0,
                "unit_combat":3,
                "special_unit":-1,
                "x":10,
                "y":11,
                "area":5,
                "group":9,
                "in_group":true,
                "group_head":true,
                "base_moves":2,
                "max_moves":4,
                "moves_left":2,
                "can_move":true,
                "has_moved":false,
                "visibility_range":1,
                "air_range":0,
                "nuke_range":-1,
                "can_build_route":false,
                "build_type":-1,
                "work_rate":0,
                "max_work_rate":0,
                "can_fight":true,
                "can_attack":true,
                "can_defend":true,
                "fighting":false,
                "attacking":false,
                "defending":false,
                "combat":false,
                "hurt":false,
                "dead":false,
                "max_hit_points":100,
                "curr_hit_points":100,
                "base_combat":3,
                "curr_combat":300,
                "combat_limit":100,
                "air_combat_limit":100,
                "fortify_modifier":0,
                "experience_needed":2,
                "attack_xp_value":4,
                "defense_xp_value":2,
                "max_xp_value":10,
                "special_cargo":-1,
                "domain_cargo":-1,
                "cargo":0,
                "cargo_space":0,
                "cargo_space_available":0,
                "has_cargo":false,
                "full":false,
                "cargo_can_move":true,
                "automated":false,
                "waiting":false,
                "fortifyable":true,
                "made_interception":false,
                "promotion_ready":false,
                "animal":false,
                "only_defensive":false,
                "rival_territory":false,
                "military_happiness":true,
                "spy":false,
                "found":false,
                "golden_age":false,
                "last_move_turn":41,
                "game_turn_created":1,
                "experience_percent":0
            }"#,
        )
        .unwrap();

        assert_eq!(detail.unit_ref(), UnitRef::new(0, 42));
        assert_eq!(detail.plot(), Plot::new(10, 11));
        assert!(detail.can_attack);
        assert_eq!(detail.curr_hit_points, 100);
    }

    #[test]
    fn decodes_unit_promotion_and_kill_results() {
        let promotion: UnitPromotionState = serde_json::from_value(json!({
            "player": 0,
            "unit": 42,
            "promotion": 3,
            "has": true
        }))
        .unwrap();
        assert_eq!(promotion.unit_ref(), UnitRef::new(0, 42));
        assert!(promotion.has);

        let killed: KilledUnit = serde_json::from_value(json!({
            "player": 0,
            "unit": 42,
            "killed": true
        }))
        .unwrap();
        assert_eq!(killed.unit_ref(), UnitRef::new(0, 42));
        assert!(killed.killed);
    }
}
