use crate::commands::{CityOrder, SpawnUnitRequest, SpawnUnitResult, SpawnedUnit};
use crate::events::{
    BridgeCallbackMessage, BridgeCallbackRequest, BridgeEvent, BridgeEventMessage,
};
use crate::protocol::{
    decode_jsonl, encode_jsonl, BridgeHello, BridgeReply, Message, BRIDGE_PROTOCOL_VERSION,
};
use crate::state::{
    CityState, GameTurnResult, MapState, ModStateResult, PlayerCitiesResult, PlayerGoldResult,
    PlayerOptions, PlayerState, PlayerUnitsResult, PlayersResult, PlotState, PlotStateResult,
    SetModStateResult, TeamTechState, UnitState,
};
use crate::types::{CityRef, InfoType, PlayerId, Plot, TeamId, UnitRef};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub type Result<T> = std::result::Result<T, BridgeError>;

#[derive(Debug)]
pub enum BridgeError {
    Io(io::Error),
    Json(serde_json::Error),
    Protocol(String),
    Bridge { code: String, message: String },
}

impl From<io::Error> for BridgeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for BridgeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Json(error) => write!(formatter, "JSON error: {error}"),
            Self::Protocol(message) => write!(formatter, "bridge protocol error: {message}"),
            Self::Bridge { code, message } => write!(formatter, "bridge error {code}: {message}"),
        }
    }
}

impl std::error::Error for BridgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Protocol(_) | Self::Bridge { .. } => None,
        }
    }
}

pub struct BridgeClient {
    next_id: u64,
    control_reader: BufReader<File>,
    control_writer: File,
    callback_reader: BufReader<File>,
    callback_writer: File,
    queued_events: VecDeque<Message>,
}

impl BridgeClient {
    pub fn connect_default() -> Result<Self> {
        Self::connect(
            r"\\.\pipe\CvGameCoreDLL-Control",
            r"\\.\pipe\CvGameCoreDLL-Callbacks",
        )
    }

    pub fn connect_default_with_handshake() -> Result<(Self, BridgeHello)> {
        let mut client = Self::connect_default()?;
        let hello = client.handshake()?;
        Ok((client, hello))
    }

    pub fn connect_with_prefix(prefix: &str) -> Result<Self> {
        Self::connect(
            format!(r"\\.\pipe\{prefix}-Control"),
            format!(r"\\.\pipe\{prefix}-Callbacks"),
        )
    }

    pub fn connect_with_prefix_and_handshake(prefix: &str) -> Result<(Self, BridgeHello)> {
        let mut client = Self::connect_with_prefix(prefix)?;
        let hello = client.handshake()?;
        Ok((client, hello))
    }

    pub fn connect<P: AsRef<Path>, Q: AsRef<Path>>(
        control_pipe: P,
        callback_pipe: Q,
    ) -> Result<Self> {
        let control = open_pipe(control_pipe)?;
        let callbacks = open_pipe(callback_pipe)?;

        Ok(Self {
            next_id: 1,
            control_reader: BufReader::new(control.try_clone()?),
            control_writer: control,
            callback_reader: BufReader::new(callbacks.try_clone()?),
            callback_writer: callbacks,
            queued_events: VecDeque::new(),
        })
    }

    pub fn handshake(&mut self) -> Result<BridgeHello> {
        let hello = self.next_hello()?;
        if hello.protocol != BRIDGE_PROTOCOL_VERSION {
            return Err(BridgeError::Protocol(format!(
                "unsupported bridge protocol {}, expected {}",
                hello.protocol, BRIDGE_PROTOCOL_VERSION
            )));
        }
        if hello.side != "dll" {
            return Err(BridgeError::Protocol(format!(
                "unexpected bridge side {:?}, expected \"dll\"",
                hello.side
            )));
        }
        Ok(hello)
    }

    pub fn next_hello(&mut self) -> Result<BridgeHello> {
        if let Some(pos) = self
            .queued_events
            .iter()
            .position(|message| matches!(message, Message::Hello { .. }))
        {
            let message = self.queued_events.remove(pos).expect("queued hello exists");
            return message
                .into_hello()
                .ok_or_else(|| BridgeError::Protocol("queued message was not hello".to_string()));
        }

        loop {
            let message = self.read_control()?;
            if let Some(hello) = message.clone().into_hello() {
                return Ok(hello);
            }
            self.queued_events.push_back(message);
        }
    }

    pub fn query<T: DeserializeOwned>(&mut self, name: &str, args: Value) -> Result<T> {
        let id = self.next_request_id();
        self.write_control(&Message::Query {
            id,
            name: name.to_string(),
            args,
        })?;
        self.wait_for_reply(id)
    }

    pub fn command<T: DeserializeOwned>(&mut self, name: &str, args: Value) -> Result<T> {
        let id = self.next_request_id();
        self.write_control(&Message::Command {
            id,
            name: name.to_string(),
            args,
        })?;
        self.wait_for_reply(id)
    }

    pub fn get_game_turn(&mut self) -> Result<i32> {
        let result: GameTurnResult = self.query("get_game_turn", json!({}))?;
        Ok(result.turn)
    }

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

    pub fn get_map_state(&mut self) -> Result<MapState> {
        self.query("get_map_state", json!({}))
    }

    pub fn get_plot_state(&mut self, plot: Plot) -> Result<PlotState> {
        let result: PlotStateResult =
            self.query("get_plot_state", json!({ "x": plot.x, "y": plot.y }))?;
        Ok(result.into())
    }

    pub fn get_city_state(&mut self, city: CityRef) -> Result<CityState> {
        self.query(
            "get_city_state",
            json!({ "player": city.player, "city": city.id }),
        )
    }

    pub fn list_player_cities<P: Into<PlayerId>>(&mut self, player: P) -> Result<Vec<CityState>> {
        let player = player.into();
        let result: PlayerCitiesResult =
            self.query("list_player_cities", json!({ "player": player.0 }))?;
        Ok(result.cities)
    }

    pub fn list_all_cities(&mut self) -> Result<Vec<CityState>> {
        let mut cities = Vec::new();
        for player in self.list_alive_players()? {
            cities.extend(self.list_player_cities(player.player_id())?);
        }
        Ok(cities)
    }

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

    pub fn set_city_population(&mut self, city: CityRef, value: i32) -> Result<CityState> {
        self.command(
            "set_city_population",
            json!({ "player": city.player, "city": city.id, "value": value }),
        )
    }

    pub fn change_city_population(&mut self, city: CityRef, change: i32) -> Result<CityState> {
        self.command(
            "change_city_population",
            json!({ "player": city.player, "city": city.id, "change": change }),
        )
    }

    pub fn set_city_culture<P: Into<PlayerId>>(
        &mut self,
        city: CityRef,
        culture_player: P,
        value: i32,
    ) -> Result<CityState> {
        let culture_player = culture_player.into();
        self.command(
            "set_city_culture",
            json!({
                "player": city.player,
                "city": city.id,
                "culture_player": culture_player.0,
                "value": value
            }),
        )
    }

    pub fn set_owner_city_culture(&mut self, city: CityRef, value: i32) -> Result<CityState> {
        self.command(
            "set_city_culture",
            json!({ "player": city.player, "city": city.id, "value": value }),
        )
    }

    pub fn set_city_production(&mut self, city: CityRef, value: i32) -> Result<CityState> {
        self.command(
            "set_city_production",
            json!({ "player": city.player, "city": city.id, "value": value }),
        )
    }

    pub fn change_city_production(&mut self, city: CityRef, change: i32) -> Result<CityState> {
        self.command(
            "change_city_production",
            json!({ "player": city.player, "city": city.id, "change": change }),
        )
    }

    pub fn set_city_unit_production<U>(
        &mut self,
        city: CityRef,
        unit_type: U,
        value: i32,
    ) -> Result<CityState>
    where
        U: Into<InfoType>,
    {
        self.command(
            "set_city_unit_production",
            json!({
                "player": city.player,
                "city": city.id,
                "unit_type": unit_type.into(),
                "value": value
            }),
        )
    }

    pub fn set_city_building_production<B>(
        &mut self,
        city: CityRef,
        building_type: B,
        value: i32,
    ) -> Result<CityState>
    where
        B: Into<InfoType>,
    {
        self.command(
            "set_city_building_production",
            json!({
                "player": city.player,
                "city": city.id,
                "building_type": building_type.into(),
                "value": value
            }),
        )
    }

    pub fn set_city_project_production<P>(
        &mut self,
        city: CityRef,
        project_type: P,
        value: i32,
    ) -> Result<CityState>
    where
        P: Into<InfoType>,
    {
        self.command(
            "set_city_project_production",
            json!({
                "player": city.player,
                "city": city.id,
                "project_type": project_type.into(),
                "value": value
            }),
        )
    }

    pub fn push_city_order(&mut self, city: CityRef, order: CityOrder) -> Result<CityState> {
        self.command("push_city_order", order.into_args(city))
    }

    pub fn clear_city_order_queue(&mut self, city: CityRef) -> Result<CityState> {
        self.command(
            "clear_city_order_queue",
            json!({ "player": city.player, "city": city.id }),
        )
    }

    pub fn pop_city_order(&mut self, city: CityRef, index: i32) -> Result<CityState> {
        self.command(
            "pop_city_order",
            json!({ "player": city.player, "city": city.id, "index": index }),
        )
    }

    pub fn set_plot_owner<P: Into<PlayerId>>(&mut self, plot: Plot, owner: P) -> Result<PlotState> {
        let owner = owner.into();
        let result: PlotStateResult = self.command(
            "set_plot_owner",
            json!({ "x": plot.x, "y": plot.y, "owner": owner.0 }),
        )?;
        Ok(result.into())
    }

    pub fn clear_plot_owner(&mut self, plot: Plot) -> Result<PlotState> {
        let result: PlotStateResult = self.command(
            "set_plot_owner",
            json!({ "x": plot.x, "y": plot.y, "owner": -1 }),
        )?;
        Ok(result.into())
    }

    pub fn set_plot_terrain<T>(&mut self, plot: Plot, terrain: T) -> Result<PlotState>
    where
        T: Into<InfoType>,
    {
        let result: PlotStateResult = self.command(
            "set_plot_terrain",
            json!({ "x": plot.x, "y": plot.y, "terrain": terrain.into() }),
        )?;
        Ok(result.into())
    }

    pub fn set_plot_feature<F>(&mut self, plot: Plot, feature: F) -> Result<PlotState>
    where
        F: Into<InfoType>,
    {
        let result: PlotStateResult = self.command(
            "set_plot_feature",
            json!({ "x": plot.x, "y": plot.y, "feature": feature.into() }),
        )?;
        Ok(result.into())
    }

    pub fn clear_plot_feature(&mut self, plot: Plot) -> Result<PlotState> {
        self.set_plot_feature(plot, -1)
    }

    pub fn set_plot_bonus<B>(&mut self, plot: Plot, bonus: B) -> Result<PlotState>
    where
        B: Into<InfoType>,
    {
        let result: PlotStateResult = self.command(
            "set_plot_bonus",
            json!({ "x": plot.x, "y": plot.y, "bonus": bonus.into() }),
        )?;
        Ok(result.into())
    }

    pub fn clear_plot_bonus(&mut self, plot: Plot) -> Result<PlotState> {
        self.set_plot_bonus(plot, -1)
    }

    pub fn set_plot_improvement<I>(&mut self, plot: Plot, improvement: I) -> Result<PlotState>
    where
        I: Into<InfoType>,
    {
        let result: PlotStateResult = self.command(
            "set_plot_improvement",
            json!({ "x": plot.x, "y": plot.y, "improvement": improvement.into() }),
        )?;
        Ok(result.into())
    }

    pub fn clear_plot_improvement(&mut self, plot: Plot) -> Result<PlotState> {
        self.set_plot_improvement(plot, -1)
    }

    pub fn set_plot_route<R>(&mut self, plot: Plot, route: R) -> Result<PlotState>
    where
        R: Into<InfoType>,
    {
        let result: PlotStateResult = self.command(
            "set_plot_route",
            json!({ "x": plot.x, "y": plot.y, "route": route.into() }),
        )?;
        Ok(result.into())
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
        let player = player.into();
        let result: PlotStateResult = self.command(
            "set_plot_culture",
            json!({ "x": plot.x, "y": plot.y, "player": player.0, "value": value }),
        )?;
        Ok(result.into())
    }

    pub fn change_plot_culture<P: Into<PlayerId>>(
        &mut self,
        plot: Plot,
        player: P,
        change: i32,
    ) -> Result<PlotState> {
        let player = player.into();
        let result: PlotStateResult = self.command(
            "change_plot_culture",
            json!({ "x": plot.x, "y": plot.y, "player": player.0, "change": change }),
        )?;
        Ok(result.into())
    }

    pub fn set_plot_revealed<T: Into<TeamId>>(
        &mut self,
        plot: Plot,
        team: T,
        revealed: bool,
    ) -> Result<PlotState> {
        let team = team.into();
        let result: PlotStateResult = self.command(
            "set_plot_revealed",
            json!({
                "x": plot.x,
                "y": plot.y,
                "team": team.0,
                "revealed": if revealed { 1 } else { 0 }
            }),
        )?;
        Ok(result.into())
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

    pub fn get_mod_state(&mut self) -> Result<String> {
        let result: ModStateResult = self.query("get_mod_state", json!({}))?;
        Ok(result.json)
    }

    pub fn set_mod_state(&mut self, json_state: &str) -> Result<usize> {
        let result: SetModStateResult =
            self.command("set_mod_state", json!({ "json": json_state }))?;
        Ok(result.bytes)
    }

    pub fn load_mod_state<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        let json_state = self.get_mod_state()?;
        if json_state.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_str(&json_state)?))
    }

    pub fn save_mod_state<T: Serialize>(&mut self, state: &T) -> Result<usize> {
        let json_state = serde_json::to_string(state)?;
        self.set_mod_state(&json_state)
    }

    pub fn next_event(&mut self) -> Result<Message> {
        if let Some(pos) = self
            .queued_events
            .iter()
            .position(|message| matches!(message, Message::Event { .. }))
        {
            return Ok(self.queued_events.remove(pos).expect("queued event exists"));
        }

        loop {
            let message = self.read_control()?;
            if matches!(message, Message::Event { .. }) {
                return Ok(message);
            }
            self.queued_events.push_back(message);
        }
    }

    pub fn next_bridge_event(&mut self) -> Result<BridgeEventMessage> {
        loop {
            match self.next_event()? {
                Message::Event { seq, name, args } => {
                    return Ok(BridgeEventMessage {
                        seq,
                        event: BridgeEvent::from_name_args(name, args)?,
                    });
                }
                other => self.queued_events.push_back(other),
            }
        }
    }

    pub fn next_control_message(&mut self) -> Result<Message> {
        if let Some(event) = self.queued_events.pop_front() {
            return Ok(event);
        }
        self.read_control()
    }

    pub fn next_callback_raw(&mut self) -> Result<Message> {
        let mut line = String::new();
        self.callback_reader.read_line(&mut line)?;
        if line.is_empty() {
            return Err(BridgeError::Protocol("callback pipe closed".to_string()));
        }
        Ok(decode_jsonl(&line)?)
    }

    pub fn next_callback_mirror(&mut self) -> Result<Message> {
        match self.next_callback_raw()? {
            msg @ Message::CallbackMirror { .. } => Ok(msg),
            other => Err(BridgeError::Protocol(format!(
                "expected callback_mirror, got {other:?}"
            ))),
        }
    }

    pub fn next_callback_event(&mut self) -> Result<BridgeEventMessage> {
        match self.next_callback_mirror()? {
            Message::CallbackMirror { seq, name, args } => Ok(BridgeEventMessage {
                seq,
                event: BridgeEvent::from_name_args(name, args)?,
            }),
            other => Err(BridgeError::Protocol(format!(
                "expected callback_mirror, got {other:?}"
            ))),
        }
    }

    pub fn next_callback_message(&mut self) -> Result<BridgeCallbackMessage> {
        match self.next_callback_raw()? {
            Message::CallbackMirror { seq, name, args } => {
                Ok(BridgeCallbackMessage::Mirror(BridgeEventMessage {
                    seq,
                    event: BridgeEvent::from_name_args(name, args)?,
                }))
            }
            Message::CallbackRequest { id, name, args } => {
                Ok(BridgeCallbackMessage::Request(BridgeCallbackRequest {
                    id,
                    event: BridgeEvent::from_name_args(name, args)?,
                }))
            }
            other => Err(BridgeError::Protocol(format!(
                "expected callback message, got {other:?}"
            ))),
        }
    }

    pub fn next_callback_request(&mut self) -> Result<BridgeCallbackRequest> {
        match self.next_callback_message()? {
            BridgeCallbackMessage::Request(request) => Ok(request),
            other => Err(BridgeError::Protocol(format!(
                "expected callback_request, got {other:?}"
            ))),
        }
    }

    pub fn write_callback_reply(&mut self, reply: BridgeReply) -> Result<()> {
        let line = encode_jsonl(&reply.into_message())?;
        self.callback_writer.write_all(line.as_bytes())?;
        self.callback_writer.flush()?;
        Ok(())
    }

    pub fn write_callback_success<T: Serialize>(&mut self, id: u64, result: &T) -> Result<()> {
        self.write_callback_reply(BridgeReply::success(id, serde_json::to_value(result)?))
    }

    pub fn write_callback_error(
        &mut self,
        id: u64,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<()> {
        self.write_callback_reply(BridgeReply::error(id, code, message))
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn write_control(&mut self, message: &Message) -> Result<()> {
        let line = encode_jsonl(message)?;
        self.control_writer.write_all(line.as_bytes())?;
        self.control_writer.flush()?;
        Ok(())
    }

    fn read_control(&mut self) -> Result<Message> {
        let mut line = String::new();
        self.control_reader.read_line(&mut line)?;
        if line.is_empty() {
            return Err(BridgeError::Protocol("control pipe closed".to_string()));
        }
        Ok(decode_jsonl(&line)?)
    }

    fn wait_for_reply<T: DeserializeOwned>(&mut self, id: u64) -> Result<T> {
        if let Some(pos) = self.queued_events.iter().position(|message| {
            matches!(
                message,
                Message::Reply {
                    id: reply_id,
                    ..
                } if *reply_id == id
            )
        }) {
            let message = self.queued_events.remove(pos).expect("queued reply exists");
            if let Message::Reply {
                id: reply_id,
                ok,
                result,
                error,
            } = message
            {
                return decode_reply(BridgeReply {
                    id: reply_id,
                    ok,
                    result,
                    error,
                });
            }
        }

        loop {
            match self.read_control()? {
                Message::Reply {
                    id: reply_id,
                    ok,
                    result,
                    error,
                } if reply_id == id => {
                    return decode_reply(BridgeReply {
                        id: reply_id,
                        ok,
                        result,
                        error,
                    })
                }
                msg @ Message::Event { .. }
                | msg @ Message::Hello { .. }
                | msg @ Message::Log { .. } => {
                    self.queued_events.push_back(msg);
                }
                other => self.queued_events.push_back(other),
            }
        }
    }
}

fn decode_reply<T: DeserializeOwned>(reply: BridgeReply) -> Result<T> {
    if !reply.ok {
        if let Some(error) = reply.error {
            return Err(BridgeError::Bridge {
                code: error.code,
                message: error.message,
            });
        }
        return Err(BridgeError::Protocol(
            "reply failed without error body".to_string(),
        ));
    }

    let result = reply
        .result
        .ok_or_else(|| BridgeError::Protocol("reply missing result".to_string()))?;
    Ok(serde_json::from_value(result)?)
}

fn open_pipe<P: AsRef<Path>>(path: P) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}
