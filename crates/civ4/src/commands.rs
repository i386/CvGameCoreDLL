use crate::types::{CityRef, InfoType, PlayerId, Plot, UnitRef};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
pub struct SpawnUnitRequest {
    pub player: PlayerId,
    pub unit_type: InfoType,
    pub plot: Plot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_ai: Option<InfoType>,
}

impl SpawnUnitRequest {
    pub fn new<P, U>(player: P, unit_type: U, plot: Plot) -> Self
    where
        P: Into<PlayerId>,
        U: Into<InfoType>,
    {
        Self {
            player: player.into(),
            unit_type: unit_type.into(),
            plot,
            unit_ai: None,
        }
    }

    pub fn with_unit_ai<U>(mut self, unit_ai: U) -> Self
    where
        U: Into<InfoType>,
    {
        self.unit_ai = Some(unit_ai.into());
        self
    }

    pub(crate) fn into_args(self) -> Value {
        let mut args = json!({
            "player": self.player.0,
            "unit_type": self.unit_type,
            "x": self.plot.x,
            "y": self.plot.y,
        });
        if let Some(unit_ai) = self.unit_ai {
            args["unit_ai"] = json!(unit_ai);
        }
        args
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnedUnit {
    pub unit: UnitRef,
    pub plot: Plot,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct SpawnUnitResult {
    pub player: i32,
    pub unit: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitGroupMission {
    pub mission: InfoType,
    pub data1: i32,
    pub data2: i32,
    pub flags: i32,
    pub append: bool,
    pub manual: bool,
}

impl UnitGroupMission {
    pub fn new<M: Into<InfoType>>(mission: M) -> Self {
        Self {
            mission: mission.into(),
            data1: -1,
            data2: -1,
            flags: 0,
            append: false,
            manual: false,
        }
    }

    pub fn move_to(plot: Plot) -> Self {
        Self::new("MISSION_MOVE_TO").with_data(plot.x, plot.y)
    }

    pub fn build(build: i32) -> Self {
        Self::new("MISSION_BUILD").with_data1(build)
    }

    pub fn with_data(mut self, data1: i32, data2: i32) -> Self {
        self.data1 = data1;
        self.data2 = data2;
        self
    }

    pub fn with_data1(mut self, data1: i32) -> Self {
        self.data1 = data1;
        self
    }

    pub fn flags(mut self, flags: i32) -> Self {
        self.flags = flags;
        self
    }

    pub fn append(mut self, append: bool) -> Self {
        self.append = append;
        self
    }

    pub fn manual(mut self, manual: bool) -> Self {
        self.manual = manual;
        self
    }

    pub(crate) fn into_args(self, unit: UnitRef) -> Value {
        json!({
            "player": unit.player,
            "unit": unit.id,
            "mission": self.mission,
            "data1": self.data1,
            "data2": self.data2,
            "flags": self.flags,
            "append": if self.append { 1 } else { 0 },
            "manual": if self.manual { 1 } else { 0 },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CityOrderType {
    Train,
    Construct,
    Create,
    Maintain,
}

impl CityOrderType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Construct => "construct",
            Self::Create => "create",
            Self::Maintain => "maintain",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityOrder {
    pub order: CityOrderType,
    pub data1: InfoType,
    pub data2: Option<i32>,
    pub save: bool,
    pub pop: bool,
    pub append: bool,
    pub force: bool,
}

impl CityOrder {
    pub fn train<U: Into<InfoType>>(unit_type: U) -> Self {
        Self::new(CityOrderType::Train, unit_type)
    }

    pub fn train_with_ai<U: Into<InfoType>>(unit_type: U, unit_ai: i32) -> Self {
        Self::train(unit_type).with_data2(unit_ai)
    }

    pub fn construct<B: Into<InfoType>>(building_type: B) -> Self {
        Self::new(CityOrderType::Construct, building_type)
    }

    pub fn create<P: Into<InfoType>>(project_type: P) -> Self {
        Self::new(CityOrderType::Create, project_type)
    }

    pub fn maintain<P: Into<InfoType>>(process_type: P) -> Self {
        Self::new(CityOrderType::Maintain, process_type)
    }

    pub fn with_data2(mut self, data2: i32) -> Self {
        self.data2 = Some(data2);
        self
    }

    pub fn saved(mut self, save: bool) -> Self {
        self.save = save;
        self
    }

    pub fn pop_current(mut self, pop: bool) -> Self {
        self.pop = pop;
        self
    }

    pub fn append(mut self, append: bool) -> Self {
        self.append = append;
        self
    }

    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    fn new<T: Into<InfoType>>(order: CityOrderType, data1: T) -> Self {
        Self {
            order,
            data1: data1.into(),
            data2: None,
            save: true,
            pop: false,
            append: false,
            force: false,
        }
    }

    pub(crate) fn into_args(self, city: CityRef) -> Value {
        let mut args = json!({
            "player": city.player,
            "city": city.id,
            "order": self.order.as_str(),
            "data1": self.data1,
            "save": if self.save { 1 } else { 0 },
            "pop": if self.pop { 1 } else { 0 },
            "append": if self.append { 1 } else { 0 },
            "force": if self.force { 1 } else { 0 },
        });
        if let Some(data2) = self.data2 {
            args["data2"] = json!(data2);
        }
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_unit_request_flattens_plot() {
        let request =
            SpawnUnitRequest::new(0, "UNIT_WARRIOR", Plot::new(3, 4)).with_unit_ai("UNITAI_ATTACK");
        let args = request.into_args();

        assert_eq!(
            args,
            json!({
                "player": 0,
                "unit_type": "UNIT_WARRIOR",
                "x": 3,
                "y": 4,
                "unit_ai": "UNITAI_ATTACK"
            })
        );
    }

    #[test]
    fn city_order_serializes_to_bridge_args() {
        let args = CityOrder::train_with_ai("UNIT_WARRIOR", 3)
            .append(true)
            .force(true)
            .into_args(CityRef::new(0, 7));

        assert_eq!(
            args,
            json!({
                "player": 0,
                "city": 7,
                "order": "train",
                "data1": "UNIT_WARRIOR",
                "data2": 3,
                "save": 1,
                "pop": 0,
                "append": 1,
                "force": 1
            })
        );
    }

    #[test]
    fn unit_group_mission_serializes_to_bridge_args() {
        let args = UnitGroupMission::move_to(Plot::new(11, 12))
            .append(true)
            .into_args(UnitRef::new(0, 42));

        assert_eq!(
            args,
            json!({
                "player": 0,
                "unit": 42,
                "mission": "MISSION_MOVE_TO",
                "data1": 11,
                "data2": 12,
                "flags": 0,
                "append": 1,
                "manual": 0
            })
        );
    }
}
