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
pub struct SelectionGroupRef {
    pub player: i32,
    pub id: i32,
}

impl SelectionGroupRef {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CityProductionRule {
    CanTrain,
    CannotTrain,
    CanConstruct,
    CannotConstruct,
    CanCreate,
    CannotCreate,
    CanMaintain,
    CannotMaintain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CityProductionItem {
    Unit(i32),
    Building(i32),
    Project(i32),
    Process(i32),
}

impl CityProductionItem {
    pub fn id(self) -> i32 {
        match self {
            Self::Unit(id) | Self::Building(id) | Self::Project(id) | Self::Process(id) => id,
        }
    }
}

impl CityProductionRule {
    pub fn name(self) -> &'static str {
        match self {
            Self::CanTrain => "can_train",
            Self::CannotTrain => "cannot_train",
            Self::CanConstruct => "can_construct",
            Self::CannotConstruct => "cannot_construct",
            Self::CanCreate => "can_create",
            Self::CannotCreate => "cannot_create",
            Self::CanMaintain => "can_maintain",
            Self::CannotMaintain => "cannot_maintain",
        }
    }

    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "can_train" => Some(Self::CanTrain),
            "cannot_train" => Some(Self::CannotTrain),
            "can_construct" => Some(Self::CanConstruct),
            "cannot_construct" => Some(Self::CannotConstruct),
            "can_create" => Some(Self::CanCreate),
            "cannot_create" => Some(Self::CannotCreate),
            "can_maintain" => Some(Self::CanMaintain),
            "cannot_maintain" => Some(Self::CannotMaintain),
            _ => None,
        }
    }

    pub fn item(self, id: i32) -> CityProductionItem {
        match self {
            Self::CanTrain | Self::CannotTrain => CityProductionItem::Unit(id),
            Self::CanConstruct | Self::CannotConstruct => CityProductionItem::Building(id),
            Self::CanCreate | Self::CannotCreate => CityProductionItem::Project(id),
            Self::CanMaintain | Self::CannotMaintain => CityProductionItem::Process(id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommerceType {
    Gold,
    Research,
    Culture,
    Espionage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfoKind {
    Unit,
    UnitAi,
    Building,
    BuildingClass,
    Project,
    Process,
    Terrain,
    Feature,
    Bonus,
    Improvement,
    Route,
    Promotion,
    Tech,
    Civic,
    CivicOption,
    Religion,
    Corporation,
    Victory,
    GameOption,
    MultiplayerOption,
    ForceControl,
    Era,
    Leader,
    Civilization,
    Handicap,
    GameSpeed,
    Hurry,
    Build,
    Goody,
    Mission,
    EspionageMission,
    Specialist,
    UnitClass,
    UnitCombat,
    PlayerOption,
    Commerce,
    Yield,
}
