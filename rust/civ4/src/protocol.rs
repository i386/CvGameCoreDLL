use serde::{Deserialize, Serialize};
use serde_json::Value;

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
        let decoded = decode_jsonl(r#"{"type":"reply","id":7,"ok":true,"result":{"gold":500}}"#).unwrap();
        match decoded {
            Message::Reply { id, ok, .. } => {
                assert_eq!(id, 7);
                assert!(ok);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }
}
