use crate::events::BridgeEvent;
use crate::types::{CityProductionItem, CityProductionRule};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BridgeEventKind {
    Init,
    Uninit,
    KbdEvent,
    MouseEvent,
    GameStart,
    GameEnd,
    PreSave,
    BeginGameTurn,
    EndGameTurn,
    BeginPlayerTurn,
    EndPlayerTurn,
    IsPlayerResearch,
    CanResearch,
    CannotResearch,
    CanDoCivic,
    CannotDoCivic,
    FirstContact,
    CombatResult,
    ImprovementBuilt,
    ImprovementDestroyed,
    RouteBuilt,
    PlotRevealed,
    PlotFeatureRemoved,
    PlotPicked,
    CanBuild,
    NukeExplosion,
    GotoPlotSet,
    CityBuilt,
    CityRazed,
    CityAcquired,
    CityAcquiredKept,
    CityLost,
    CityGrowth,
    CultureExpansion,
    CityDoTurn,
    CityBuildingUnit,
    CityBuildingBuilding,
    BuildingCostMod,
    CityRename,
    CityHurry,
    CityProductionRule(CityProductionRule),
    SelectionGroupPushMission,
    UnitMove,
    UnitCannotMoveInto,
    UnitSetXY,
    UnitCreated,
    UnitBuilt,
    UnitKilled,
    UnitLost,
    UnitPromoted,
    UnitSelected,
    UnitRename,
    UnitPillage,
    UnitSpreadReligionAttempt,
    UnitGifted,
    UnitBuildImprovement,
    GoodyReceived,
    GreatPersonBorn,
    BuildingBuilt,
    ProjectBuilt,
    TechAcquired,
    TechSelected,
    ReligionFounded,
    ReligionSpread,
    ReligionRemove,
    CorporationFounded,
    CorporationSpread,
    CorporationRemove,
    GoldenAge,
    EndGoldenAge,
    ChangeWar,
    PlayerGoldTrade,
    SetPlayerAlive,
    PlayerChangeStateReligion,
    Victory,
    VassalState,
}

impl BridgeEventKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Uninit => "uninit",
            Self::KbdEvent => "kbd_event",
            Self::MouseEvent => "mouse_event",
            Self::GameStart => "game_start",
            Self::GameEnd => "game_end",
            Self::PreSave => "pre_save",
            Self::BeginGameTurn => "begin_game_turn",
            Self::EndGameTurn => "end_game_turn",
            Self::BeginPlayerTurn => "begin_player_turn",
            Self::EndPlayerTurn => "end_player_turn",
            Self::IsPlayerResearch => "is_player_research",
            Self::CanResearch => "can_research",
            Self::CannotResearch => "cannot_research",
            Self::CanDoCivic => "can_do_civic",
            Self::CannotDoCivic => "cannot_do_civic",
            Self::FirstContact => "first_contact",
            Self::CombatResult => "combat_result",
            Self::ImprovementBuilt => "improvement_built",
            Self::ImprovementDestroyed => "improvement_destroyed",
            Self::RouteBuilt => "route_built",
            Self::PlotRevealed => "plot_revealed",
            Self::PlotFeatureRemoved => "plot_feature_removed",
            Self::PlotPicked => "plot_picked",
            Self::CanBuild => "can_build",
            Self::NukeExplosion => "nuke_explosion",
            Self::GotoPlotSet => "goto_plot_set",
            Self::CityBuilt => "city_built",
            Self::CityRazed => "city_razed",
            Self::CityAcquired => "city_acquired",
            Self::CityAcquiredKept => "city_acquired_kept",
            Self::CityLost => "city_lost",
            Self::CityGrowth => "city_growth",
            Self::CultureExpansion => "culture_expansion",
            Self::CityDoTurn => "city_do_turn",
            Self::CityBuildingUnit => "city_building_unit",
            Self::CityBuildingBuilding => "city_building_building",
            Self::BuildingCostMod => "get_building_cost_mod",
            Self::CityRename => "city_rename",
            Self::CityHurry => "city_hurry",
            Self::CityProductionRule(rule) => rule.name(),
            Self::SelectionGroupPushMission => "selection_group_push_mission",
            Self::UnitMove => "unit_move",
            Self::UnitCannotMoveInto => "unit_cannot_move_into",
            Self::UnitSetXY => "unit_set_xy",
            Self::UnitCreated => "unit_created",
            Self::UnitBuilt => "unit_built",
            Self::UnitKilled => "unit_killed",
            Self::UnitLost => "unit_lost",
            Self::UnitPromoted => "unit_promoted",
            Self::UnitSelected => "unit_selected",
            Self::UnitRename => "unit_rename",
            Self::UnitPillage => "unit_pillage",
            Self::UnitSpreadReligionAttempt => "unit_spread_religion_attempt",
            Self::UnitGifted => "unit_gifted",
            Self::UnitBuildImprovement => "unit_build_improvement",
            Self::GoodyReceived => "goody_received",
            Self::GreatPersonBorn => "great_person_born",
            Self::BuildingBuilt => "building_built",
            Self::ProjectBuilt => "project_built",
            Self::TechAcquired => "tech_acquired",
            Self::TechSelected => "tech_selected",
            Self::ReligionFounded => "religion_founded",
            Self::ReligionSpread => "religion_spread",
            Self::ReligionRemove => "religion_remove",
            Self::CorporationFounded => "corporation_founded",
            Self::CorporationSpread => "corporation_spread",
            Self::CorporationRemove => "corporation_remove",
            Self::GoldenAge => "golden_age",
            Self::EndGoldenAge => "end_golden_age",
            Self::ChangeWar => "change_war",
            Self::PlayerGoldTrade => "player_gold_trade",
            Self::SetPlayerAlive => "set_player_alive",
            Self::PlayerChangeStateReligion => "player_change_state_religion",
            Self::Victory => "victory",
            Self::VassalState => "vassal_state",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "init" => Self::Init,
            "uninit" => Self::Uninit,
            "kbd_event" => Self::KbdEvent,
            "mouse_event" => Self::MouseEvent,
            "game_start" => Self::GameStart,
            "game_end" => Self::GameEnd,
            "pre_save" => Self::PreSave,
            "begin_game_turn" => Self::BeginGameTurn,
            "end_game_turn" => Self::EndGameTurn,
            "begin_player_turn" => Self::BeginPlayerTurn,
            "end_player_turn" => Self::EndPlayerTurn,
            "is_player_research" => Self::IsPlayerResearch,
            "can_research" => Self::CanResearch,
            "cannot_research" => Self::CannotResearch,
            "can_do_civic" => Self::CanDoCivic,
            "cannot_do_civic" => Self::CannotDoCivic,
            "first_contact" => Self::FirstContact,
            "combat_result" => Self::CombatResult,
            "improvement_built" => Self::ImprovementBuilt,
            "improvement_destroyed" => Self::ImprovementDestroyed,
            "route_built" => Self::RouteBuilt,
            "plot_revealed" => Self::PlotRevealed,
            "plot_feature_removed" => Self::PlotFeatureRemoved,
            "plot_picked" => Self::PlotPicked,
            "can_build" => Self::CanBuild,
            "nuke_explosion" => Self::NukeExplosion,
            "goto_plot_set" => Self::GotoPlotSet,
            "city_built" => Self::CityBuilt,
            "city_razed" => Self::CityRazed,
            "city_acquired" => Self::CityAcquired,
            "city_acquired_kept" => Self::CityAcquiredKept,
            "city_lost" => Self::CityLost,
            "city_growth" => Self::CityGrowth,
            "culture_expansion" => Self::CultureExpansion,
            "city_do_turn" => Self::CityDoTurn,
            "city_building_unit" => Self::CityBuildingUnit,
            "city_building_building" => Self::CityBuildingBuilding,
            "get_building_cost_mod" => Self::BuildingCostMod,
            "city_rename" => Self::CityRename,
            "city_hurry" => Self::CityHurry,
            "selection_group_push_mission" => Self::SelectionGroupPushMission,
            "unit_move" => Self::UnitMove,
            "unit_cannot_move_into" => Self::UnitCannotMoveInto,
            "unit_set_xy" => Self::UnitSetXY,
            "unit_created" => Self::UnitCreated,
            "unit_built" => Self::UnitBuilt,
            "unit_killed" => Self::UnitKilled,
            "unit_lost" => Self::UnitLost,
            "unit_promoted" => Self::UnitPromoted,
            "unit_selected" => Self::UnitSelected,
            "unit_rename" => Self::UnitRename,
            "unit_pillage" => Self::UnitPillage,
            "unit_spread_religion_attempt" => Self::UnitSpreadReligionAttempt,
            "unit_gifted" => Self::UnitGifted,
            "unit_build_improvement" => Self::UnitBuildImprovement,
            "goody_received" => Self::GoodyReceived,
            "great_person_born" => Self::GreatPersonBorn,
            "building_built" => Self::BuildingBuilt,
            "project_built" => Self::ProjectBuilt,
            "tech_acquired" => Self::TechAcquired,
            "tech_selected" => Self::TechSelected,
            "religion_founded" => Self::ReligionFounded,
            "religion_spread" => Self::ReligionSpread,
            "religion_remove" => Self::ReligionRemove,
            "corporation_founded" => Self::CorporationFounded,
            "corporation_spread" => Self::CorporationSpread,
            "corporation_remove" => Self::CorporationRemove,
            "golden_age" => Self::GoldenAge,
            "end_golden_age" => Self::EndGoldenAge,
            "change_war" => Self::ChangeWar,
            "player_gold_trade" => Self::PlayerGoldTrade,
            "set_player_alive" => Self::SetPlayerAlive,
            "player_change_state_religion" => Self::PlayerChangeStateReligion,
            "victory" => Self::Victory,
            "vassal_state" => Self::VassalState,
            other => Self::CityProductionRule(CityProductionRule::from_name(other)?),
        })
    }
}

impl BridgeEvent {
    pub fn kind(&self) -> Option<BridgeEventKind> {
        BridgeEventKind::from_name(self.name())
    }

    pub fn city_production_item(&self) -> Option<CityProductionItem> {
        match self {
            Self::CityProductionRule { rule, item, .. } => Some(rule.item(*item)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PlayerId;

    #[test]
    fn maps_known_event_kind_names() {
        assert_eq!(BridgeEventKind::BeginPlayerTurn.name(), "begin_player_turn");
        assert_eq!(
            BridgeEventKind::BuildingCostMod.name(),
            "get_building_cost_mod"
        );
        assert_eq!(
            BridgeEventKind::from_name("cannot_train"),
            Some(BridgeEventKind::CityProductionRule(
                CityProductionRule::CannotTrain
            ))
        );
        assert_eq!(
            BridgeEventKind::from_name("get_building_cost_mod"),
            Some(BridgeEventKind::BuildingCostMod)
        );
        assert_eq!(
            BridgeEventKind::from_name("unit_cannot_move_into"),
            Some(BridgeEventKind::UnitCannotMoveInto)
        );
        assert_eq!(
            BridgeEventKind::from_name("cannot_research"),
            Some(BridgeEventKind::CannotResearch)
        );
        assert_eq!(
            BridgeEventKind::from_name("can_do_civic"),
            Some(BridgeEventKind::CanDoCivic)
        );
        assert_eq!(
            BridgeEventKind::from_name("can_build"),
            Some(BridgeEventKind::CanBuild)
        );
        assert_eq!(BridgeEventKind::from_name("future_event"), None);
    }

    #[test]
    fn gets_kind_from_bridge_event() {
        let event = BridgeEvent::BeginPlayerTurn {
            turn: 1,
            player: PlayerId(0),
        };

        assert_eq!(event.kind(), Some(BridgeEventKind::BeginPlayerTurn));
        assert_eq!(
            BridgeEvent::Unknown {
                name: "future_event".to_string(),
                args: serde_json::json!({})
            }
            .kind(),
            None
        );
    }

    #[test]
    fn gets_typed_city_production_item_from_event() {
        let event = BridgeEvent::CityProductionRule {
            rule: CityProductionRule::CannotConstruct,
            city: crate::types::CityRef::new(0, 1),
            plot: crate::types::Plot::new(2, 3),
            item: 12,
            continue_current: false,
            test_visible: false,
            ignore_cost: false,
            ignore_upgrades: false,
        };

        assert_eq!(
            event.city_production_item(),
            Some(CityProductionItem::Building(12))
        );
        assert_eq!(event.city_production_item().unwrap().id(), 12);
        assert_eq!(BridgeEvent::GameStart.city_production_item(), None);
    }
}
