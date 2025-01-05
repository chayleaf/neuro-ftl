use neuro_ftl_derive::JsonSchemaNoRef;
use neuro_sama::derive::Actions;
use schemars::JsonSchema;
use serde::Deserialize;

// a SystemName without is_referenceable, so it isn't put in $ref, to make the schema simpler
#[derive(Copy, Clone, Debug, Deserialize, JsonSchemaNoRef)]
#[serde(rename_all = "camelCase")]
pub enum SystemName {
    Shields = 0,
    Engines = 1,
    Oxygen = 2,
    Weapons = 3,
    Drones = 4,
    Medbay = 5,
    Pilot = 6,
    Sensors = 7,
    Doors = 8,
    Teleporter = 9,
    Cloaking = 10,
    Artillery = 11,
    Battery = 12,
    Clonebay = 13,
    Mind = 14,
    Hacking = 15,
    Reactor = 17,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SkipCredits;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Continue;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NewGame;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Confirm;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Deny;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MainMenu;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameShip {
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartGame;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameCrew {
    pub crew_member_index: u8,
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose0;
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose1;
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose2;
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose3;
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose4;
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose5;
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose6;
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose7;
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose8;
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Choose9;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IncreasePower {
    pub system: SystemName,
    pub amount: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DecreasePower {
    pub system: SystemName,
    pub amount: u8,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, JsonSchemaNoRef)]
#[serde(rename_all = "camelCase")]
pub enum TargetShip {
    Player,
    Enemy,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetWeaponTargets {
    pub weapon_index: u8,
    pub target_ship: TargetShip,
    pub target_room_ids: Vec<u8>,
    pub autofire: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActivateWeapon {
    pub weapon_index: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeactivateWeapon {
    pub weapon_index: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActivateDrone {
    pub drone_index: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeactivateDrone {
    pub drone_index: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HackSystem {
    pub system: SystemName,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActivateHacking;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MindControl {
    pub target_ship: TargetShip,
    pub target_room_id: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActivateCloaking;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActivateBattery;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TeleportSend {
    pub target_room_id: Option<u8>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TeleportReturn {
    pub source_room_id: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenDoors {
    pub door_ids: Vec<u8>,
    #[serde(default)]
    pub include_airlocks: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CloseDoors {
    pub door_ids: Vec<u8>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlanDoorRoute {
    pub first_room_id: i8,
    pub second_room_id: i8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveCrew {
    pub crew_member_indices: Vec<u8>,
    pub room_id: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShipOverview;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenStore;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Back;

#[derive(Debug, Deserialize, JsonSchemaNoRef)]
#[serde(rename_all = "camelCase")]
pub enum InventorySlotType {
    OverCapacity,
    AugmentationOverCapacity,
    Weapon,
    Cargo,
    Drone,
    Augmentation,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpgradeSystem {
    pub system: SystemName,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UndoUpgrades;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FireCrew {
    pub crew_member_index: u8,
}

#[derive(Debug, Deserialize, JsonSchemaNoRef)]
pub struct InventorySlot {
    pub r#type: InventorySlotType,
    // must be max 3 normally and 2 for augmentations
    pub index: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SwapInventorySlots {
    pub slot1: InventorySlot,
    pub slot2: InventorySlot,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Sell {
    pub slot: InventorySlot,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuyAugmentation {
    pub index: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuyWeapon {
    pub index: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuyDrone {
    pub index: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuyConsumable {
    pub item: ItemType,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuySystem {
    pub system: SystemName,
}

#[derive(Copy, Clone, Debug, Deserialize, JsonSchemaNoRef, Hash, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Jump {
    pub direction: Direction,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChooseNextSector {
    pub direction: Direction,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Starmap;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NextSector;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Repair1;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepairAll;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SwitchStorePage;

#[derive(Debug, Deserialize, JsonSchemaNoRef)]
#[serde(rename_all = "camelCase")]
pub enum ItemType {
    Fuel,
    Missiles,
    DronePart,
}

impl ItemType {
    pub fn blueprint_name(&self) -> &'static str {
        match self {
            Self::Fuel => "fuel",
            Self::Missiles => "missiles",
            Self::DronePart => "drones",
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SellScreen;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuyScreen;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SystemsScreen;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CrewScreen;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InventoryScreen;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PauseGame;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnpauseGame;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Wait {
    #[serde(default)]
    pub distress_signal: bool,
}

#[derive(Actions, Debug)]
pub enum FtlActions {
    /// Skip credits
    #[name = "skip_credits"]
    SkipCredits(SkipCredits),
    /// Continue from an existing save
    #[name = "continue"]
    Continue(Continue),
    /// Start a new game
    #[name = "new_game"]
    NewGame(NewGame),
    /// Confirm the action
    #[name = "confirm"]
    Confirm(Confirm),
    /// Deny the action
    #[name = "deny"]
    Deny(Deny),
    /// Rename the ship
    #[name = "rename_ship"]
    RenameShip(RenameShip),
    /// Rename a crew member
    #[name = "rename_crew"]
    RenameCrew(RenameCrew),
    /// Start game
    #[name = "start_game"]
    StartGame(StartGame),
    /// Go to the main menu
    #[name = "main_menu"]
    MainMenu(MainMenu),
    /// Choose an event option
    #[name = "choose0"]
    Choose0(Choose0),
    /// Choose an event option
    #[name = "choose1"]
    Choose1(Choose1),
    /// Choose an event option
    #[name = "choose2"]
    Choose2(Choose2),
    /// Choose an event option
    #[name = "choose3"]
    Choose3(Choose3),
    /// Choose an event option
    #[name = "choose4"]
    Choose4(Choose4),
    /// Choose an event option
    #[name = "choose5"]
    Choose5(Choose5),
    /// Choose an event option
    #[name = "choose6"]
    Choose6(Choose6),
    /// Choose an event option
    #[name = "choose7"]
    Choose7(Choose7),
    /// Choose an event option
    #[name = "choose8"]
    Choose8(Choose8),
    /// Choose an event option
    #[name = "choose9"]
    Choose9(Choose9),
    /// Increase a system's power
    #[name = "increase_system_power"]
    IncreaseSystemPower(IncreasePower),
    /// Decrease a system's power
    #[name = "decrease_system_power"]
    DecreaseSystemPower(DecreasePower),
    /// Set a weapon's targets. The weapon will fire as soon as it's charged. If autofire is
    /// enabled, the weapon will continue shooting at its target until you deactivate it. Depending
    /// on the weapon, it may consume missiles. Typically, a weapon must have a single target room,
    /// but some weapons require multiple targets to be set (in that case you may pass the same
    /// target multiple times). If the weapon is a beam weapon, the beam will pass through the two
    /// rooms you choose.
    #[name = "set_weapon_targets"]
    SetWeaponTargets(SetWeaponTargets),
    /// Activate a mounted weapon. The weapon will start charging, but will not shoot without a
    /// target.
    #[name = "activate_weapon"]
    ActivateWeapon(ActivateWeapon),
    /// Deactivate a mounted weapon.
    #[name = "deactivate_weapon"]
    DeactivateWeapon(DeactivateWeapon),
    /// Activate a drone. This will consume a drone part if the drone isn't already spawned. The
    /// drone will typically be left behind when you jump into another system.
    #[name = "activate_drone"]
    ActivateDrone(ActivateDrone),
    /// Deactivate a drone. This will not destroy it, it will just depower it.
    #[name = "deactivate_drone"]
    DeactivateDrone(DeactivateDrone),
    /// Launch a hacking drone towards one of the enemy ship's systems, locking the system room
    /// down and periodically allowing you to disrupt the system's function and stun the crew. This
    /// will consume a drone part. You can only launch one hacking drone towards a single ship.
    #[name = "launch_hacking_drone"]
    HackSystem(HackSystem),
    /// Activate the hacking drone, disrupting the system that it's attached to.
    #[name = "activate_hacking"]
    ActivateHacking(ActivateHacking),
    /// Mind control a random enemy crew member in a particular room, temporarily making it your
    /// ally.
    #[name = "mind_control"]
    MindControl(MindControl),
    /// Activate the cloaking system, partially disappearing into another dimension, adding +60% to
    /// evasion and preventing enemy weapons from charging and aiming. This can be activated on a
    /// cooldown.
    #[name = "activate_cloaking"]
    ActivateCloaking(ActivateCloaking),
    /// Activate the battery subsystem, temporarily increasing available reactor power.
    #[name = "activate_battery"]
    ActivateBattery(ActivateBattery),
    /// Use the teleport system, sending everyone in the teleporter room to board a specific room
    /// in the enemy ship. Be careful - if you then destroy the enemy ship, your boarders will die,
    /// and if you jump away, your boarders will be left behind! You can use the `teleport_return`
    /// actions to return everyone to your own ship. If you choose null as the target room, the
    /// room is chosen at random.
    #[name = "teleport_send"]
    TeleportSend(TeleportSend),
    /// Use the teleport system, returning all of your crew from a specific room in the enemy ship
    /// to your own ship.
    #[name = "teleport_return"]
    TeleportReturn(TeleportReturn),
    /// List all the doors along the shortest path between two places in your ship, or space. The
    /// most useful use case for this command is planning a route between a specific room and space
    /// to find out which doors to open for venting oxygen out of a room, which can help you stop
    /// fires. This command takes room IDs, with ID -1 being space.
    #[name = "plan_door_route"]
    PlanDoorRoute(PlanDoorRoute),
    /// Open doors in your ship by their IDs. If you don't pass any doors, all doors in your ship
    /// will be opened, except the ones that lead to space. This can be helpful for rebalancing
    /// oxygen to help crew members not suffocate while repairing breaches, or for venting oxygen
    /// out of a specific room. You can use the `plan_door_route` command to help you find out
    /// which doors to open.
    ///
    /// `include_airlocks` determines whether doors to space are allowed to open (defaults to
    /// `false`).
    #[name = "open_doors"]
    OpenDoors(OpenDoors),
    /// Close doors in your ship by their IDs. If you don't pass any doors, all doors in your ship
    /// will be closed. It's a good idea to have all doors closed by default, because they prevent
    /// breaches from draining oxygen out of your entire ship.
    #[name = "close_doors"]
    CloseDoors(CloseDoors),
    /// Move a crew member to a different room. If they are currently onboard the enemy ship, you
    /// have to pick a room ID from the enemy ship, but by default you have to pick a room ID from
    /// your own ship. You can use this for fighting intruders, reparing breaches, fighting fires,
    /// manning ship systems and subsystems.
    #[name = "move_crew"]
    MoveCrew(MoveCrew),
    /// Go to the ship overview screen, where you can upgrade the ship's reactor, systems and
    /// subsystems, where you can fire and rename crew members and manage your inventory to swap
    /// drones and weapons.
    #[name = "ship_overview"]
    ShipOverview(ShipOverview),
    /// Reopen the store, where you can buy and sell items.
    #[name = "open_store"]
    OpenStore(OpenStore),
    /// Close the current menu, allowing you to return to controlling the ship.
    #[name = "back"]
    Back(Back),
    /// Upgrade a ship system, subsystem, or reactor for scrap.
    #[name = "upgrade_system"]
    UpgradeSystem(UpgradeSystem),
    /// Undo ship upgrades that you just did, refunding the scrap.
    #[name = "undo_upgrades"]
    UndoUpgrades(UndoUpgrades),
    /// Permanently remove a crew member from your ship.
    #[name = "fire_crew"]
    FireCrew(FireCrew),
    /// Swap two inventory slots. This can be used for choosing which weapons and drones your ship
    /// has equipped. The `overCapacity` slot type is used for an extra slot for weapons and drones
    /// that will be deleted when you jump to a different system. The `augmentationOverCapacity`
    /// slot type is the same, but for augments.
    #[name = "swap_inventory_slots"]
    SwapInventorySlots(SwapInventorySlots),
    /// Sell an inventory item. The inventory slot determines which ship inventory item you want to
    /// sell.
    #[name = "sell"]
    Sell(Sell),
    /// Open the starmap, allowing you to move to a different location.
    #[name = "starmap"]
    Starmap(Starmap),
    /// Do a FTL jump to a different star system in a specific direction.
    #[name = "jump"]
    Jump(Jump),
    /// Open the next sector selection, allowing you to progress in the game.
    #[name = "open_next_sector_selection"]
    NextSector(NextSector),
    /// Jump to the next sector, progressing the game.
    #[name = "choose_next_sector"]
    ChooseNextSector(ChooseNextSector),
    /// Repair ship hull for 1 H.
    #[name = "repair_1"]
    Repair1(Repair1),
    /// Repair the entire ship hull.
    #[name = "repair_all"]
    RepairAll(RepairAll),
    /// Buy a system from the shop.
    #[name = "buy_system"]
    BuySystem(BuySystem),
    /// Buy a consumable item from the shop.
    #[name = "buy_consumable"]
    BuyConsumable(BuyConsumable),
    /// Buy a weapon from the shop.
    #[name = "buy_weapon"]
    BuyWeapon(BuyWeapon),
    /// Buy a drone from the shop.
    #[name = "buy_drone"]
    BuyDrone(BuyDrone),
    /// Buy a ship augmentation from the shop.
    #[name = "buy_augmentation"]
    BuyAugmentation(BuyAugmentation),
    /// Switch the store page, this will reveal different items you could buy. There's a total of 2
    /// pages.
    #[name = "switch_store_page"]
    SwitchStorePage(SwitchStorePage),
    /// Switch to selling items, this will allow you to sell items, giving you back half of their
    /// purchase price in scrap.
    #[name = "switch_to_selling"]
    SellScreen(SellScreen),
    /// Switch to buying items, this will allow you to buy items from the store.
    #[name = "switch_to_buying"]
    BuyScreen(BuyScreen),
    /// Switch to the systems screen, this will allow you to upgrade ship systems.
    #[name = "switch_to_systems"]
    SystemsScreen(SystemsScreen),
    /// Switch to the crew screen, this will allow you to rename and remove crew members.
    #[name = "switch_to_crew"]
    CrewScreen(CrewScreen),
    /// Switch to the inventory screen, this will allow you to manage your inventory, equipping
    /// drones and weapons.
    #[name = "switch_to_inventory"]
    InventoryScreen(InventoryScreen),
    /// Pause the game, giving you more time to think about your next actions and micromanage your
    /// crew and weapons.
    #[name = "pause"]
    Pause(PauseGame),
    /// Unpause the game, progressing the game.
    #[name = "unpause"]
    Unpause(UnpauseGame),
    /// Wait for a single turn instead of doing an FTL jump. The rebels' fleet will progress
    /// towards you as usual. If you set `distress_signal` to `true`, your ship will send out a
    /// distress signal, broadcasting your location to ships that pass nearby. If you have no fuel,
    /// this is your only way to get fuel.
    #[name = "skip_turn"]
    Wait(Wait),
}
