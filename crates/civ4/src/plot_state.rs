use crate::types::{CityRef, PlayerId, Plot, TeamId};
use serde::Deserialize;

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
    pub area: i32,
    pub area_water: bool,
    pub area_tiles: i32,
    pub area_cities: i32,
    pub owner_area_cities: i32,
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
    #[serde(default = "default_negative_one")]
    pub area: i32,
    #[serde(default)]
    pub area_water: bool,
    #[serde(default)]
    pub area_tiles: i32,
    #[serde(default)]
    pub area_cities: i32,
    #[serde(default)]
    pub owner_area_cities: i32,
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
            area: value.area,
            area_water: value.area_water,
            area_tiles: value.area_tiles,
            area_cities: value.area_cities,
            owner_area_cities: value.owner_area_cities,
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

fn default_negative_one() -> i32 {
    -1
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlotCultureState {
    pub x: i32,
    pub y: i32,
    pub player: i32,
    pub culture: i32,
    pub total_culture: i32,
    pub culture_percent: i32,
}

impl PlotCultureState {
    pub fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }

    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlotVisibilityState {
    pub x: i32,
    pub y: i32,
    pub team: i32,
    pub debug: bool,
    pub visible: bool,
    pub revealed: bool,
    pub revealed_owner: i32,
    pub revealed_team: i32,
    pub revealed_improvement: i32,
    pub revealed_route: i32,
}

impl PlotVisibilityState {
    pub fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }

    pub fn team_id(&self) -> TeamId {
        TeamId(self.team)
    }

    pub fn revealed_owner_id(&self) -> Option<PlayerId> {
        (self.revealed_owner >= 0).then_some(PlayerId(self.revealed_owner))
    }

    pub fn revealed_team_id(&self) -> Option<TeamId> {
        (self.revealed_team >= 0).then_some(TeamId(self.revealed_team))
    }

    pub fn revealed_improvement(&self) -> Option<i32> {
        (self.revealed_improvement >= 0).then_some(self.revealed_improvement)
    }

    pub fn revealed_route(&self) -> Option<i32> {
        (self.revealed_route >= 0).then_some(self.revealed_route)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plot_state_maps_negative_owner_and_city_to_none() {
        let result = PlotStateResult {
            x: 1,
            y: 2,
            owner: -1,
            area: -1,
            area_water: false,
            area_tiles: 0,
            area_cities: 0,
            owner_area_cities: 0,
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
    fn decodes_plot_culture_and_visibility_state() {
        let culture: PlotCultureState = serde_json::from_value(json!({
            "x": 10,
            "y": 12,
            "player": 0,
            "culture": 42,
            "total_culture": 50,
            "culture_percent": 84
        }))
        .unwrap();
        assert_eq!(culture.plot(), Plot::new(10, 12));
        assert_eq!(culture.player_id(), PlayerId(0));
        assert_eq!(culture.culture_percent, 84);

        let visibility: PlotVisibilityState = serde_json::from_value(json!({
            "x": 10,
            "y": 12,
            "team": 1,
            "debug": false,
            "visible": true,
            "revealed": true,
            "revealed_owner": -1,
            "revealed_team": -1,
            "revealed_improvement": 3,
            "revealed_route": -1
        }))
        .unwrap();
        assert_eq!(visibility.plot(), Plot::new(10, 12));
        assert_eq!(visibility.team_id(), TeamId(1));
        assert_eq!(visibility.revealed_owner_id(), None);
        assert_eq!(visibility.revealed_team_id(), None);
        assert_eq!(visibility.revealed_improvement(), Some(3));
        assert_eq!(visibility.revealed_route(), None);
    }
}
