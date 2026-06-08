pub mod callbacks;
mod city_api;
pub mod client;
pub mod commands;
mod event_kind;
mod event_payloads;
#[cfg(test)]
mod event_tests;
pub mod events;
mod game_api;
mod info_api;
pub mod metadata;
mod mod_state_api;
mod player_api;
mod player_identity_state;
mod plot_api;
mod plot_state;
pub mod protocol;
mod selection_group_api;
mod selection_group_state;
pub mod state;
mod team_api;
pub mod types;
mod unit_api;
mod unit_state;

pub use callbacks::{
    CallbackControl, CallbackDispatch, CallbackDispatcher, InputCallbackReply, RuleCallbackReply,
};
pub use city_api::CityProductionOptionsQuery;
pub use client::{BridgeClient, BridgeError, Result};
pub use commands::{
    CityOrder, CityOrderType, SpawnUnitRequest, SpawnedUnit, UnitCommandName, UnitCommandType,
    UnitGroupCommand, UnitGroupJoin, UnitGroupMission,
};
pub use event_kind::BridgeEventKind;
pub use events::{BridgeCallbackMessage, BridgeCallbackRequest, BridgeEvent, BridgeEventMessage};
pub use metadata::{InfoCount, InfoTypeEntry, InfoTypeState, InfoTypesResult};
pub use plot_api::{
    PlotCultureChangeOptions, PlotCultureOptions, PlotFeatureOptions, PlotOwnerOptions,
    PlotRevealedOptions, PlotRouteOptions, PlotTerrainOptions,
};
pub use protocol::{BridgeCapability, BridgeHello, BridgeReply, Message, BRIDGE_PROTOCOL_VERSION};
pub use state::{
    CityBuildingClassChange, CityBuildingState, CityCorporationState, CityDetailState,
    CityProductionOptions, CityReligionState, CityState, ForceControlState, GameOptionState,
    GameState, KilledUnit, MapState, MultiplayerOptionState, PlayerEconomyState,
    PlayerGoldPerTurnState, PlayerIdentityState, PlayerOptions, PlayerState, PlotCultureState,
    PlotState, PlotVisibilityState, SelectionGroupCommandCheck, SelectionGroupMissionCheck,
    SelectionGroupMissionState, SelectionGroupState, TeamRelationState, TeamState, TeamTechState,
    UnitCommandResult, UnitDetailState, UnitGroupJoinCheck, UnitPromotionState, UnitState,
};
pub use types::{
    CityProductionItem, CityProductionRule, CityRef, CommerceType, GameStatus, InfoKind, InfoType,
    PlayerId, Plot, SelectionGroupRef, TeamId, UnitRef, WarPlan,
};
