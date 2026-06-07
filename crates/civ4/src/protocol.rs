use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const BRIDGE_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Hello {
        protocol: u32,
        side: String,
        capabilities: Vec<String>,
    },
    Event {
        seq: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    CallbackMirror {
        seq: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    CallbackRequest {
        id: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    Query {
        id: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    Command {
        id: u64,
        name: String,
        #[serde(default)]
        args: Value,
    },
    Reply {
        id: u64,
        ok: bool,
        #[serde(default)]
        result: Option<Value>,
        #[serde(default)]
        error: Option<BridgeErrorBody>,
    },
    Log {
        level: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeHello {
    pub protocol: u32,
    pub side: String,
    pub capabilities: Vec<String>,
}

impl BridgeHello {
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|available| available == capability)
    }

    pub fn missing_capabilities<'a>(&self, required: &'a [&'a str]) -> Vec<&'a str> {
        required
            .iter()
            .copied()
            .filter(|capability| !self.has_capability(capability))
            .collect()
    }

    pub fn into_message(self) -> Message {
        Message::Hello {
            protocol: self.protocol,
            side: self.side,
            capabilities: self.capabilities,
        }
    }
}

impl Message {
    pub fn into_hello(self) -> Option<BridgeHello> {
        match self {
            Self::Hello {
                protocol,
                side,
                capabilities,
            } => Some(BridgeHello {
                protocol,
                side,
                capabilities,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeReply {
    pub id: u64,
    pub ok: bool,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<BridgeErrorBody>,
}

impl BridgeReply {
    pub fn success(id: u64, result: Value) -> Self {
        Self {
            id,
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: u64, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id,
            ok: false,
            result: None,
            error: Some(BridgeErrorBody {
                code: code.into(),
                message: message.into(),
            }),
        }
    }

    pub fn into_message(self) -> Message {
        Message::Reply {
            id: self.id,
            ok: self.ok,
            result: self.result,
            error: self.error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeErrorBody {
    pub code: String,
    pub message: String,
}

pub fn encode_jsonl(message: &Message) -> serde_json::Result<String> {
    let mut line = serde_json::to_string(message)?;
    line.push('\n');
    Ok(line)
}

pub fn decode_jsonl(line: &str) -> serde_json::Result<Message> {
    serde_json::from_str(line.trim_end_matches(['\r', '\n']))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_event() {
        let msg = Message::Event {
            seq: 1,
            name: "begin_game_turn".to_string(),
            args: serde_json::json!({ "turn": 42 }),
        };
        let line = encode_jsonl(&msg).unwrap();
        let decoded = decode_jsonl(&line).unwrap();
        assert!(matches!(decoded, Message::Event { seq: 1, .. }));
    }

    #[test]
    fn decodes_reply() {
        let decoded =
            decode_jsonl(r#"{"type":"reply","id":7,"ok":true,"result":{"gold":500}}"#).unwrap();
        match decoded {
            Message::Reply { id, ok, .. } => {
                assert_eq!(id, 7);
                assert!(ok);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn decodes_callback_request() {
        let decoded = decode_jsonl(
            r#"{"type":"callback_request","id":9,"name":"kbd_event","args":{"key":65}}"#,
        )
        .unwrap();
        match decoded {
            Message::CallbackRequest { id, name, args } => {
                assert_eq!(id, 9);
                assert_eq!(name, "kbd_event");
                assert_eq!(args["key"], 65);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn hello_reports_capabilities() {
        let hello = decode_jsonl(
            r#"{"type":"hello","protocol":1,"side":"dll","capabilities":["events","queries","callback_requests"]}"#,
        )
        .unwrap()
        .into_hello()
        .unwrap();

        assert_eq!(hello.protocol, BRIDGE_PROTOCOL_VERSION);
        assert_eq!(hello.side, "dll");
        assert!(hello.has_capability("queries"));
        assert_eq!(
            hello.missing_capabilities(&["queries", "commands", "callback_requests"]),
            vec!["commands"]
        );
    }
}
