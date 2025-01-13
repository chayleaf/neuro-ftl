use std::borrow::Cow;

use neuro_ftl_derive::Delta;
use serde::Serialize;

use crate::impl_delta;

use super::strings::{self, text};

pub mod util;

use util::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerShipInfo {
    pub hull: Help<i32>,
    pub max_hull: i32,
    pub drone_count: Help<i32>,
    pub fuel_count: Help<i32>,
    pub missile_count: Help<i32>,
    pub scrap_count: Help<i32>,
}

#[derive(Clone, Debug, Serialize, Delta, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WeaponInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[delta1]
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tooltip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tip: String,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub damage: u8,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub healing: u8,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub missiles_per_shot: u8,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub shield_piercing: u8,
    // *10
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub fire_chance_percentage: u8,
    // *10
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub breach_chance_percentage: u8,
    // *10
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub stun_chance_percentage: u8,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub stun_duration: u8,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub ion_damage: u8,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub system_damage: u8,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub crew_damage: u8,
    // must be set to damage * 2 if hull_buster != 0
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub hull_damage: u8,
    #[serde(skip_serializing_if = "is_false")]
    pub lockdowns_room: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub can_target_own_ship: bool,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub missiles_required: u8,

    #[serde(skip_serializing_if = "is_zero_u8")]
    pub projectile_speed: u8,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub cooldown: u8,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub required_power: u8,
    pub powered: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub hacked: bool,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
pub struct DroneInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[delta1]
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tooltip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tip: String,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub required_power: u8,
    #[serde(skip_serializing_if = "is_false")]
    pub deploying: bool,
    pub deployed: bool,
    pub powered: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub hacked: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub dead: bool,
    pub health: QuantizedU8<20>,
    pub max_health: QuantizedU8<20>,
    // for crew
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "snake_case")]
pub struct AugmentInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[delta1]
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tooltip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tip: String,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "snake_case")]
pub struct ItemInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[delta1]
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

// "repair_{one,all}_*"
#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "snake_case")]
pub struct RepairInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[delta1]
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
pub struct SystemLevel {
    pub effect: Cow<'static, str>,
    #[delta1]
    pub level: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<u8>,
    pub purchased: bool,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    #[delta1]
    pub ship: ShipId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<u8>,
    #[delta1]
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub hp: u8,
    pub max_hp: u8,
    pub can_be_manned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_manned_bonus: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_power: Option<u8>,
    pub levels: Vec<SystemLevel>,
    pub active: bool,
    pub level: u8,
    pub max_level: u8,
    // Some(false) or Some(true) if this is e.g. cloaking, None if this is something that doesnt
    // get locked down normally
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_cooldown: Option<bool>,
    #[serde(skip_serializing_if = "is_false")]
    pub hacked: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub on_fire: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub breached: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub being_damaged: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub being_repaired: bool,
    // for piloting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_evasion_chance_percentage: Option<u8>,
    // for weapons
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub weapon_names: Vec<String>,
    // for drones
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub drone_names: Vec<String>,
    // for shields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shields: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_shields: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub super_shields: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_super_shields: Option<u8>,
    // for hacking
    #[serde(skip_serializing_if = "is_false")]
    pub hacking_in_progress: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub hacking_allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hacking_drone_system: Option<String>,
    // for battery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_power: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_battery_power: Option<u8>,
    // for oxygen
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_oxygen_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_max_oxygen_level: Option<u8>,
    // for artillery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artillery_weapon: Option<WeaponInfo>,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "snake_case")]
pub struct StoreItem {
    pub drones: Vec<DroneInfo>,
    pub weapons: Vec<WeaponInfo>,
    pub crew: Vec<CrewInfo>,
    pub augments: Vec<AugmentInfo>,
    pub items: Vec<ItemInfo>,
    pub repair: Vec<RepairInfo>,
}

#[derive(Clone, Debug, Serialize, Delta, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DoorInfoShort {
    #[delta1]
    pub door_id: i8,
    #[delta1]
    pub room_id: i8,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfo {
    pub ship: ShipId,
    #[delta1]
    pub room_id: i8,
    pub doors: Vec<DoorInfoShort>,
    pub crew_member_names: Vec<String>,
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub fire_percentage: u8,
    pub oxygen_percentage: u8,
    #[serde(skip_serializing_if = "is_false")]
    pub hacked: bool,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
pub struct DoorInfo {
    pub ship: ShipId,
    #[delta1]
    pub door_id: i8,
    pub room_id_1: i8,
    pub room_id_2: i8,
    pub open: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub hacked: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Species {
    Human,
    Engi,
    Mantis,
    Slug,
    Rock,
    Crystal,
    Energy,
    Anaerobic,
}

#[derive(Clone, Debug, Serialize, Delta, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub ship: ShipId,
    pub room_id: u8,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
pub struct Skills {
    pub piloting_evasion: Help<u8>,
    pub engines_evasion: Help<u8>,
    pub shields_recharge: Help<u8>,
    pub weapons_recharge: Help<u8>,
    pub repairing_speed: Help<u8>,
    pub fighting_strength: Help<u8>,
}

impl Skills {
    pub fn new() -> Self {
        Self {
            piloting_evasion: Help::new(strings::SKILL_PILOTING, 0),
            engines_evasion: Help::new(strings::SKILL_ENGINES, 0),
            shields_recharge: Help::new(strings::SKILL_SHIELDS, 0),
            weapons_recharge: Help::new(strings::SKILL_WEAPONS, 0),
            repairing_speed: Help::new(strings::SKILL_REPAIRING, 0),
            fighting_strength: Help::new(strings::SKILL_FIGHTING, 0),
        }
    }
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
pub struct CrewInfo {
    #[delta1]
    pub name: String,
    pub species: Species,
    pub faction: ShipId,
    pub location: Location,
    pub bonus_percentage_added: Skills,
    pub health: QuantizedU8<20>,
    pub max_health: QuantizedU8<20>,
    pub is_drone: bool,
    // reuse on_fire for this because who cares
    #[serde(skip_serializing_if = "is_false")]
    pub fighting_fire: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub suffocating: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub fighting: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub healing: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub dead: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub mind_controlled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manning: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repairing: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sabotaging: Option<Cow<'static, str>>,
}

#[derive(Clone, Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
struct ShipInfo {
    pub ship_name: String,
    pub destroyed: bool,
    pub rooms: Vec<RoomInfo>,
    pub doors: Vec<DoorInfo>,
    pub systems: Vec<SystemInfo>,
    pub crew: Vec<CrewInfo>,
    pub weapons: Vec<WeaponInfo>,
    pub drones: Vec<WeaponInfo>,
}

impl PlayerShipInfo {
    pub fn new() -> Self {
        Self {
            hull: Help::new(text("tooltip_hull"), 0),
            max_hull: 0,
            drone_count: Help::new(text("tooltip_droneCount"), 0),
            fuel_count: Help::new(text("tooltip_fuelCount"), 0),
            missile_count: Help::new(text("tooltip_missileCount"), 0),
            scrap_count: Help::new(text("tooltip_scrapCount"), 0),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Pair<T> {
    pub current: T,
    pub max: T,
}

#[derive(Copy, Clone, Debug, Default, Delta, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Point<T: std::fmt::Debug + Ord + Serialize> {
    #[delta1]
    pub x: T,
    #[delta1]
    pub y: T,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipId {
    Player,
    Enemy,
    AllShips,
}

#[derive(Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
pub struct LocationInfo {
    pub map_position: Point<i32>,
    pub map_routes: Vec<Point<i32>>,
    /// Your current location.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub current: Help<bool>,
    /// Rebel Flagship
    #[serde(skip_serializing_if = "Help::is_false")]
    pub boss: Help<bool>,
    /// This is the Federation Base.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub base: Help<bool>,
    /// This is the exit beacon. Go here to travel to the next sector.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub exit: Help<bool>,
    /// The Rebels are about to gain control of this beacon!
    #[serde(skip_serializing_if = "Help::is_false")]
    pub rebels: Help<bool>,
    /// You previously found a store at this location.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub store: Help<bool>,
    /// Federation Repair Station. Repairs hull and provides supplies.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub repair: Help<bool>,
    /// The Rebels have control of this location. Very dangerous.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub fleet: Help<bool>,
    /// A hostile enemy was left behind at this location.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub hostile: Help<bool>,
    /// Explored location. Nothing left of interest.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub nothing: Help<bool>,
    /// Distress beacon. Someone might need help.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub distress: Help<bool>,
    /// Unvisited. Possible ship detected.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub ship: Help<bool>,
    /// Unvisited. Quest destination.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub quest: Help<bool>,
    /// Unvisited. Reported merchant location.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub merchant: Help<bool>,
    /// An unvisited location.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub unvisited: Help<bool>,
    /// The Rebel Fleet was prepared for the nebula in this sector, so it won't be as effective a hiding spot. The nebula will still disrupt your sensors.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub nebula_fleet: Help<bool>,
    /// The nebula here will make fleet pursuit slower but will disrupt your sensors.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub nebula: Help<bool>,
    /// Asteroid field detected in this location.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub asteroids: Help<bool>,
    /// Beacon coordinates appear to be very close to a nearby sun.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub sun: Help<bool>,
    /// This section of the nebula is experiencing an ion storm.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub ion: Help<bool>,
    /// A pulsar is flooding this area with dangerous electromagnetic forces.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub pulsar: Help<bool>,
    /// Planet-side anti-ship batteries are detected in this system.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub planetary_defense_system: Help<bool>,
    /// The Fleet's Anti-Ship Batteries are targeting you.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub planetary_defense_system_fleet: Help<bool>,
    /// An Anti-Ship Battery on the planet is targeting you.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub planetary_defense_system_player: Help<bool>,
    /// An Anti-Ship Battery on the planet is targeting your enemies.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub planetary_defense_system_enemy: Help<bool>,
    /// An Anti-Ship Battery on the planet is firing on all ships in the area.
    #[serde(skip_serializing_if = "Help::is_false")]
    pub planetary_defense_system_all: Help<bool>,
}

impl LocationInfo {
    pub fn new_map() -> Self {
        Self {
            map_position: Point::default(),
            map_routes: vec![],
            current: Help::new(text("map_current_loc"), false),
            boss: Help::new(text("map_boss_loc"), false),
            base: Help::new(text("map_base_loc"), false),
            exit: Help::new(text("map_exit_loc"), false),
            rebels: Help::new(text("map_rebels_loc"), false),
            store: Help::new(text("map_store_loc"), false),
            repair: Help::new(text("map_repair_loc"), false),
            fleet: Help::new(text("map_fleet_loc"), false),
            hostile: Help::new(text("map_hostile_loc"), false),
            nothing: Help::new(text("map_nothing_loc"), false),
            distress: Help::new(text("map_distress_loc"), false),
            ship: Help::new(text("map_ship_loc"), false),
            quest: Help::new(text("map_quest_loc"), false),
            merchant: Help::new(text("map_merchant_loc"), false),
            unvisited: Help::new(text("map_unvisited_loc"), false),
            nebula_fleet: Help::new(strings::LOC_NEBULA_FLEET, false),
            nebula: Help::new(text("map_nebula_loc"), false),
            asteroids: Help::new(text("map_asteroid_loc"), false),
            sun: Help::new(text("map_sun_loc"), false),
            ion: Help::new(text("map_ion_loc"), false),
            pulsar: Help::new(text("map_pulsar_loc"), false),
            planetary_defense_system: Help::new(text("map_pds_loc"), false),
            planetary_defense_system_fleet: Help::new(text("map_pds_fleet"), false),
            planetary_defense_system_all: Help::new(strings::BUG, false),
            planetary_defense_system_enemy: Help::new(strings::BUG, false),
            planetary_defense_system_player: Help::new(strings::BUG, false),
        }
    }
    pub fn new_current() -> Self {
        Self {
            map_position: Point::default(),
            map_routes: vec![],
            current: Help::new(strings::BUG, false),
            boss: Help::new(strings::BUG, false),
            base: Help::new(strings::LOC_BASE, false),
            exit: Help::new(strings::LOC_EXIT, false),
            rebels: Help::new(strings::LOC_REBELS, false),
            store: Help::new(strings::LOC_STORE, false),
            repair: Help::new(strings::BUG, false),
            fleet: Help::new(strings::BUG, false),
            hostile: Help::new(strings::BUG, false),
            nothing: Help::new(strings::BUG, false),
            distress: Help::new(strings::BUG, false),
            ship: Help::new(strings::BUG, false),
            quest: Help::new(strings::BUG, false),
            merchant: Help::new(strings::BUG, false),
            unvisited: Help::new(strings::BUG, false),
            nebula_fleet: Help::new(strings::LOC_NEBULA_FLEET, false),
            nebula: Help::new(text("tooltip_nebula"), false),
            asteroids: Help::new(text("tooltip_asteroids"), false),
            sun: Help::new(text("tooltip_sun"), false),
            ion: Help::new(text("tooltip_storm"), false),
            pulsar: Help::new(text("tooltip_pulsar"), false),
            planetary_defense_system: Help::new(strings::BUG, false),
            planetary_defense_system_fleet: Help::new(text("tooltip_PDS_FLEET"), false),
            planetary_defense_system_player: Help::new(text("tooltip_PDS_PLAYER"), false),
            planetary_defense_system_enemy: Help::new(text("tooltip_PDS_ENEMY"), false),
            planetary_defense_system_all: Help::new(text("tooltip_PDS_ALL"), false),
        }
    }
}

#[derive(Debug, Serialize, Delta)]
#[serde(rename_all = "camelCase")]
pub struct SectorInfo {
    pub map_position: Point<i32>,
    pub map_routes: Vec<Point<i32>>,
    // only add this if this is immediately reachable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub hostile: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub civilian: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub nebula: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub unknown: bool,
}

impl_delta!(ShipId, Species);
