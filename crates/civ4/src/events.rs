use crate::types::{CityRef, PlayerId, Plot, TeamId, UnitRef};
use serde::de::{self, DeserializeOwned, Deserializer, Visitor};
use serde::Deserialize;
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeEventMessage {
    pub seq: u64,
    pub event: BridgeEvent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BridgeEvent {
    Init,
    Uninit,
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
    UnitMove {
        unit: UnitRef,
        from: Plot,
        to: Plot,
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
    BuildingBuilt {
        city: CityRef,
        building: i32,
    },
    TechAcquired {
        team: TeamId,
        player: PlayerId,
        tech: i32,
        announce: bool,
    },
    ReligionFounded {
        player: PlayerId,
        religion: i32,
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
    Victory {
        team: TeamId,
        victory: i32,
    },
    Unknown {
        name: String,
        args: Value,
    },
}

impl BridgeEvent {
    pub fn from_name_args(name: String, args: Value) -> serde_json::Result<Self> {
        Ok(match name.as_str() {
            "init" => Self::Init,
            "uninit" => Self::Uninit,
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
            "unit_move" => {
                let payload: UnitMovePayload = decode(args)?;
                Self::UnitMove {
                    unit: payload.unit(),
                    from: Plot::new(payload.from_x, payload.from_y),
                    to: Plot::new(payload.x, payload.y),
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
            "building_built" => {
                let payload: BuildingBuiltPayload = decode(args)?;
                Self::BuildingBuilt {
                    city: payload.city(),
                    building: payload.building,
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
            "religion_founded" => {
                let payload: ReligionFoundedPayload = decode(args)?;
                Self::ReligionFounded {
                    player: PlayerId(payload.player),
                    religion: payload.religion,
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
            "victory" => {
                let payload: VictoryPayload = decode(args)?;
                Self::Victory {
                    team: TeamId(payload.team),
                    victory: payload.victory,
                }
            }
            _ => Self::Unknown { name, args },
        })
    }
}

fn decode<T: DeserializeOwned>(args: Value) -> serde_json::Result<T> {
    serde_json::from_value(args)
}

#[derive(Deserialize)]
struct TurnPayload {
    turn: i32,
}

#[derive(Deserialize)]
struct PlayerTurnPayload {
    turn: i32,
    player: i32,
}

#[derive(Deserialize)]
struct CityPlotPayload {
    player: i32,
    city: i32,
    x: i32,
    y: i32,
}

impl CityPlotPayload {
    fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
struct CityRazedPayload {
    player: i32,
    city: i32,
    razed_by: i32,
    x: i32,
    y: i32,
}

impl CityRazedPayload {
    fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
struct CityAcquiredPayload {
    old_player: i32,
    player: i32,
    city: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    conquest: bool,
    #[serde(deserialize_with = "deserialize_int_bool")]
    trade: bool,
    x: i32,
    y: i32,
}

impl CityAcquiredPayload {
    fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
struct CityGrowthPayload {
    player: i32,
    city: i32,
    population: i32,
}

impl CityGrowthPayload {
    fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
struct UnitMovePayload {
    player: i32,
    unit: i32,
    from_x: i32,
    from_y: i32,
    x: i32,
    y: i32,
}

impl UnitMovePayload {
    fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }
}

#[derive(Deserialize)]
struct UnitPlotPayload {
    player: i32,
    unit: i32,
    unit_type: i32,
    x: i32,
    y: i32,
}

impl UnitPlotPayload {
    fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
struct UnitBuiltPayload {
    player: i32,
    city: i32,
    unit: i32,
    unit_type: i32,
    x: i32,
    y: i32,
}

impl UnitBuiltPayload {
    fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }

    fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
struct UnitKilledPayload {
    player: i32,
    unit: i32,
    unit_type: i32,
    attacker: i32,
    x: i32,
    y: i32,
}

impl UnitKilledPayload {
    fn unit(&self) -> UnitRef {
        UnitRef {
            player: self.player,
            id: self.unit,
        }
    }

    fn plot(&self) -> Plot {
        Plot::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
struct BuildingBuiltPayload {
    player: i32,
    city: i32,
    building: i32,
}

impl BuildingBuiltPayload {
    fn city(&self) -> CityRef {
        CityRef {
            player: self.player,
            id: self.city,
        }
    }
}

#[derive(Deserialize)]
struct TechAcquiredPayload {
    team: i32,
    player: i32,
    tech: i32,
    #[serde(deserialize_with = "deserialize_int_bool")]
    announce: bool,
}

#[derive(Deserialize)]
struct ReligionFoundedPayload {
    player: i32,
    religion: i32,
}

#[derive(Deserialize)]
struct PlayerPayload {
    player: i32,
}

#[derive(Deserialize)]
struct ChangeWarPayload {
    #[serde(deserialize_with = "deserialize_int_bool")]
    war: bool,
    team: i32,
    other_team: i32,
}

#[derive(Deserialize)]
struct PlayerGoldTradePayload {
    from_player: i32,
    to_player: i32,
    amount: i32,
}

#[derive(Deserialize)]
struct VictoryPayload {
    team: i32,
    victory: i32,
}

fn deserialize_int_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_known_event_payload() {
        let event = BridgeEvent::from_name_args(
            "city_acquired".to_string(),
            json!({
                "old_player": 1,
                "player": 2,
                "city": 9,
                "conquest": 1,
                "trade": 0,
                "x": 10,
                "y": 11
            }),
        )
        .unwrap();

        assert_eq!(
            event,
            BridgeEvent::CityAcquired {
                old_player: PlayerId(1),
                city: CityRef { player: 2, id: 9 },
                conquest: true,
                trade: false,
                plot: Plot::new(10, 11),
            }
        );
    }

    #[test]
    fn preserves_unknown_event_payload() {
        let event =
            BridgeEvent::from_name_args("future_event".to_string(), json!({ "payload": 1 }))
                .unwrap();

        assert_eq!(
            event,
            BridgeEvent::Unknown {
                name: "future_event".to_string(),
                args: json!({ "payload": 1 }),
            }
        );
    }
}
