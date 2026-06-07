use crate::protocol::{decode_jsonl, encode_jsonl, BridgeReply, Message};
use serde::de::DeserializeOwned;
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
        let result: Value = self.query("get_game_turn", json!({}))?;
        result
            .get("turn")
            .and_then(Value::as_i64)
            .map(|turn| turn as i32)
            .ok_or_else(|| BridgeError::Protocol("get_game_turn reply missing turn".to_string()))
    }

    pub fn get_player_gold(&mut self, player: i32) -> Result<i32> {
        let result: Value = self.query("get_player_gold", json!({ "player": player }))?;
        result
            .get("gold")
            .and_then(Value::as_i64)
            .map(|gold| gold as i32)
            .ok_or_else(|| BridgeError::Protocol("get_player_gold reply missing gold".to_string()))
    }

    pub fn set_player_gold(&mut self, player: i32, value: i32) -> Result<i32> {
        let result: Value = self.command(
            "set_player_gold",
            json!({ "player": player, "value": value }),
        )?;
        result
            .get("gold")
            .and_then(Value::as_i64)
            .map(|gold| gold as i32)
            .ok_or_else(|| BridgeError::Protocol("set_player_gold reply missing gold".to_string()))
    }

    pub fn get_mod_state(&mut self) -> Result<String> {
        let result: Value = self.query("get_mod_state", json!({}))?;
        result
            .get("json")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| BridgeError::Protocol("get_mod_state reply missing json".to_string()))
    }

    pub fn set_mod_state(&mut self, json_state: &str) -> Result<usize> {
        let result: Value = self.command("set_mod_state", json!({ "json": json_state }))?;
        result
            .get("bytes")
            .and_then(Value::as_u64)
            .map(|bytes| bytes as usize)
            .ok_or_else(|| BridgeError::Protocol("set_mod_state reply missing bytes".to_string()))
    }

    pub fn next_event(&mut self) -> Result<Message> {
        if let Some(event) = self.queued_events.pop_front() {
            return Ok(event);
        }

        loop {
            let message = self.read_control()?;
            match message {
                Message::Event { .. } | Message::Hello { .. } | Message::Log { .. } => {
                    return Ok(message);
                }
                other => self.queued_events.push_back(other),
            }
        }
    }

    pub fn next_callback_mirror(&mut self) -> Result<Message> {
        let mut line = String::new();
        self.callback_reader.read_line(&mut line)?;
        Ok(decode_jsonl(&line)?)
    }

    pub fn write_callback_reply(&mut self, reply: BridgeReply) -> Result<()> {
        let line = encode_jsonl(&reply.into_message())?;
        self.callback_writer.write_all(line.as_bytes())?;
        self.callback_writer.flush()?;
        Ok(())
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
