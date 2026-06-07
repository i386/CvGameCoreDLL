use crate::events::{
    BridgeCallbackMessage, BridgeCallbackRequest, BridgeEvent, BridgeEventMessage,
};
use crate::protocol::{decode_jsonl, encode_jsonl, BridgeReply, Message};
use crate::types::{CityRef, InfoType, PlayerId, Plot, TeamId, UnitRef};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
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

    pub fn connect_with_prefix(prefix: &str) -> Result<Self> {
        Self::connect(
            format!(r"\\.\pipe\{prefix}-Control"),
            format!(r"\\.\pipe\{prefix}-Callbacks"),
        )
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

    pub fn get_unit_state(&mut self, unit: UnitRef) -> Result<UnitState> {
        self.query(
            "get_unit_state",
            json!({ "player": unit.player, "unit": unit.id }),
        )
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

#[derive(Deserialize)]
struct GameTurnResult {
    turn: i32,
}

#[derive(Deserialize)]
struct PlayerGoldResult {
    gold: i32,
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
}
