pub(crate) use crate::unit_event_payloads::*;

use crate::types::{CityRef, Plot};
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
pub(crate) struct ModNetMessagePayload {
    pub(crate) data1: i32,
    pub(crate) data2: i32,
    pub(crate) data3: i32,
    pub(crate) data4: i32,
    pub(crate) data5: i32,
}

#[derive(Deserialize)]
pub(crate) struct UpdatePayload {
    pub(crate) delta_time: f64,
}

#[derive(Deserialize)]
pub(crate) struct WindowActivationPayload {
    #[serde(deserialize_with = "deserialize_int_bool")]
    pub(crate) active: bool,
}

#[derive(Deserialize)]
pub(crate) struct ChatPayload {
    pub(crate) text: String,
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
