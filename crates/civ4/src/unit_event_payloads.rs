use crate::event_payloads::deserialize_int_bool;
use crate::types::{CityRef, Plot, UnitRef};
use serde::Deserialize;

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
