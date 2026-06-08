use crate::client::{BridgeClient, Result};
use crate::metadata::{InfoCount, InfoTypeEntry, InfoTypeState, InfoTypesResult};
use crate::types::{InfoKind, InfoType};
use serde_json::json;

impl BridgeClient {
    pub fn get_info_count(&mut self, kind: InfoKind) -> Result<i32> {
        let result: InfoCount = self.query("get_info_count", json!({ "kind": kind }))?;
        Ok(result.count)
    }

    pub fn get_info_type<I>(&mut self, kind: InfoKind, value: I) -> Result<InfoTypeState>
    where
        I: Into<InfoType>,
    {
        self.query(
            "get_info_type",
            json!({ "kind": kind, "value": value.into() }),
        )
    }

    pub fn resolve_info_id<I>(&mut self, kind: InfoKind, value: I) -> Result<i32>
    where
        I: Into<InfoType>,
    {
        Ok(self.get_info_type(kind, value)?.id)
    }

    pub fn list_info_types(&mut self, kind: InfoKind) -> Result<Vec<InfoTypeEntry>> {
        let result: InfoTypesResult = self.query("list_info_types", json!({ "kind": kind }))?;
        Ok(result.types)
    }
}
