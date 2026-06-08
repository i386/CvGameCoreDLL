pub mod callbacks;
mod city_api;
pub mod client;
pub mod commands;
pub mod events;
mod game_api;
pub mod protocol;
pub mod state;
mod team_api;
pub mod types;
mod unit_api;

pub use callbacks::{CallbackControl, CallbackDispatch, CallbackDispatcher};
pub use client::{BridgeClient, BridgeError, Result};
pub use commands::{CityOrder, CityOrderType, SpawnUnitRequest, SpawnedUnit};
pub use events::{BridgeCallbackMessage, BridgeCallbackRequest, BridgeEvent, BridgeEventMessage};
pub use protocol::{BridgeHello, BridgeReply, Message, BRIDGE_PROTOCOL_VERSION};
pub use state::{
    CityBuildingClassChange, CityBuildingState, CityCorporationState, CityReligionState, CityState,
    ForceControlState, GameOptionState, GameState, KilledUnit, MapState, MultiplayerOptionState,
    PlayerOptions, PlayerState, PlotState, TeamRelationState, TeamState, TeamTechState,
    UnitPromotionState, UnitState,
};
pub use types::{CityRef, GameStatus, InfoType, PlayerId, Plot, TeamId, UnitRef, WarPlan};
