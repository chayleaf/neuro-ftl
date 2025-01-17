pub const BUG: &str = "??? BUG IN THE MOD";

pub const SENSORS_LEVEL1: &str = "See all room info in the player ship. Without this, only rooms with your crew members are shown.";
pub const SENSORS_LEVEL2: &str = "See all room and crew info in the enemy ship";
pub const SENSORS_LEVEL3: &str = "See enemy weapon and drone info";
pub const SENSORS_LEVEL4: &str = "See enemy system info";

pub const LOC_BASE: &str =
    "This is the Federation Base. You have to protect it from the rebels' flagship.";
pub const LOC_EXIT: &str = "This is the exit beacon. Here you can travel to the next sector.";
pub const LOC_REBELS: &str = "The Rebels are about to gain control of this location!";
pub const LOC_STORE: &str = "There is a store at this location.";
pub const LOC_BOSS1: &str = "Rebel Flagship will be here in this many turns.";
pub const LOC_FLEET1: &str = "The Rebels will have control of this location in this many turns.";

pub const LOC_NEBULA_FLEET: &str = "You're inside a nebula. Your sensors will not function.";

pub const TOOLTIP_WEAPONS: &str =
    "Weapons: Activate a weapon to charge and target it to fire. Manning reduces charge time.";
pub const TOOLTIP_REACTOR: &str = "Reactor: Reactor energy powers your systems.";
pub const TOOLTIP_REACTOR_ENEMY: &str = "Reactor: Reactor energy powers enemy systems.";

pub const SKILL_PILOTING: &str = "Evasion bonus percentage when manning Piloting";
pub const SKILL_ENGINES: &str = "Evasion bonus percentage when manning Engines";
pub const SKILL_SHIELDS: &str = "Shield recharge speed bonus percentage when manning Shields";
pub const SKILL_WEAPONS: &str = "Weapons recharge speed bonus percentage when manning Weapons";
pub const SKILL_REPAIRING: &str = "Repair speed, with 100 being the base repair speed";
pub const SKILL_FIGHTING: &str = "Hand-to-hand combat power, with 100 being the base combat power";
pub const SKILL_MOVEMENT: &str = "Movement speed, with 100 being the base movement speed";
pub const SKILL_SUFFOCATION_RES: &str =
    "Suffocation resistance, with 100 being 0 damage from suffocation";
pub const SKILL_LOCKDOWN: &str = "Lockdown power, activate with the `lockdown` action";

pub fn text(s: &str) -> &'static str {
    crate::library().text(s).unwrap()
}
