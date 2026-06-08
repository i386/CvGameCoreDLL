use crate::client::{BridgeClient, Result};
use crate::state::{TeamRelationState, TeamState, TeamTechState, TeamsResult};
use crate::types::{InfoType, PlayerId, TeamId, WarPlan};
use serde_json::{json, Value};

impl BridgeClient {
    pub fn get_team_tech_state<T, I>(&mut self, team: T, tech: I) -> Result<TeamTechState>
    where
        T: Into<TeamId>,
        I: Into<InfoType>,
    {
        let team = team.into();
        self.query(
            "get_team_tech_state",
            json!({ "team": team.0, "tech": tech.into() }),
        )
    }

    pub fn set_team_has_tech<T, I>(
        &mut self,
        team: T,
        tech: I,
        has: bool,
        player: Option<PlayerId>,
    ) -> Result<TeamTechState>
    where
        T: Into<TeamId>,
        I: Into<InfoType>,
    {
        let team = team.into();
        let mut args = json!({
            "team": team.0,
            "tech": tech.into(),
            "has": if has { 1 } else { 0 },
        });
        if let Some(player) = player {
            args["player"] = json!(player.0);
        }
        self.command("set_team_has_tech", args)
    }

    pub fn grant_team_tech<T, I>(
        &mut self,
        team: T,
        tech: I,
        player: Option<PlayerId>,
    ) -> Result<TeamTechState>
    where
        T: Into<TeamId>,
        I: Into<InfoType>,
    {
        self.set_team_has_tech(team, tech, true, player)
    }

    pub fn change_team_research_progress<T, I>(
        &mut self,
        team: T,
        tech: I,
        change: i32,
        player: Option<PlayerId>,
    ) -> Result<TeamTechState>
    where
        T: Into<TeamId>,
        I: Into<InfoType>,
    {
        let team = team.into();
        let mut args = json!({
            "team": team.0,
            "tech": tech.into(),
            "change": change,
        });
        if let Some(player) = player {
            args["player"] = json!(player.0);
        }
        self.command("change_team_research_progress", args)
    }

    pub fn get_team_state<T: Into<TeamId>>(&mut self, team: T) -> Result<TeamState> {
        let team = team.into();
        self.query("get_team_state", json!({ "team": team.0 }))
    }

    pub fn list_teams(&mut self) -> Result<Vec<TeamState>> {
        let result: TeamsResult = self.query("list_teams", json!({}))?;
        Ok(result.teams)
    }

    pub fn get_team_relation_state<T, O>(
        &mut self,
        team: T,
        other_team: O,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        self.query(
            "get_team_relation_state",
            relation_args(team.into(), other_team.into()),
        )
    }

    pub fn meet_team<T, O>(
        &mut self,
        team: T,
        other_team: O,
        new_diplo: bool,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["new_diplo"] = json!(if new_diplo { 1 } else { 0 });
        self.command("meet_team", args)
    }

    pub fn declare_war<T, O>(
        &mut self,
        team: T,
        other_team: O,
        war_plan: Option<WarPlan>,
        new_diplo: bool,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["new_diplo"] = json!(if new_diplo { 1 } else { 0 });
        if let Some(war_plan) = war_plan {
            args["war_plan"] = json!(war_plan);
        }
        self.command("declare_war", args)
    }

    pub fn make_peace<T, O>(
        &mut self,
        team: T,
        other_team: O,
        bump_units: bool,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["bump_units"] = json!(if bump_units { 1 } else { 0 });
        self.command("make_peace", args)
    }

    pub fn set_team_open_borders<T, O>(
        &mut self,
        team: T,
        other_team: O,
        open: bool,
        reciprocal: bool,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["open"] = json!(if open { 1 } else { 0 });
        args["reciprocal"] = json!(if reciprocal { 1 } else { 0 });
        self.command("set_team_open_borders", args)
    }

    pub fn set_team_defensive_pact<T, O>(
        &mut self,
        team: T,
        other_team: O,
        pact: bool,
        reciprocal: bool,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["pact"] = json!(if pact { 1 } else { 0 });
        args["reciprocal"] = json!(if reciprocal { 1 } else { 0 });
        self.command("set_team_defensive_pact", args)
    }

    pub fn set_team_force_peace<T, O>(
        &mut self,
        team: T,
        other_team: O,
        peace: bool,
        reciprocal: bool,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["peace"] = json!(if peace { 1 } else { 0 });
        args["reciprocal"] = json!(if reciprocal { 1 } else { 0 });
        self.command("set_team_force_peace", args)
    }

    pub fn set_team_permanent_war_peace<T, O>(
        &mut self,
        team: T,
        other_team: O,
        permanent: bool,
        reciprocal: bool,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["permanent"] = json!(if permanent { 1 } else { 0 });
        args["reciprocal"] = json!(if reciprocal { 1 } else { 0 });
        self.command("set_team_permanent_war_peace", args)
    }

    pub fn set_team_vassal<T, O>(
        &mut self,
        team: T,
        master: O,
        vassal: bool,
        capitulated: bool,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), master.into());
        args["vassal"] = json!(if vassal { 1 } else { 0 });
        args["capitulated"] = json!(if capitulated { 1 } else { 0 });
        self.command("set_team_vassal", args)
    }

    pub fn set_team_war_weariness<T, O>(
        &mut self,
        team: T,
        other_team: O,
        value: i32,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["value"] = json!(value);
        self.command("set_team_war_weariness", args)
    }

    pub fn change_team_war_weariness<T, O>(
        &mut self,
        team: T,
        other_team: O,
        change: i32,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["change"] = json!(change);
        self.command("change_team_war_weariness", args)
    }

    pub fn set_team_stolen_visibility_timer<T, O>(
        &mut self,
        team: T,
        other_team: O,
        value: i32,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["value"] = json!(value);
        self.command("set_team_stolen_visibility_timer", args)
    }

    pub fn change_team_stolen_visibility_timer<T, O>(
        &mut self,
        team: T,
        other_team: O,
        change: i32,
    ) -> Result<TeamRelationState>
    where
        T: Into<TeamId>,
        O: Into<TeamId>,
    {
        let mut args = relation_args(team.into(), other_team.into());
        args["change"] = json!(change);
        self.command("change_team_stolen_visibility_timer", args)
    }
}

fn relation_args(team: TeamId, other_team: TeamId) -> Value {
    json!({ "team": team.0, "other_team": other_team.0 })
}
