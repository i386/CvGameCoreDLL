use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InfoCount {
    pub kind: String,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InfoTypeState {
    pub kind: String,
    pub id: i32,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InfoTypeEntry {
    pub id: i32,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InfoTypesResult {
    pub kind: String,
    pub types: Vec<InfoTypeEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_info_metadata_results() {
        let count: InfoCount = serde_json::from_value(json!({
            "kind": "unit",
            "count": 5
        }))
        .unwrap();
        assert_eq!(count.count, 5);

        let info: InfoTypeState = serde_json::from_value(json!({
            "kind": "unit",
            "id": 1,
            "type": "UNIT_WARRIOR"
        }))
        .unwrap();
        assert_eq!(info.type_name, "UNIT_WARRIOR");

        let list: InfoTypesResult = serde_json::from_value(json!({
            "kind": "tech",
            "types": [
                { "id": 0, "type": "TECH_AGRICULTURE" },
                { "id": 1, "type": "TECH_MINING" }
            ]
        }))
        .unwrap();
        assert_eq!(list.types[1].id, 1);
        assert_eq!(list.types[1].type_name, "TECH_MINING");
    }
}
