use crate::types::{CityProductionRule, CityRef, PlayerId, Plot, TeamId, UnitRef};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeEventMessage {
    pub seq: u64,
    pub event: BridgeEvent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeCallbackRequest {
    pub id: u64,
    pub event: BridgeEvent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BridgeCallbackMessage {
    Mirror(BridgeEventMessage),
    Request(BridgeCallbackRequest),
}

impl BridgeCallbackMessage {
    pub fn event(&self) -> &BridgeEvent {
        match self {
            Self::Mirror(message) => &message.event,
            Self::Request(request) => &request.event,
        }
    }

    pub fn name(&self) -> &str {
        self.event().name()
    }

    pub fn request_id(&self) -> Option<u64> {
        match self {
            Self::Mirror(_) => None,
            Self::Request(request) => Some(request.id),
        }
    }

    pub fn is_request(&self) -> bool {
        self.request_id().is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BridgeEvent {
    Init,
    Uninit,
    KbdEvent {
        evt: i32,
        key: i32,
        cursor_x: i32,
        cursor_y: i32,
        plot: Option<Plot>,
    },
    MouseEvent {
        evt: i32,
        cursor_x: i32,
        cursor_y: i32,
        plot: Option<Plot>,
        interface_consumed: bool,
    },
    GameStart,
    GameEnd,
    PreSave,
    ModNetMessage {
        data1: i32,
        data2: i32,
        data3: i32,
        data4: i32,
        data5: i32,
    },
    Update {
        delta_time: f64,
    },
    WindowActivation {
        active: bool,
    },
    Chat {
        text: String,
    },
    UiText {
        args: Value,
    },
    BeginGameTurn {
        turn: i32,
    },
    EndGameTurn {
        turn: i32,
    },
    BeginPlayerTurn {
        turn: i32,
        player: PlayerId,
    },
    EndPlayerTurn {
        turn: i32,
        player: PlayerId,
    },
    IsPlayerResearch {
        player: PlayerId,
    },
    CanResearch {
        player: PlayerId,
        tech: i32,
        trade: bool,
    },
    CannotResearch {
        player: PlayerId,
        tech: i32,
        trade: bool,
    },
    CanDoCivic {
        player: PlayerId,
        civic: i32,
    },
    CannotDoCivic {
        player: PlayerId,
        civic: i32,
    },
    CannotFoundCity {
        player: PlayerId,
        plot: Plot,
        test_visible: bool,
    },
    CanFoundCitiesOnWater {
        player: PlayerId,
        plot: Plot,
        test_visible: bool,
    },
    CityFoundValue {
        player: PlayerId,
        plot: Plot,
    },
    FirstContact {
        team: TeamId,
        other_team: TeamId,
    },
    CanDeclareWar {
        team: TeamId,
        other_team: TeamId,
    },
    CombatResult {
        winner: UnitRef,
        winner_unit_type: i32,
        winner_plot: Plot,
        loser: UnitRef,
        loser_unit_type: i32,
        loser_plot: Plot,
    },
    ImprovementBuilt {
        improvement: i32,
        plot: Plot,
    },
    ImprovementDestroyed {
        improvement: i32,
        player: PlayerId,
        plot: Plot,
    },
    RouteBuilt {
        route: i32,
        plot: Plot,
    },
    PlotRevealed {
        plot: Plot,
        team: TeamId,
    },
    PlotFeatureRemoved {
        plot: Plot,
        feature: i32,
        city: Option<CityRef>,
    },
    PlotPicked {
        plot: Plot,
    },
    CanBuild {
        plot: Plot,
        build: i32,
        player: PlayerId,
        test_visible: bool,
    },
    NukeExplosion {
        plot: Plot,
        unit: Option<UnitRef>,
        unit_type: i32,
    },
    GotoPlotSet {
        plot: Plot,
        player: PlayerId,
    },
    CityBuilt {
        city: CityRef,
        plot: Plot,
    },
    CityRazed {
        city: CityRef,
        razed_by: PlayerId,
        plot: Plot,
    },
    CityAcquired {
        old_player: PlayerId,
        city: CityRef,
        conquest: bool,
        trade: bool,
        plot: Plot,
    },
    CityAcquiredKept {
        city: CityRef,
        plot: Plot,
    },
    CityLost {
        city: CityRef,
        plot: Plot,
    },
    CityGrowth {
        city: CityRef,
        population: i32,
    },
    CultureExpansion {
        city: CityRef,
        plot: Plot,
    },
    CityDoTurn {
        city: CityRef,
        plot: Plot,
    },
    CityBuildingUnit {
        city: CityRef,
        unit_type: i32,
    },
    CityBuildingBuilding {
        city: CityRef,
        building: i32,
    },
    BuildingCostMod {
        city: CityRef,
        plot: Plot,
        building: i32,
    },
    CityRename {
        city: CityRef,
        plot: Plot,
    },
    CityHurry {
        city: CityRef,
        hurry: i32,
    },
    CityProductionRule {
        rule: CityProductionRule,
        city: CityRef,
        plot: Plot,
        item: i32,
        continue_current: bool,
        test_visible: bool,
        ignore_cost: bool,
        ignore_upgrades: bool,
    },
    SelectionGroupPushMission {
        player: PlayerId,
        group: i32,
        mission: i32,
    },
    UnitMove {
        unit: UnitRef,
        from: Plot,
        to: Plot,
    },
    UnitCannotMoveInto {
        unit: UnitRef,
        unit_type: i32,
        plot: Plot,
        attack: bool,
        declare_war: bool,
        ignore_load: bool,
    },
    UnitSetXY {
        unit: UnitRef,
        unit_type: i32,
        plot: Plot,
    },
    UnitCreated {
        unit: UnitRef,
        unit_type: i32,
        plot: Plot,
    },
    UnitBuilt {
        city: CityRef,
        unit: UnitRef,
        unit_type: i32,
        plot: Plot,
    },
    UnitKilled {
        unit: UnitRef,
        unit_type: i32,
        attacker: PlayerId,
        plot: Plot,
    },
    UnitLost {
        unit: UnitRef,
        unit_type: i32,
        plot: Plot,
    },
    UnitPromoted {
        unit: UnitRef,
        unit_type: i32,
        promotion: i32,
        plot: Plot,
    },
    UnitSelected {
        unit: UnitRef,
        unit_type: i32,
        plot: Plot,
    },
    UnitRename {
        unit: UnitRef,
        unit_type: i32,
        plot: Plot,
    },
    UnitPillage {
        unit: UnitRef,
        unit_type: i32,
        improvement: i32,
        route: i32,
        pillage_player: PlayerId,
        plot: Plot,
    },
    UnitSpreadReligionAttempt {
        unit: UnitRef,
        unit_type: i32,
        religion: i32,
        success: bool,
        plot: Plot,
    },
    UnitGifted {
        unit: UnitRef,
        unit_type: i32,
        gifting_player: PlayerId,
        plot: Plot,
    },
    UnitBuildImprovement {
        unit: UnitRef,
        unit_type: i32,
        build: i32,
        finished: bool,
        plot: Plot,
    },
    GoodyReceived {
        player: PlayerId,
        plot: Plot,
        unit: Option<UnitRef>,
        unit_type: i32,
        goody: i32,
    },
    GreatPersonBorn {
        player: PlayerId,
        city: CityRef,
        unit: UnitRef,
        unit_type: i32,
        plot: Plot,
    },
    BuildingBuilt {
        city: CityRef,
        building: i32,
    },
    ProjectBuilt {
        city: CityRef,
        project: i32,
    },
    TechAcquired {
        team: TeamId,
        player: PlayerId,
        tech: i32,
        announce: bool,
    },
    TechSelected {
        player: PlayerId,
        tech: i32,
    },
    ReligionFounded {
        player: PlayerId,
        religion: i32,
    },
    ReligionSpread {
        city: CityRef,
        religion: i32,
    },
    ReligionRemove {
        city: CityRef,
        religion: i32,
    },
    CorporationFounded {
        player: PlayerId,
        corporation: i32,
    },
    CorporationSpread {
        city: CityRef,
        corporation: i32,
    },
    CorporationRemove {
        city: CityRef,
        corporation: i32,
    },
    GoldenAge {
        player: PlayerId,
    },
    EndGoldenAge {
        player: PlayerId,
    },
    ChangeWar {
        war: bool,
        team: TeamId,
        other_team: TeamId,
    },
    PlayerGoldTrade {
        from_player: PlayerId,
        to_player: PlayerId,
        amount: i32,
    },
    SetPlayerAlive {
        player: PlayerId,
        alive: bool,
    },
    PlayerChangeStateReligion {
        player: PlayerId,
        new_religion: i32,
        old_religion: i32,
    },
    Victory {
        team: TeamId,
        victory: i32,
    },
    VassalState {
        master: TeamId,
        vassal: TeamId,
        is_vassal: bool,
    },
    Unknown {
        name: String,
        args: Value,
    },
}
