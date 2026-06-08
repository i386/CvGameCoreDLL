use crate::types::{PlayerId, TeamId};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlayerIdentityState {
    pub player: i32,
    pub team: i32,
    pub civilization: i32,
    pub leader: i32,
    pub personality: i32,
    pub name: String,
    pub name_key: String,
    pub civilization_description: String,
    pub civilization_description_key: String,
    pub civilization_short_description: String,
    pub civilization_short_description_key: String,
    pub civilization_adjective: String,
    pub civilization_adjective_key: String,
}

impl PlayerIdentityState {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(self.player)
    }

    pub fn team_id(&self) -> TeamId {
        TeamId(self.team)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_player_identity_state() {
        let identity: PlayerIdentityState = serde_json::from_value(json!({
            "player": 0,
            "team": 0,
            "civilization": 1,
            "leader": 2,
            "personality": 2,
            "name": "Gandhi",
            "name_key": "TXT_KEY_LEADER_GANDHI",
            "civilization_description": "Indian Empire",
            "civilization_description_key": "TXT_KEY_CIV_INDIA_DESC",
            "civilization_short_description": "India",
            "civilization_short_description_key": "TXT_KEY_CIV_INDIA_SHORT_DESC",
            "civilization_adjective": "Indian",
            "civilization_adjective_key": "TXT_KEY_CIV_INDIA_ADJECTIVE"
        }))
        .unwrap();

        assert_eq!(identity.player_id(), PlayerId(0));
        assert_eq!(identity.team_id(), TeamId(0));
        assert_eq!(identity.name, "Gandhi");
        assert_eq!(identity.civilization_short_description, "India");
    }
}
