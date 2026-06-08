use crate::client::{BridgeClient, Result};
use crate::state::{PlayerSelectionGroupsResult, SelectionGroupState};
use crate::types::{PlayerId, SelectionGroupRef};
use serde_json::json;

impl BridgeClient {
    pub fn get_selection_group_state(
        &mut self,
        group: SelectionGroupRef,
    ) -> Result<SelectionGroupState> {
        self.query(
            "get_selection_group_state",
            json!({ "player": group.player, "group": group.id }),
        )
    }

    pub fn list_player_selection_groups<P: Into<PlayerId>>(
        &mut self,
        player: P,
    ) -> Result<Vec<SelectionGroupState>> {
        let player = player.into();
        let result: PlayerSelectionGroupsResult = self.query(
            "list_player_selection_groups",
            json!({ "player": player.0 }),
        )?;
        Ok(result.groups)
    }

    pub fn list_all_selection_groups(&mut self) -> Result<Vec<SelectionGroupState>> {
        let mut groups = Vec::new();
        for player in self.list_alive_players()? {
            groups.extend(self.list_player_selection_groups(player.player_id())?);
        }
        Ok(groups)
    }
}
