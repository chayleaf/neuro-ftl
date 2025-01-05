#[derive(Debug)]
pub struct CrewContext {
    pub name: String,
}

#[derive(Debug)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Debug)]
pub struct ShipBuilderContext {
    pub ship_name: String,
    pub crew_members: Vec<CrewContext>,
    pub advanced_edition: bool,
    pub difficulty: Difficulty,
}

#[derive(Debug)]
pub struct PowerEffect {
    pub required_power: usize,
    pub currently_active: bool,
    pub purchased: bool,
    pub scrap_cost: usize,
    pub effect: &'static str,
}

#[derive(Debug)]
pub enum SystemSpecificContext {
    Shields {},
    Engine {},
    Oxygen {},
    Weapons {},
    Drones {},
    Medbay {},
    Pilot {},
    Sensors {},
    Doors {},
    Teleporter {},
    Cloaking {},
    Artillery {},
    Battery {},
    Clonebay {},
    Mind {},
    Hacking {},
    Reactor {},
    Room {},
}

pub struct SystemContext {
    id: &'static str,
    // #[serde(flatten)]
    inner: SystemSpecificContext,
    // true if bNeedsManned or system id == 6
    requires_manning: bool,
    // iActiveManned
    manned: bool,
    needs_power: bool,
    allocated_power: i32,
    current_level: i32,
    max_level: i32,
}

impl SystemSpecificContext {
    pub fn shields(allocated_power: i32, max_power: i32) -> Self {
        Self::Shields {
            id: "shields",
            allocated_power,
            max_power,
            max_power_with_upgrades: 8,
        }
    }
    pub fn engine() -> Self {
        Self::Engine { id: "engine" }
    }
    pub fn oxygen() -> Self {
        Self::Oxygen { id: "oxygen" }
    }
    pub fn weapons() -> Self {
        Self::Weapons { id: "weapons" }
    }
    pub fn drones() -> Self {
        Self::Drones { id: "drones" }
    }
    pub fn medbay() -> Self {
        Self::Medbay { id: "medbay" }
    }
    pub fn pilot() -> Self {
        Self::Pilot { id: "pilot" }
    }
    pub fn sensors() -> Self {
        Self::Sensors { id: "sensors" }
    }
    pub fn doors() -> Self {
        Self::Doors { id: "doors" }
    }
    pub fn teleporter() -> Self {
        Self::Teleporter { id: "teleporter" }
    }
    pub fn cloaking() -> Self {
        Self::Cloaking { id: "cloaking" }
    }
    pub fn artillery() -> Self {
        Self::Artillery { id: "artillery" }
    }
    pub fn battery() -> Self {
        Self::Battery { id: "battery" }
    }
    pub fn clonebay() -> Self {
        Self::Clonebay { id: "clonebay" }
    }
    pub fn mind() -> Self {
        Self::Mind { id: "mind" }
    }
    pub fn hacking() -> Self {
        Self::Hacking { id: "hacking" }
    }
    pub fn reactor() -> Self {
        Self::Reactor { id: "reactor" }
    }
    pub fn room() -> Self {
        Self::Room { id: "room" }
    }
}

#[derive(Debug)]
pub struct GameContext {
    pub ship_name: String,
    pub crew_members: Vec<CrewContext>,
    pub systems: Vec<SystemSpecificContext>,
}

#[derive(Debug)]
pub enum Context {
    MainMenu,
    ShipBuilder(ShipBuilderContext),
}
