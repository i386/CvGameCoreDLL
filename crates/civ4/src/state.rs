use crate::types::{CityRef, PlayerId, Plot, TeamId, UnitRef};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlayerState {
    pub player: i32,
    pub team: i32,
    pub alive: bool,
    pub human: bool,
    pub gold: i32,
    pub cities: i32,
    pub units: i32,
    pub population: i32,
}

impl PlayerState {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }

    pub fn team_id(&self) -> TeamId {
        TeamId(self.team)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlayerOptions {
    pub player: i32,
    pub team: i32,
    pub state_religion: i32,
    pub current_research: i32,
    pub civics: Vec<i32>,
}

impl PlayerOptions {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }

    pub fn team_id(&self) -> TeamId {
        TeamId(self.team)
    }

    pub fn state_religion(&self) -> Option<i32> {
        (self.state_religion >= 0).then_some(self.state_religion)
    }

    pub fn current_research(&self) -> Option<i32> {
        (self.current_research >= 0).then_some(self.current_research)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MapState {
    pub width: i32,
    pub height: i32,
    pub plots: i32,
    pub land_plots: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlotState {
    pub plot: Plot,
    pub owner: Option<PlayerId>,
    pub terrain: i32,
    pub feature: i32,
    pub bonus: i32,
    pub improvement: i32,
    pub route: i32,
    pub water: bool,
    pub peak: bool,
    pub units: i32,
    pub city: Option<CityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct PlotStateResult {
    pub x: i32,
    pub y: i32,
    pub owner: i32,
    pub terrain: i32,
    pub feature: i32,
    pub bonus: i32,
    pub improvement: i32,
    pub route: i32,
    pub water: bool,
    pub peak: bool,
    pub units: i32,
    pub city_player: i32,
    pub city: i32,
}

impl From<PlotStateResult> for PlotState {
    fn from(value: PlotStateResult) -> Self {
        Self {
            plot: Plot::new(value.x, value.y),
            owner: (value.owner >= 0).then_some(PlayerId(value.owner)),
            terrain: value.terrain,
            feature: value.feature,
            bonus: value.bonus,
            improvement: value.improvement,
            route: value.route,
            water: value.water,
            peak: value.peak,
            units: value.units,
            city: (value.city_player >= 0 && value.city >= 0)
                .then_some(CityRef::new(value.city_player, value.city)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CityState {
    pub player: i32,
    pub city: i32,
    pub x: i32,
    pub y: i32,
    pub population: i32,
    pub culture: i32,
    pub production: i32,
    pub production_needed: i32,
    pub production_unit: i32,
    pub production_unit_ai: i32,
    pub production_building: i32,
    pub production_project: i32,
    pub production_process: i32,
    pub order_queue_length: i32,
    pub occupation_timer: i32,
    pub hurry_anger_timer: i32,
}

impl CityState {
    pub fn city_ref(&self) -> CityRef {
        CityRef::new(self.player, self.city)
    }

    pub fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CityBuildingState {
    pub player: i32,
    pub city: i32,
    pub building: i32,
    pub real: i32,
    pub free: i32,
    pub active: bool,
}

impl CityBuildingState {
    pub fn city_ref(&self) -> CityRef {
        CityRef::new(self.player, self.city)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CityReligionState {
    pub player: i32,
    pub city: i32,
    pub religion: i32,
    pub has: bool,
}

impl CityReligionState {
    pub fn city_ref(&self) -> CityRef {
        CityRef::new(self.player, self.city)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CityCorporationState {
    pub player: i32,
    pub city: i32,
    pub corporation: i32,
    pub has: bool,
}

impl CityCorporationState {
    pub fn city_ref(&self) -> CityRef {
        CityRef::new(self.player, self.city)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CityBuildingClassChange {
    pub player: i32,
    pub city: i32,
    pub building_class: i32,
    pub happiness: i32,
    pub health: i32,
}

impl CityBuildingClassChange {
    pub fn city_ref(&self) -> CityRef {
        CityRef::new(self.player, self.city)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TeamTechState {
    pub team: i32,
    pub tech: i32,
    pub has: bool,
    pub progress: i32,
}

impl TeamTechState {
    pub fn team_id(&self) -> TeamId {
        TeamId(self.team)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TeamState {
    pub team: i32,
    pub alive: bool,
    pub ever_alive: bool,
    pub human: bool,
    pub barbarian: bool,
    pub minor: bool,
    pub leader: i32,
    pub secretary: i32,
    pub members: i32,
    pub cities: i32,
    pub population: i32,
    pub land: i32,
    pub assets: i32,
    pub power: i32,
    pub defensive_power: i32,
    pub at_war_count: i32,
    pub has_met_count: i32,
    pub defensive_pact_count: i32,
    pub vassal_count: i32,
    pub vassal: bool,
    pub nuke_interception: i32,
    pub map_trading: bool,
    pub tech_trading: bool,
    pub gold_trading: bool,
    pub open_borders_trading: bool,
    pub defensive_pact_trading: bool,
    pub permanent_alliance_trading: bool,
    pub vassal_trading: bool,
}

impl TeamState {
    pub fn team_id(&self) -> TeamId {
        TeamId(self.team)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TeamRelationState {
    pub team: i32,
    pub other_team: i32,
    pub has_met: bool,
    pub at_war: bool,
    pub can_declare_war: bool,
    pub can_change_war_peace: bool,
    pub permanent_war_peace: bool,
    pub open_borders: bool,
    pub defensive_pact: bool,
    pub force_peace: bool,
    pub vassal: bool,
    pub master: bool,
    pub war_weariness: i32,
    pub stolen_visibility_timer: i32,
    pub war_plan: i32,
}

impl TeamRelationState {
    pub fn team_id(&self) -> TeamId {
        TeamId(self.team)
    }

    pub fn other_team_id(&self) -> TeamId {
        TeamId(self.other_team)
    }
}

#[derive(Deserialize)]
pub(crate) struct GameTurnResult {
    pub turn: i32,
}

#[derive(Deserialize)]
pub(crate) struct PlayerGoldResult {
    pub gold: i32,
}

#[derive(Deserialize)]
pub(crate) struct PlayersResult {
    pub players: Vec<PlayerState>,
}

#[derive(Deserialize)]
pub(crate) struct PlayerCitiesResult {
    pub cities: Vec<CityState>,
}

#[derive(Deserialize)]
pub(crate) struct PlayerUnitsResult {
    pub units: Vec<UnitState>,
}

#[derive(Deserialize)]
pub(crate) struct TeamsResult {
    pub teams: Vec<TeamState>,
}

#[derive(Deserialize)]
pub(crate) struct ModStateResult {
    pub json: String,
}

#[derive(Deserialize)]
pub(crate) struct SetModStateResult {
    pub bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestState {
        schema_version: u32,
        enabled: bool,
    }

    #[test]
    fn plot_state_maps_negative_owner_and_city_to_none() {
        let result = PlotStateResult {
            x: 1,
            y: 2,
            owner: -1,
            terrain: 3,
            feature: -1,
            bonus: -1,
            improvement: -1,
            route: -1,
            water: false,
            peak: false,
            units: 0,
            city_player: -1,
            city: -1,
        };

        let state = PlotState::from(result);
        assert_eq!(state.plot, Plot::new(1, 2));
        assert_eq!(state.owner, None);
        assert_eq!(state.city, None);
    }

    #[test]
    fn mod_state_round_trips_through_json() {
        let state = TestState {
            schema_version: 1,
            enabled: true,
        };
        let json_state = serde_json::to_string(&state).unwrap();
        let decoded: TestState = serde_json::from_str(&json_state).unwrap();

        assert_eq!(decoded, state);
    }

    #[test]
    fn decodes_collection_query_results() {
        let players: PlayersResult = serde_json::from_value(json!({
            "players": [{
                "player": 0,
                "team": 0,
                "alive": true,
                "human": true,
                "gold": 50,
                "cities": 1,
                "units": 2,
                "population": 3
            }]
        }))
        .unwrap();
        assert_eq!(players.players[0].player_id(), PlayerId(0));

        let cities: PlayerCitiesResult = serde_json::from_value(json!({
            "player": 0,
            "cities": [{
                "player": 0,
                "city": 7,
                "x": 10,
                "y": 11,
                "population": 4,
                "culture": 99,
                "production": 10,
                "production_needed": 35,
                "production_unit": 1,
                "production_unit_ai": 2,
                "production_building": -1,
                "production_project": -1,
                "production_process": -1,
                "order_queue_length": 1,
                "occupation_timer": 0,
                "hurry_anger_timer": 0
            }]
        }))
        .unwrap();
        assert_eq!(cities.cities[0].city_ref(), CityRef::new(0, 7));

        let units: PlayerUnitsResult = serde_json::from_value(json!({
            "player": 0,
            "units": [{
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
            }]
        }))
        .unwrap();
        assert_eq!(units.units[0].unit_ref(), UnitRef::new(0, 42));
        assert_eq!(units.units[0].promotions, vec![1, 4]);
    }

    #[test]
    fn decodes_city_relation_states() {
        let building: CityBuildingState = serde_json::from_value(json!({
            "player": 0,
            "city": 7,
            "building": 12,
            "real": 1,
            "free": 0,
            "active": true
        }))
        .unwrap();
        assert_eq!(building.city_ref(), CityRef::new(0, 7));
        assert_eq!(building.building, 12);
        assert!(building.active);

        let religion: CityReligionState = serde_json::from_value(json!({
            "player": 0,
            "city": 7,
            "religion": 2,
            "has": true
        }))
        .unwrap();
        assert_eq!(religion.city_ref(), CityRef::new(0, 7));
        assert!(religion.has);

        let corporation: CityCorporationState = serde_json::from_value(json!({
            "player": 0,
            "city": 7,
            "corporation": 3,
            "has": false
        }))
        .unwrap();
        assert_eq!(corporation.city_ref(), CityRef::new(0, 7));
        assert!(!corporation.has);

        let building_class: CityBuildingClassChange = serde_json::from_value(json!({
            "player": 0,
            "city": 7,
            "building_class": 4,
            "happiness": 1,
            "health": -1
        }))
        .unwrap();
        assert_eq!(building_class.city_ref(), CityRef::new(0, 7));
        assert_eq!(building_class.happiness, 1);
        assert_eq!(building_class.health, -1);
    }

    #[test]
    fn decodes_player_options_and_team_tech_state() {
        let options: PlayerOptions = serde_json::from_value(json!({
            "player": 0,
            "team": 0,
            "state_religion": -1,
            "current_research": 3,
            "civics": [1, 2, 3, 4, 5]
        }))
        .unwrap();

        assert_eq!(options.player_id(), PlayerId(0));
        assert_eq!(options.team_id(), TeamId(0));
        assert_eq!(options.state_religion(), None);
        assert_eq!(options.current_research(), Some(3));
        assert_eq!(options.civics, vec![1, 2, 3, 4, 5]);

        let tech: TeamTechState = serde_json::from_value(json!({
            "team": 0,
            "tech": 7,
            "has": true,
            "progress": 42
        }))
        .unwrap();

        assert_eq!(tech.team_id(), TeamId(0));
        assert_eq!(tech.tech, 7);
        assert!(tech.has);
        assert_eq!(tech.progress, 42);
    }

    #[test]
    fn decodes_team_state_and_relation_state() {
        let teams: TeamsResult = serde_json::from_value(json!({
            "teams": [{
                "team": 0,
                "alive": true,
                "ever_alive": true,
                "human": true,
                "barbarian": false,
                "minor": false,
                "leader": 0,
                "secretary": 0,
                "members": 1,
                "cities": 2,
                "population": 5,
                "land": 10,
                "assets": 100,
                "power": 50,
                "defensive_power": 40,
                "at_war_count": 1,
                "has_met_count": 3,
                "defensive_pact_count": 0,
                "vassal_count": 0,
                "vassal": false,
                "nuke_interception": 0,
                "map_trading": true,
                "tech_trading": true,
                "gold_trading": true,
                "open_borders_trading": true,
                "defensive_pact_trading": false,
                "permanent_alliance_trading": false,
                "vassal_trading": false
            }]
        }))
        .unwrap();
        assert_eq!(teams.teams[0].team_id(), TeamId(0));
        assert!(teams.teams[0].alive);

        let relation: TeamRelationState = serde_json::from_value(json!({
            "team": 0,
            "other_team": 1,
            "has_met": true,
            "at_war": false,
            "can_declare_war": true,
            "can_change_war_peace": true,
            "permanent_war_peace": false,
            "open_borders": true,
            "defensive_pact": false,
            "force_peace": false,
            "vassal": false,
            "master": false,
            "war_weariness": 0,
            "stolen_visibility_timer": 0,
            "war_plan": -1
        }))
        .unwrap();
        assert_eq!(relation.team_id(), TeamId(0));
        assert_eq!(relation.other_team_id(), TeamId(1));
        assert!(relation.has_met);
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
