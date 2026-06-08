use crate::types::{CityRef, Plot, UnitRef};
use serde::de::{self, DeserializeOwned, Deserializer, Visitor};
use serde::Deserialize;
use serde_json::Value;
use std::fmt;

pub(crate) fn decode<T: DeserializeOwned>(args: Value) -> serde_json::Result<T> {
    serde_json::from_value(args)
}

pub(crate) fn plot_from_xy(x: i32, y: i32) -> Option<Plot> {
    (x >= 0 && y >= 0).then_some(Plot::new(x, y))
}

#[derive(Deserialize)]
pub(crate) struct TurnPayload {
    pub(crate) turn: i32,
}

#[derive(Deserialize)]
pub(crate) struct PlayerTurnPayload {
    pub(crate) turn: i32,
    pub(crate) player: i32,
}

#[derive(Deserialize)]
pub(crate) struct PlayerResearchPayload {
    pub(crate) player: i32,
    pub(crate) tech: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) trade: bool,
}

#[derive(Deserialize)]
pub(crate) struct PlayerCivicPayload {
    pub(crate) player: i32,
    pub(crate) civic: i32,
}

#[derive(Deserialize)]
pub(crate) struct KbdEventPayload {
    pub(crate) evt: i32,
    pub(crate) key: i32,
    pub(crate) cursor_x: i32,
    pub(crate) cursor_y: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl KbdEventPayload {
    pub(crate) fn plot(&self) -> Option<Plot> {
        plot_from_xy(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct MouseEventPayload {
    pub(crate) evt: i32,
    pub(crate) cursor_x: i32,
    pub(crate) cursor_y: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) interface_consumed: bool,
}

impl MouseEventPayload {
    pub(crate) fn plot(&self) -> Option<Plot> {
        plot_from_xy(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct TeamPairPayload {
    pub(crate) team: i32,
    pub(crate) other_team: i32,
}

#[derive(Deserialize)]
pub(crate) struct PlotPayload {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl PlotPayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct PlotTeamPayload {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) team: i32,
}

impl PlotTeamPayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct PlotPlayerPayload {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) player: i32,
}

#[derive(Deserialize)]
pub(crate) struct PlayerPlotRulePayload {
    pub(crate) player: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) test_visible: bool,
}

impl PlayerPlotRulePayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct PlotBuildPayload {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) build: i32,
    pub(crate) player: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) test_visible: bool,
}

impl PlotBuildPayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitOptionalPayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitOptionalPayload {
    pub(crate) fn unit(&self) -> Option<UnitRef> {
        (self.player >= 0 && self.unit >= 0).then_some(UnitRef {
            player: self.player,
            id: self.unit,
        })
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct CombatResultPayload {
    pub(crate) winner_player: i32,
    pub(crate) winner_unit: i32,
    pub(crate) winner_unit_type: i32,
    pub(crate) winner_x: i32,
    pub(crate) winner_y: i32,
    pub(crate) loser_player: i32,
    pub(crate) loser_unit: i32,
    pub(crate) loser_unit_type: i32,
    pub(crate) loser_x: i32,
    pub(crate) loser_y: i32,
}

impl CombatResultPayload {
    pub(crate) fn winner(&self) -> UnitRef {
        UnitRef {
            player: self.winner_player,
            id: self.winner_unit,
        }
    }

    pub(crate) fn loser(&self) -> UnitRef {
        UnitRef {
            player: self.loser_player,
            id: self.loser_unit,
        }
    }

    pub(crate) fn winner_plot(&self) -> Plot {
        Plot::new(self.winner_x, self.winner_y)
    }

    pub(crate) fn loser_plot(&self) -> Plot {
        Plot::new(self.loser_x, self.loser_y)
    }
}

impl PlotPlayerPayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct ImprovementBuiltPayload {
    pub(crate) improvement: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl ImprovementBuiltPayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct ImprovementDestroyedPayload {
    pub(crate) improvement: i32,
    pub(crate) player: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl ImprovementDestroyedPayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct RouteBuiltPayload {
    pub(crate) route: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl RouteBuiltPayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct PlotFeatureRemovedPayload {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) feature: i32,
    pub(crate) city_player: i32,
    pub(crate) city: i32,
}

impl PlotFeatureRemovedPayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }

    pub(crate) fn city(&self) -> Option<CityRef> {
        (self.city_player >= 0 && self.city >= 0).then_some(CityRef {
            player: self.city_player,
            id: self.city,
        })
    }
}

#[derive(Deserialize)]
pub(crate) struct CityPlotPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl CityPlotPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct CityRazedPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) razed_by: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl CityRazedPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct CityAcquiredPayload {
    pub(crate) old_player: i32,
    pub(crate) player: i32,
    pub(crate) city: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) conquest: bool,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) trade: bool,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl CityAcquiredPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct CityGrowthPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) population: i32,
}

impl CityGrowthPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct CityUnitTypePayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) unit_type: i32,
}

impl CityUnitTypePayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct CityBuildingPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) building: i32,
}

impl CityBuildingPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct CityBuildingPlotPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) building: i32,
}

impl CityBuildingPlotPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct CityHurryPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) hurry: i32,
}

impl CityHurryPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct CityProductionRulePayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    #[serde(default)]
    pub(crate) unit: Option<i32>,
    #[serde(default)]
    pub(crate) building: Option<i32>,
    #[serde(default)]
    pub(crate) project: Option<i32>,
    #[serde(default)]
    pub(crate) process: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_int_bool")]
    pub(crate) continue_current: bool,
    #[serde(default, deserialize_with = "deserialize_int_bool")]
    pub(crate) test_visible: bool,
    #[serde(default, deserialize_with = "deserialize_int_bool")]
    pub(crate) ignore_cost: bool,
    #[serde(default, deserialize_with = "deserialize_int_bool")]
    pub(crate) ignore_upgrades: bool,
}

impl CityProductionRulePayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct CityProjectPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) project: i32,
}

impl CityProjectPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct SelectionGroupMissionPayload {
    pub(crate) player: i32,
    pub(crate) group: i32,
    pub(crate) mission: i32,
}

#[derive(Deserialize)]
pub(crate) struct UnitMovePayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) from_x: i32,
    pub(crate) from_y: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitMovePayload {
    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitPlotPayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitPlotPayload {
    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitMoveIntoPayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) attack: bool,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) declare_war: bool,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) ignore_load: bool,
}

impl UnitMoveIntoPayload {
    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitBuiltPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitBuiltPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitKilledPayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) attacker: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitKilledPayload {
    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitPromotionPayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) promotion: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitPromotionPayload {
    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitPillagePayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) improvement: i32,
    pub(crate) route: i32,
    pub(crate) pillage_player: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitPillagePayload {
    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitReligionAttemptPayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) religion: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) success: bool,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitReligionAttemptPayload {
    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitGiftedPayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) gifting_player: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitGiftedPayload {
    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnitBuildImprovementPayload {
    pub(crate) player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) build: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) finished: bool,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl UnitBuildImprovementPayload {
    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct GoodyReceivedPayload {
    pub(crate) player: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) unit_player: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) goody: i32,
}

impl GoodyReceivedPayload {
    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }

    pub(crate) fn unit(&self) -> Option<UnitRef> {
        (self.unit_player >= 0 && self.unit >= 0).then_some(UnitRef {
            player: self.unit_player,
            id: self.unit,
        })
    }
}

#[derive(Deserialize)]
pub(crate) struct GreatPersonBornPayload {
    pub(crate) player: i32,
    pub(crate) city_player: i32,
    pub(crate) city: i32,
    pub(crate) unit: i32,
    pub(crate) unit_type: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl GreatPersonBornPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.city_player,
            id: self.city,
        }
    }

    pub(crate) fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    pub(crate) fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub(crate) struct BuildingBuiltPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) building: i32,
}

impl BuildingBuiltPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct TechAcquiredPayload {
    pub(crate) team: i32,
    pub(crate) player: i32,
    pub(crate) tech: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) announce: bool,
}

#[derive(Deserialize)]
pub(crate) struct TechSelectedPayload {
    pub(crate) player: i32,
    pub(crate) tech: i32,
}

#[derive(Deserialize)]
pub(crate) struct ReligionFoundedPayload {
    pub(crate) player: i32,
    pub(crate) religion: i32,
}

#[derive(Deserialize)]
pub(crate) struct CityReligionPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) religion: i32,
}

impl CityReligionPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct CorporationFoundedPayload {
    pub(crate) player: i32,
    pub(crate) corporation: i32,
}

#[derive(Deserialize)]
pub(crate) struct CityCorporationPayload {
    pub(crate) player: i32,
    pub(crate) city: i32,
    pub(crate) corporation: i32,
}

impl CityCorporationPayload {
    pub(crate) fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct PlayerPayload {
    pub(crate) player: i32,
}

#[derive(Deserialize)]
pub(crate) struct PlayerAlivePayload {
    pub(crate) player: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) alive: bool,
}

#[derive(Deserialize)]
pub(crate) struct PlayerStateReligionPayload {
    pub(crate) player: i32,
    pub(crate) new_religion: i32,
    pub(crate) old_religion: i32,
}

#[derive(Deserialize)]
pub(crate) struct ChangeWarPayload {
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) war: bool,
    pub(crate) team: i32,
    pub(crate) other_team: i32,
}

#[derive(Deserialize)]
pub(crate) struct VassalStatePayload {
    pub(crate) master: i32,
    pub(crate) vassal: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) is_vassal: bool,
}

#[derive(Deserialize)]
pub(crate) struct PlayerGoldTradePayload {
    pub(crate) from_player: i32,
    pub(crate) to_player: i32,
    pub(crate) amount: i32,
}

#[derive(Deserialize)]
pub(crate) struct VictoryPayload {
    pub(crate) team: i32,
    pub(crate) victory: i32,
}

pub(crate) fn deserialize_int_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct IntBoolVisitor;

    impl<'de> Visitor<'de> for IntBoolVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a boolean or integer boolean")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value != 0)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value != 0)
        }
    }

    deserializer.deserialize_any(IntBoolVisitor)
}
