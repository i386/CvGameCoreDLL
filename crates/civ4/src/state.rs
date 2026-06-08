pub use crate::plot_state::{MapState, PlotCultureState, PlotState, PlotVisibilityState};
pub use crate::selection_group_state::{
    SelectionGroupCommandCheck, SelectionGroupMissionCheck, SelectionGroupMissionState,
    SelectionGroupState, UnitCommandResult, UnitGroupJoinCheck,
};
use crate::types::{CityRef, PlayerId, Plot, TeamId};
pub use crate::unit_state::{KilledUnit, UnitDetailState, UnitPromotionState, UnitState};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GameState {
    pub turn: i32,
    pub year: i32,
    pub elapsed_turns: i32,
    pub start_turn: i32,
    pub start_year: i32,
    pub estimate_end_turn: i32,
    pub max_turns: i32,
    pub max_city_elimination: i32,
    pub advanced_start_points: i32,
    pub target_score: i32,
    pub active_player: i32,
    pub active_team: i32,
    pub pause_player: i32,
    pub paused: bool,
    pub winner: i32,
    pub victory: i32,
    pub game_state: i32,
    pub start_era: i32,
    pub current_era: i32,
    pub calendar: i32,
    pub game_speed: i32,
    pub handicap: i32,
    pub num_cities: i32,
    pub num_civ_cities: i32,
    pub total_population: i32,
    pub num_human_players: i32,
    pub num_deals: i32,
    pub nukes_exploded: i32,
    pub ai_auto_play: i32,
    pub network_multiplayer: bool,
    pub game_multiplayer: bool,
    pub team_game: bool,
    pub debug_mode: bool,
    pub final_initialized: bool,
}

impl GameState {
    pub fn active_player_id(&self) -> Option<PlayerId> {
        (self.active_player >= 0).then_some(PlayerId(self.active_player))
    }

    pub fn active_team_id(&self) -> Option<TeamId> {
        (self.active_team >= 0).then_some(TeamId(self.active_team))
    }

    pub fn pause_player_id(&self) -> Option<PlayerId> {
        (self.pause_player >= 0).then_some(PlayerId(self.pause_player))
    }

    pub fn winner_team_id(&self) -> Option<TeamId> {
        (self.winner >= 0).then_some(TeamId(self.winner))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GameOptionState {
    pub option: i32,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MultiplayerOptionState {
    pub option: i32,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ForceControlState {
    pub control: i32,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlayerState {
    pub player: i32,
    pub team: i32,
    pub alive: bool,
    pub ever_alive: bool,
    pub human: bool,
    pub barbarian: bool,
    pub minor: bool,
    pub playable: bool,
    pub founded_first_city: bool,
    pub extended_game: bool,
    pub turn_active: bool,
    pub turn_done: bool,
    pub end_turn: bool,
    pub auto_moves: bool,
    pub strike: bool,
    pub handicap: i32,
    pub civilization: i32,
    pub leader: i32,
    pub personality: i32,
    pub current_era: i32,
    pub parent: i32,
    pub player_color: i32,
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

    pub fn parent_id(&self) -> Option<PlayerId> {
        (self.parent >= 0).then_some(PlayerId(self.parent))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlayerEconomyState {
    pub player: i32,
    pub gold: i32,
    pub gold_per_turn: i32,
    pub advanced_start_points: i32,
    pub golden_age_turns: i32,
    pub golden_age_length: i32,
    pub golden_age: bool,
    pub num_unit_golden_ages: i32,
    pub units_required_for_golden_age: i32,
    pub units_golden_age_ready: i32,
    pub anarchy_turns: i32,
    pub anarchy: bool,
    pub strike_turns: i32,
    pub strike: bool,
    pub combat_experience: i32,
    pub gold_per_unit: i32,
    pub gold_per_military_unit: i32,
    pub total_culture: i32,
    pub commerce_percent: Vec<i32>,
    pub commerce_rate: Vec<i32>,
    pub commerce_rate_modifier: Vec<i32>,
}

impl PlayerEconomyState {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlayerGoldPerTurnState {
    pub player: i32,
    pub other_player: i32,
    pub value: i32,
}

impl PlayerGoldPerTurnState {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }

    pub fn other_player_id(&self) -> PlayerId {
        PlayerId(self.other_player)
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
pub struct CityDetailState {
    pub player: i32,
    pub city: i32,
    pub x: i32,
    pub y: i32,
    pub production: bool,
    pub food_production: bool,
    pub disorder: bool,
    pub occupation: bool,
    pub we_love_the_king_day: bool,
    pub food: i32,
    pub food_kept: i32,
    pub growth_threshold: i32,
    pub food_consumption: i32,
    pub food_difference: i32,
    pub happy_level: i32,
    pub unhappy_level: i32,
    pub angry_population: i32,
    pub good_health: i32,
    pub bad_health: i32,
    pub health_rate: i32,
    pub unhealthy_population: i32,
    pub maintenance: i32,
    pub distance_maintenance: i32,
    pub num_cities_maintenance: i32,
    pub colony_maintenance: i32,
    pub corporation_maintenance: i32,
    pub production_left: i32,
    pub current_production_difference: i32,
    pub defense_damage: i32,
    pub total_defense: i32,
    pub defense_modifier: i32,
    pub yield_rate: Vec<i32>,
    pub commerce_rate: Vec<i32>,
    pub commerce_rate_times100: Vec<i32>,
}

impl CityDetailState {
    pub fn city_ref(&self) -> CityRef {
        CityRef::new(self.player, self.city)
    }

    pub fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CityProductionOptions {
    pub player: i32,
    pub city: i32,
    pub continue_current: bool,
    pub test_visible: bool,
    pub ignore_cost: bool,
    pub ignore_upgrades: bool,
    pub units: Vec<i32>,
    pub buildings: Vec<i32>,
    pub projects: Vec<i32>,
    pub processes: Vec<i32>,
}

impl CityProductionOptions {
    pub fn city_ref(&self) -> CityRef {
        CityRef::new(self.player, self.city)
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
    fn decodes_game_state_and_options() {
        let state: GameState = serde_json::from_value(json!({
            "turn": 42,
            "year": 1000,
            "elapsed_turns": 40,
            "start_turn": 0,
            "start_year": -4000,
            "estimate_end_turn": 500,
            "max_turns": 460,
            "max_city_elimination": 0,
            "advanced_start_points": 0,
            "target_score": 0,
            "active_player": 0,
            "active_team": 0,
            "pause_player": -1,
            "paused": false,
            "winner": -1,
            "victory": -1,
            "game_state": 0,
            "start_era": 0,
            "current_era": 1,
            "calendar": 0,
            "game_speed": 2,
            "handicap": 3,
            "num_cities": 12,
            "num_civ_cities": 11,
            "total_population": 42,
            "num_human_players": 1,
            "num_deals": 2,
            "nukes_exploded": 0,
            "ai_auto_play": 0,
            "network_multiplayer": false,
            "game_multiplayer": false,
            "team_game": false,
            "debug_mode": false,
            "final_initialized": true
        }))
        .unwrap();
        assert_eq!(state.active_player_id(), Some(PlayerId(0)));
        assert_eq!(state.active_team_id(), Some(TeamId(0)));
        assert_eq!(state.pause_player_id(), None);
        assert_eq!(state.winner_team_id(), None);

        let option: GameOptionState = serde_json::from_value(json!({
            "option": 1,
            "enabled": true
        }))
        .unwrap();
        assert!(option.enabled);

        let mp_option: MultiplayerOptionState = serde_json::from_value(json!({
            "option": 2,
            "enabled": false
        }))
        .unwrap();
        assert!(!mp_option.enabled);

        let force_control: ForceControlState = serde_json::from_value(json!({
            "control": 3,
            "enabled": true
        }))
        .unwrap();
        assert!(force_control.enabled);
    }

    #[test]
    fn decodes_collection_query_results() {
        let players: PlayersResult = serde_json::from_value(json!({
            "players": [{
                "player": 0,
                "team": 0,
                "alive": true,
                "ever_alive": true,
                "human": true,
                "barbarian": false,
                "minor": false,
                "playable": true,
                "founded_first_city": true,
                "extended_game": false,
                "turn_active": true,
                "turn_done": false,
                "end_turn": false,
                "auto_moves": false,
                "strike": false,
                "handicap": 3,
                "civilization": 1,
                "leader": 2,
                "personality": 2,
                "current_era": 1,
                "parent": -1,
                "player_color": 4,
                "gold": 50,
                "cities": 1,
                "units": 2,
                "population": 3
            }]
        }))
        .unwrap();
        assert_eq!(players.players[0].player_id(), PlayerId(0));
        assert_eq!(players.players[0].parent_id(), None);

        let economy: PlayerEconomyState = serde_json::from_value(json!({
            "player": 0,
            "gold": 50,
            "gold_per_turn": 2,
            "advanced_start_points": -1,
            "golden_age_turns": 0,
            "golden_age_length": 8,
            "golden_age": false,
            "num_unit_golden_ages": 1,
            "units_required_for_golden_age": 3,
            "units_golden_age_ready": 2,
            "anarchy_turns": 0,
            "anarchy": false,
            "strike_turns": 0,
            "strike": false,
            "combat_experience": 4,
            "gold_per_unit": 1,
            "gold_per_military_unit": 1,
            "total_culture": 99,
            "commerce_percent": [0, 80, 20, 0],
            "commerce_rate": [4, 12, 2, 0],
            "commerce_rate_modifier": [0, 25, 0, 0]
        }))
        .unwrap();
        assert_eq!(economy.player_id(), PlayerId(0));
        assert_eq!(economy.commerce_percent[1], 80);

        let gold_per_turn: PlayerGoldPerTurnState = serde_json::from_value(json!({
            "player": 0,
            "other_player": 1,
            "value": -3
        }))
        .unwrap();
        assert_eq!(gold_per_turn.player_id(), PlayerId(0));
        assert_eq!(gold_per_turn.other_player_id(), PlayerId(1));

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

        let city_detail: CityDetailState = serde_json::from_value(json!({
            "player": 0,
            "city": 7,
            "x": 10,
            "y": 11,
            "production": true,
            "food_production": false,
            "disorder": false,
            "occupation": false,
            "we_love_the_king_day": true,
            "food": 12,
            "food_kept": 4,
            "growth_threshold": 26,
            "food_consumption": 8,
            "food_difference": 3,
            "happy_level": 7,
            "unhappy_level": 5,
            "angry_population": 0,
            "good_health": 6,
            "bad_health": 4,
            "health_rate": 0,
            "unhealthy_population": 0,
            "maintenance": 3,
            "distance_maintenance": 1,
            "num_cities_maintenance": 2,
            "colony_maintenance": 0,
            "corporation_maintenance": 0,
            "production_left": 12,
            "current_production_difference": 5,
            "defense_damage": 0,
            "total_defense": 40,
            "defense_modifier": 40,
            "yield_rate": [11, 8, 12],
            "commerce_rate": [6, 14, 2, 0],
            "commerce_rate_times100": [600, 1400, 200, 0]
        }))
        .unwrap();
        assert_eq!(city_detail.city_ref(), CityRef::new(0, 7));
        assert_eq!(city_detail.plot(), Plot::new(10, 11));
        assert_eq!(city_detail.yield_rate[0], 11);
        assert_eq!(city_detail.commerce_rate_times100[1], 1400);

        let production_options: CityProductionOptions = serde_json::from_value(json!({
            "player": 0,
            "city": 7,
            "continue_current": true,
            "test_visible": false,
            "ignore_cost": false,
            "ignore_upgrades": true,
            "units": [1, 2],
            "buildings": [3],
            "projects": [4],
            "processes": [5, 6]
        }))
        .unwrap();
        assert_eq!(production_options.city_ref(), CityRef::new(0, 7));
        assert!(production_options.continue_current);
        assert_eq!(production_options.units, vec![1, 2]);
        assert_eq!(production_options.processes, vec![5, 6]);

        let units: PlayerUnitsResult = serde_json::from_value(json!({
            "player": 0,
            "units": []
        }))
        .unwrap();
        assert!(units.units.is_empty());
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
}
