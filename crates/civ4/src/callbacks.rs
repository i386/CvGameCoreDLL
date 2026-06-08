use crate::client::{BridgeClient, Result};
use crate::events::{BridgeCallbackMessage, BridgeEventMessage};
use serde::Serialize;
use serde_json::{json, Value};

pub type CallbackHandler =
    dyn FnMut(&mut BridgeClient, &BridgeCallbackMessage) -> Result<CallbackControl>;

#[derive(Debug, Clone, PartialEq)]
pub enum CallbackControl {
    Continue,
    Stop,
    Respond(Value),
    RespondAndStop(Value),
}

impl CallbackControl {
    pub fn consume(consume: bool) -> Self {
        Self::Respond(InputCallbackReply::new(consume).into_value())
    }

    pub fn consume_and_stop(consume: bool) -> Self {
        Self::RespondAndStop(InputCallbackReply::new(consume).into_value())
    }

    pub fn rule_value(value: bool) -> Self {
        Self::Respond(RuleCallbackReply::new(value).into_value())
    }

    pub fn rule_value_and_stop(value: bool) -> Self {
        Self::RespondAndStop(RuleCallbackReply::new(value).into_value())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct InputCallbackReply {
    pub consume: bool,
}

impl InputCallbackReply {
    pub fn new(consume: bool) -> Self {
        Self { consume }
    }

    pub fn into_value(self) -> Value {
        serde_json::to_value(self).expect("input callback reply serializes")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RuleCallbackReply {
    pub value: bool,
}

impl RuleCallbackReply {
    pub fn new(value: bool) -> Self {
        Self { value }
    }

    pub fn into_value(self) -> Value {
        serde_json::to_value(self).expect("rule callback reply serializes")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallbackDispatch {
    pub callback: BridgeCallbackMessage,
    pub handlers_run: usize,
    pub stopped: bool,
    pub reply_sent: bool,
}

pub struct CallbackDispatcher {
    handlers: Vec<RegisteredHandler>,
}

struct RegisteredHandler {
    name: Option<String>,
    handler: Box<CallbackHandler>,
}

impl CallbackDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn on_any<F>(&mut self, handler: F) -> &mut Self
    where
        F: FnMut(&mut BridgeClient, &BridgeCallbackMessage) -> Result<CallbackControl> + 'static,
    {
        self.handlers.push(RegisteredHandler {
            name: None,
            handler: Box::new(handler),
        });
        self
    }

    pub fn on_name<F>(&mut self, name: impl Into<String>, handler: F) -> &mut Self
    where
        F: FnMut(&mut BridgeClient, &BridgeCallbackMessage) -> Result<CallbackControl> + 'static,
    {
        self.handlers.push(RegisteredHandler {
            name: Some(name.into()),
            handler: Box::new(handler),
        });
        self
    }

    pub fn dispatch_next(&mut self, client: &mut BridgeClient) -> Result<CallbackDispatch> {
        let callback = client.next_callback_message()?;
        self.dispatch_callback(client, callback)
    }

    pub fn dispatch_event(
        &mut self,
        client: &mut BridgeClient,
        event: BridgeEventMessage,
    ) -> Result<CallbackDispatch> {
        self.dispatch_callback(client, BridgeCallbackMessage::Mirror(event))
    }

    pub fn dispatch_callback(
        &mut self,
        client: &mut BridgeClient,
        callback: BridgeCallbackMessage,
    ) -> Result<CallbackDispatch> {
        let mut handlers_run = 0;
        let mut stopped = false;
        let mut response = None;

        for registered in &mut self.handlers {
            if !registered.matches(&callback) {
                continue;
            }

            handlers_run += 1;
            match (registered.handler)(client, &callback)? {
                CallbackControl::Continue => {}
                CallbackControl::Stop => {
                    stopped = true;
                    break;
                }
                CallbackControl::Respond(value) => {
                    response = Some(value);
                }
                CallbackControl::RespondAndStop(value) => {
                    response = Some(value);
                    stopped = true;
                    break;
                }
            }
        }

        let mut reply_sent = false;
        if let Some(id) = callback.request_id() {
            let result = response.unwrap_or_else(|| json!({}));
            client.write_callback_success(id, &result)?;
            reply_sent = true;
        }

        Ok(CallbackDispatch {
            callback,
            handlers_run,
            stopped,
            reply_sent,
        })
    }

    pub fn run_until_stopped(&mut self, client: &mut BridgeClient) -> Result<CallbackDispatch> {
        loop {
            let dispatch = self.dispatch_next(client)?;
            if dispatch.stopped {
                return Ok(dispatch);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    pub fn len(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for CallbackDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RegisteredHandler {
    fn matches(&self, callback: &BridgeCallbackMessage) -> bool {
        match &self.name {
            Some(name) => name == callback.name(),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{BridgeEvent, BridgeEventMessage};
    use crate::types::PlayerId;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn typed_callback_replies_match_protocol_fields() {
        assert_eq!(
            InputCallbackReply::new(true).into_value(),
            json!({ "consume": true })
        );
        assert_eq!(
            RuleCallbackReply::new(false).into_value(),
            json!({ "value": false })
        );
    }

    #[test]
    fn callback_control_builds_typed_replies() {
        assert_eq!(
            CallbackControl::consume(false),
            CallbackControl::Respond(json!({ "consume": false }))
        );
        assert_eq!(
            CallbackControl::consume_and_stop(true),
            CallbackControl::RespondAndStop(json!({ "consume": true }))
        );
        assert_eq!(
            CallbackControl::rule_value(true),
            CallbackControl::Respond(json!({ "value": true }))
        );
        assert_eq!(
            CallbackControl::rule_value_and_stop(false),
            CallbackControl::RespondAndStop(json!({ "value": false }))
        );
    }

    #[test]
    fn dispatches_matching_handlers_in_order() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut dispatcher = CallbackDispatcher::new();

        {
            let calls = calls.clone();
            dispatcher.on_name("begin_player_turn", move |_client, callback| {
                calls.borrow_mut().push(callback.name().to_string());
                Ok(CallbackControl::Continue)
            });
        }
        {
            let calls = calls.clone();
            dispatcher.on_any(move |_client, _event| {
                calls.borrow_mut().push("any".to_string());
                Ok(CallbackControl::Continue)
            });
        }

        let event = BridgeEventMessage {
            seq: 1,
            event: BridgeEvent::BeginPlayerTurn {
                turn: 3,
                player: PlayerId(0),
            },
        };

        let mut client = dummy_client();
        let dispatch = dispatcher.dispatch_event(&mut client, event).unwrap();

        assert_eq!(dispatch.handlers_run, 2);
        assert!(!dispatch.stopped);
        assert!(!dispatch.reply_sent);
        assert_eq!(
            calls.borrow().as_slice(),
            ["begin_player_turn".to_string(), "any".to_string()]
        );
    }

    #[test]
    fn stop_prevents_later_handlers() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut dispatcher = CallbackDispatcher::new();

        {
            let calls = calls.clone();
            dispatcher.on_any(move |_client, _event| {
                calls.borrow_mut().push("first".to_string());
                Ok(CallbackControl::Stop)
            });
        }
        {
            let calls = calls.clone();
            dispatcher.on_any(move |_client, _event| {
                calls.borrow_mut().push("second".to_string());
                Ok(CallbackControl::Continue)
            });
        }

        let event = BridgeEventMessage {
            seq: 2,
            event: BridgeEvent::GameStart,
        };

        let mut client = dummy_client();
        let dispatch = dispatcher.dispatch_event(&mut client, event).unwrap();

        assert_eq!(dispatch.handlers_run, 1);
        assert!(dispatch.stopped);
        assert!(!dispatch.reply_sent);
        assert_eq!(calls.borrow().as_slice(), ["first".to_string()]);
    }

    #[test]
    fn callback_request_sends_default_reply() {
        let mut dispatcher = CallbackDispatcher::new();
        dispatcher.on_any(|_client, _callback| Ok(CallbackControl::Continue));

        let callback = BridgeCallbackMessage::Request(crate::events::BridgeCallbackRequest {
            id: 42,
            event: BridgeEvent::GameStart,
        });

        let mut client = dummy_client();
        let dispatch = dispatcher.dispatch_callback(&mut client, callback).unwrap();

        assert_eq!(dispatch.handlers_run, 1);
        assert!(dispatch.reply_sent);
    }

    #[test]
    fn callback_request_can_set_response_and_stop() {
        let mut dispatcher = CallbackDispatcher::new();
        dispatcher.on_any(|_client, _callback| {
            Ok(CallbackControl::RespondAndStop(json!({ "consume": true })))
        });

        let callback = BridgeCallbackMessage::Request(crate::events::BridgeCallbackRequest {
            id: 43,
            event: BridgeEvent::GameStart,
        });

        let mut client = dummy_client();
        let dispatch = dispatcher.dispatch_callback(&mut client, callback).unwrap();

        assert_eq!(dispatch.handlers_run, 1);
        assert!(dispatch.stopped);
        assert!(dispatch.reply_sent);
    }

    #[cfg(unix)]
    fn dummy_client() -> BridgeClient {
        BridgeClient::connect("/dev/null", "/dev/null").unwrap()
    }

    #[cfg(windows)]
    fn dummy_client() -> BridgeClient {
        let path = std::env::temp_dir().join("civ4-callback-dispatcher-test.tmp");
        std::fs::write(&path, b"").unwrap();
        BridgeClient::connect(&path, &path).unwrap()
    }
}
