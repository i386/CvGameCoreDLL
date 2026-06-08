use crate::client::{BridgeClient, Result};
use crate::state::{ModStateResult, SetModStateResult};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;

impl BridgeClient {
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
}
