use crate::event_payloads::*;
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
    FirstContact {
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

impl BridgeEvent {
    pub fn name(&self) -> &str {
        match self {
            Self::Init => "init",
            Self::Uninit => "uninit",
            Self::KbdEvent { .. } => "kbd_event",
            Self::MouseEvent { .. } => "mouse_event",
            Self::GameStart => "game_start",
            Self::GameEnd => "game_end",
            Self::PreSave => "pre_save",
            Self::BeginGameTurn { .. } => "begin_game_turn",
            Self::EndGameTurn { .. } => "end_game_turn",
            Self::BeginPlayerTurn { .. } => "begin_player_turn",
            Self::EndPlayerTurn { .. } => "end_player_turn",
            Self::FirstContact { .. } => "first_contact",
            Self::CombatResult { .. } => "combat_result",
            Self::ImprovementBuilt { .. } => "improvement_built",
            Self::ImprovementDestroyed { .. } => "improvement_destroyed",
            Self::RouteBuilt { .. } => "route_built",
            Self::PlotRevealed { .. } => "plot_revealed",
            Self::PlotFeatureRemoved { .. } => "plot_feature_removed",
            Self::PlotPicked { .. } => "plot_picked",
            Self::NukeExplosion { .. } => "nuke_explosion",
            Self::GotoPlotSet { .. } => "goto_plot_set",
            Self::CityBuilt { .. } => "city_built",
            Self::CityRazed { .. } => "city_razed",
            Self::CityAcquired { .. } => "city_acquired",
            Self::CityAcquiredKept { .. } => "city_acquired_kept",
            Self::CityLost { .. } => "city_lost",
            Self::CityGrowth { .. } => "city_growth",
            Self::CultureExpansion { .. } => "culture_expansion",
            Self::CityDoTurn { .. } => "city_do_turn",
            Self::CityBuildingUnit { .. } => "city_building_unit",
            Self::CityBuildingBuilding { .. } => "city_building_building",
            Self::BuildingCostMod { .. } => "get_building_cost_mod",
            Self::CityRename { .. } => "city_rename",
            Self::CityHurry { .. } => "city_hurry",
            Self::CityProductionRule { rule, .. } => rule.name(),
            Self::SelectionGroupPushMission { .. } => "selection_group_push_mission",
            Self::UnitMove { .. } => "unit_move",
            Self::UnitSetXY { .. } => "unit_set_xy",
            Self::UnitCreated { .. } => "unit_created",
            Self::UnitBuilt { .. } => "unit_built",
            Self::UnitKilled { .. } => "unit_killed",
            Self::UnitLost { .. } => "unit_lost",
            Self::UnitPromoted { .. } => "unit_promoted",
            Self::UnitSelected { .. } => "unit_selected",
            Self::UnitRename { .. } => "unit_rename",
            Self::UnitPillage { .. } => "unit_pillage",
            Self::UnitSpreadReligionAttempt { .. } => "unit_spread_religion_attempt",
            Self::UnitGifted { .. } => "unit_gifted",
            Self::UnitBuildImprovement { .. } => "unit_build_improvement",
            Self::GoodyReceived { .. } => "goody_received",
            Self::GreatPersonBorn { .. } => "great_person_born",
            Self::BuildingBuilt { .. } => "building_built",
            Self::ProjectBuilt { .. } => "project_built",
            Self::TechAcquired { .. } => "tech_acquired",
            Self::TechSelected { .. } => "tech_selected",
            Self::ReligionFounded { .. } => "religion_founded",
            Self::ReligionSpread { .. } => "religion_spread",
            Self::ReligionRemove { .. } => "religion_remove",
            Self::CorporationFounded { .. } => "corporation_founded",
            Self::CorporationSpread { .. } => "corporation_spread",
            Self::CorporationRemove { .. } => "corporation_remove",
            Self::GoldenAge { .. } => "golden_age",
            Self::EndGoldenAge { .. } => "end_golden_age",
            Self::ChangeWar { .. } => "change_war",
            Self::PlayerGoldTrade { .. } => "player_gold_trade",
            Self::SetPlayerAlive { .. } => "set_player_alive",
            Self::PlayerChangeStateReligion { .. } => "player_change_state_religion",
            Self::Victory { .. } => "victory",
            Self::VassalState { .. } => "vassal_state",
            Self::Unknown { name, .. } => name,
        }
    }

    pub fn from_name_args(name: String, args: Value) -> serde_json::Result<Self> {
        Ok(match name.as_str() {
            "init" => Self::Init,
            "uninit" => Self::Uninit,
            "kbd_event" => {
                let payload: KbdEventPayload = decode(args)?;
                Self::KbdEvent {
                    evt: payload.evt,
                    key: payload.key,
                    cursor_x: payload.cursor_x,
                    cursor_y: payload.cursor_y,
                    plot: payload.plot(),
                }
            }
            "mouse_event" => {
                let payload: MouseEventPayload = decode(args)?;
                Self::MouseEvent {
                    evt: payload.evt,
                    cursor_x: payload.cursor_x,
                    cursor_y: payload.cursor_y,
                    plot: payload.plot(),
                    interface_consumed: payload.interface_consumed,
                }
            }
            "game_start" => Self::GameStart,
            "game_end" => Self::GameEnd,
            "pre_save" => Self::PreSave,
            "begin_game_turn" => {
                let payload: TurnPayload = decode(args)?;
                Self::BeginGameTurn { turn: payload.turn }
            }
            "end_game_turn" => {
                let payload: TurnPayload = decode(args)?;
                Self::EndGameTurn { turn: payload.turn }
            }
            "begin_player_turn" => {
                let payload: PlayerTurnPayload = decode(args)?;
                Self::BeginPlayerTurn {
                    turn: payload.turn,
                    player: PlayerId(payload.player),
                }
            }
            "end_player_turn" => {
                let payload: PlayerTurnPayload = decode(args)?;
                Self::EndPlayerTurn {
                    turn: payload.turn,
                    player: PlayerId(payload.player),
                }
            }
            "first_contact" => {
                let payload: TeamPairPayload = decode(args)?;
                Self::FirstContact {
                    team: TeamId(payload.team),
                    other_team: TeamId(payload.other_team),
                }
            }
            "combat_result" => {
                let payload: CombatResultPayload = decode(args)?;
                Self::CombatResult {
                    winner: payload.winner(),
                    winner_unit_type: payload.winner_unit_type,
                    winner_plot: payload.winner_plot(),
                    loser: payload.loser(),
                    loser_unit_type: payload.loser_unit_type,
                    loser_plot: payload.loser_plot(),
                }
            }
            "improvement_built" => {
                let payload: ImprovementBuiltPayload = decode(args)?;
                Self::ImprovementBuilt {
                    improvement: payload.improvement,
                    plot: payload.plot(),
                }
            }
            "improvement_destroyed" => {
                let payload: ImprovementDestroyedPayload = decode(args)?;
                Self::ImprovementDestroyed {
                    improvement: payload.improvement,
                    player: PlayerId(payload.player),
                    plot: payload.plot(),
                }
            }
            "route_built" => {
                let payload: RouteBuiltPayload = decode(args)?;
                Self::RouteBuilt {
                    route: payload.route,
                    plot: payload.plot(),
                }
            }
            "plot_revealed" => {
                let payload: PlotTeamPayload = decode(args)?;
                Self::PlotRevealed {
                    plot: payload.plot(),
                    team: TeamId(payload.team),
                }
            }
            "plot_feature_removed" => {
                let payload: PlotFeatureRemovedPayload = decode(args)?;
                Self::PlotFeatureRemoved {
                    plot: payload.plot(),
                    feature: payload.feature,
                    city: payload.city(),
                }
            }
            "plot_picked" => {
                let payload: PlotPayload = decode(args)?;
                Self::PlotPicked {
                    plot: payload.plot(),
                }
            }
            "nuke_explosion" => {
                let payload: UnitOptionalPayload = decode(args)?;
                Self::NukeExplosion {
                    plot: payload.plot(),
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                }
            }
            "goto_plot_set" => {
                let payload: PlotPlayerPayload = decode(args)?;
                Self::GotoPlotSet {
                    plot: payload.plot(),
                    player: PlayerId(payload.player),
                }
            }
            "city_built" => {
                let payload: CityPlotPayload = decode(args)?;
                Self::CityBuilt {
                    city: payload.city(),
                    plot: payload.plot(),
                }
            }
            "city_razed" => {
                let payload: CityRazedPayload = decode(args)?;
                Self::CityRazed {
                    city: payload.city(),
                    razed_by: PlayerId(payload.razed_by),
                    plot: payload.plot(),
                }
            }
            "city_acquired" => {
                let payload: CityAcquiredPayload = decode(args)?;
                Self::CityAcquired {
                    old_player: PlayerId(payload.old_player),
                    city: payload.city(),
                    conquest: payload.conquest,
                    trade: payload.trade,
                    plot: payload.plot(),
                }
            }
            "city_acquired_kept" => {
                let payload: CityPlotPayload = decode(args)?;
                Self::CityAcquiredKept {
                    city: payload.city(),
                    plot: payload.plot(),
                }
            }
            "city_lost" => {
                let payload: CityPlotPayload = decode(args)?;
                Self::CityLost {
                    city: payload.city(),
                    plot: payload.plot(),
                }
            }
            "city_growth" => {
                let payload: CityGrowthPayload = decode(args)?;
                Self::CityGrowth {
                    city: payload.city(),
                    population: payload.population,
                }
            }
            "culture_expansion" => {
                let payload: CityPlotPayload = decode(args)?;
                Self::CultureExpansion {
                    city: payload.city(),
                    plot: payload.plot(),
                }
            }
            "city_do_turn" => {
                let payload: CityPlotPayload = decode(args)?;
                Self::CityDoTurn {
                    city: payload.city(),
                    plot: payload.plot(),
                }
            }
            "city_building_unit" => {
                let payload: CityUnitTypePayload = decode(args)?;
                Self::CityBuildingUnit {
                    city: payload.city(),
                    unit_type: payload.unit_type,
                }
            }
            "city_building_building" => {
                let payload: CityBuildingPayload = decode(args)?;
                Self::CityBuildingBuilding {
                    city: payload.city(),
                    building: payload.building,
                }
            }
            "get_building_cost_mod" => {
                let payload: CityBuildingPlotPayload = decode(args)?;
                Self::BuildingCostMod {
                    city: payload.city(),
                    plot: payload.plot(),
                    building: payload.building,
                }
            }
            "city_rename" => {
                let payload: CityPlotPayload = decode(args)?;
                Self::CityRename {
                    city: payload.city(),
                    plot: payload.plot(),
                }
            }
            "city_hurry" => {
                let payload: CityHurryPayload = decode(args)?;
                Self::CityHurry {
                    city: payload.city(),
                    hurry: payload.hurry,
                }
            }
            "can_train" | "cannot_train" | "can_construct" | "cannot_construct" | "can_create"
            | "cannot_create" | "can_maintain" | "cannot_maintain" => {
                let rule = CityProductionRule::from_name(name.as_str()).unwrap();
                let payload: CityProductionRulePayload = decode(args)?;
                let item = match rule {
                    CityProductionRule::CanTrain | CityProductionRule::CannotTrain => {
                        payload.unit.unwrap_or(-1)
                    }
                    CityProductionRule::CanConstruct | CityProductionRule::CannotConstruct => {
                        payload.building.unwrap_or(-1)
                    }
                    CityProductionRule::CanCreate | CityProductionRule::CannotCreate => {
                        payload.project.unwrap_or(-1)
                    }
                    CityProductionRule::CanMaintain | CityProductionRule::CannotMaintain => {
                        payload.process.unwrap_or(-1)
                    }
                };
                Self::CityProductionRule {
                    rule,
                    city: payload.city(),
                    plot: payload.plot(),
                    item,
                    continue_current: payload.continue_current,
                    test_visible: payload.test_visible,
                    ignore_cost: payload.ignore_cost,
                    ignore_upgrades: payload.ignore_upgrades,
                }
            }
            "selection_group_push_mission" => {
                let payload: SelectionGroupMissionPayload = decode(args)?;
                Self::SelectionGroupPushMission {
                    player: PlayerId(payload.player),
                    group: payload.group,
                    mission: payload.mission,
                }
            }
            "unit_move" => {
                let payload: UnitMovePayload = decode(args)?;
                Self::UnitMove {
                    unit: payload.unit(),
                    from: Plot::new(payload.from_x, payload.from_y),
                    to: Plot::new(payload.x, payload.y),
                }
            }
            "unit_set_xy" => {
                let payload: UnitPlotPayload = decode(args)?;
                Self::UnitSetXY {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    plot: payload.plot(),
                }
            }
            "unit_created" => {
                let payload: UnitPlotPayload = decode(args)?;
                Self::UnitCreated {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    plot: payload.plot(),
                }
            }
            "unit_built" => {
                let payload: UnitBuiltPayload = decode(args)?;
                Self::UnitBuilt {
                    city: payload.city(),
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    plot: payload.plot(),
                }
            }
            "unit_killed" => {
                let payload: UnitKilledPayload = decode(args)?;
                Self::UnitKilled {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    attacker: PlayerId(payload.attacker),
                    plot: payload.plot(),
                }
            }
            "unit_lost" => {
                let payload: UnitPlotPayload = decode(args)?;
                Self::UnitLost {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    plot: payload.plot(),
                }
            }
            "unit_promoted" => {
                let payload: UnitPromotionPayload = decode(args)?;
                Self::UnitPromoted {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    promotion: payload.promotion,
                    plot: payload.plot(),
                }
            }
            "unit_selected" => {
                let payload: UnitPlotPayload = decode(args)?;
                Self::UnitSelected {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    plot: payload.plot(),
                }
            }
            "unit_rename" => {
                let payload: UnitPlotPayload = decode(args)?;
                Self::UnitRename {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    plot: payload.plot(),
                }
            }
            "unit_pillage" => {
                let payload: UnitPillagePayload = decode(args)?;
                Self::UnitPillage {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    improvement: payload.improvement,
                    route: payload.route,
                    pillage_player: PlayerId(payload.pillage_player),
                    plot: payload.plot(),
                }
            }
            "unit_spread_religion_attempt" => {
                let payload: UnitReligionAttemptPayload = decode(args)?;
                Self::UnitSpreadReligionAttempt {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    religion: payload.religion,
                    success: payload.success,
                    plot: payload.plot(),
                }
            }
            "unit_gifted" => {
                let payload: UnitGiftedPayload = decode(args)?;
                Self::UnitGifted {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    gifting_player: PlayerId(payload.gifting_player),
                    plot: payload.plot(),
                }
            }
            "unit_build_improvement" => {
                let payload: UnitBuildImprovementPayload = decode(args)?;
                Self::UnitBuildImprovement {
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    build: payload.build,
                    finished: payload.finished,
                    plot: payload.plot(),
                }
            }
            "goody_received" => {
                let payload: GoodyReceivedPayload = decode(args)?;
                Self::GoodyReceived {
                    player: PlayerId(payload.player),
                    plot: payload.plot(),
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    goody: payload.goody,
                }
            }
            "great_person_born" => {
                let payload: GreatPersonBornPayload = decode(args)?;
                Self::GreatPersonBorn {
                    player: PlayerId(payload.player),
                    city: payload.city(),
                    unit: payload.unit(),
                    unit_type: payload.unit_type,
                    plot: payload.plot(),
                }
            }
            "building_built" => {
                let payload: BuildingBuiltPayload = decode(args)?;
                Self::BuildingBuilt {
                    city: payload.city(),
                    building: payload.building,
                }
            }
            "project_built" => {
                let payload: CityProjectPayload = decode(args)?;
                Self::ProjectBuilt {
                    city: payload.city(),
                    project: payload.project,
                }
            }
            "tech_acquired" => {
                let payload: TechAcquiredPayload = decode(args)?;
                Self::TechAcquired {
                    team: TeamId(payload.team),
                    player: PlayerId(payload.player),
                    tech: payload.tech,
                    announce: payload.announce,
                }
            }
            "tech_selected" => {
                let payload: TechSelectedPayload = decode(args)?;
                Self::TechSelected {
                    player: PlayerId(payload.player),
                    tech: payload.tech,
                }
            }
            "religion_founded" => {
                let payload: ReligionFoundedPayload = decode(args)?;
                Self::ReligionFounded {
                    player: PlayerId(payload.player),
                    religion: payload.religion,
                }
            }
            "religion_spread" => {
                let payload: CityReligionPayload = decode(args)?;
                Self::ReligionSpread {
                    city: payload.city(),
                    religion: payload.religion,
                }
            }
            "religion_remove" => {
                let payload: CityReligionPayload = decode(args)?;
                Self::ReligionRemove {
                    city: payload.city(),
                    religion: payload.religion,
                }
            }
            "corporation_founded" => {
                let payload: CorporationFoundedPayload = decode(args)?;
                Self::CorporationFounded {
                    player: PlayerId(payload.player),
                    corporation: payload.corporation,
                }
            }
            "corporation_spread" => {
                let payload: CityCorporationPayload = decode(args)?;
                Self::CorporationSpread {
                    city: payload.city(),
                    corporation: payload.corporation,
                }
            }
            "corporation_remove" => {
                let payload: CityCorporationPayload = decode(args)?;
                Self::CorporationRemove {
                    city: payload.city(),
                    corporation: payload.corporation,
                }
            }
            "golden_age" => {
                let payload: PlayerPayload = decode(args)?;
                Self::GoldenAge {
                    player: PlayerId(payload.player),
                }
            }
            "end_golden_age" => {
                let payload: PlayerPayload = decode(args)?;
                Self::EndGoldenAge {
                    player: PlayerId(payload.player),
                }
            }
            "change_war" => {
                let payload: ChangeWarPayload = decode(args)?;
                Self::ChangeWar {
                    war: payload.war,
                    team: TeamId(payload.team),
                    other_team: TeamId(payload.other_team),
                }
            }
            "player_gold_trade" => {
                let payload: PlayerGoldTradePayload = decode(args)?;
                Self::PlayerGoldTrade {
                    from_player: PlayerId(payload.from_player),
                    to_player: PlayerId(payload.to_player),
                    amount: payload.amount,
                }
            }
            "set_player_alive" => {
                let payload: PlayerAlivePayload = decode(args)?;
                Self::SetPlayerAlive {
                    player: PlayerId(payload.player),
                    alive: payload.alive,
                }
            }
            "player_change_state_religion" => {
                let payload: PlayerStateReligionPayload = decode(args)?;
                Self::PlayerChangeStateReligion {
                    player: PlayerId(payload.player),
                    new_religion: payload.new_religion,
                    old_religion: payload.old_religion,
                }
            }
            "victory" => {
                let payload: VictoryPayload = decode(args)?;
                Self::Victory {
                    team: TeamId(payload.team),
                    victory: payload.victory,
                }
            }
            "vassal_state" => {
                let payload: VassalStatePayload = decode(args)?;
                Self::VassalState {
                    master: TeamId(payload.master),
                    vassal: TeamId(payload.vassal),
                    is_vassal: payload.is_vassal,
                }
            }
            _ => Self::Unknown { name, args },
        })
    }
}
