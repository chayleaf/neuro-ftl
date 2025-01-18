use std::{borrow::Cow, collections::BTreeMap};

use neuro_ftl_derive::Delta;
use serde::Serialize;

use crate::impl_delta;

pub mod util;

use util::*;

pub use util::Help;

#[derive(Clone, Debug, Delta, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    #[delta1]
    pub ship: ShipId,
    #[delta1]
    pub room_id: i32,
}

#[derive(Clone, Debug, Delta, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WeaponInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[delta1]
    pub weapon_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tooltip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tip: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub cost: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub rarity: i32,
    // #[delta2]
    pub faction: ShipId,
    #[serde(skip_serializing_if = "is_zero")]
    pub damage: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub healing: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub missiles_per_shot: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub shield_piercing: i32,
    // *10
    #[serde(skip_serializing_if = "is_zero")]
    pub fire_chance_percentage: i32,
    // *10
    #[serde(skip_serializing_if = "is_zero")]
    pub breach_chance_percentage: i32,
    // *10
    #[serde(skip_serializing_if = "is_zero")]
    pub stun_chance_percentage: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub stun_duration: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub ion_damage: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub system_damage: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub crew_damage: i32,
    // must be set to damage * 2 if hull_buster != 0
    #[serde(skip_serializing_if = "is_zero")]
    pub hull_damage: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub lockdowns_room: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub can_target_own_ship: bool,

    #[serde(skip_serializing_if = "is_zero")]
    pub projectile_speed: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub cooldown: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub remaining_cooldown: QuantizedI32<3>,
    #[serde(skip_serializing_if = "is_zero")]
    pub required_power: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activated: Option<bool>,
    #[serde(skip_serializing_if = "is_zero")]
    pub hacked: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub stale_info: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targeted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autofiring: Option<bool>,
}

#[derive(Clone, Debug, Delta, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DroneInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[delta1]
    pub drone_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tooltip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tip: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub cost: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub rarity: i32,
    // #[delta2]
    pub faction: ShipId,
    #[serde(skip_serializing_if = "is_zero")]
    pub required_power: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub deploying: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activated: Option<bool>,
    #[serde(skip_serializing_if = "is_zero")]
    pub hacked: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub dead: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<Pair<QuantizedI32<25>>>,
    // for crew
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    // for space
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weapon: Option<WeaponInfo>,
    #[serde(skip_serializing_if = "is_zero")]
    pub stale_info: bool,
}

#[derive(Clone, Debug, Delta, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AugmentInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[delta1]
    pub augment_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub cost: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub rarity: i32,
}

#[derive(Clone, Debug, Delta, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ItemInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[delta1]
    pub item_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub cost: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub rarity: i32,
}

// "repair_{one,all}_*"
#[derive(Clone, Debug, Delta, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct RepairInfo {
    #[serde(skip_serializing_if = "str::is_empty")]
    #[delta1]
    pub repair_name: &'static str,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub description: &'static str,
    #[serde(skip_serializing_if = "is_zero")]
    pub cost: i32,
}

#[derive(Clone, Debug, Delta, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SystemLevel {
    #[delta2]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[delta(skip_serializing_if = "Option::is_none")]
    pub effect: Option<Cow<'static, str>>,
    #[delta1]
    pub level: usize,
    #[serde(skip_serializing_if = "is_zero")]
    pub cost: i32,
    pub purchased: bool,
    pub active: bool,
}

#[derive(Copy, Clone, Debug, Default, Delta, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReactorState {
    #[serde(skip_serializing_if = "is_zero")]
    pub power: Pair<i32>,
    #[serde(skip_serializing_if = "is_zero")]
    pub battery_power: Pair<i32>,
    #[serde(skip_serializing_if = "is_zero")]
    pub reduced_capacity: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub hacked: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub stale_info: bool,
}

#[derive(Clone, Debug, Delta, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    #[delta1]
    pub system_name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    // #[delta2]
    #[serde(skip_serializing_if = "Option::is_none")]
    // #[delta(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<&'static str>,
    #[serde(skip_serializing_if = "is_zero")]
    pub cost: i32,
    #[serde(skip_serializing_if = "is_zero")]
    pub rarity: i32,
    // #[delta2]
    pub faction: ShipId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp: Option<Pair<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_be_manned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_manned_bonus: Option<Cow<'static, str>>,
    #[delta2]
    pub requires_power: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<Pair<i32>>,
    pub levels: Vec<SystemLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    pub level: Pair<i32>,
    // Some(false) or Some(true) if this is e.g. cloaking, None if this is something that doesnt
    // get locked down normally
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_cooldown: Option<bool>,
    #[delta2]
    #[serde(skip_serializing_if = "is_zero")]
    #[delta(skip_serializing_if = "is_zero")]
    pub hacked: bool,
    #[delta2]
    #[serde(skip_serializing_if = "is_zero")]
    #[delta(skip_serializing_if = "is_zero")]
    pub on_fire: bool,
    #[delta2]
    #[serde(skip_serializing_if = "is_zero")]
    #[delta(skip_serializing_if = "is_zero")]
    pub breached: bool,
    #[delta2]
    #[serde(skip_serializing_if = "is_zero")]
    #[delta(skip_serializing_if = "is_zero")]
    pub being_damaged: bool,
    #[delta2]
    #[serde(skip_serializing_if = "is_zero")]
    #[delta(skip_serializing_if = "is_zero")]
    pub being_repaired: bool,
    // for piloting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evasion_bonus: Option<i32>,
    // for weapons
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub weapon_names: Vec<String>,
    // for drones
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub drone_names: Vec<String>,
    // for shields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shields: Option<Pair<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub super_shields: Option<Pair<i32>>,
    // for hacking
    #[serde(skip_serializing_if = "is_zero")]
    pub hacking_in_progress: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub hacking_allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hacking_drone_system: Option<&'static str>,
    // for battery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_power: Option<Pair<i32>>,
    // for oxygen
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_oxygen_level: Option<Pair<QuantizedI32<10>>>,
    // for artillery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artillery_weapon: Option<WeaponInfo>,
    #[serde(skip_serializing_if = "is_zero")]
    pub stale_info: bool,
}

impl<T: IsZero> IsZero for Option<T> {
    fn is_zero(&self) -> bool {
        !self.as_ref().is_some_and(|x| !x.is_zero())
    }
}

#[derive(Clone, Debug, Delta, Eq, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub struct StoreItems {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub drones: Vec<DroneInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub weapons: Vec<WeaponInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub crew: Vec<CrewInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub augments: Vec<AugmentInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub repair: Vec<RepairInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub systems: Vec<SystemInfo>,
}

#[derive(Clone, Debug, Delta, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DoorInfoShort {
    #[delta1]
    pub door_id: i32,
    #[delta2]
    pub room_id: i32,
}

#[derive(Clone, Debug, Delta, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfo {
    // #[delta2]
    pub faction: ShipId,
    #[delta1]
    pub room_id: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub doors: Vec<DoorInfoShort>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub crew_member_names: Vec<String>,
    #[delta2]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[delta(skip_serializing_if = "Vec::is_empty")]
    pub intruder_names: Vec<String>,
    #[delta2]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[delta(skip_serializing_if = "Option::is_none")]
    pub system_name: Option<String>,
    #[delta2]
    #[serde(skip_serializing_if = "is_zero")]
    #[delta(skip_serializing_if = "is_zero")]
    pub fire_level: i32,
    #[delta2]
    #[serde(skip_serializing_if = "is_zero")]
    #[delta(skip_serializing_if = "is_zero")]
    pub breached: bool,
    pub oxygen_percentage: QuantizedI32<25>,
    #[delta2]
    #[serde(skip_serializing_if = "is_zero")]
    #[delta(skip_serializing_if = "is_zero")]
    pub hacked: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub stale_info: bool,
}

#[derive(Clone, Debug, Delta, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DoorInfo {
    #[delta1]
    pub door_id: i32,
    pub room_id_1: i32,
    pub room_id_2: i32,
    pub open: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub hacked: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub lockdown: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Species {
    Human,
    Engi,
    Mantis,
    Slug,
    Rockman,
    Crystal,
    Zoltan,
    Lanius,
    // not techically a species
    BattleDrone,
    RepairDrone,
    Drone,
}

impl Species {
    pub fn from_id(id: &str) -> Self {
        match id {
            "human" => Self::Human,
            "engi" => Self::Engi,
            "mantis" => Self::Mantis,
            "slug" => Self::Slug,
            "rock" => Self::Rockman,
            "crystal" => Self::Crystal,
            "energy" => Self::Zoltan,
            "anaerobic" => Self::Lanius,
            "battle" => Self::BattleDrone,
            "repair" => Self::RepairDrone,
            _ => Self::Drone,
        }
    }
}

#[derive(Clone, Debug, Delta, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub ship: ShipId,
    pub room_id: u32,
}

#[derive(Clone, Debug, Delta, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Skills {
    pub piloting_evasion: Help<u8>,
    pub engines_evasion: Help<u8>,
    pub shields_recharge: Help<u8>,
    pub weapons_recharge: Help<u8>,
    pub repairing_speed: Help<u8>,
    pub fighting_strength: Help<u8>,
    pub movement_speed: Help<u8>,
    // Rockman
    #[serde(skip_serializing_if = "is_zero")]
    pub fire_immunity: Help<bool>,
    // Slug
    #[serde(skip_serializing_if = "is_zero")]
    pub mind_control_immunity: Help<bool>,
    // Slug
    #[serde(skip_serializing_if = "is_zero")]
    pub telepathic_sensors: Help<bool>,
    // Lanius/Crystal
    #[serde(skip_serializing_if = "is_zero")]
    pub suffocation_resistance: Help<u8>,
    // Lanius
    #[serde(skip_serializing_if = "is_zero")]
    pub drains_room_oxygen: Help<bool>,
    // Crystal
    #[serde(skip_serializing_if = "is_zero")]
    pub can_lockdown_rooms: Help<bool>,
    // Zoltan
    #[serde(skip_serializing_if = "is_zero")]
    pub damage_on_death: Help<u8>,
    // Zoltan
    #[serde(skip_serializing_if = "is_zero")]
    pub powers_systems: Help<bool>,
}

#[derive(Clone, Debug, Delta, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CrewInfo {
    #[delta1]
    pub crew_member_name: String,
    pub species: Species,
    #[delta2]
    pub faction: ShipId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    pub bonuses: Skills,
    pub health: Pair<QuantizedI32<25>>,
    // reuse on_fire for this because who cares
    #[serde(skip_serializing_if = "is_zero")]
    pub fighting_fire: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub suffocating: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub fighting: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub healing: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub dead: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub mind_controlled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manning: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repairing: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sabotaging: Option<&'static str>,
    #[serde(skip_serializing_if = "is_zero")]
    pub stale_info: bool,
}

#[derive(Clone, Debug, Delta, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShipInfo {
    #[delta1]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ship_name: String,
    // #[delta2]
    pub faction: ShipId,
    #[serde(skip_serializing_if = "is_zero")]
    pub destroyed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reactor: Option<ReactorState>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rooms: Vec<RoomInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub doors: Vec<DoorInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub systems: Vec<SystemInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub crew: Vec<CrewInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub weapons: Vec<ItemSlot<WeaponInfo>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub drones: Vec<ItemSlot<DroneInfo>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub augments: Vec<ItemSlot<AugmentInfo>>,
    pub hull: Help<Pair<i32>>,
    pub evasion_chance_percentage: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnyItemInfo {
    Weapon(WeaponInfo),
    Drone(DroneInfo),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnyItemInfoSer<'a> {
    Weapon(WeaponInfoSer<'a>),
    Drone(DroneInfoSer<'a>),
}

impl<'a> HasId<'a> for AnyItemInfo {
    type Id = (&'a String,);
    fn id(&'a self) -> Self::Id {
        match self {
            Self::Weapon(x) => x.id(),
            Self::Drone(x) => x.id(),
        }
    }
}

impl<'a> Delta<'a> for AnyItemInfo {
    type Delta = AnyItemInfoSer<'a>;
    fn delta(&'a self, prev: &'a Self, ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        (self != prev).then(|| self.serializable(ctx))
    }
}

impl<'a> Serializable<'a> for AnyItemInfo {
    type Ser = AnyItemInfoSer<'a>;
    fn serializable(&'a self, ctx: &mut SerContext<'a>) -> Self::Ser {
        match self {
            Self::Weapon(x) => AnyItemInfoSer::Weapon(x.serializable(ctx)),
            Self::Drone(x) => AnyItemInfoSer::Drone(x.serializable(ctx)),
        }
    }
}

#[derive(Clone, Debug, Delta, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub scrap_count: Help<i32>,
    pub drone_part_count: Help<i32>,
    pub fuel_count: Help<i32>,
    pub missile_count: Help<i32>,
    #[serde(skip_serializing_if = "is_zero")]
    pub overcapacity_slot: ItemSlot<AnyItemInfo>,
    #[serde(skip_serializing_if = "is_zero")]
    pub augment_overcapacity_slot: ItemSlot<AugmentInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cargo_slots: Vec<ItemSlot<AnyItemInfo>>,
}

#[derive(Copy, Clone, Debug, Serialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum InventorySlotType {
    OverCapacity,
    AugmentationOverCapacity,
    Weapon,
    Cargo,
    Drone,
    Augmentation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSlot<T> {
    pub r#type: InventorySlotType,
    pub index: usize,
    pub contents: Option<T>,
}

impl<'a, T> HasId<'a> for ItemSlot<T> {
    type Id = (InventorySlotType, usize);
    fn id(&'a self) -> Self::Id {
        (self.r#type, self.index)
    }
}
impl<'a, T: 'a + Delta<'a> + Serializable<'a>> Delta<'a> for ItemSlot<T> {
    type Delta = ItemSlot<util::Opt3<T::Ser, T::Delta>>;
    fn delta(&'a self, prev: &'a Self, ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        self.contents
            .delta(&prev.contents, ctx)
            .map(|contents| ItemSlot {
                r#type: self.r#type,
                index: self.index,
                contents: Some(contents),
            })
    }
}
impl<'a, T: 'a + Serializable<'a>> Serializable<'a> for ItemSlot<T> {
    type Ser = ItemSlot<T::Ser>;
    fn serializable(&'a self, ctx: &mut SerContext<'a>) -> Self::Ser {
        ItemSlot {
            r#type: self.r#type,
            index: self.index,
            contents: self.contents.as_ref().map(|x| x.serializable(ctx)),
        }
    }
}

impl<T> ItemSlot<T> {
    pub fn new(r#type: InventorySlotType, index: usize) -> Self {
        Self {
            r#type,
            index,
            contents: None,
        }
    }
    pub fn new1(r#type: InventorySlotType, index: usize, contents: T) -> Self {
        Self {
            r#type,
            index,
            contents: Some(contents),
        }
    }
}

impl<T> IsZero for ItemSlot<T> {
    fn is_zero(&self) -> bool {
        self.contents.is_none()
    }
}

#[derive(Clone, Debug, Default, Delta, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShipDesc {
    #[delta1]
    pub layout_id: usize,
    #[delta1]
    pub layout_variation_id: usize,
    #[delta2]
    pub ship_name: String,
    pub class: String,
    pub description: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub unlocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_condition: Option<String>,
}

#[derive(Clone, Debug, Default, Delta)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub confirmation_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_text: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub event_options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory: Option<Inventory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_store_page: Option<StoreItems>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_location: Option<CurrentLocationInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<LocationInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sectors: Vec<SectorInfo>,
    #[serde(skip_serializing_if = "is_zero")]
    pub in_main_menu: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub can_continue_saved_game: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub game_over: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub in_credits: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub in_new_game_config: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub victory: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub available_ships: Vec<ShipDesc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_ship: Option<ShipDesc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_ship: Option<ShipInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enemy_ship: Option<ShipInfo>,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Pair<T> {
    pub current: T,
    pub max: T,
}

impl<T: IsZero> IsZero for Pair<T> {
    fn is_zero(&self) -> bool {
        is_zero(&self.current) && is_zero(&self.max)
    }
}

impl<'a, T: Delta<'a> + Serializable<'a>> Delta<'a> for Pair<T> {
    type Delta = Pair<T::Ser>;
    fn delta(&'a self, prev: &'a Self, ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        (self.current.delta(&prev.current, ctx).is_some()
            || self.max.delta(&prev.max, ctx).is_some())
        .then(|| self.serializable(ctx))
    }
}

impl<'a, T: Serializable<'a>> Serializable<'a> for Pair<T> {
    type Ser = Pair<T::Ser>;
    fn serializable(&'a self, ctx: &mut SerContext<'a>) -> Self::Ser {
        Pair {
            current: self.current.serializable(ctx),
            max: self.max.serializable(ctx),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<'a, T: 'a + Ord + std::fmt::Debug> HasId<'a> for Point<T> {
    type Id = &'a Self;
    fn id(&'a self) -> Self::Id {
        self
    }
}

impl<'a, T: Delta<'a> + Serializable<'a>> Delta<'a> for Point<T> {
    type Delta = Point<T::Ser>;
    fn delta(&'a self, prev: &'a Self, ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        (self.x.delta(&prev.x, ctx).is_some() || self.y.delta(&prev.y, ctx).is_some())
            .then(|| self.serializable(ctx))
    }
}

impl<'a, T: Serializable<'a>> Serializable<'a> for Point<T> {
    type Ser = Point<T::Ser>;
    fn serializable(&'a self, ctx: &mut SerContext<'a>) -> Self::Ser {
        Point {
            x: self.x.serializable(ctx),
            y: self.y.serializable(ctx),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    TopLeft,
    Left,
    BottomLeft,
    Top,
    Bottom,
    TopRight,
    Right,
    BottomRight,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipId {
    Player,
    Enemy,
}

#[derive(Clone, Debug, Delta)]
#[serde(rename_all = "camelCase")]
pub struct LocationInfo {
    #[delta1]
    pub map_position: Point<i32>,
    pub map_routes: Vec<Point<i32>>,
    /// Your current location.
    #[serde(skip_serializing_if = "is_zero")]
    pub current: Help<bool>,
    /// Rebel Flagship
    #[serde(skip_serializing_if = "is_zero")]
    pub boss: Help<bool>,
    #[serde(skip_serializing_if = "is_zero")]
    pub boss_comes_in: Help<usize>,
    /// This is the Federation Base.
    #[serde(skip_serializing_if = "is_zero")]
    pub base: Help<bool>,
    /// This is the exit beacon. Go here to travel to the next sector.
    #[serde(skip_serializing_if = "is_zero")]
    pub exit: Help<bool>,
    /// The Rebels are about to gain control of this beacon!
    #[serde(skip_serializing_if = "is_zero")]
    pub rebels: Help<bool>,
    /// You previously found a store at this location.
    #[serde(skip_serializing_if = "is_zero")]
    pub store: Help<bool>,
    /// Federation Repair Station. Repairs hull and provides supplies.
    #[serde(skip_serializing_if = "is_zero")]
    pub repair: Help<bool>,
    /// The Rebels have control of this location. Very dangerous.
    #[serde(skip_serializing_if = "is_zero")]
    pub fleet: Help<bool>,
    #[serde(skip_serializing_if = "is_zero")]
    pub fleet_comes_in: Help<f64>,
    /// A hostile enemy was left behind at this location.
    #[serde(skip_serializing_if = "is_zero")]
    pub hostile: Help<bool>,
    /// Explored location. Nothing left of interest.
    #[serde(skip_serializing_if = "is_zero")]
    pub nothing: Help<bool>,
    /// Distress beacon. Someone might need help.
    #[serde(skip_serializing_if = "is_zero")]
    pub distress: Help<bool>,
    /// Unvisited. Possible ship detected.
    #[serde(skip_serializing_if = "is_zero")]
    pub ship: Help<bool>,
    /// Unvisited. Quest destination.
    #[serde(skip_serializing_if = "is_zero")]
    pub quest: Help<bool>,
    /// Unvisited. Reported merchant location.
    #[serde(skip_serializing_if = "is_zero")]
    pub merchant: Help<bool>,
    /// An unvisited location.
    #[serde(skip_serializing_if = "is_zero")]
    pub unvisited: Help<bool>,
    /// The Rebel Fleet was prepared for the nebula in this sector, so it won't be as effective a hiding spot. The nebula will still disrupt your sensors.
    #[serde(skip_serializing_if = "is_zero")]
    pub nebula_fleet: Help<bool>,
    /// The nebula here will make fleet pursuit slower but will disrupt your sensors.
    #[serde(skip_serializing_if = "is_zero")]
    pub nebula: Help<bool>,
    /// Asteroid field detected in this location.
    #[serde(skip_serializing_if = "is_zero")]
    pub asteroids: Help<bool>,
    /// Beacon coordinates appear to be very close to a nearby sun.
    #[serde(skip_serializing_if = "is_zero")]
    pub sun: Help<bool>,
    /// This section of the nebula is experiencing an ion storm.
    #[serde(skip_serializing_if = "is_zero")]
    pub ion: Help<bool>,
    /// A pulsar is flooding this area with dangerous electromagnetic forces.
    #[serde(skip_serializing_if = "is_zero")]
    pub pulsar: Help<bool>,
    /// Planet-side anti-ship batteries are detected in this system.
    #[serde(skip_serializing_if = "is_zero")]
    pub planetary_defense_system: Help<bool>,
    /// The Fleet's Anti-Ship Batteries are targeting you.
    #[serde(skip_serializing_if = "is_zero")]
    pub planetary_defense_system_fleet: Help<bool>,
}

#[derive(Clone, Debug, Delta, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentLocationInfo {
    #[delta1]
    pub map_position: Point<i32>,
    pub map_routes: BTreeMap<Direction, Point<i32>>,
    /// This is the Federation Base.
    #[serde(skip_serializing_if = "is_zero")]
    pub base: Help<bool>,
    /// This is the exit beacon. Go here to travel to the next sector.
    #[serde(skip_serializing_if = "is_zero")]
    pub exit: Help<bool>,
    /// You previously found a store at this location.
    #[serde(skip_serializing_if = "is_zero")]
    pub store: Help<bool>,
    /// The Rebel Fleet was prepared for the nebula in this sector, so it won't be as effective a hiding spot. The nebula will still disrupt your sensors.
    #[serde(skip_serializing_if = "is_zero")]
    pub nebula_fleet: Help<bool>,
    /// The nebula here will make fleet pursuit slower but will disrupt your sensors.
    #[serde(skip_serializing_if = "is_zero")]
    pub nebula: Help<bool>,
    /// Asteroid field detected in this location.
    #[serde(skip_serializing_if = "is_zero")]
    pub asteroids: Help<bool>,
    /// Beacon coordinates appear to be very close to a nearby sun.
    #[serde(skip_serializing_if = "is_zero")]
    pub sun: Help<bool>,
    /// This section of the nebula is experiencing an ion storm.
    #[serde(skip_serializing_if = "is_zero")]
    pub ion: Help<bool>,
    /// A pulsar is flooding this area with dangerous electromagnetic forces.
    #[serde(skip_serializing_if = "is_zero")]
    pub pulsar: Help<bool>,
    /// The Fleet's Anti-Ship Batteries are targeting you.
    #[serde(skip_serializing_if = "is_zero")]
    pub planetary_defense_system_fleet: Help<bool>,
    /// An Anti-Ship Battery on the planet is targeting you.
    #[serde(skip_serializing_if = "is_zero")]
    pub planetary_defense_system_player: Help<bool>,
    /// An Anti-Ship Battery on the planet is targeting your enemies.
    #[serde(skip_serializing_if = "is_zero")]
    pub planetary_defense_system_enemy: Help<bool>,
    /// An Anti-Ship Battery on the planet is firing on all ships in the area.
    #[serde(skip_serializing_if = "is_zero")]
    pub planetary_defense_system_all: Help<bool>,
}

#[derive(Clone, Debug, Delta)]
#[serde(rename_all = "camelCase")]
pub struct SectorInfo {
    #[delta1]
    pub map_position: Point<i32>,
    // only add this if this is immediately reachable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_name: Option<String>,
    pub map_routes: BTreeMap<Direction, Point<i32>>,
    #[serde(skip_serializing_if = "is_zero")]
    pub hostile: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub civilian: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub nebula: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub unknown: bool,
}

impl_delta!(ShipId, Species, InventorySlotType);
