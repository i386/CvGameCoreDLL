use crate::types::{PlayerId, Plot, TeamId, UnitRef};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SelectionGroupMissionState {
    pub mission: i32,
    pub data1: i32,
    pub data2: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SelectionGroupMissionCheck {
    pub player: i32,
    pub group: i32,
    pub mission: i32,
    pub data1: i32,
    pub data2: i32,
    pub x: i32,
    pub y: i32,
    pub test_visible: bool,
    pub use_cache: bool,
    pub can_start: bool,
}

impl SelectionGroupMissionCheck {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }

    pub fn plot(&self) -> Option<Plot> {
        (self.x >= 0 && self.y >= 0).then_some(Plot::new(self.x, self.y))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SelectionGroupCommandCheck {
    pub player: i32,
    pub group: i32,
    pub command: i32,
    pub data1: i32,
    pub data2: i32,
    pub test_visible: bool,
    pub use_cache: bool,
    pub can_do: bool,
}

impl SelectionGroupCommandCheck {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnitCommandResult {
    pub player: i32,
    pub group: i32,
    pub command: i32,
    pub data1: i32,
    pub data2: i32,
    pub executed: bool,
    pub executing_player: i32,
    pub executing_unit: i32,
    pub unit_exists: bool,
    pub x: i32,
    pub y: i32,
    pub current_group: i32,
}

impl UnitCommandResult {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }

    pub fn executing_unit_ref(&self) -> Option<UnitRef> {
        (self.executing_player >= 0 && self.executing_unit >= 0)
            .then_some(UnitRef::new(self.executing_player, self.executing_unit))
    }

    pub fn plot(&self) -> Option<Plot> {
        (self.x >= 0 && self.y >= 0).then_some(Plot::new(self.x, self.y))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SelectionGroupState {
    pub player: i32,
    pub group: i32,
    pub team: i32,
    pub x: i32,
    pub y: i32,
    pub area: i32,
    pub domain: i32,
    pub head_player: i32,
    pub head_unit: i32,
    pub head_unit_type: i32,
    pub activity: i32,
    pub automate: i32,
    pub automated: bool,
    pub mission_timer: i32,
    pub units: i32,
    pub cargo: i32,
    pub base_moves: i32,
    pub can_all_move: bool,
    pub can_any_move: bool,
    pub has_moved: bool,
    pub waiting: bool,
    pub full: bool,
    pub has_cargo: bool,
    pub can_fight: bool,
    pub can_defend: bool,
    pub has_worker: bool,
    pub ready_to_select: bool,
    pub ready_to_move: bool,
    pub ready_to_auto: bool,
    pub mission_queue_length: i32,
    pub missions: Vec<SelectionGroupMissionState>,
}

impl SelectionGroupState {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }

    pub fn team_id(&self) -> TeamId {
        TeamId(self.team)
    }

    pub fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }

    pub fn head_unit_ref(&self) -> Option<UnitRef> {
        (self.head_player >= 0 && self.head_unit >= 0)
            .then_some(UnitRef::new(self.head_player, self.head_unit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_selection_group_state() {
        let state: SelectionGroupState = serde_json::from_value(json!({
            "player": 0,
            "group": 9,
            "team": 0,
            "x": 10,
            "y": 12,
            "area": 5,
            "domain": 0,
            "head_player": 0,
            "head_unit": 123,
            "head_unit_type": 1,
            "activity": 0,
            "automate": -1,
            "automated": false,
            "mission_timer": 0,
            "units": 2,
            "cargo": 0,
            "base_moves": 1,
            "can_all_move": true,
            "can_any_move": true,
            "has_moved": false,
            "waiting": false,
            "full": false,
            "has_cargo": false,
            "can_fight": true,
            "can_defend": true,
            "has_worker": false,
            "ready_to_select": true,
            "ready_to_move": true,
            "ready_to_auto": false,
            "mission_queue_length": 1,
            "missions": [
                { "mission": 1, "data1": 11, "data2": 12 }
            ]
        }))
        .unwrap();

        assert_eq!(state.player_id(), PlayerId(0));
        assert_eq!(state.team_id(), TeamId(0));
        assert_eq!(state.plot(), Plot::new(10, 12));
        assert_eq!(state.head_unit_ref(), Some(UnitRef::new(0, 123)));
        assert_eq!(state.missions[0].data1, 11);
    }

    #[test]
    fn decodes_selection_group_mission_check() {
        let check: SelectionGroupMissionCheck = serde_json::from_value(json!({
            "player": 0,
            "group": 9,
            "mission": 1,
            "data1": 11,
            "data2": 12,
            "x": 11,
            "y": 12,
            "test_visible": false,
            "use_cache": true,
            "can_start": true
        }))
        .unwrap();

        assert_eq!(check.player_id(), PlayerId(0));
        assert_eq!(check.plot(), Some(Plot::new(11, 12)));
        assert!(check.can_start);
    }

    #[test]
    fn decodes_selection_group_command_check() {
        let check: SelectionGroupCommandCheck = serde_json::from_value(json!({
            "player": 0,
            "group": 9,
            "command": 10,
            "data1": 1,
            "data2": 99,
            "test_visible": true,
            "use_cache": false,
            "can_do": true
        }))
        .unwrap();

        assert_eq!(check.player_id(), PlayerId(0));
        assert!(check.can_do);
    }

    #[test]
    fn decodes_unit_command_result() {
        let result: UnitCommandResult = serde_json::from_value(json!({
            "player": 0,
            "group": 9,
            "command": 10,
            "data1": 1,
            "data2": 99,
            "executed": true,
            "executing_player": 0,
            "executing_unit": 42,
            "unit_exists": true,
            "x": 11,
            "y": 12,
            "current_group": 9
        }))
        .unwrap();

        assert_eq!(result.player_id(), PlayerId(0));
        assert_eq!(result.executing_unit_ref(), Some(UnitRef::new(0, 42)));
        assert_eq!(result.plot(), Some(Plot::new(11, 12)));
    }
}
