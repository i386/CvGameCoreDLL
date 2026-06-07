use crate::client::{BridgeClient, Result};
use crate::events::BridgeEventMessage;

pub type CallbackHandler =
    dyn FnMut(&mut BridgeClient, &BridgeEventMessage) -> Result<CallbackControl>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackControl {
    Continue,
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallbackDispatch {
    pub event: BridgeEventMessage,
    pub handlers_run: usize,
    pub stopped: bool,
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
        F: FnMut(&mut BridgeClient, &BridgeEventMessage) -> Result<CallbackControl> + 'static,
    {
        self.handlers.push(RegisteredHandler {
            name: None,
            handler: Box::new(handler),
        });
        self
    }

    pub fn on_name<F>(&mut self, name: impl Into<String>, handler: F) -> &mut Self
    where
        F: FnMut(&mut BridgeClient, &BridgeEventMessage) -> Result<CallbackControl> + 'static,
    {
        self.handlers.push(RegisteredHandler {
            name: Some(name.into()),
            handler: Box::new(handler),
        });
        self
    }

    pub fn dispatch_next(&mut self, client: &mut BridgeClient) -> Result<CallbackDispatch> {
        let event = client.next_callback_event()?;
        Ok(self.dispatch_event(client, event)?)
    }

    pub fn dispatch_event(
        &mut self,
        client: &mut BridgeClient,
        event: BridgeEventMessage,
    ) -> Result<CallbackDispatch> {
        let mut handlers_run = 0;
        let mut stopped = false;

        for registered in &mut self.handlers {
            if !registered.matches(&event) {
                continue;
            }

            handlers_run += 1;
            if (registered.handler)(client, &event)? == CallbackControl::Stop {
                stopped = true;
                break;
            }
        }

        Ok(CallbackDispatch {
            event,
            handlers_run,
            stopped,
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
    fn matches(&self, event: &BridgeEventMessage) -> bool {
        match &self.name {
            Some(name) => name == event.event.name(),
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
    fn dispatches_matching_handlers_in_order() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut dispatcher = CallbackDispatcher::new();

        {
            let calls = calls.clone();
            dispatcher.on_name("begin_player_turn", move |_client, event| {
                calls.borrow_mut().push(event.event.name().to_string());
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
        assert_eq!(calls.borrow().as_slice(), ["first".to_string()]);
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
