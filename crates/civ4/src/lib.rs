pub mod callbacks;
mod city_api;
pub mod client;
pub mod commands;
mod event_payloads;
#[cfg(test)]
mod event_tests;
pub mod events;
mod game_api;
mod info_api;
pub mod metadata;
mod mod_state_api;
mod player_api;
mod plot_api;
mod plot_state;
pub mod protocol;
pub mod state;
mod team_api;
pub mod types;
mod unit_api;

pub use callbacks::{CallbackControl, CallbackDispatch, CallbackDispatcher};
pub use client::{BridgeClient, BridgeError, Result};
pub use commands::{CityOrder, CityOrderType, SpawnUnitRequest, SpawnedUnit};
pub use events::{BridgeCallbackMessage, BridgeCallbackRequest, BridgeEvent, BridgeEventMessage};
pub use metadata::{InfoCount, InfoTypeEntry, InfoTypeState, InfoTypesResult};
pub use plot_api::{
    PlotCultureChangeOptions, PlotCultureOptions, PlotFeatureOptions, PlotOwnerOptions,
    PlotRevealedOptions, PlotRouteOptions, PlotTerrainOptions,
};
pub use protocol::{BridgeHello, BridgeReply, Message, BRIDGE_PROTOCOL_VERSION};
pub use state::{
    CityBuildingClassChange, CityBuildingState, CityCorporationState, CityReligionState, CityState,
    ForceControlState, GameOptionState, GameState, KilledUnit, MapState, MultiplayerOptionState,
    PlayerEconomyState, PlayerGoldPerTurnState, PlayerOptions, PlayerState, PlotCultureState,
    PlotState, PlotVisibilityState, TeamRelationState, TeamState, TeamTechState,
    UnitPromotionState, UnitState,
};
pub use types::{
    CityRef, CommerceType, GameStatus, InfoKind, InfoType, PlayerId, Plot, TeamId, UnitRef, WarPlan,
};
