use crate::client::{BridgeClient, Result};
use crate::commands::CityOrder;
use crate::state::{
    CityBuildingClassChange, CityBuildingState, CityCorporationState, CityReligionState, CityState,
    PlayerCitiesResult,
};
use crate::types::{CityRef, InfoType, PlayerId};
use serde_json::{json, Value};

impl BridgeClient {
    pub fn get_city_state(&mut self, city: CityRef) -> Result<CityState> {
        self.query(
            "get_city_state",
            json!({ "player": city.player, "city": city.id }),
        )
    }

    pub fn list_player_cities<P: Into<PlayerId>>(&mut self, player: P) -> Result<Vec<CityState>> {
        let player = player.into();
        let result: PlayerCitiesResult =
            self.query("list_player_cities", json!({ "player": player.0 }))?;
        Ok(result.cities)
    }

    pub fn list_all_cities(&mut self) -> Result<Vec<CityState>> {
        let mut cities = Vec::new();
        for player in self.list_alive_players()? {
            cities.extend(self.list_player_cities(player.player_id())?);
        }
        Ok(cities)
    }

    pub fn set_city_population(&mut self, city: CityRef, value: i32) -> Result<CityState> {
        self.command(
            "set_city_population",
            json!({ "player": city.player, "city": city.id, "value": value }),
        )
    }

    pub fn change_city_population(&mut self, city: CityRef, change: i32) -> Result<CityState> {
        self.command(
            "change_city_population",
            json!({ "player": city.player, "city": city.id, "change": change }),
        )
    }

    pub fn set_city_culture<P: Into<PlayerId>>(
        &mut self,
        city: CityRef,
        culture_player: P,
        value: i32,
    ) -> Result<CityState> {
        let culture_player = culture_player.into();
        self.command(
            "set_city_culture",
            json!({
                "player": city.player,
                "city": city.id,
                "culture_player": culture_player.0,
                "value": value
            }),
        )
    }

    pub fn set_owner_city_culture(&mut self, city: CityRef, value: i32) -> Result<CityState> {
        self.command(
            "set_city_culture",
            json!({ "player": city.player, "city": city.id, "value": value }),
        )
    }

    pub fn set_city_production(&mut self, city: CityRef, value: i32) -> Result<CityState> {
        self.command(
            "set_city_production",
            json!({ "player": city.player, "city": city.id, "value": value }),
        )
    }

    pub fn change_city_production(&mut self, city: CityRef, change: i32) -> Result<CityState> {
        self.command(
            "change_city_production",
            json!({ "player": city.player, "city": city.id, "change": change }),
        )
    }

    pub fn set_city_unit_production<U>(
        &mut self,
        city: CityRef,
        unit_type: U,
        value: i32,
    ) -> Result<CityState>
    where
        U: Into<InfoType>,
    {
        self.command(
            "set_city_unit_production",
            json!({
                "player": city.player,
                "city": city.id,
                "unit_type": unit_type.into(),
                "value": value
            }),
        )
    }

    pub fn set_city_building_production<B>(
        &mut self,
        city: CityRef,
        building_type: B,
        value: i32,
    ) -> Result<CityState>
    where
        B: Into<InfoType>,
    {
        self.command(
            "set_city_building_production",
            json!({
                "player": city.player,
                "city": city.id,
                "building_type": building_type.into(),
                "value": value
            }),
        )
    }

    pub fn set_city_project_production<P>(
        &mut self,
        city: CityRef,
        project_type: P,
        value: i32,
    ) -> Result<CityState>
    where
        P: Into<InfoType>,
    {
        self.command(
            "set_city_project_production",
            json!({
                "player": city.player,
                "city": city.id,
                "project_type": project_type.into(),
                "value": value
            }),
        )
    }

    pub fn push_city_order(&mut self, city: CityRef, order: CityOrder) -> Result<CityState> {
        self.command("push_city_order", order.into_args(city))
    }

    pub fn clear_city_order_queue(&mut self, city: CityRef) -> Result<CityState> {
        self.command(
            "clear_city_order_queue",
            json!({ "player": city.player, "city": city.id }),
        )
    }

    pub fn pop_city_order(&mut self, city: CityRef, index: i32) -> Result<CityState> {
        self.command(
            "pop_city_order",
            json!({ "player": city.player, "city": city.id, "index": index }),
        )
    }

    pub fn get_city_building_state<B>(
        &mut self,
        city: CityRef,
        building: B,
    ) -> Result<CityBuildingState>
    where
        B: Into<InfoType>,
    {
        self.query(
            "get_city_building_state",
            json!({ "player": city.player, "city": city.id, "building": building.into() }),
        )
    }

    pub fn get_city_religion_state<R>(
        &mut self,
        city: CityRef,
        religion: R,
    ) -> Result<CityReligionState>
    where
        R: Into<InfoType>,
    {
        self.query(
            "get_city_religion_state",
            json!({ "player": city.player, "city": city.id, "religion": religion.into() }),
        )
    }

    pub fn get_city_corporation_state<C>(
        &mut self,
        city: CityRef,
        corporation: C,
    ) -> Result<CityCorporationState>
    where
        C: Into<InfoType>,
    {
        self.query(
            "get_city_corporation_state",
            json!({ "player": city.player, "city": city.id, "corporation": corporation.into() }),
        )
    }

    pub fn get_city_building_class_change<B>(
        &mut self,
        city: CityRef,
        building_class: B,
    ) -> Result<CityBuildingClassChange>
    where
        B: Into<InfoType>,
    {
        self.query(
            "get_city_building_class_change",
            json!({ "player": city.player, "city": city.id, "building_class": building_class.into() }),
        )
    }

    pub fn set_city_occupation_timer(&mut self, city: CityRef, value: i32) -> Result<CityState> {
        self.command(
            "set_city_occupation_timer",
            json!({ "player": city.player, "city": city.id, "value": value }),
        )
    }

    pub fn change_city_occupation_timer(
        &mut self,
        city: CityRef,
        change: i32,
    ) -> Result<CityState> {
        self.command(
            "change_city_occupation_timer",
            json!({ "player": city.player, "city": city.id, "change": change }),
        )
    }

    pub fn change_city_hurry_anger_timer(
        &mut self,
        city: CityRef,
        change: i32,
    ) -> Result<CityState> {
        self.command(
            "change_city_hurry_anger_timer",
            json!({ "player": city.player, "city": city.id, "change": change }),
        )
    }

    pub fn set_city_real_building<B>(
        &mut self,
        city: CityRef,
        building: B,
        value: i32,
    ) -> Result<CityBuildingState>
    where
        B: Into<InfoType>,
    {
        self.command(
            "set_city_real_building",
            json!({
                "player": city.player,
                "city": city.id,
                "building": building.into(),
                "value": value
            }),
        )
    }

    pub fn set_city_free_building<B>(
        &mut self,
        city: CityRef,
        building: B,
        value: i32,
    ) -> Result<CityBuildingState>
    where
        B: Into<InfoType>,
    {
        self.command(
            "set_city_free_building",
            json!({
                "player": city.player,
                "city": city.id,
                "building": building.into(),
                "value": value
            }),
        )
    }

    pub fn set_city_religion<R>(
        &mut self,
        city: CityRef,
        religion: R,
        has: bool,
        announce: bool,
    ) -> Result<CityReligionState>
    where
        R: Into<InfoType>,
    {
        self.command(
            "set_city_religion",
            city_flag_args(city, "religion", religion.into(), has, announce),
        )
    }

    pub fn add_city_religion<R>(&mut self, city: CityRef, religion: R) -> Result<CityReligionState>
    where
        R: Into<InfoType>,
    {
        self.set_city_religion(city, religion, true, false)
    }

    pub fn remove_city_religion<R>(
        &mut self,
        city: CityRef,
        religion: R,
    ) -> Result<CityReligionState>
    where
        R: Into<InfoType>,
    {
        self.set_city_religion(city, religion, false, false)
    }

    pub fn set_city_corporation<C>(
        &mut self,
        city: CityRef,
        corporation: C,
        has: bool,
        announce: bool,
    ) -> Result<CityCorporationState>
    where
        C: Into<InfoType>,
    {
        self.command(
            "set_city_corporation",
            city_flag_args(city, "corporation", corporation.into(), has, announce),
        )
    }

    pub fn add_city_corporation<C>(
        &mut self,
        city: CityRef,
        corporation: C,
    ) -> Result<CityCorporationState>
    where
        C: Into<InfoType>,
    {
        self.set_city_corporation(city, corporation, true, false)
    }

    pub fn remove_city_corporation<C>(
        &mut self,
        city: CityRef,
        corporation: C,
    ) -> Result<CityCorporationState>
    where
        C: Into<InfoType>,
    {
        self.set_city_corporation(city, corporation, false, false)
    }

    pub fn set_city_building_happiness_change<B>(
        &mut self,
        city: CityRef,
        building_class: B,
        value: i32,
    ) -> Result<CityBuildingClassChange>
    where
        B: Into<InfoType>,
    {
        self.command(
            "set_city_building_happiness_change",
            json!({
                "player": city.player,
                "city": city.id,
                "building_class": building_class.into(),
                "value": value
            }),
        )
    }

    pub fn set_city_building_health_change<B>(
        &mut self,
        city: CityRef,
        building_class: B,
        value: i32,
    ) -> Result<CityBuildingClassChange>
    where
        B: Into<InfoType>,
    {
        self.command(
            "set_city_building_health_change",
            json!({
                "player": city.player,
                "city": city.id,
                "building_class": building_class.into(),
                "value": value
            }),
        )
    }
}

fn city_flag_args(city: CityRef, key: &str, info: InfoType, has: bool, announce: bool) -> Value {
    let mut args = json!({
        "player": city.player,
        "city": city.id,
        "has": if has { 1 } else { 0 },
        "announce": if announce { 1 } else { 0 },
    });
    args[key] = json!(info);
    args
}
