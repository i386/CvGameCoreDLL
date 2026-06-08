use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub i32);

impl From<i32> for PlayerId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeamId(pub i32);

impl From<i32> for TeamId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CityRef {
    pub player: i32,
    pub id: i32,
}

impl CityRef {
    pub fn new<P: Into<PlayerId>>(player: P, id: i32) -> Self {
        Self {
            player: player.into().0,
            id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitRef {
    pub player: i32,
    pub id: i32,
}

impl UnitRef {
    pub fn new<P: Into<PlayerId>>(player: P, id: i32) -> Self {
        Self {
            player: player.into().0,
            id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Plot {
    pub x: i32,
    pub y: i32,
}

impl Plot {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InfoType {
    Id(i32),
    Name(String),
}

impl From<i32> for InfoType {
    fn from(value: i32) -> Self {
        Self::Id(value)
    }
}

impl From<&str> for InfoType {
    fn from(value: &str) -> Self {
        Self::Name(value.to_string())
    }
}

impl From<String> for InfoType {
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarPlan {
    None,
    AttackedRecent,
    Attacked,
    PreparingLimited,
    PreparingTotal,
    Limited,
    Total,
    Dogpile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    On,
    Over,
    Extended,
}
