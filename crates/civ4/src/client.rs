use crate::events::{
    BridgeCallbackMessage, BridgeCallbackRequest, BridgeEvent, BridgeEventMessage,
};
use crate::protocol::{
    decode_jsonl, encode_jsonl, BridgeHello, BridgeReply, Message, BRIDGE_PROTOCOL_VERSION,
};
use crate::types::{CityRef, InfoType, PlayerId, Plot, TeamId, UnitRef};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
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
        let args = request.into_args()?;
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

#[derive(Debug, Clone, Serialize)]
pub struct SpawnUnitRequest {
    pub player: PlayerId,
    pub unit_type: InfoType,
    pub plot: Plot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_ai: Option<InfoType>,
}

impl SpawnUnitRequest {
    pub fn new<P, U>(player: P, unit_type: U, plot: Plot) -> Self
    where
        P: Into<PlayerId>,
        U: Into<InfoType>,
    {
        Self {
            player: player.into(),
            unit_type: unit_type.into(),
            plot,
            unit_ai: None,
        }
    }

    pub fn with_unit_ai<U>(mut self, unit_ai: U) -> Self
    where
        U: Into<InfoType>,
    {
        self.unit_ai = Some(unit_ai.into());
        self
    }

    fn into_args(self) -> Result<Value> {
        let mut value = serde_json::to_value(self)?;
        let object = value.as_object_mut().ok_or_else(|| {
            BridgeError::Protocol("spawn_unit args were not an object".to_string())
        })?;
        if let Some(plot) = object.remove("plot") {
            let plot = plot.as_object().ok_or_else(|| {
                BridgeError::Protocol("spawn_unit plot was not an object".to_string())
            })?;
            let x = plot
                .get("x")
                .cloned()
                .ok_or_else(|| BridgeError::Protocol("spawn_unit plot missing x".to_string()))?;
            let y = plot
                .get("y")
                .cloned()
                .ok_or_else(|| BridgeError::Protocol("spawn_unit plot missing y".to_string()))?;
            object.insert("x".to_string(), x);
            object.insert("y".to_string(), y);
        }
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnedUnit {
    pub unit: UnitRef,
    pub plot: Plot,
}

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
    pub water: bool,
    pub peak: bool,
    pub units: i32,
    pub city: Option<CityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct PlotStateResult {
    x: i32,
    y: i32,
    owner: i32,
    terrain: i32,
    feature: i32,
    bonus: i32,
    improvement: i32,
    water: bool,
    peak: bool,
    units: i32,
    city_player: i32,
    city: i32,
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
pub struct UnitState {
    pub player: i32,
    pub unit: i32,
    pub unit_type: i32,
    pub x: i32,
    pub y: i32,
    pub damage: i32,
    pub experience: i32,
    pub level: i32,
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

#[derive(Deserialize)]
struct GameTurnResult {
    turn: i32,
}

#[derive(Deserialize)]
struct PlayerGoldResult {
    gold: i32,
}

#[derive(Deserialize)]
struct PlayersResult {
    players: Vec<PlayerState>,
}

#[derive(Deserialize)]
struct PlayerCitiesResult {
    cities: Vec<CityState>,
}

#[derive(Deserialize)]
struct PlayerUnitsResult {
    units: Vec<UnitState>,
}

#[derive(Deserialize)]
struct ModStateResult {
    json: String,
}

#[derive(Deserialize)]
struct SetModStateResult {
    bytes: usize,
}

#[derive(Deserialize)]
struct SpawnUnitResult {
    player: i32,
    unit: i32,
    x: i32,
    y: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestState {
        schema_version: u32,
        enabled: bool,
    }

    #[test]
    fn spawn_unit_request_flattens_plot() {
        let request =
            SpawnUnitRequest::new(0, "UNIT_WARRIOR", Plot::new(3, 4)).with_unit_ai("UNITAI_ATTACK");
        let args = request.into_args().unwrap();

        assert_eq!(
            args,
            json!({
                "player": 0,
                "unit_type": "UNIT_WARRIOR",
                "x": 3,
                "y": 4,
                "unit_ai": "UNITAI_ATTACK"
            })
        );
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
                "culture": 99
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
                "x": 10,
                "y": 11,
                "damage": 0,
                "experience": 2,
                "level": 1
            }]
        }))
        .unwrap();
        assert_eq!(units.units[0].unit_ref(), UnitRef::new(0, 42));
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
}
