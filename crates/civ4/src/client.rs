use crate::callbacks::{InputCallbackReply, RuleCallbackReply};
use crate::events::{
    BridgeCallbackMessage, BridgeCallbackRequest, BridgeEvent, BridgeEventMessage,
};
use crate::protocol::{
    decode_jsonl, encode_jsonl, BridgeCapability, BridgeHello, BridgeReply, Message,
    BRIDGE_PROTOCOL_VERSION,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
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

    pub fn connect_default_requiring(required: &[BridgeCapability]) -> Result<(Self, BridgeHello)> {
        let mut client = Self::connect_default()?;
        let hello = client.handshake_requiring(required)?;
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

    pub fn handshake_requiring(&mut self, required: &[BridgeCapability]) -> Result<BridgeHello> {
        let hello = self.handshake()?;
        let missing = hello.missing_bridge_capabilities(required);
        if !missing.is_empty() {
            let missing_names: Vec<&str> =
                missing.iter().map(|capability| capability.name()).collect();
            return Err(BridgeError::Protocol(format!(
                "bridge is missing capabilities: {}",
                missing_names.join(", ")
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

    pub fn write_input_callback_reply(&mut self, id: u64, consume: bool) -> Result<()> {
        self.write_callback_success(id, &InputCallbackReply::new(consume))
    }

    pub fn write_rule_callback_reply(&mut self, id: u64, value: bool) -> Result<()> {
        self.write_callback_success(id, &RuleCallbackReply::new(value))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_typed_input_callback_reply() {
        let (mut client, control_path, callback_path) = temp_file_client("input");

        client.write_input_callback_reply(7, true).unwrap();

        assert_callback_reply(&callback_path, 7, json!({ "consume": true }));
        let _ = fs::remove_file(control_path);
        let _ = fs::remove_file(callback_path);
    }

    #[test]
    fn writes_typed_rule_callback_reply() {
        let (mut client, control_path, callback_path) = temp_file_client("rule");

        client.write_rule_callback_reply(8, false).unwrap();

        assert_callback_reply(&callback_path, 8, json!({ "value": false }));
        let _ = fs::remove_file(control_path);
        let _ = fs::remove_file(callback_path);
    }

    #[test]
    fn handshake_requiring_accepts_present_capabilities() {
        let (mut client, control_path, callback_path) = temp_file_client_with_control(
            "capabilities-ok",
            r#"{"type":"hello","protocol":1,"side":"dll","capabilities":["events","queries","commands"]}"#,
        );

        let hello = client
            .handshake_requiring(&[
                BridgeCapability::Events,
                BridgeCapability::Queries,
                BridgeCapability::Commands,
            ])
            .unwrap();

        assert!(hello.has_bridge_capability(BridgeCapability::Commands));
        let _ = fs::remove_file(control_path);
        let _ = fs::remove_file(callback_path);
    }

    #[test]
    fn handshake_requiring_reports_missing_capabilities() {
        let (mut client, control_path, callback_path) = temp_file_client_with_control(
            "capabilities-missing",
            r#"{"type":"hello","protocol":1,"side":"dll","capabilities":["events"]}"#,
        );

        let error = client
            .handshake_requiring(&[BridgeCapability::Events, BridgeCapability::Commands])
            .unwrap_err();

        assert!(matches!(error, BridgeError::Protocol(message) if message.contains("commands")));
        let _ = fs::remove_file(control_path);
        let _ = fs::remove_file(callback_path);
    }

    fn temp_file_client(test_name: &str) -> (BridgeClient, PathBuf, PathBuf) {
        temp_file_client_with_control(test_name, "")
    }

    fn temp_file_client_with_control(
        test_name: &str,
        control_body: &str,
    ) -> (BridgeClient, PathBuf, PathBuf) {
        let unique = format!(
            "civ4-client-{test_name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let control_path = std::env::temp_dir().join(format!("{unique}-control.jsonl"));
        let callback_path = std::env::temp_dir().join(format!("{unique}-callback.jsonl"));

        let control_body = if control_body.is_empty() {
            String::new()
        } else {
            format!("{control_body}\n")
        };
        fs::write(&control_path, control_body).unwrap();
        fs::write(&callback_path, b"").unwrap();

        let client = BridgeClient::connect(&control_path, &callback_path).unwrap();
        (client, control_path, callback_path)
    }

    fn assert_callback_reply(path: &PathBuf, id: u64, expected_result: serde_json::Value) {
        let line = fs::read_to_string(path).unwrap();
        match decode_jsonl(&line).unwrap() {
            Message::Reply {
                id: reply_id,
                ok,
                result,
                error,
            } => {
                assert_eq!(reply_id, id);
                assert!(ok);
                assert_eq!(result, Some(expected_result));
                assert!(error.is_none());
            }
            other => panic!("expected reply, got {other:?}"),
        }
    }
}
