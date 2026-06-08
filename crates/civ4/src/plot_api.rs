use crate::client::{BridgeClient, Result};
use crate::plot_state::{
    MapState, PlotCultureState, PlotState, PlotStateResult, PlotVisibilityState,
};
use crate::types::{InfoType, PlayerId, Plot, TeamId};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotOwnerOptions {
    pub check_units: bool,
    pub update_plot_group: bool,
}

impl Default for PlotOwnerOptions {
    fn default() -> Self {
        Self {
            check_units: true,
            update_plot_group: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotTerrainOptions {
    pub recalculate: bool,
    pub rebuild_graphics: bool,
}

impl Default for PlotTerrainOptions {
    fn default() -> Self {
        Self {
            recalculate: true,
            rebuild_graphics: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlotFeatureOptions {
    pub variety: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotRouteOptions {
    pub update_plot_group: bool,
}

impl Default for PlotRouteOptions {
    fn default() -> Self {
        Self {
            update_plot_group: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotCultureOptions {
    pub update: bool,
    pub update_plot_groups: bool,
}

impl Default for PlotCultureOptions {
    fn default() -> Self {
        Self {
            update: true,
            update_plot_groups: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotCultureChangeOptions {
    pub update: bool,
}

impl Default for PlotCultureChangeOptions {
    fn default() -> Self {
        Self { update: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotRevealedOptions {
    pub terrain_only: bool,
    pub from_team: Option<TeamId>,
    pub update_plot_group: bool,
}

impl Default for PlotRevealedOptions {
    fn default() -> Self {
        Self {
            terrain_only: false,
            from_team: None,
            update_plot_group: true,
        }
    }
}

impl BridgeClient {
    pub fn get_map_state(&mut self) -> Result<MapState> {
        self.query("get_map_state", json!({}))
    }

    pub fn get_plot_state(&mut self, plot: Plot) -> Result<PlotState> {
        let result: PlotStateResult =
            self.query("get_plot_state", json!({ "x": plot.x, "y": plot.y }))?;
        Ok(result.into())
    }

    pub fn get_plot_culture_state<P: Into<PlayerId>>(
        &mut self,
        plot: Plot,
        player: P,
    ) -> Result<PlotCultureState> {
        let player = player.into();
        self.query(
            "get_plot_culture_state",
            json!({ "x": plot.x, "y": plot.y, "player": player.0 }),
        )
    }

    pub fn get_plot_visibility_state<T: Into<TeamId>>(
        &mut self,
        plot: Plot,
        team: T,
    ) -> Result<PlotVisibilityState> {
        self.get_plot_visibility_state_with_debug(plot, team, false)
    }

    pub fn get_plot_visibility_state_with_debug<T: Into<TeamId>>(
        &mut self,
        plot: Plot,
        team: T,
        debug: bool,
    ) -> Result<PlotVisibilityState> {
        let team = team.into();
        self.query(
            "get_plot_visibility_state",
            json!({
                "x": plot.x,
                "y": plot.y,
                "team": team.0,
                "debug": if debug { 1 } else { 0 }
            }),
        )
    }

    pub fn set_plot_owner<P: Into<PlayerId>>(&mut self, plot: Plot, owner: P) -> Result<PlotState> {
        self.set_plot_owner_with_options(plot, owner, PlotOwnerOptions::default())
    }

    pub fn set_plot_owner_with_options<P: Into<PlayerId>>(
        &mut self,
        plot: Plot,
        owner: P,
        options: PlotOwnerOptions,
    ) -> Result<PlotState> {
        self.command_plot_state(
            "set_plot_owner",
            plot_owner_args(plot, owner.into().0, options),
        )
    }

    pub fn clear_plot_owner(&mut self, plot: Plot) -> Result<PlotState> {
        self.clear_plot_owner_with_options(plot, PlotOwnerOptions::default())
    }

    pub fn clear_plot_owner_with_options(
        &mut self,
        plot: Plot,
        options: PlotOwnerOptions,
    ) -> Result<PlotState> {
        self.command_plot_state("set_plot_owner", plot_owner_args(plot, -1, options))
    }

    pub fn set_plot_terrain<T>(&mut self, plot: Plot, terrain: T) -> Result<PlotState>
    where
        T: Into<InfoType>,
    {
        self.set_plot_terrain_with_options(plot, terrain, PlotTerrainOptions::default())
    }

    pub fn set_plot_terrain_with_options<T>(
        &mut self,
        plot: Plot,
        terrain: T,
        options: PlotTerrainOptions,
    ) -> Result<PlotState>
    where
        T: Into<InfoType>,
    {
        let mut args = plot_info_args(plot, "terrain", terrain.into());
        args["recalculate"] = json_bool(options.recalculate);
        args["rebuild_graphics"] = json_bool(options.rebuild_graphics);
        self.command_plot_state("set_plot_terrain", args)
    }

    pub fn set_plot_feature<F>(&mut self, plot: Plot, feature: F) -> Result<PlotState>
    where
        F: Into<InfoType>,
    {
        self.set_plot_feature_with_options(plot, feature, PlotFeatureOptions::default())
    }

    pub fn set_plot_feature_with_options<F>(
        &mut self,
        plot: Plot,
        feature: F,
        options: PlotFeatureOptions,
    ) -> Result<PlotState>
    where
        F: Into<InfoType>,
    {
        let mut args = plot_info_args(plot, "feature", feature.into());
        if let Some(variety) = options.variety {
            args["variety"] = json!(variety);
        }
        self.command_plot_state("set_plot_feature", args)
    }

    pub fn clear_plot_feature(&mut self, plot: Plot) -> Result<PlotState> {
        self.set_plot_feature(plot, -1)
    }

    pub fn set_plot_bonus<B>(&mut self, plot: Plot, bonus: B) -> Result<PlotState>
    where
        B: Into<InfoType>,
    {
        self.command_plot_state(
            "set_plot_bonus",
            plot_info_args(plot, "bonus", bonus.into()),
        )
    }

    pub fn clear_plot_bonus(&mut self, plot: Plot) -> Result<PlotState> {
        self.set_plot_bonus(plot, -1)
    }

    pub fn set_plot_improvement<I>(&mut self, plot: Plot, improvement: I) -> Result<PlotState>
    where
        I: Into<InfoType>,
    {
        self.command_plot_state(
            "set_plot_improvement",
            plot_info_args(plot, "improvement", improvement.into()),
        )
    }

    pub fn clear_plot_improvement(&mut self, plot: Plot) -> Result<PlotState> {
        self.set_plot_improvement(plot, -1)
    }

    pub fn set_plot_route<R>(&mut self, plot: Plot, route: R) -> Result<PlotState>
    where
        R: Into<InfoType>,
    {
        self.set_plot_route_with_options(plot, route, PlotRouteOptions::default())
    }

    pub fn set_plot_route_with_options<R>(
        &mut self,
        plot: Plot,
        route: R,
        options: PlotRouteOptions,
    ) -> Result<PlotState>
    where
        R: Into<InfoType>,
    {
        let mut args = plot_info_args(plot, "route", route.into());
        args["update_plot_group"] = json_bool(options.update_plot_group);
        self.command_plot_state("set_plot_route", args)
    }

    pub fn clear_plot_route(&mut self, plot: Plot) -> Result<PlotState> {
        self.set_plot_route(plot, -1)
    }

    pub fn set_plot_culture<P: Into<PlayerId>>(
        &mut self,
        plot: Plot,
        player: P,
        value: i32,
    ) -> Result<PlotState> {
        self.set_plot_culture_with_options(plot, player, value, PlotCultureOptions::default())
    }

    pub fn set_plot_culture_with_options<P: Into<PlayerId>>(
        &mut self,
        plot: Plot,
        player: P,
        value: i32,
        options: PlotCultureOptions,
    ) -> Result<PlotState> {
        let player = player.into();
        let mut args = json!({
            "x": plot.x,
            "y": plot.y,
            "player": player.0,
            "value": value,
        });
        args["update"] = json_bool(options.update);
        args["update_plot_groups"] = json_bool(options.update_plot_groups);
        self.command_plot_state("set_plot_culture", args)
    }

    pub fn change_plot_culture<P: Into<PlayerId>>(
        &mut self,
        plot: Plot,
        player: P,
        change: i32,
    ) -> Result<PlotState> {
        self.change_plot_culture_with_options(
            plot,
            player,
            change,
            PlotCultureChangeOptions::default(),
        )
    }

    pub fn change_plot_culture_with_options<P: Into<PlayerId>>(
        &mut self,
        plot: Plot,
        player: P,
        change: i32,
        options: PlotCultureChangeOptions,
    ) -> Result<PlotState> {
        let player = player.into();
        let args = json!({
            "x": plot.x,
            "y": plot.y,
            "player": player.0,
            "change": change,
            "update": if options.update { 1 } else { 0 },
        });
        self.command_plot_state("change_plot_culture", args)
    }

    pub fn set_plot_revealed<T: Into<TeamId>>(
        &mut self,
        plot: Plot,
        team: T,
        revealed: bool,
    ) -> Result<PlotState> {
        self.set_plot_revealed_with_options(plot, team, revealed, PlotRevealedOptions::default())
    }

    pub fn set_plot_revealed_with_options<T: Into<TeamId>>(
        &mut self,
        plot: Plot,
        team: T,
        revealed: bool,
        options: PlotRevealedOptions,
    ) -> Result<PlotState> {
        let team = team.into();
        let mut args = json!({
            "x": plot.x,
            "y": plot.y,
            "team": team.0,
            "revealed": if revealed { 1 } else { 0 },
            "terrain_only": if options.terrain_only { 1 } else { 0 },
            "update_plot_group": if options.update_plot_group { 1 } else { 0 },
        });
        if let Some(from_team) = options.from_team {
            args["from_team"] = json!(from_team.0);
        }
        self.command_plot_state("set_plot_revealed", args)
    }

    fn command_plot_state(&mut self, name: &str, args: Value) -> Result<PlotState> {
        let result: PlotStateResult = self.command(name, args)?;
        Ok(result.into())
    }
}

fn plot_owner_args(plot: Plot, owner: i32, options: PlotOwnerOptions) -> Value {
    json!({
        "x": plot.x,
        "y": plot.y,
        "owner": owner,
        "check_units": if options.check_units { 1 } else { 0 },
        "update_plot_group": if options.update_plot_group { 1 } else { 0 },
    })
}

fn plot_info_args(plot: Plot, key: &str, info: InfoType) -> Value {
    let mut args = json!({
        "x": plot.x,
        "y": plot.y,
    });
    args[key] = json!(info);
    args
}

fn json_bool(value: bool) -> Value {
    json!(if value { 1 } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plot_owner_options_serialize_bridge_flags() {
        let args = plot_owner_args(
            Plot::new(3, 4),
            -1,
            PlotOwnerOptions {
                check_units: false,
                update_plot_group: true,
            },
        );

        assert_eq!(
            args,
            json!({
                "x": 3,
                "y": 4,
                "owner": -1,
                "check_units": 0,
                "update_plot_group": 1
            })
        );
    }

    #[test]
    fn plot_info_args_keep_xml_type_names() {
        let args = plot_info_args(Plot::new(3, 4), "terrain", "TERRAIN_GRASS".into());

        assert_eq!(
            args,
            json!({
                "x": 3,
                "y": 4,
                "terrain": "TERRAIN_GRASS"
            })
        );
    }

    #[test]
    fn plot_revealed_options_can_copy_visibility_from_team() {
        let options = PlotRevealedOptions {
            terrain_only: true,
            from_team: Some(TeamId(2)),
            update_plot_group: false,
        };
        let mut args = json!({
            "x": 3,
            "y": 4,
            "team": 1,
            "revealed": 1,
            "terrain_only": if options.terrain_only { 1 } else { 0 },
            "update_plot_group": if options.update_plot_group { 1 } else { 0 },
        });
        if let Some(from_team) = options.from_team {
            args["from_team"] = json!(from_team.0);
        }

        assert_eq!(
            args,
            json!({
                "x": 3,
                "y": 4,
                "team": 1,
                "revealed": 1,
                "terrain_only": 1,
                "from_team": 2,
                "update_plot_group": 0
            })
        );
    }
}
