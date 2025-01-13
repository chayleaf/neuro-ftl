use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Text {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@language")]
    pub language: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TextString {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub contents: Option<String>,
    #[serde(rename = "@load")]
    pub load: Option<String>,

    #[serde(rename = "@planet")]
    pub planet: Option<String>,
    #[serde(rename = "@back")]
    pub back: Option<String>,
}

impl TextString {
    pub fn to_str(&'static self) -> &'static str {
        if let Some(id) = &self.load {
            super::library().text(id).unwrap()
        } else {
            self.contents.as_ref().unwrap()
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Achievement {
    #[serde(rename = "@id")]
    pub id: String,
    pub name: TextString,
    pub desc: TextString,
    pub shortname: Option<TextString>,
    pub img: String,
    pub ship: Option<String>,
    #[serde(default, rename = "multiDifficulty")]
    pub multi_difficulty: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename = "FTL")]
#[serde(deny_unknown_fields)]
pub struct XmlText {
    #[serde(default)]
    pub text: Vec<Text>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename = "FTL")]
#[serde(deny_unknown_fields)]
pub struct XmlAchievements {
    #[serde(default, rename = "achievement")]
    pub achievements: Vec<Achievement>,
}

#[derive(Deserialize, Serialize, Debug, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rect {
    #[serde(rename = "@x")]
    pub x: i32,
    #[serde(rename = "@y")]
    pub y: i32,
    #[serde(rename = "@w")]
    pub w: i32,
    #[serde(rename = "@h")]
    pub h: i32,
}

#[derive(Deserialize, Serialize, Debug, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Point {
    #[serde(rename = "@x")]
    pub x: i32,
    #[serde(rename = "@y")]
    pub y: i32,
}

#[derive(Deserialize, Serialize, Debug, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Offsets {
    pub floor: Option<Point>,
    pub cloak: Option<Point>,
}

#[derive(Deserialize, Serialize, Debug, Hash, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Deserialize, Serialize, Debug, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Mount {
    #[serde(rename = "@x")]
    pub x: i32,
    #[serde(rename = "@y")]
    pub y: i32,
    #[serde(rename = "@rotate")]
    pub rotate: bool,
    #[serde(rename = "@mirror")]
    pub mirror: bool,
    #[serde(rename = "@gib")]
    pub gib: i32,
    #[serde(rename = "@slide")]
    pub slide: Direction,
}

#[derive(Deserialize, Serialize, Debug, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WeaponMounts {
    #[serde(default, rename = "mount")]
    pub mounts: Vec<Mount>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Range {
    #[serde(rename = "@min")]
    pub min: f64,
    #[serde(rename = "@max")]
    pub max: f64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Gib {
    pub x: i32,
    pub y: i32,
    pub velocity: Range,
    pub direction: Range,
    pub angular: Range,
    pub glow_offset: Point,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Explosion {
    pub gib1: Gib,
    pub gib2: Gib,
    pub gib3: Gib,
    pub gib4: Gib,
    pub gib5: Option<Gib>,
    pub gib6: Option<Gib>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename = "FTL")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct XmlShip {
    pub img: Rect,
    pub glow_offset: Point,
    pub offsets: Offsets,
    pub weapon_mounts: WeaponMounts,
    pub explosion: Explosion,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AnimSheet {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@w")]
    pub w: i32,
    #[serde(rename = "@h")]
    pub h: i32,
    #[serde(rename = "@fw")]
    pub fw: i32,
    #[serde(rename = "@fh")]
    pub fh: i32,
    #[serde(rename = "$text")]
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AnimDesc {
    #[serde(rename = "@length")]
    pub length: i32,
    #[serde(rename = "@x")]
    pub x: i32,
    #[serde(rename = "@y")]
    pub y: i32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WeaponAnim {
    #[serde(rename = "@name")]
    pub name: String,
    pub sheet: String,
    pub desc: AnimDesc,
    #[serde(rename = "chargedFrame")]
    pub charged_frame: u32,
    #[serde(rename = "fireFrame")]
    pub fire_frame: u32,
    #[serde(rename = "firePoint")]
    pub fire_point: Point,
    #[serde(rename = "mountPoint")]
    pub mount_point: Point,
    #[serde(rename = "chargeImage")]
    pub charge_image: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Anim {
    #[serde(rename = "@name")]
    pub name: String,
    pub sheet: String,
    pub desc: AnimDesc,
    pub time: f64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename = "FTL")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct XmlAnim {
    #[serde(default)]
    pub anim_sheet: Vec<AnimSheet>,
    #[serde(default)]
    pub anim: Vec<Anim>,
    #[serde(default)]
    pub weapon_anim: Vec<WeaponAnim>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BlueprintList {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(default, rename = "name")]
    pub names: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Amount {
    #[serde(rename = "@amount")]
    pub amount: i32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CrewCount {
    #[serde(rename = "@amount")]
    pub amount: u32,
    #[serde(rename = "@max")]
    pub max: Option<u32>,
    #[serde(rename = "@class")]
    pub class: String,
}

const fn true_fn() -> bool {
    true
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SystemSlot {
    direction: Option<Direction>,
    number: i32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct System {
    #[serde(rename = "@power")]
    pub power: u32,
    #[serde(rename = "@max")]
    pub max: Option<u32>,
    #[serde(rename = "@room")]
    pub room: u32,
    #[serde(default = "true_fn")]
    #[serde(rename = "@start")]
    pub start: bool,
    #[serde(rename = "@img")]
    pub img: Option<String>,
    #[serde(default, rename = "slot")]
    pub slots: Vec<SystemSlot>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ArtillerySystem {
    #[serde(rename = "@power")]
    pub power: u32,
    #[serde(rename = "@max")]
    pub max: Option<u32>,
    #[serde(rename = "@room")]
    pub room: u32,
    #[serde(default = "true_fn")]
    #[serde(rename = "@start")]
    pub start: bool,
    #[serde(rename = "@img")]
    pub img: Option<String>,
    #[serde(rename = "@weapon")]
    pub weapon: String,
    #[serde(default, rename = "slot")]
    pub slots: Vec<SystemSlot>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SystemList {
    pub shields: Option<System>,
    pub engines: System,
    pub oxygen: Option<System>,
    pub weapons: Option<System>,
    pub drones: Option<System>,
    pub medbay: Option<System>,
    pub pilot: System,
    pub sensors: Option<System>,
    pub doors: Option<System>,
    pub teleporter: Option<System>,
    pub cloaking: Option<System>,
    #[serde(default)]
    pub artillery: Vec<ArtillerySystem>,
    pub battery: Option<System>,
    pub clonebay: Option<System>,
    pub mind: Option<System>,
    pub hacking: Option<System>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DroneList {
    #[serde(rename = "@load")]
    pub load: Option<String>,
    #[serde(rename = "@count")]
    pub count: Option<u32>,
    /// This is an option because theres a typo where a droneList is supposed to be a weaponList
    #[serde(rename = "@drones")]
    pub drones: Option<u32>,
    /// This is only here because theres a typo where a droneList is supposed to be a weaponList
    #[serde(rename = "@missiles")]
    pub missiles: Option<u32>,
    #[serde(default, rename = "drone")]
    pub blueprints: Vec<BlueprintRef>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WeaponList {
    #[serde(rename = "@count")]
    pub count: Option<u32>,
    #[serde(rename = "@missiles")]
    pub missiles: u32,
    #[serde(rename = "@load")]
    pub load: Option<String>,
    #[serde(default, rename = "weapon")]
    pub blueprints: Vec<BlueprintRef>,
}

#[derive(Deserialize, Serialize, Debug, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BlueprintRef {
    #[serde(rename = "@name")]
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ShipBlueprint {
    #[serde(default, rename = "@NOLOC")]
    pub noloc: bool,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@layout")]
    pub layout: String,
    #[serde(rename = "@img")]
    pub img: String,
    pub class: TextString,
    #[serde(rename = "minSector")]
    pub min_sector: Option<u32>,
    #[serde(rename = "maxSector")]
    pub max_sector: Option<u32>,
    #[serde(rename = "name")]
    pub title: Option<TextString>,
    pub unlock: Option<TextString>,
    pub desc: Option<TextString>,
    #[serde(rename = "systemList")]
    pub system_list: SystemList,
    #[serde(default, rename = "droneList")]
    pub drone_list: Vec<DroneList>,
    #[serde(rename = "droneSlots")]
    pub drone_slots: Option<u32>,
    #[serde(rename = "weaponList")]
    pub weapon_list: Option<WeaponList>,
    #[serde(rename = "weaponSlots")]
    pub weapon_slots: Option<u32>,
    pub health: Amount,
    #[serde(rename = "maxPower")]
    pub max_power: Amount,
    #[serde(default, rename = "crewCount")]
    pub crew_count: Vec<CrewCount>,
    #[serde(rename = "boardingAI")]
    pub boarding_ai: Option<String>,
    #[serde(default)]
    pub aug: Vec<BlueprintRef>,
    #[serde(rename = "cloakImage")]
    pub cloak_image: Option<String>,
    #[serde(rename = "shieldImage")]
    pub shield_image: Option<String>,
    #[serde(rename = "floorImage")]
    pub floor_image: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PowerList {
    #[serde(default, rename = "power")]
    pub powers: Vec<TextString>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rgba {
    #[serde(rename = "@r")]
    pub r: f64,
    #[serde(rename = "@g")]
    pub g: f64,
    #[serde(rename = "@b")]
    pub b: f64,
    #[serde(rename = "@a")]
    pub a: f64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    #[serde(default, rename = "color")]
    pub colors: Vec<Rgba>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ColorList {
    #[serde(default, rename = "layer")]
    pub layers: Vec<Layer>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CrewBlueprint {
    #[serde(default, rename = "@NOLOC")]
    pub noloc: bool,
    #[serde(rename = "@name")]
    pub name: String,
    pub desc: TextString,
    pub cost: u32,
    pub bp: u32,
    pub title: TextString,
    pub short: TextString,
    pub rarity: u32,
    #[serde(rename = "powerList")]
    pub power_list: PowerList,
    #[serde(rename = "colorList")]
    pub color_list: Option<ColorList>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UpgradeCost {
    #[serde(default, rename = "level")]
    pub levels: Vec<u32>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SystemBlueprint {
    #[serde(default, rename = "@NOLOC")]
    pub noloc: bool,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "type")]
    pub type_: SystemName,
    pub title: TextString,
    pub desc: TextString,
    #[serde(rename = "startPower")]
    pub start_power: u32,
    #[serde(rename = "maxPower")]
    pub max_power: u32,
    pub rarity: u32,
    #[serde(rename = "upgradeCost")]
    pub upgrade_cost: UpgradeCost,
    pub cost: u32,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sounds {
    #[serde(default, rename = "sound")]
    pub sounds: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Projectile {
    #[serde(rename = "@count")]
    pub count: u32,
    #[serde(rename = "@fake")]
    pub fake: bool,
    #[serde(rename = "$text")]
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Projectiles {
    #[serde(default, rename = "projectile")]
    pub projectiles: Vec<Projectile>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Boost {
    #[serde(rename = "type")]
    pub type_: BoostType,
    pub amount: u32,
    pub count: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WeaponBlueprint {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(default, rename = "@NOLOC")]
    pub noloc: bool,
    #[serde(rename = "type")]
    pub type_: WeaponType,
    pub title: TextString,
    #[serde(default)]
    pub short: Vec<TextString>,
    pub desc: Option<TextString>,
    pub tooltip: Option<TextString>,
    #[serde(rename = "flavorType")]
    pub flavor_type: Option<TextString>,
    #[serde(default)]
    pub tip: Vec<TextString>,
    // damage.i_damage
    pub damage: u32,
    pub missiles: Option<u32>,
    // shots
    pub shots: Option<u32>,
    // damage.i_shield_piercing
    pub sp: Option<u32>,
    #[serde(rename = "chargeLevels")]
    pub charge_levels: Option<u32>,
    // damage.fire_chance
    #[serde(rename = "fireChance")]
    pub fire_chance: u32,
    // damage.breach_chance
    #[serde(rename = "breachChance")]
    pub breach_chance: u32,
    // cooldown
    pub cooldown: Option<f64>,
    pub power: Option<u32>,
    pub cost: Option<u32>,
    // color
    pub color: Option<Rgb>,
    pub bp: Option<u32>,
    pub rarity: u32,
    // damage.i_ion_damage
    pub ion: Option<u32>,
    // damage.b_hull_buster
    #[serde(rename = "hullBust")]
    pub hull_bust: Option<bool>,
    // damage.i_pers_damage
    #[serde(rename = "persDamage")]
    pub pers_damage: Option<i32>,
    // damage.b_lockdown
    pub lockdown: Option<bool>,
    // damage.i_system_damage
    #[serde(rename = "sysDamage")]
    pub sys_damage: Option<i32>,
    pub speed: Option<u32>,
    // damage.stun_chance
    #[serde(rename = "stunChance")]
    pub stun_chance: Option<u32>,
    #[serde(rename = "iconImage")]
    pub icon_image: Option<String>,
    // length
    pub length: Option<u32>,
    pub image: Option<String>,
    #[serde(rename = "launchSounds")]
    pub launch_sounds: Sounds,
    #[serde(rename = "hitShipSounds")]
    pub hit_ship_sounds: Option<Sounds>,
    #[serde(rename = "hitShieldSounds")]
    pub hit_shield_sounds: Option<Sounds>,
    #[serde(rename = "missSounds")]
    pub miss_sounds: Option<Sounds>,
    #[serde(rename = "weaponArt")]
    pub weapon_art: String,
    pub explosion: Option<String>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub drone_targetable: Vec<bool>,
    pub radius: Option<u32>,
    // damage.i_stun
    pub stun: Option<u32>,
    pub boost: Option<Boost>,
    pub spin: Option<u32>,
    pub projectiles: Option<Projectiles>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DroneBlueprint {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(default, rename = "@NOLOC")]
    pub noloc: bool,
    #[serde(rename = "type")]
    pub type_: DroneType,
    pub tip: Option<TextString>,
    pub title: TextString,
    pub short: TextString,
    pub desc: TextString,
    pub power: u32,
    pub cooldown: Option<u32>,
    pub dodge: Option<u32>,
    pub speed: Option<u32>,
    pub cost: u32,
    pub bp: Option<u32>,
    #[serde(rename = "droneImage")]
    pub drone_image: Option<String>,
    #[serde(rename = "iconImage")]
    pub icon_image: Option<String>,
    pub image: Option<String>,
    #[serde(rename = "weaponBlueprint")]
    pub weapon_blueprint: Option<String>,
    pub target: Option<DroneTarget>,
    pub rarity: u32,
    pub level: Option<u32>,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AugBlueprint {
    #[serde(rename = "@name")]
    pub name: String,
    pub title: TextString,
    pub desc: TextString,
    pub cost: u32,
    pub bp: Option<u32>,
    pub rarity: u32,
    pub stackable: bool,
    pub value: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ItemBlueprint {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ItemType2,
    pub title: TextString,
    pub desc: TextString,
    pub cost: u32,
    pub rarity: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename = "FTL")]
#[serde(deny_unknown_fields)]
pub struct XmlBlueprints {
    #[serde(default, rename = "blueprintList")]
    pub blueprint_lists: Vec<BlueprintList>,
    #[serde(default, rename = "crewBlueprint")]
    pub crew_blueprints: Vec<CrewBlueprint>,
    #[serde(default, rename = "systemBlueprint")]
    pub system_blueprints: Vec<SystemBlueprint>,
    #[serde(default, rename = "weaponBlueprint")]
    pub weapon_blueprints: Vec<WeaponBlueprint>,
    #[serde(default, rename = "droneBlueprint")]
    pub drone_blueprints: Vec<DroneBlueprint>,
    #[serde(default, rename = "augBlueprint")]
    pub aug_blueprints: Vec<AugBlueprint>,
    #[serde(default, rename = "itemBlueprint")]
    pub item_blueprints: Vec<ItemBlueprint>,
    #[serde(default, rename = "shipBlueprint")]
    pub ship_blueprints: Vec<ShipBlueprint>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum EnvironmentType {
    #[serde(rename = "asteroid")]
    Asteroid,
    #[serde(rename = "pulsar")]
    Pulsar,
    #[serde(rename = "PDS")]
    Pds,
    #[serde(rename = "sun")]
    Sun,
    #[serde(rename = "storm")]
    Storm,
    #[serde(rename = "nebula")]
    Nebula,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum Target {
    Enemy,
    Player,
    All,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Crew {
    #[serde(default, rename = "crewMember")]
    pub crew_member: Vec<CrewMember2>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WeaponOverride {
    #[serde(rename = "@count")]
    pub count: u32,
    #[serde(default, rename = "name")]
    pub names: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Ship {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@blueprint")]
    pub blueprint: Option<String>,
    #[serde(rename = "@auto_blueprint")]
    pub auto_blueprint: Option<String>,
    pub surrender: Option<Event>,
    #[serde(default)]
    pub escape: Vec<Event>,
    pub gotaway: Option<Event>,
    pub destroyed: Option<Event>,
    #[serde(rename = "deadCrew")]
    pub dead_crew: Option<Event>,
    pub crew: Option<Crew>,
    #[serde(rename = "weaponOverride")]
    pub weapon_override: Option<WeaponOverride>,
}

fn case_insensitive_de<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::de::Error;
    let x: &'de str = <&'de str as serde::Deserialize>::deserialize(d)?;
    match x {
        "1" => Ok(true),
        "0" => Ok(true),
        "true" => Ok(true),
        "false" => Ok(false),
        "TRUE" => Ok(true),
        "FALSE" => Ok(false),
        _ => Err(D::Error::invalid_value(
            serde::de::Unexpected::Str(x),
            &"a boolean",
        )),
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ShipRef {
    #[serde(rename = "@load")]
    pub load: Option<String>,
    #[serde(default, rename = "@hostile", deserialize_with = "case_insensitive_de")]
    pub hostile: bool,
    #[serde(flatten)]
    pub ship: Option<Box<Ship>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    #[serde(rename = "@type")]
    pub type_: EnvironmentType,
    #[serde(rename = "@target")]
    pub target: Option<Target>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TextList {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(default, rename = "@unique")]
    pub unique: bool,
    #[serde(default)]
    pub text: Vec<TextString>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum SystemName {
    Shields,
    #[serde(alias = "engine")]
    Engines,
    Oxygen,
    Weapons,
    Drones,
    Medbay,
    Pilot,
    Sensors,
    Doors,
    Teleporter,
    Cloaking,
    Artillery,
    Battery,
    Clonebay,
    Mind,
    Hacking,
    Reactor,
    Random,
    Room,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum DamageEffect {
    None,
    Fire,
    Breach,
    Random,
    All,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum BoostType {
    Damage,
    Cooldown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum ItemType {
    Drones,
    Scrap,
    #[serde(alias = "missile")]
    Missiles,
    Fuel,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum ItemType2 {
    #[serde(rename = "ITEM_DRONE")]
    Drones,
    #[serde(rename = "ITEM_MISSILE")]
    Missiles,
    #[serde(rename = "ITEM_FUEL")]
    Fuel,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Item {
    #[serde(rename = "@type")]
    pub type_: ItemType,
    #[serde(rename = "@min")]
    pub min: i32,
    #[serde(rename = "@max")]
    pub max: i32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    #[serde(rename = "@req")]
    pub req: Option<String>,
    #[serde(default, rename = "@hidden")]
    pub hidden: bool,
    #[serde(default, rename = "@hiiden")]
    pub hiiden: bool,
    #[serde(default, rename = "@blue")]
    pub blue: bool,
    #[serde(rename = "@lvl")]
    pub lvl: Option<u32>,
    #[serde(rename = "@max_lvl")]
    pub max_lvl: Option<u32>,
    #[serde(rename = "@min_level")]
    pub min_level: Option<u32>,
    #[serde(rename = "@max_group")]
    pub max_group: Option<u32>,
    pub text: TextString,
    pub event: Event,
    pub choice: Option<Box<Choice>>,
    #[serde(rename = "autoReward")]
    pub auto_reward: Option<AutoReward>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Items {
    #[serde(rename = "item")]
    pub items: Vec<Item>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "UPPERCASE")]
pub enum AutoRewardLevel {
    #[serde(alias = "low")]
    Low,
    #[serde(alias = "MEDIUM")]
    Med,
    High,
    Random,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "UPPERCASE")]
pub enum WeaponType {
    Laser = 0,
    Beam = 2,
    Missiles = 1,
    Bomb = 3,
    Burst = 4,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DroneType {
    // shoots down incoming projectiles
    Defense = 0,
    // attacks the enemy ship
    Combat = 1,
    // crew member, repairs
    Repair = 2,
    // crew member, kills
    Battle = 3,
    // no teleporter needed, breaches through
    Boarder = 4,
    // repairs hull and dies via SetDestroyed
    ShipRepair = 5,
    // internal
    Hacking = 6,
    // supershield
    Shield = 7,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DroneTarget {
    Lasers,
    Drones,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AutoReward {
    #[serde(rename = "@level")]
    pub level: AutoRewardLevel,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CrewMember {
    #[serde(rename = "@amount")]
    pub amount: i32,
    #[serde(rename = "@class")]
    pub class: Option<String>,
    #[serde(rename = "@type")]
    pub type_: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@weapons")]
    pub weapons: Option<u32>,
    #[serde(rename = "@shields")]
    pub shields: Option<u32>,
    #[serde(rename = "@pilot")]
    pub pilot: Option<u32>,
    #[serde(rename = "@engines")]
    pub engines: Option<u32>,
    #[serde(rename = "@combat")]
    pub combat: Option<u32>,
    #[serde(rename = "@repair")]
    pub repair: Option<u32>,
    #[serde(rename = "@all_skills")]
    pub all_skills: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CrewMember2 {
    #[serde(rename = "@class")]
    pub class: Option<String>,
    #[serde(rename = "@prop")]
    pub prop: Option<f64>,
    #[serde(rename = "@type")]
    pub type_: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Boarders {
    #[serde(rename = "@min")]
    pub min: u32,
    #[serde(rename = "@max")]
    pub max: u32,
    #[serde(rename = "@class")]
    pub class: Option<String>,
    #[serde(default, rename = "@breach")]
    pub breach: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RemoveCrew {
    #[serde(rename = "@class")]
    pub class: Option<String>,
    pub clone: bool,
    pub text: TextString,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ItemModify {
    #[serde(default, rename = "@steal")]
    pub steal: bool,
    #[serde(default)]
    pub item: Vec<Item>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Image {
    #[serde(rename = "@w")]
    pub w: u32,
    #[serde(rename = "@h")]
    pub h: u32,
    #[serde(rename = "$text")]
    pub path: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ImageList {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@ui")]
    pub ui: Option<String>,
    pub img: Vec<Image>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventImage {
    #[serde(rename = "@back")]
    pub back: Option<String>,
    #[serde(rename = "@planet")]
    pub planet: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Damage {
    #[serde(rename = "@amount")]
    pub amount: i32,
    #[serde(rename = "@system")]
    pub system: Option<SystemName>,
    #[serde(rename = "@effect")]
    pub effect: Option<DamageEffect>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Quest {
    #[serde(rename = "@event")]
    pub amount: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum StatusType {
    Limit,
    Divide,
    Clear,
    Loss,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Status {
    #[serde(rename = "@type")]
    pub type_: StatusType,
    #[serde(rename = "@target")]
    pub target: Target,
    #[serde(rename = "@system")]
    pub system: SystemName,
    #[serde(rename = "@amount")]
    pub amount: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Upgrade {
    #[serde(rename = "@system")]
    pub system: SystemName,
    #[serde(rename = "@amount")]
    pub amount: u32,
}

#[derive(Deserialize, Serialize, Debug, Default, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UnlockShip {
    #[serde(rename = "@id")]
    pub id: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Event {
    #[serde(rename = "@load")]
    pub load: Option<String>,
    #[serde(rename = "@chance")]
    pub chance: Option<f64>,
    #[serde(rename = "@timer")]
    pub timer: Option<u32>,
    #[serde(rename = "@min")]
    pub min: Option<u32>,
    #[serde(rename = "@max")]
    pub max: Option<u32>,
    #[serde(rename = "@loss1")]
    pub loss1: Option<String>,

    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@unique")]
    pub unique: Option<bool>,
    pub text: Option<TextString>,
    pub img: Option<EventImage>,
    pub ship: Option<ShipRef>,
    pub environment: Option<Environment>,
    pub item_modify: Option<ItemModify>,
    pub weapon: Option<BlueprintRef>,
    pub drone: Option<BlueprintRef>,
    pub augment: Option<BlueprintRef>,
    pub remove: Option<BlueprintRef>,
    pub quest: Option<Quest>,
    #[serde(default)]
    pub status: Vec<Status>,
    pub fleet: Option<String>,
    #[serde(rename = "modifyPursuit")]
    pub modiy_pursuit: Option<Amount>,
    #[serde(rename = "crewMember")]
    pub crew_member: Option<CrewMember>,
    pub boarders: Option<Boarders>,
    #[serde(default)]
    pub damage: Vec<Damage>,
    #[serde(rename = "removeCrew")]
    pub remove_crew: Option<RemoveCrew>,
    #[serde(rename = "autoReward")]
    pub auto_reward: Option<AutoReward>,
    pub store: Option<()>,
    pub repair: Option<()>,
    pub upgrade: Option<Upgrade>,
    #[serde(default, rename = "distressBeacon")]
    pub distress_beacon: Option<()>,
    #[serde(default, rename = "secretSector")]
    pub secret_sector: Option<()>,
    pub reveal_map: Option<()>,
    #[serde(rename = "unlockShip")]
    pub unlock_ship: Option<()>,
    #[serde(default, rename = "choice")]
    pub choices: Vec<Choice>,
    pub event: Option<Box<Event>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventList {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(default, rename = "event")]
    pub events: Vec<Event>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventCount {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@min")]
    pub min: u32,
    #[serde(rename = "@max")]
    pub max: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventCounts {
    #[serde(rename = "@sector")]
    pub sector: u32,
    #[serde(default, rename = "event")]
    pub events: Vec<EventCount>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename = "FTL")]
#[serde(deny_unknown_fields)]
pub struct XmlEvents {
    #[serde(default, rename = "event")]
    pub events: Vec<Event>,
    #[serde(default, rename = "eventList")]
    pub event_lists: Vec<EventList>,
    #[serde(default, rename = "textList")]
    pub text_lists: Vec<TextList>,
    #[serde(default, rename = "ship")]
    pub ships: Vec<Ship>,
    #[serde(default, rename = "eventCounts")]
    pub event_counts: Vec<EventCounts>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sound {
    #[serde(rename = "@volume")]
    pub volume: u32,
    #[serde(default, rename = "@loop")]
    pub loop_: bool,
    #[serde(rename = "@count")]
    pub count: Option<u32>,
    #[serde(rename = "$text")]
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename = "FTL")]
#[serde(deny_unknown_fields)]
pub struct XmlSounds {
    #[serde(rename = "cloneArrive")]
    pub clone_arrive: Sound,
    #[serde(rename = "cloneBroken")]
    pub clone_broken: Sound,
    #[serde(rename = "batteryStop")]
    pub battery_stop: Sound,
    #[serde(rename = "batteryStart")]
    pub battery_start: Sound,
    #[serde(rename = "mindControl")]
    pub mind_control: Sound,
    #[serde(rename = "mindControlEnd")]
    pub mind_control_end: Sound,
    #[serde(rename = "pulsar")]
    pub pulsar: Sound,
    #[serde(rename = "hackLand")]
    pub hack_land: Sound,
    #[serde(rename = "hackLoop")]
    pub hack_loop: Sound,
    #[serde(rename = "hackStart")]
    pub hack_start: Sound,
    #[serde(rename = "shrikeDeath")]
    pub shrike_death: Sound,
    #[serde(rename = "shrikeRepair")]
    pub shrike_repair: Sound,
    #[serde(rename = "flakImpact1")]
    pub flak_impact1: Sound,
    #[serde(rename = "flakImpact2")]
    pub flak_impact2: Sound,
    #[serde(rename = "flakImpact3")]
    pub flak_impact3: Sound,
    #[serde(rename = "flakImpact4")]
    pub flak_impact4: Sound,
    #[serde(rename = "crystalExplosion")]
    pub crystal_explosion: Sound,
    #[serde(rename = "crystalExplosion_small")]
    pub crystal_explosion_small: Sound,
    #[serde(rename = "fleetFireHeavy")]
    pub fleet_fire_heavy: Sound,
    #[serde(rename = "fleetFireLight")]
    pub fleet_fire_light: Sound,
    #[serde(rename = "ionBoarderCharge")]
    pub ion_boarder_charge: Sound,
    #[serde(rename = "shieldDroneCharge")]
    pub shield_drone_charge: Sound,
    #[serde(rename = "shieldDroneActivate")]
    pub shield_drone_activate: Sound,
    #[serde(rename = "weaponReady")]
    pub weapon_ready: Sound,
    #[serde(rename = "hullBreach")]
    pub hull_breach: Sound,
    #[serde(rename = "eventDamage")]
    pub event_damage: Sound,
}

#[cfg(test)]
mod test {
    #[test]
    fn test_text() {
        let text = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <FTL>
            <text name="a">Test 1</text>
            <text name="b" language="test">Test 2</text>
            </FTL>
        "#;
        let x: super::XmlText = quick_xml::de::from_str(text).unwrap();
        assert_eq!(
            &x.text,
            &[
                super::Text {
                    name: "a".to_owned(),
                    value: "Test 1".to_owned(),
                    language: None,
                },
                super::Text {
                    name: "b".to_owned(),
                    value: "Test 2".to_owned(),
                    language: Some("test".to_owned()),
                },
            ]
        );
    }

    #[test]
    fn test_achievements() {
        let text = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <FTL>
            <achievement id="ACH_UNLOCK_ALL">
              <name id="ACH_UNLOCK_ALL_name"/>
              <desc id="ACH_UNLOCK_ALL_desc"/>
              <img>achievements/4.png</img>
            </achievement>
            <achievement id="ACH_ROCK_MISSILES">
              <name id="ACH_ROCK_MISSILES_name"/>
              <shortname id="ACH_ROCK_MISSILES_shortname"/>
              <desc id="ACH_ROCK_MISSILES_desc"/>
              <img>achievements/rock_cruiser_2.png</img>
              <ship>PLAYER_SHIP_ROCK</ship>
              <multiDifficulty>1</multiDifficulty>
            </achievement>
            </FTL>
        "#;
        let x: super::XmlAchievements = quick_xml::de::from_str(text).unwrap();
        assert_eq!(
            &x.achievements,
            &[
                super::Achievement {
                    id: "ACH_UNLOCK_ALL".to_owned(),
                    name: super::TextString {
                        id: Some("ACH_UNLOCK_ALL_name".to_owned()),
                        ..Default::default()
                    },
                    desc: super::TextString {
                        id: Some("ACH_UNLOCK_ALL_desc".to_owned()),
                        ..Default::default()
                    },
                    img: "achievements/4.png".to_owned(),
                    shortname: None,
                    ship: None,
                    multi_difficulty: false,
                },
                super::Achievement {
                    id: "ACH_ROCK_MISSILES".to_owned(),
                    name: super::TextString {
                        id: Some("ACH_ROCK_MISSILES_name".to_owned()),
                        ..Default::default()
                    },
                    shortname: Some(super::TextString {
                        id: Some("ACH_ROCK_MISSILES_shortname".to_owned()),
                        ..Default::default()
                    }),
                    desc: super::TextString {
                        id: Some("ACH_ROCK_MISSILES_desc".to_owned()),
                        ..Default::default()
                    },
                    img: "achievements/rock_cruiser_2.png".to_owned(),
                    ship: Some("PLAYER_SHIP_ROCK".to_owned()),
                    multi_difficulty: true,
                }
            ]
        );
    }

    #[test]
    fn test_ship() {
        let text = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <FTL>
            <img x="-99" y="-44" w="443" h="354"/>
            <glowOffset x="25" y="26"/>
            <offsets><cloak x="17" y="17"/></offsets>
            <weaponMounts>
              <mount x="150" y="25" rotate="false" mirror="true" gib="2" slide="up"/>
              <mount x="295" y="25" rotate="false" mirror="false" gib="3" slide="up"/>
            </weaponMounts>
            <explosion>
              <gib1>
                <velocity min="0.1" max="1"/>
                <direction min="180" max="240"/>
                <angular min="0" max="1"/>
                <glowOffset x="28" y="25"/>
                <x>213</x>
                <y>49</y>
              </gib1>
              <gib2>
                <velocity min="0.4" max="0.8"/>
                <direction min="-20" max="60"/>
                <angular min="-0.6" max="-0.1"/>
                <glowOffset x="25" y="26"/>
                <x>0</x>
                <y>0</y>
              </gib2>
              <gib3>
                <velocity min="0.4" max="1"/>
                <direction min="280" max="350"/>
                <angular min="0.1" max="0.5"/>
                <glowOffset x="26" y="26"/>
                <x>159</x>
                <y>0</y>
              </gib3>
              <gib4>
                <velocity min="0.1" max="0.3"/>
                <direction min="90" max="180"/>
                <angular min="-0.5" max="0.5"/>
                <glowOffset x="24" y="27"/>
                <x>50</x>
                <y>57</y>
              </gib4>
            </explosion>
            </FTL>
        "#;
        let _: super::XmlShip = quick_xml::de::from_str(text).unwrap();
    }

    #[test]
    fn test_anim() {
        let text = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <FTL>
            <animSheet name="fire_large" w="256" h="32" fw="32" fh="32">effects/fire_L1_strip8.png</animSheet>
            <animSheet name="fire_small" w="256" h="32" fw="32" fh="32">effects/fire_S1_strip8.png</animSheet>
            <anim name="fire_large">
              <sheet>fire_large</sheet>
              <desc length="8" x="0" y="0"/>
              <time>1.0</time>
            </anim>
            <weaponAnim name="artillery_fed">
              <sheet>artillery_fed</sheet>
              <desc length="10" x="0" y="0"/>
              <chargedFrame>1</chargedFrame>
              <fireFrame>2</fireFrame>
              <firePoint  x="0" y="50"/>
              <mountPoint x="0" y="50"/>
              <chargeImage>weapons/blank.png</chargeImage>
            </weaponAnim>
            <weaponAnim name="boss_1">
              <sheet>boss_1</sheet>
              <desc length="12" x="0" y="0"/>
              <chargedFrame>5</chargedFrame>
              <fireFrame>8</fireFrame>
              <firePoint  x="16" y="20"/>
              <mountPoint x="0" y="0"/>
            </weaponAnim>
            </FTL>
        "#;

        let _: super::XmlAnim = quick_xml::de::from_str(text).unwrap();
    }

    #[test]
    fn test_blueprints() {
        let text = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <FTL>
            <blueprintList name="DRONES_STANDARD">
              <name>COMBAT_1</name>
              <name>COMBAT_2</name>
            </blueprintList>
            <shipBlueprint name="REBEL_FAT" layout="rebel_squat" img="rebel_squat">
              <class id="ship_REBEL_FAT_class"/>
              <systemList>
                <pilot power="1" max="2" room="0"/>
                <oxygen power="1" max="2" room="2"/>
                <shields power="2" max="8" room="4"/>
                <engines power="2" max="4" room="7"/>
                <weapons power="1" max="6" room="6"/>
                <drones power="2" max="8" room="1"/>
                <medbay power="1" max="3" room="3" start="false"/>
                <doors power="1" room="5" start="true" img="room_doors_4">
                  <slot>
                    <direction>right</direction>
                    <number>0</number>
                  </slot>
		</doors>
                <artillery power="4" room="9" weapon="ARTILLERY_BOSS_1"/>
		<artillery power="4" room="10" weapon="ARTILLERY_BOSS_2"/>
		<artillery power="3" room="14" weapon="ARTILLERY_BOSS_3"/>
              </systemList>
              <droneList drones="4" load="DRONES_STANDARD"/>
              <weaponList missiles="10" load="WEAPONS_REBEL"/>
              <health amount="9"/>
              <maxPower amount ="8"/>
              <crewCount amount = "3" max="5" class="human"/>
              <crewCount amount = "1" class="engi"/>
              <boardingAI>sabotage</boardingAI>
              <aug name="SLUG_GEL"/>
              <cloakImage>rebel_squat</cloakImage>
            </shipBlueprint>
            </FTL>
        "#;

        let _: super::XmlBlueprints = quick_xml::de::from_str(text).unwrap();
    }

    #[test]
    fn test_events() {
        let header = r#"<?xml version="1.0" encoding="utf-8"?><FTL>"#;
        let trailer = r#"</FTL>"#;
        let text = [
            r#"
            <textList name="FRIENDLY_BEACON">
                    <text id="text_FRIENDLY_BEACON_1"/>
                    <text id="text_FRIENDLY_BEACON_2"/>
                    <text id="text_FRIENDLY_BEACON_3"/>
                    <text id="text_FRIENDLY_BEACON_4"/>
                    <text id="text_FRIENDLY_BEACON_5"/>
            </textList>"#,
            r#"
            <eventList name="ASTEROID_EXPLORE_RESULTS">
                    <event>
                            <text id="event_ASTEROID_EXPLORE_RESULTS_1_text"/>
                    </event>
                    <event>
                            <text id="event_ASTEROID_EXPLORE_RESULTS_2_text"/>
                            <autoReward level="HIGH">fuel_only</autoReward>
                    </event>
                    <event>
                            <text id="event_ASTEROID_EXPLORE_RESULTS_3_text"/>
                            <autoReward level="MED">missiles</autoReward>
                    </event>
                    <event>
                            <text id="event_ASTEROID_EXPLORE_RESULTS_4_text"/>
                            <autoReward level="MED">droneparts</autoReward>
                    </event>
                    <event>
                            <text id="event_ASTEROID_EXPLORE_RESULTS_5_text"/>
                            <damage amount="3"/>
                            <damage amount="1" system="random"/>
                            <damage amount="1" system="room" effect="fire"/>
                    </event>
                    <event>
                            <text id="event_ASTEROID_EXPLORE_RESULTS_6_text"/>
                            <ship load="PIRATE" hostile="true"/>
                            <environment type="asteroid" target="player"/>
                    </event>
            </eventList>"#,
            r#"
            <event name="STRANDED_BEACON" unique="true">
                    <text id="event_STRANDED_BEACON_text"/>
                    <distressBeacon/>
                    <choice hidden="true">
                            <text id="event_STRANDED_BEACON_c1_choice"/>
                            <event load="STRANDED"/>
                    </choice>
                    <choice>
                            <text id="event_STRANDED_BEACON_c2_choice"/>
                            <event/>
                    </choice>
            </event>"#,
            r#"
            <event name="BOARDERS" unique="true">
                    <text load="BOARDERS_TEXT"/>
                    <boarders min="3" max="5" class="human"/>
            </event>"#,
            r#"
            <event>
                    <text id="event_STATION_SICK_DRONE_LIST_1_text"/>
                    <item_modify>
                            <item type="drones" min="-1" max="-1"/>
                    </item_modify>
            </event>"#,
            r#"
            <event>
                    <text id="event_REPAIR_STATION_c1_text"/>
                    <item_modify>
                            <item type="scrap" min="-40" max="-40"/>
                    </item_modify>
                    <damage amount="-20"/>
            </event>"#,
            r#"
            <event>
                    <text id="event_STATION_SICK_LIST_3_c1_text"/>
                    <boarders min="3" max="4" class="human"/>
                    <crewMember amount="-1" class="traitor"/>
            </event>"#,
            r#"
            <event>
                    <text id="event_STATION_SICK_LIST_2_c2_text"/>
                    <crewMember repair="1" amount="1"/>
                    <autoReward level="MED">scrap_only</autoReward>
            </event>"#,
            r#"
            <ship name="DONOR_MANTIS_CHASE2" auto_blueprint="MANTIS_BOMBER">
                    <escape timer="12" min="6" max="6">
                            <text id="ship_DONOR_MANTIS_CHASE2_escape_text"/>
                    </escape>
                    <gotaway>
                            <text id="ship_DONOR_MANTIS_CHASE2_gotaway_text"/>
                            <autoReward level="HIGH">standard</autoReward>
                    </gotaway>
                    <surrender  min="2" max="2">
                            <text id="ship_DONOR_MANTIS_CHASE2_surrender_text"/>
                            <choice>
                                    <text id="ship_DONOR_MANTIS_CHASE2_surrender_c1_choice"/>
                                    <event>
                                            <text id="ship_DONOR_MANTIS_CHASE2_surrender_c1_text"/>
                                            <autoReward level="HIGH">weapon</autoReward>
                                            <ship hostile="false"/>
                                    </event>
                            </choice>
                            <choice>
                                    <text id="ship_DONOR_MANTIS_CHASE2_surrender_c2_choice"/>
                                    <event>
                                            <text id="ship_DONOR_MANTIS_CHASE2_surrender_c2_text"/>
                                    </event>
                            </choice>
                    </surrender>
                    <destroyed>
                            <text id="ship_DONOR_MANTIS_CHASE2_destroyed_text"/>
                            <weapon name="RANDOM"/>
                            <autoReward level="MED">standard</autoReward>
                    </destroyed>
                    <deadCrew>
                            <text id="ship_DONOR_MANTIS_CHASE2_deadCrew_text"/>
                            <weapon name="RANDOM"/>
                            <autoReward level="HIGH">standard</autoReward>
                    </deadCrew>
            </ship>"#,
            r#"
            <event>
                    <text id="event_ZOLTAN_CREW_STUDY_c3_text"/>
                    <drone name="RANDOM"/>
                    <autoReward level="LOW">stuff</autoReward>
            </event>"#,
            r#"
            <event>
                    <remove name="STASIS_POD"/>
                    <text id="event_ZOLTAN_CREW_STUDY_c4_text"/>
                    <choice hidden="true">
                            <text id="continue"/>
                            <event>
                                    <text id="event_ZOLTAN_CREW_STUDY_c4_c1_text"/>
                                    <choice hidden="true">
                                            <text id="continue"/>
                                            <event>
                                                    <text id="event_ZOLTAN_CREW_STUDY_c4_c1_c1_text"/>
                                                    <crewMember amount="1" class="crystal" id="name_Ruwen"/>
                                            </event>
                                    </choice>
                            </event>
                    </choice>
            </event>"#,
            r#"
            <event load="ROCK_CRYSTAL_BEACON_LIST"/>"#,
            r#"
            <event>
                    <text id="event_ROCK_CRYSTAL_BEACON_LIST_2_text"/>
                    <ship load="ROCK_SHIP" hostile="true"/>
            </event>"#,
            r#"
            <event>
                    <text id="ship_DONOR_MANTIS_CHASE1_gotaway_c1_text"/>
                    <quest event="DONOR_MANTIS_CHASE2"/>
            </event>"#,
            r#"
            <event>
                    <text id="event_ALISON_DEFECTOR_HELP_6_text"/>
                    <boarders min="1" max="1" class="human"/>
                    <removeCrew>
                            <clone>true</clone>
                            <text id="event_ALISON_DEFECTOR_HELP_6_c0_clone"/>
                    </removeCrew>
            </event>"#,
            r#"
            <event name="STORE">
                    <text load="STORE_TEXT"/>
                    <store/>
            </event>"#,
            r#"
            <event>
                    <text id="event_ALISON_MANTIS_CREW_REJECT_2_text"/>
                    <autoReward level="HIGH">scrap_only</autoReward>
                    <damage amount="4"/>
                    <damage amount="1" system="room" effect="fire"/>
            </event>"#,
            r#"
            <event>
                    <text id="event_STRANDED_CHARLIES_4_text"/>
                    <crewMember engines="1" amount="1" id="name_Charlie"/>
            </event>"#,
            r#"
            <ship name="MANTIS_LANDING_PARTY" auto_blueprint="SHIPS_MANTIS">
                    <destroyed>
                            <text id="ship_MANTIS_LANDING_PARTY_destroyed_text"/>
                            <autoReward level="MED">standard</autoReward>
                    </destroyed>
                    <deadCrew>
                            <text id="ship_MANTIS_LANDING_PARTY_deadCrew_text"/>
                            <autoReward level="HIGH">standard</autoReward>
                    </deadCrew>
                    <crew>
                               <crewMember type="mantis" prop="0.80"/>
                               <crewMember type="engi" prop="0.20"/>
                    </crew>
            </ship>"#,
            r#"
            <event>
                    <distressBeacon/>
                    <text id="event_CIVILIAN_ASTEROIDS_BEACON_LIST4_1_text"/>
                    <modifyPursuit amount="1"/>
            </event>"#,
            r#"
            <event>
                    <text id="event_BOARDERS_HACKING_c1_text"/>
                    <status type="limit" target="player" system="sensors" amount="0"/>
            </event>"#,
            r#"
            <event name = "FLEET_EASY_BEACON_DLC">
                    <fleet>rebel</fleet>
                    <text id="event_FLEET_EASY_BEACON_DLC_text"/>
                    <ship load="LONG_FLEET" hostile ="true"/>
                    <environment type="PDS" target="player"/>
            </event>"#,
            r#"
            <event>
                    <text id="event_ASTEROID_EXPLORE_RESULTS_5_text"/>
                    <damage amount="3"/>
                    <damage amount="1" system="random"/> 
                    <damage amount="1" system="room" effect="fire"/>
            </event>"#,
            r#"
            <event>
                    <text id="event_MERCENARY_c2_text"/>
                    <item_modify>
                            <item type="scrap" min="-20" max="-10"/>
                    </item_modify>
                    <reveal_map/>
            </event>"#,
            r#"
            <event>
                    <text id="event_RANDOM_GIFT_5_text"/>
                    <upgrade amount="1" system="reactor"/>
            </event>"#,
            r#"
            <event name="BOARDER_TEST">
                    <text>Oh no enemies!</text>
                    <boarders min="3" max="5" class="human"/>
                    <ship load="PIRATE"/>
            </event>"#,
            r#"
            <event>
                    <secretSector/>
            </event>"#,
            r#"
            <event name="TEST_EVENT2">
                    <text>Limited systems!</text>
                    <status type="limit" target="player" system="sensors" amount="1"/>
                    <status type="limit" target="player" system="doors" amount="1"/>
                    <status type="limit" target="player" system="engines" amount="1"/>
                    <status type="limit" target="player" system="weapons" amount="1"/>
            </event>"#,
            r#"
            <event/>"#,
            r#"
            <event>
                    <text id="event_STRANDED_2_c4_text"/>
                    <crewMember amount="1" all_skills="1" id="name_Charlie"/>
            </event>
            <event/>
            "#,
            r#"
            <eventList name="HOSTILE_ROCK">
                    <event load="ROCK_SHIP"/>
                    <event load="ROCK_PIRATE"/>
                    <event load="ROCK_FIGHT_ASTEROID"/>
                    <event load="ROCK_PIRATE_ASTEROID"/>
                    <event load="ROCK_PIRATE_SUN"/>
            </eventList>
            "#,
        ];
        for elem in text {
            let text = header.to_owned() + elem + trailer;
            if let Err(err) = quick_xml::de::from_str::<super::XmlEvents>(&text) {
                eprintln!("{text}");
                panic!("{err}");
            }
        }
    }
}
