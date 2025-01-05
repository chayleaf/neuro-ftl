#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use std::{
    borrow::Cow,
    ffi::{c_char, c_double, c_float, c_int, c_uint, CStr},
    fmt,
    marker::PhantomData,
    mem,
    ops::Range,
    ops::{Deref, DerefMut},
    ptr,
};

use neuro_ftl_derive::{vtable, TestOffsets};

// 0 1 5 13 2 3 4 6 7 8 9 10 11 12 14 15 - systems
// 6 7 8 12 - subsystems
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
pub enum System {
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
    Total = 16,
    Reactor = 17,
    Random = 18,
    Room = 20,
}

impl System {
    pub fn from_id(id: c_int) -> Option<System> {
        Some(match id {
            0 => Self::Shields,
            1 => Self::Engines,
            2 => Self::Oxygen,
            3 => Self::Weapons,
            4 => Self::Drones,
            5 => Self::Medbay,
            6 => Self::Pilot,
            7 => Self::Sensors,
            8 => Self::Doors,
            9 => Self::Teleporter,
            10 => Self::Cloaking,
            11 => Self::Artillery,
            12 => Self::Battery,
            13 => Self::Clonebay,
            14 => Self::Mind,
            15 => Self::Hacking,
            17 => Self::Reactor,
            _ => return None,
        })
    }
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "shields" => Self::Shields,
            "engines" => Self::Engines,
            "oxygen" => Self::Oxygen,
            "weapons" => Self::Weapons,
            "drones" => Self::Drones,
            "medbay" => Self::Medbay,
            "pilot" => Self::Pilot,
            "sensors" => Self::Sensors,
            "doors" => Self::Doors,
            "teleporter" => Self::Teleporter,
            "cloaking" => Self::Cloaking,
            "artillery" => Self::Artillery,
            "battery" => Self::Battery,
            "clonebay" => Self::Clonebay,
            "mind" => Self::Mind,
            "hacking" => Self::Hacking,
            "reactor" => Self::Reactor,
            _ => return None,
        })
    }
    pub fn name(&self) -> &'static str {
        match self {
            Self::Shields => "shields",
            Self::Engines => "engines",
            Self::Oxygen => "oxygen",
            Self::Weapons => "weapons",
            Self::Drones => "drones",
            Self::Medbay => "medbay",
            Self::Pilot => "pilot",
            Self::Sensors => "sensors",
            Self::Doors => "doors",
            Self::Teleporter => "teleporter",
            Self::Cloaking => "cloaking",
            Self::Artillery => "artillery",
            Self::Battery => "battery",
            Self::Clonebay => "clonebay",
            Self::Mind => "mind",
            Self::Hacking => "hacking",
            Self::Reactor => "reactor",
            _ => "invalid system id",
        }
    }
}

impl fmt::Display for System {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[repr(u32)]
pub enum TextEvent {
    Confirm = 0,
    Cancel = 1,
    Clear = 2,
    Backspace = 3,
    Delete = 4,
    Left = 5,
    Right = 6,
    Home = 7,
    End = 8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct JoystickInputEvent {
    #[test_offset = 0x0]
    pub device: c_int,
    #[test_offset = 0x4]
    pub index: c_int,
    #[test_offset = 0x8]
    pub x: c_float,
    #[test_offset = 0xc]
    pub y: c_float,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct KeyboardInputEvent {
    #[test_offset = 0x0]
    pub key: c_int,
    #[test_offset = 0x4]
    pub system_key: c_int,
    #[test_offset = 0x8]
    pub modifiers: c_uint,
    #[test_offset = 0xc]
    pub is_repeat: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct MemoryInputEvent {
    #[test_offset = 0x0]
    pub used_bytes: i64,
    #[test_offset = 0x8]
    pub free_bytes: i64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct MouseInputEvent {
    #[test_offset = 0x0]
    pub x: c_float,
    #[test_offset = 0x4]
    pub y: c_float,
    #[test_offset = 0x8]
    pub scroll: c_float,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct TextInputEvent {
    #[test_offset = 0x0]
    pub ch: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct TouchInputEvent {
    #[test_offset = 0x0]
    pub id: c_uint,
    #[test_offset = 0x4]
    pub x: c_float,
    #[test_offset = 0x8]
    pub y: c_float,
    #[test_offset = 0xc]
    pub initial_x: c_float,
    #[test_offset = 0x10]
    pub initial_y: c_float,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union EventInner {
    pub joystick: JoystickInputEvent,
    pub keyboard: KeyboardInputEvent,
    pub memory: MemoryInputEvent,
    pub mouse: MouseInputEvent,
    pub text: TextInputEvent,
    pub touch: TouchInputEvent,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputEventType {
    Joystick = 1,
    Keyboard = 2,
    Memory = 3,
    Mouse = 4,
    Text = 5,
    Touch = 6,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputEventDetail {
    JoystickConnected = 0x1,
    JoystickDisconnected = 0x2,
    JoystickButtonDown = 0x3,
    JoystickButtonUp = 0x4,
    JoystickDpadChange = 0x5,
    JoystickStickChange = 0x6,
    KeyboardKeyDown = 0x7,
    KeyboardKeyUp = 0x8,
    KeyboardSystemKeyDown = 0x9,
    KeyboardSystemKeyUp = 0xa,
    MemoryLow = 0xb,
    MouseMove = 0xc,
    MouseLmbDown = 0xd,
    MouseLmbUp = 0xe,
    MouseMmbDown = 0xf,
    MouseMmbUp = 0x10,
    MouseRmbDown = 0x11,
    MouseRmbUp = 0x12,
    MouseScrollH = 0x13,
    MouseScrollV = 0x14,
    TextInput = 0x15,
    TextDone = 0x16,
    TextCancelled = 0x17,
    TextClear = 0x18,
    TextBackspace = 0x19,
    TextDelete = 0x1a,
    TextCursorLeft = 0x1b,
    TextCursorRight = 0x1c,
    TextCursorHome = 0x1d,
    TextCursorEnd = 0x1e,
    TouchDown = 0x1f,
    TouchMove = 0x20,
    TouchUp = 0x21,
    TouchCancel = 0x22,
}

#[repr(C)]
#[derive(Copy, Clone, TestOffsets)]
pub struct InputEvent {
    #[test_offset = 0x0]
    pub type_: InputEventType,
    #[test_offset = 0x4]
    pub detail: InputEventDetail,
    #[test_offset = 0x8]
    pub timestamp: c_double,
    #[test_offset = 0x10]
    pub inner: EventInner,
}

pub type EventType = InputEvent;

pub type SDLKey = c_int;

#[vtable]
pub struct VtableCEvent {
    pub dtor: Option<fn(*mut CEvent)>,
    pub delete_dtor: Option<fn(*mut CEvent)>,
    pub on_event: Option<fn(*mut CEvent, *const EventType)>,
    pub on_input_focus: Option<fn(*mut CEvent)>,
    pub on_input_blur: Option<fn(*mut CEvent)>,
    pub on_key_down: Option<fn(*mut CEvent, SDLKey)>,
    pub on_key_up: Option<fn(*mut CEvent, SDLKey)>,
    pub on_text_input: Option<fn(*mut CEvent, c_int)>,
    pub on_text_event: Option<fn(*mut CEvent, TextEvent)>,
    pub on_mouse_move: Option<fn(*mut CEvent, c_int, c_int, c_int, c_int, bool, bool, bool)>,
    pub on_mouse_wheel: Option<fn(*mut CEvent, bool, bool)>,
    pub on_l_button_down: Option<fn(*mut CEvent, c_int, c_int)>,
    pub on_l_button_up: Option<fn(*mut CEvent, c_int, c_int)>,
    pub on_r_button_down: Option<fn(*mut CEvent, c_int, c_int)>,
    pub on_r_button_up: Option<fn(*mut CEvent, c_int, c_int)>,
    pub on_m_button_down: Option<fn(*mut CEvent, c_int, c_int)>,
    pub on_m_button_up: Option<fn(*mut CEvent, c_int, c_int)>,
    pub on_touch_down: Option<fn(*mut CEvent, c_int, c_int, c_int)>,
    pub on_touch_move: Option<fn(*mut CEvent, c_int, c_int, c_int, c_int, c_int)>,
    pub on_touch_up: Option<fn(*mut CEvent, c_int, c_int, c_int, c_int, c_int)>,
    pub on_touch_cancel: Option<fn(*mut CEvent, c_int, c_int, c_int, c_int, c_int)>,
    pub on_request_exit: Option<fn(*mut CEvent)>,
    pub on_exit: Option<fn(*mut CEvent)>,
    pub on_window_resize: Option<fn(*mut CEvent, c_int, c_int)>,
    pub on_language_change: Option<fn(*mut CEvent)>,
}

#[repr(C)]
#[derive(Debug)]
pub struct CEvent {
    pub vtable: *mut VtableCEvent,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct UnlockArrow {
    #[test_offset = 0x0]
    pub direction: c_int,
    #[test_offset = 0x4]
    pub status: c_int,
    #[test_offset = 0x8]
    pub shape: Rect,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipButton {
    #[test_offset = 0x0]
    pub base: Button,
    #[test_offset = 0x90]
    pub i_ship_image: *mut GL_Texture,
    #[test_offset = 0x98]
    pub b_ship_locked: bool,
    #[test_offset = 0x99]
    pub b_layout_locked: bool,
    #[test_offset = 0x9a]
    pub b_no_exist: bool,
    #[test_offset = 0xa0]
    pub achievements: Vector<*mut CAchievement>,
    #[test_offset = 0xb8]
    pub i_selected_ach: c_int,
    #[test_offset = 0xbc]
    pub b_selected: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipSelect {
    #[test_offset = 0x0]
    pub position: Point,
    #[test_offset = 0x8]
    pub title_pos: Point,
    #[test_offset = 0x10]
    pub ship_list_base: Vector<*mut GL_Primitive>,
    #[test_offset = 0x28]
    pub ship_buttons: Vector<*mut ShipButton>,
    #[test_offset = 0x40]
    pub arrows: Vector<UnlockArrow>,
    #[test_offset = 0x58]
    pub b_open: bool,
    #[test_offset = 0x5c]
    pub selected_ship: c_int,
    #[test_offset = 0x60]
    pub info_box: InfoBox,
    #[test_offset = 0x138]
    pub current_type: c_int,
    #[test_offset = 0x140]
    pub type_a: TextButton,
    #[test_offset = 0x240]
    pub type_b: TextButton,
    #[test_offset = 0x340]
    pub type_c: TextButton,
    #[test_offset = 0x440]
    pub confirm: TextButton,
    #[test_offset = 0x540]
    pub b_confirmed: bool,
    #[test_offset = 0x544]
    pub active_touch: c_int,
    #[test_offset = 0x548]
    pub tutorial: ChoiceBox,
    #[test_offset = 0x760]
    pub tutorial_page: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemCustomBox {
    #[test_offset = 0x0]
    pub base: SystemBox,
    #[test_offset = 0x268]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x270]
    pub button: Button,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewCustomizeBox {
    #[test_offset = 0x0]
    pub base: CrewEquipBox,
    #[test_offset = 0x338]
    pub customize_button: TextButton,
    #[test_offset = 0x438]
    pub b_customizing: bool,
    #[test_offset = 0x43c]
    pub customize_location: Point,
    #[test_offset = 0x448]
    pub accept_button: TextButton,
    #[test_offset = 0x548]
    pub big_rename_button: TextButton,
    #[test_offset = 0x648]
    pub left_button: Button,
    #[test_offset = 0x6d8]
    pub right_button: Button,
    #[test_offset = 0x768]
    pub b_renaming: bool,
    #[test_offset = 0x769]
    pub have_customize_touch: bool,
    #[test_offset = 0x76a]
    pub customize_activated: bool,
    #[test_offset = 0x770]
    pub box_: *mut GL_Primitive,
    #[test_offset = 0x778]
    pub box_on: *mut GL_Primitive,
    #[test_offset = 0x780]
    pub big_box: *mut GL_Texture,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipBuilder {
    #[test_offset = 0x0]
    pub current_ship: *mut ShipManager,
    #[test_offset = 0x8]
    pub name_box_primitive: *mut GL_Primitive,
    #[test_offset = 0x10]
    pub enable_advanced_primitive: *mut GL_Primitive,
    #[test_offset = 0x18]
    pub reset_button: Button,
    #[test_offset = 0xa8]
    pub clear_button: Button,
    #[test_offset = 0x138]
    pub start_button: TextButton,
    #[test_offset = 0x238]
    pub back_button: TextButton,
    #[test_offset = 0x338]
    pub rename_button: TextButton,
    #[test_offset = 0x438]
    pub left_button: Button,
    #[test_offset = 0x4c8]
    pub right_button: Button,
    #[test_offset = 0x558]
    pub list_button: TextButton,
    #[test_offset = 0x658]
    pub show_button: TextButton,
    #[test_offset = 0x758]
    pub easy_button: TextButton,
    #[test_offset = 0x858]
    pub normal_button: TextButton,
    #[test_offset = 0x958]
    pub hard_button: TextButton,
    #[test_offset = 0xa58]
    pub type_a: TextButton,
    #[test_offset = 0xb58]
    pub type_b: TextButton,
    #[test_offset = 0xc58]
    pub type_c: TextButton,
    #[test_offset = 0xd58]
    pub type_a_loc: Point,
    #[test_offset = 0xd60]
    pub type_b_loc: Point,
    #[test_offset = 0xd68]
    pub type_c_loc: Point,
    #[test_offset = 0xd70]
    pub random_button: TextButton,
    #[test_offset = 0xe70]
    pub advanced_off_button: TextButton,
    #[test_offset = 0xf70]
    pub advanced_on_button: TextButton,
    #[test_offset = 0x1070]
    pub buttons: Vector<*mut GenericButton>,
    #[test_offset = 0x1088]
    pub animations: Vector<Animation>,
    #[test_offset = 0x10a0]
    pub v_crew_boxes: Vector<*mut CrewCustomizeBox>,
    #[test_offset = 0x10b8]
    pub b_open: bool,
    #[test_offset = 0x10c0]
    pub base_image: *mut GL_Primitive,
    #[test_offset = 0x10c8]
    pub ship_select_box: *mut GL_Primitive,
    #[test_offset = 0x10d0]
    pub ship_ach_box: *mut GL_Primitive,
    #[test_offset = 0x10d8]
    pub ship_equip_box: *mut GL_Primitive,
    #[test_offset = 0x10e0]
    pub start_button_box: *mut GL_Primitive,
    #[test_offset = 0x10e8]
    pub advanced_button_box: *mut GL_Primitive,
    #[test_offset = 0x10f0]
    pub type_a_offset: c_int,
    #[test_offset = 0x10f4]
    pub type_b_offset: c_int,
    #[test_offset = 0x10f8]
    pub type_c_offset: c_int,
    #[test_offset = 0x10fc]
    pub ship_ach_padding: c_int,
    #[test_offset = 0x1100]
    pub advanced_title_offset: c_int,
    #[test_offset = 0x1108]
    pub v_equipment_boxes: Vector<*mut EquipmentBox>,
    #[test_offset = 0x1120]
    pub info_box: InfoBox,
    #[test_offset = 0x11f8]
    pub sys_boxes: Vector<*mut SystemCustomBox>,
    #[test_offset = 0x1210]
    pub shopping_id: c_int,
    #[test_offset = 0x1214]
    pub current_slot: c_int,
    #[test_offset = 0x1218]
    pub current_box: c_int,
    #[test_offset = 0x121c]
    pub b_done: bool,
    #[test_offset = 0x1220]
    pub ships: [[*const ShipBlueprint; 10]; 3],
    #[test_offset = 0x1310]
    pub current_ship_id: c_int,
    #[test_offset = 0x1314]
    pub store_ids: [c_int; 4],
    #[test_offset = 0x1324]
    pub b_renaming: bool,
    #[test_offset = 0x1328]
    pub current_name: StdString,
    #[test_offset = 0x1330]
    pub b_show_rooms: bool,
    #[test_offset = 0x1331]
    pub b_customizing_crew: bool,
    #[test_offset = 0x1338]
    pub walking_man: Animation,
    #[test_offset = 0x13f8]
    pub walking_man_pos: Pointf,
    #[test_offset = 0x1400]
    pub ship_select: ShipSelect,
    #[test_offset = 0x1b68]
    pub intro_screen: ChoiceBox,
    #[test_offset = 0x1d80]
    pub b_showed_intro: bool,
    #[test_offset = 0x1d84]
    pub current_type: c_int,
    #[test_offset = 0x1d88]
    pub name_input: TextInput,
    #[test_offset = 0x1de8]
    pub active_touch: c_int,
    #[test_offset = 0x1dec]
    pub active_touch_is_ship: bool,
    #[test_offset = 0x1ded]
    pub ship_drag_active: bool,
    #[test_offset = 0x1dee]
    pub ship_drag_vertical: bool,
    #[test_offset = 0x1df0]
    pub ship_drag_offset: Point,
    #[test_offset = 0x1df8]
    pub ship_achievements: Vector<ShipAchievementInfo>,
    #[test_offset = 0x1e10]
    pub selected_ach: c_int,
    #[test_offset = 0x1e18]
    pub arrow: *mut GL_Texture,
    #[test_offset = 0x1e20]
    pub desc_box: *mut WindowFrame,
    #[test_offset = 0x1e28]
    pub tracker: AnimationTracker,
    #[test_offset = 0x1e48]
    pub encourage_ship_list: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct MainMenu {
    #[test_offset = 0x0]
    pub b_open: bool,
    #[test_offset = 0x4]
    pub active_touch: c_int,
    #[test_offset = 0x8]
    pub background: *mut GL_Texture,
    #[test_offset = 0x10]
    pub glowy: *mut GL_Texture,
    #[test_offset = 0x18]
    pub glow_tracker: AnimationTracker,
    #[test_offset = 0x38]
    pub continue_button: Button,
    #[test_offset = 0xc8]
    pub start_button: Button,
    #[test_offset = 0x158]
    pub help_button: Button,
    #[test_offset = 0x1e8]
    pub stat_button: Button,
    #[test_offset = 0x278]
    pub options_button: Button,
    #[test_offset = 0x308]
    pub credits_button: Button,
    #[test_offset = 0x398]
    pub quit_button: Button,
    #[test_offset = 0x428]
    pub buttons: Vector<*mut Button>,
    #[test_offset = 0x440]
    pub final_choice: c_int,
    #[test_offset = 0x448]
    pub ship_builder: ShipBuilder,
    #[test_offset = 0x2298]
    pub b_score_screen: bool,
    #[test_offset = 0x22a0]
    pub option_screen: OptionsScreen,
    #[test_offset = 0x33a0]
    pub b_select_save: bool,
    #[test_offset = 0x33a8]
    pub confirm_new_game: ConfirmWindow,
    #[test_offset = 0x3628]
    pub changelog: ChoiceBox,
    #[test_offset = 0x3840]
    pub b_credit_screen: bool,
    #[test_offset = 0x3848]
    pub credits: CreditScreen,
    #[test_offset = 0x38a0]
    pub b_changed_login: bool,
    #[test_offset = 0x38a8]
    pub test_crew: Vector<*mut CrewMember>,
    #[test_offset = 0x38c0]
    pub b_changed_screen: bool,
    #[test_offset = 0x38c1]
    pub b_sync_screen: bool,
    #[test_offset = 0x38c8]
    pub error: StdString,
}

#[repr(C)]
pub struct BossShip {
    pub base: CompleteShip,
    pub current_stage: c_int,
    pub power_timer: TimerHelper,
    pub power_count: c_int,
    pub crew_counts: Vector<c_int>,
    pub b_death_began: bool,
    pub next_stage: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WorldManager {
    #[test_offset = 0x0]
    pub player_ship: *mut CompleteShip,
    #[test_offset = 0x8]
    pub boss_ship: *mut BossShip,
    #[test_offset = 0x10]
    pub space: SpaceManager,
    #[test_offset = 0x4c8]
    pub current_difficulty: c_int,
    #[test_offset = 0x4d0]
    pub ships: Vector<*mut CompleteShip>,
    #[test_offset = 0x4e8]
    pub star_map: StarMap,
    #[test_offset = 0x11b8]
    pub command_gui: *mut CommandGui,
    #[test_offset = 0x11c0]
    pub base_location_event: *mut LocationEvent,
    #[test_offset = 0x11c8]
    pub last_location_event: *mut LocationEvent,
    #[test_offset = 0x11d0]
    pub current_ship_event: ShipEvent,
    #[test_offset = 0x1500]
    pub current_effects: Vector<StatusEffect>,
    #[test_offset = 0x1518]
    pub starting_text: StdString,
    #[test_offset = 0x1520]
    pub new_location: *mut Location,
    #[test_offset = 0x1528]
    pub b_started_game: bool,
    #[test_offset = 0x1529]
    pub b_loading_game: bool,
    #[test_offset = 0x152a]
    pub v_auto_saved: bool,
    #[test_offset = 0x152b]
    pub b_extra_choice: bool,
    #[test_offset = 0x1530]
    pub choice_history: Vector<c_int>,
    #[test_offset = 0x1548]
    pub generated_event: StdString,
    #[test_offset = 0x1550]
    pub last_main_text: TextString,
    #[test_offset = 0x1560]
    pub player_crew_count: c_int,
    #[test_offset = 0x1564]
    pub killed_crew: c_int,
    #[test_offset = 0x1568]
    pub player_hull: c_int,
    #[test_offset = 0x1570]
    pub blue_race_choices: Vector<c_int>,
    #[test_offset = 0x1588]
    pub last_selected_crew_seed: c_int,
    #[test_offset = 0x158c]
    pub testing_blueprints: bool,
    #[test_offset = 0x1590]
    pub original_choice_list: Vector<Choice>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CApp {
    #[test_offset = 0x0]
    pub base: CEvent,
    #[test_offset = 0x8]
    pub running: bool,
    #[test_offset = 0x9]
    pub shift_held: bool,
    #[test_offset = 0x10]
    pub gui: *mut CommandGui,
    #[test_offset = 0x18]
    pub world: *mut WorldManager,
    #[test_offset = 0x20]
    pub menu: MainMenu,
    #[test_offset = 0x38f0]
    pub lang_chooser: LanguageChooser,
    #[test_offset = 0x3930]
    pub screen_x: c_int,
    #[test_offset = 0x3934]
    pub screen_y: c_int,
    #[test_offset = 0x3938]
    pub modifier_x: c_int,
    #[test_offset = 0x393c]
    pub modifier_y: c_int,
    #[test_offset = 0x3940]
    pub full_screen_last_state: bool,
    #[test_offset = 0x3941]
    pub minimized: bool,
    #[test_offset = 0x3942]
    pub min_last_state: bool,
    #[test_offset = 0x3943]
    pub focus: bool,
    #[test_offset = 0x3944]
    pub focus_last_state: bool,
    #[test_offset = 0x3945]
    pub steam_overlay: bool,
    #[test_offset = 0x3946]
    pub steam_overlay_last_state: bool,
    #[test_offset = 0x3947]
    pub rendering: bool,
    #[test_offset = 0x3948]
    pub game_logic: bool,
    #[test_offset = 0x394c]
    pub mouse_modifier_x: c_float,
    #[test_offset = 0x3950]
    pub mouse_modifier_y: c_float,
    #[test_offset = 0x3958]
    pub framebuffer: *mut GL_FrameBuffer,
    #[test_offset = 0x3960]
    pub fbo_support: bool,
    #[test_offset = 0x3964]
    pub x_bar: c_int,
    #[test_offset = 0x3968]
    pub y_bar: c_int,
    #[test_offset = 0x396c]
    pub l_ctrl: bool,
    #[test_offset = 0x396d]
    pub use_frame_buffer: bool,
    #[test_offset = 0x396e]
    pub manual_resolution_error: bool,
    #[test_offset = 0x3970]
    pub manual_res_error_x: c_int,
    #[test_offset = 0x3974]
    pub manual_res_error_y: c_int,
    #[test_offset = 0x3978]
    pub native_full_screen_error: bool,
    #[test_offset = 0x3979]
    pub fb_stretch_error: bool,
    #[test_offset = 0x3980]
    pub last_language: StdString,
    #[test_offset = 0x3988]
    pub input_focus: bool,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum TouchPauseReason {
    // DW_AT_const_value = 0x0
    CrewSelect = 0,
    // DW_AT_const_value = 0x1
    CrewMove = 1,
    // DW_AT_const_value = 0x2
    Doors = 2,
    // DW_AT_const_value = 0x3
    Hacking = 3,
    // DW_AT_const_value = 0x4
    Mind = 4,
    // DW_AT_const_value = 0x5
    Targeting = 5,
    // DW_AT_const_value = 0x6
    TeleportArrive = 6,
    // DW_AT_const_value = 0x7
    TeleportLeave = 7,
}

#[allow(non_camel_case_types)]
pub type GL_FrameBuffer = c_int;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct GL_Texture {
    pub id: c_int,
    pub width: c_int,
    pub height: c_int,
    pub is_logical: bool,
    pub u_base: c_float,
    pub v_base: c_float,
    pub u_size: c_float,
    pub v_size: c_float,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct GL_Color {
    pub r: c_float,
    pub g: c_float,
    pub b: c_float,
    pub a: c_float,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct GL_Primitive {
    pub type_: c_int,
    pub line_width: c_float,
    pub has_texture: bool,
    pub texture: *mut GL_Texture,
    pub texture_antialias: bool,
    pub has_color: bool,
    pub color: GL_Color,
    pub id: c_int,
}

#[repr(C)]
#[derive(Debug)]
pub struct Vector<T> {
    pub start: *mut T,
    pub finish: *mut T,
    pub end_of_storage: *mut T,
}

impl<T> Vector<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let start = unsafe { libc::malloc(mem::size_of::<T>() * capacity) }.cast();
        Self {
            start,
            finish: start,
            end_of_storage: unsafe { start.add(capacity) },
        }
    }
    pub fn push(&mut self, val: T) {
        assert!(self.finish != self.end_of_storage);
        unsafe {
            ptr::write(self.finish, val);
            self.finish = self.finish.add(1);
        }
    }
    pub fn get_ptr(&self, index: usize) -> *mut T {
        self.start.wrapping_add(index)
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        (index < self.len()).then(|| unsafe { &*self.get_ptr(index) })
    }
    pub fn len(&self) -> usize {
        unsafe { self.finish.offset_from(self.start) as usize }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        unsafe { std::slice::from_raw_parts(self.start, self.len()) }.iter()
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        unsafe {
            if !self.start.is_null() {
                libc::free(self.start.cast());
            }
        }
    }
}

pub type VectorBoolStorage = usize;

#[repr(C)]
#[derive(Debug)]
pub struct VectorBoolIter {
    pub ptr: *mut VectorBoolStorage,
    pub offset: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct VectorBool {
    pub start: VectorBoolIter,
    pub finish: VectorBoolIter,
    pub end_of_storage: *mut VectorBoolStorage,
}

#[repr(C)]
#[derive(Debug)]
pub struct QueueIter<T> {
    pub cur: *mut T,
    pub first: *mut T,
    pub last: *mut T,
    pub node: *mut *mut T,
}

#[repr(C)]
#[derive(Debug)]
pub struct Queue<T> {
    pub map: *mut *mut T,
    pub size: usize,
    pub start: QueueIter<T>,
    pub finish: QueueIter<T>,
}

#[repr(C)]
#[derive(Debug)]
pub struct StdString {
    pub data: *const c_char,
}

impl StdString {
    pub fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.data) }
    }
    pub fn to_str(&self) -> Cow<'_, str> {
        self.as_c_str().to_string_lossy()
    }
}

#[vtable]
pub struct VtableAnimationTracker {
    pub dtor: Option<fn(*mut AnimationTracker)>,
    pub delete_dtor: Option<fn(*mut AnimationTracker)>,
    pub update: Option<fn(*mut AnimationTracker)>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AnimationTracker {
    pub vtable: *const VtableAnimationTracker,
    #[test_offset = 0x8]
    pub time: c_float,
    #[test_offset = 0xc]
    pub loop_: bool,
    #[test_offset = 0x10]
    pub current_time: c_float,
    #[test_offset = 0x14]
    pub running: bool,
    #[test_offset = 0x15]
    pub reverse: bool,
    #[test_offset = 0x16]
    pub done: bool,
    #[test_offset = 0x18]
    pub loop_delay: c_float,
    #[test_offset = 0x1c]
    pub current_delay: c_float,
}

#[vtable]
pub struct VtableGenericButton {
    pub dtor: Option<fn(*mut GenericButton)>,
    pub delete_dtor: Option<fn(*mut GenericButton)>,
    pub reset: Option<fn(*mut GenericButton)>,
    pub set_location: Option<fn(*mut GenericButton, Point)>,
    pub set_hit_box: Option<fn(*mut GenericButton, *const Rect)>,
    pub set_active: Option<fn(*mut GenericButton, bool)>,
    pub on_loop: Option<fn(*mut GenericButton)>,
    pub on_render: Option<fn(*mut GenericButton)>,
    pub mouse_move: Option<fn(*mut GenericButton, c_int, c_int, bool)>,
    pub on_click: Option<fn(*mut GenericButton)>,
    pub on_right_click: Option<fn(*mut GenericButton)>,
    pub on_touch:
        Option<fn(*mut GenericButton, TouchAction, c_int, c_int, c_int, c_int, c_int) -> bool>,
    pub reset_primitives: Option<fn(*mut GenericButton)>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct GenericButton {
    pub vtable: *const VtableGenericButton,
    #[test_offset = 0x8]
    pub position: Point,
    #[test_offset = 0x10]
    pub hitbox: Rect,
    #[test_offset = 0x20]
    pub allow_any_touch: bool,
    #[test_offset = 0x21]
    pub touch_selectable: bool,
    #[test_offset = 0x22]
    pub b_render_off: bool,
    #[test_offset = 0x23]
    pub b_render_selected: bool,
    #[test_offset = 0x24]
    pub b_flashing: bool,
    #[test_offset = 0x28]
    pub flashing: AnimationTracker,
    #[test_offset = 0x48]
    pub b_active: bool,
    #[test_offset = 0x49]
    pub b_hover: bool,
    #[test_offset = 0x4a]
    pub b_activated: bool,
    #[test_offset = 0x4b]
    pub b_selected: bool,
    #[test_offset = 0x4c]
    pub active_touch: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TextButton {
    pub base: GenericButton,
    #[test_offset = 0x50]
    pub primitives: [*mut GL_Primitive; 3],
    #[test_offset = 0x68]
    pub base_image: *mut GL_Texture,
    #[test_offset = 0x70]
    pub base_image_offset: Point,
    #[test_offset = 0x78]
    pub base_primitive: *mut GL_Primitive,
    #[test_offset = 0x80]
    pub colors_set: bool,
    #[test_offset = 0x84]
    pub colors: [GL_Color; 3],
    #[test_offset = 0xb4]
    pub text_color: GL_Color,
    #[test_offset = 0xc4]
    pub button_size: Point,
    #[test_offset = 0xcc]
    pub corner_inset: c_int,
    #[test_offset = 0xd0]
    pub auto_width: bool,
    #[test_offset = 0xd4]
    pub auto_width_margin: c_int,
    #[test_offset = 0xd8]
    pub auto_width_min: c_int,
    #[test_offset = 0xdc]
    pub auto_right_align: bool,
    #[test_offset = 0xe0]
    pub label: TextString,
    #[test_offset = 0xf0]
    pub font: c_int,
    #[test_offset = 0xf4]
    pub line_height: c_int,
    #[test_offset = 0xf8]
    pub text_y_offset: c_int,
    #[test_offset = 0xfc]
    pub auto_shrink: bool,
}

/// TextButton without the end for better alignment
#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TextButtonPrime {
    pub base: GenericButton,
    #[test_offset = 0x50]
    pub primitives: [*mut GL_Primitive; 3],
    #[test_offset = 0x68]
    pub base_image: *mut GL_Texture,
    #[test_offset = 0x70]
    pub base_image_offset: Point,
    #[test_offset = 0x78]
    pub base_primitive: *mut GL_Primitive,
    #[test_offset = 0x80]
    pub colors_set: bool,
    #[test_offset = 0x84]
    pub colors: [GL_Color; 3],
    #[test_offset = 0xb4]
    pub text_color: GL_Color,
    #[test_offset = 0xc4]
    pub button_size: Point,
    #[test_offset = 0xcc]
    pub corner_inset: c_int,
    #[test_offset = 0xd0]
    pub auto_width: bool,
    #[test_offset = 0xd4]
    pub auto_width_margin: c_int,
    #[test_offset = 0xd8]
    pub auto_width_min: c_int,
    #[test_offset = 0xdc]
    pub auto_right_align: bool,
    #[test_offset = 0xe0]
    pub label: TextString,
    #[test_offset = 0xf0]
    pub font: c_int,
    #[test_offset = 0xf4]
    pub line_height: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ConfirmWindow {
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub text: TextString,
    #[test_offset = 0x30]
    pub text_height: c_int,
    #[test_offset = 0x34]
    pub min_width: c_int,
    #[test_offset = 0x38]
    pub window_width: c_int,
    #[test_offset = 0x40]
    pub yes_text: TextString,
    #[test_offset = 0x50]
    pub no_text: TextString,
    #[test_offset = 0x60]
    pub auto_center: bool,
    #[test_offset = 0x68]
    pub window_image: *mut GL_Texture,
    #[test_offset = 0x70]
    pub window: *mut GL_Primitive,
    #[test_offset = 0x78]
    pub yes_button: TextButton,
    #[test_offset = 0x178]
    pub no_button: TextButton,
    #[test_offset = 0x278]
    pub result: bool,
}

#[vtable]
pub struct VtableCompleteShip {
    pub dtor: Option<fn(*mut CompleteShip)>,
    pub delete_dtor: Option<fn(*mut CompleteShip)>,
    pub on_loop: Option<fn(*mut CompleteShip)>,
    pub pause_loop: Option<fn(*mut CompleteShip)>,
    pub is_boss: Option<fn(*mut CompleteShip) -> bool>,
    pub restart: Option<fn(*mut CompleteShip)>,
    pub incoming_fire: Option<fn(*mut CompleteShip) -> bool>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct PowerProfile {
    #[test_offset = 0x0]
    pub system_id: c_int,
    #[test_offset = 0x4]
    pub allotment: Pair<c_int, c_int>,
    #[test_offset = 0x10]
    pub sub_indices: Vector<c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CombatAI {
    #[test_offset = 0x0]
    pub target: *mut ShipManager,
    #[test_offset = 0x8]
    pub weapons: Vector<*mut ProjectileFactory>,
    #[test_offset = 0x20]
    pub drones: Vector<*mut SpaceDrone>,
    #[test_offset = 0x38]
    pub stance: c_int,
    #[test_offset = 0x40]
    pub system_targets: Vector<c_int>,
    #[test_offset = 0x58]
    pub b_firing_while_cloaked: bool,
    #[test_offset = 0x60]
    pub self_: *mut ShipManager,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewAI {
    #[test_offset = 0x0]
    pub ship: *mut ShipManager,
    #[test_offset = 0x8]
    pub b_a_ion: bool,
    #[test_offset = 0x9]
    pub b_airlock_requested: bool,
    #[test_offset = 0xa]
    pub b_medbay_requested: bool,
    #[test_offset = 0xb]
    pub b_hurt_crew: bool,
    #[test_offset = 0xc]
    pub b_calm_ship: bool,
    #[test_offset = 0x10]
    pub crew_list: Vector<*mut CrewMember>,
    #[test_offset = 0x28]
    pub intruder_list: Vector<*mut CrewMember>,
    #[test_offset = 0x40]
    pub hull_breaches: Vector<*mut Repairable>,
    #[test_offset = 0x58]
    pub desired_task_list: Vector<CrewTask>,
    #[test_offset = 0x70]
    pub bonus_task_list: Vector<CrewTask>,
    #[test_offset = 0x88]
    pub breached_rooms: VectorBool,
    #[test_offset = 0xb0]
    pub i_teleport_request: c_int,
    #[test_offset = 0xb4]
    pub b_urgent_teleport: bool,
    #[test_offset = 0xb8]
    pub starting_crew_count: c_int,
    #[test_offset = 0xbc]
    pub b_multiracial_crew: bool,
    #[test_offset = 0xbd]
    pub b_override_race: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipAI {
    #[test_offset = 0x0]
    pub ship: *mut ShipManager,
    #[test_offset = 0x8]
    pub target: *mut ShipManager,
    #[test_offset = 0x10]
    pub crew_ai: CrewAI,
    #[test_offset = 0xd0]
    pub combat_ai: CombatAI,
    #[test_offset = 0x138]
    pub player_ship: bool,
    #[test_offset = 0x139]
    pub surrendered: bool,
    #[test_offset = 0x13a]
    pub escaping: bool,
    #[test_offset = 0x13b]
    pub destroyed: bool,
    #[test_offset = 0x13c]
    pub surrender_threshold: c_int,
    #[test_offset = 0x140]
    pub escape_threshold: c_int,
    #[test_offset = 0x144]
    pub escape_timer: c_float,
    #[test_offset = 0x148]
    pub last_max_power: c_int,
    #[test_offset = 0x150]
    pub power_profiles: Map<StdString, PowerProfile>,
    #[test_offset = 0x180]
    pub boarding_profile: c_int,
    #[test_offset = 0x184]
    pub i_teleport_request: c_int,
    #[test_offset = 0x188]
    pub i_teleport_target: c_int,
    #[test_offset = 0x18c]
    pub broken_systems: c_int,
    #[test_offset = 0x190]
    pub boarding_ai: c_int,
    #[test_offset = 0x194]
    pub i_crew_needed: c_int,
    #[test_offset = 0x198]
    pub b_stalemate_trigger: bool,
    #[test_offset = 0x19c]
    pub f_stalemate_timer: c_float,
    #[test_offset = 0x1a0]
    pub last_health: c_int,
    #[test_offset = 0x1a4]
    pub b_boss: bool,
    #[test_offset = 0x1a8]
    pub i_times_teleported: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CompleteShip {
    pub vtable: *const VtableCompleteShip,
    #[test_offset = 0x8]
    pub i_ship_id: c_int,
    #[test_offset = 0x10]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x18]
    pub space_manager: *mut SpaceManager,
    #[test_offset = 0x20]
    pub enemy_ship: *mut CompleteShip,
    #[test_offset = 0x28]
    pub b_player_ship: bool,
    #[test_offset = 0x30]
    pub ship_ai: ShipAI,
    #[test_offset = 0x1e0]
    pub arriving_party: Vector<*mut CrewMember>,
    #[test_offset = 0x1f8]
    pub leaving_party: Vector<*mut CrewMember>,
    #[test_offset = 0x210]
    pub tele_target_room: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TimerHelper {
    #[test_offset = 0x0]
    pub max_time: c_int,
    #[test_offset = 0x4]
    pub min_time: c_int,
    #[test_offset = 0x8]
    pub curr_time: c_float,
    #[test_offset = 0xc]
    pub curr_goal: c_float,
    #[test_offset = 0x10]
    pub loop_: bool,
    #[test_offset = 0x11]
    pub running: bool,
}

#[vtable]
pub struct VtableStoreBox {
    pub dtor: Option<fn(*mut StoreBox)>,
    pub delete_dtor: Option<fn(*mut StoreBox)>,
    pub on_loop: Option<fn(*mut StoreBox)>,
    pub on_render: Option<fn(*mut StoreBox)>,
    pub mouse_move: Option<fn(*mut StoreBox, c_int, c_int)>,
    pub mouse_click: Option<fn(*mut StoreBox, c_int, c_int)>,
    pub on_touch: Option<fn(*mut StoreBox, TouchAction, c_int, c_int, c_int, c_int, c_int)>,
    pub activate: Option<fn(*mut StoreBox)>,
    pub purchase: Option<fn(*mut StoreBox)>,
    pub set_info_box: Option<fn(*mut StoreBox, *mut InfoBox, c_int) -> c_int>,
    pub can_hold: Option<fn(*mut StoreBox) -> bool>,
    pub requires_confirm: Option<fn(*mut StoreBox) -> bool>,
    pub confirm: Option<fn(*mut StoreBox, bool)>,
    pub get_confirm_text: Option<fn(*mut StoreBox) -> TextString>,
    pub get_extra_data: Option<fn(*mut StoreBox) -> c_int>,
    pub set_extra_data: Option<fn(*mut StoreBox, c_int)>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct StoreBox {
    #[test_offset = 0x0]
    pub vtable: *const VtableStoreBox,
    #[test_offset = 0x8]
    pub item_id: c_int,
    #[test_offset = 0xc]
    pub item_box: c_int,
    #[test_offset = 0x10]
    pub button_image: StdString,
    #[test_offset = 0x18]
    pub button: Button,
    #[test_offset = 0xa8]
    pub symbol: *mut GL_Primitive,
    #[test_offset = 0xb0]
    pub desc: Description,
    #[test_offset = 0x110]
    pub count: c_int,
    #[test_offset = 0x114]
    pub cost_position: c_int,
    #[test_offset = 0x118]
    pub shopper: *mut ShipManager,
    #[test_offset = 0x120]
    pub equip_screen: *mut Equipment,
    #[test_offset = 0x128]
    pub p_blueprint: *const Blueprint,
    #[test_offset = 0x130]
    pub b_equipment_box: bool,
    #[test_offset = 0x134]
    pub f_icon_scale: c_float,
    #[test_offset = 0x138]
    pub push_icon: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponStoreBox {
    #[test_offset = 0x0]
    pub base: StoreBox,
    #[test_offset = 0x140]
    pub blueprint: *const WeaponBlueprint,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemStoreBox {
    #[test_offset = 0x0]
    pub base: StoreBox,
    #[test_offset = 0x140]
    pub blueprint: *const SystemBlueprint,
    #[test_offset = 0x148]
    pub type_: c_int,
    #[test_offset = 0x14c]
    pub b_confirming: bool,
    #[test_offset = 0x150]
    pub confirm_string: StdString,
    #[test_offset = 0x158]
    pub free_blueprint: StdString,
    #[test_offset = 0x160]
    pub drone_choice: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct RepairStoreBox {
    #[test_offset = 0x0]
    pub base: StoreBox,
    #[test_offset = 0x140]
    pub repair_all: bool,
    #[test_offset = 0x144]
    pub repair_cost: c_int,
    #[test_offset = 0x148]
    pub button_text: TextString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ItemBlueprint {
    #[test_offset = 0x0]
    pub base: Blueprint,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ItemStoreBox {
    #[test_offset = 0x0]
    pub base: StoreBox,
    #[test_offset = 0x140]
    pub blueprint: *const ItemBlueprint,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneStoreBox {
    #[test_offset = 0x0]
    pub base: StoreBox,
    #[test_offset = 0x140]
    pub blueprint: *const DroneBlueprint,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewStoreBox {
    #[test_offset = 0x0]
    pub base: StoreBox,
    #[test_offset = 0x140]
    pub name: StdString,
    #[test_offset = 0x148]
    pub crew_portrait: Animation,
    #[test_offset = 0x208]
    pub blueprint: CrewBlueprint,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AugmentStoreBox {
    #[test_offset = 0x0]
    pub base: StoreBox,
    #[test_offset = 0x140]
    pub blueprint: *const AugmentBlueprint,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StoreType {
    Weapons = 0,
    Systems = 4,
    Drones = 1,
    Augments = 2,
    Crew = 3,
    Total = 5,
    Items = 6,
    None = 7,
}

impl fmt::Display for StoreType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Weapons => "weapons",
            Self::Systems => "systems",
            Self::Drones => "drones",
            Self::Augments => "augments",
            Self::Crew => "crew",
            Self::Total => "total",
            Self::Items => "items",
            Self::None => "none",
        })
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Store {
    #[test_offset = 0x0]
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub box_: *mut GL_Texture,
    #[test_offset = 0x28]
    pub heading_title: [TextString; 4],
    #[test_offset = 0x68]
    pub page1: Button,
    #[test_offset = 0xf8]
    pub page2: Button,
    #[test_offset = 0x188]
    pub confirm_dialog: ConfirmWindow,
    #[test_offset = 0x408]
    pub current_button: *mut Button,
    #[test_offset = 0x410]
    pub current_description: Description,
    #[test_offset = 0x470]
    pub unavailable: StdString,
    // 6 elements
    #[test_offset = 0x478]
    pub v_store_boxes: Vector<*mut StoreBox>,
    #[test_offset = 0x490]
    pub v_item_boxes: Vector<*mut StoreBox>,
    #[test_offset = 0x4a8]
    pub shopper: *mut ShipManager,
    #[test_offset = 0x4b0]
    pub selected_weapon: c_int,
    #[test_offset = 0x4b4]
    pub selected_drone: c_int,
    #[test_offset = 0x4b8]
    pub info_box: InfoBox,
    #[test_offset = 0x590]
    pub info_box_loc: Point,
    #[test_offset = 0x598]
    pub exit_button: Button,
    #[test_offset = 0x628]
    pub world_level: c_int,
    #[test_offset = 0x62c]
    pub section_count: c_int,
    // see StoreType
    #[test_offset = 0x630]
    pub types: [c_int; 4],
    #[test_offset = 0x640]
    pub b_show_page2: bool,
    #[test_offset = 0x648]
    pub confirm_buy: *mut StoreBox,
    #[test_offset = 0x650]
    pub force_system_info_width: c_int,
}

pub enum Either<A, B> {
    A(A),
    B(B),
}

pub trait StoreBoxTrait {
    const TYPE: Either<StoreType, usize>;
    const IGNORE_COUNT: bool = false;
}

impl StoreBoxTrait for WeaponStoreBox {
    const TYPE: Either<StoreType, usize> = Either::A(StoreType::Weapons);
}

impl StoreBoxTrait for DroneStoreBox {
    const TYPE: Either<StoreType, usize> = Either::A(StoreType::Drones);
}

impl StoreBoxTrait for SystemStoreBox {
    const TYPE: Either<StoreType, usize> = Either::A(StoreType::Systems);
}

impl StoreBoxTrait for AugmentStoreBox {
    const TYPE: Either<StoreType, usize> = Either::A(StoreType::Augments);
}

impl StoreBoxTrait for CrewStoreBox {
    const TYPE: Either<StoreType, usize> = Either::A(StoreType::Crew);
}

impl StoreBoxTrait for ItemStoreBox {
    const TYPE: Either<StoreType, usize> = Either::B(0);
}

impl StoreBoxTrait for RepairStoreBox {
    const TYPE: Either<StoreType, usize> = Either::B(3);
    const IGNORE_COUNT: bool = true;
}

impl Store {
    pub fn active_boxes_for(&self, t: StoreType) -> Vec<*mut StoreBox> {
        match t {
            StoreType::Augments => self
                .active_boxes::<AugmentStoreBox>()
                .into_iter()
                .map(|x| x.cast())
                .collect(),
            StoreType::Weapons => self
                .active_boxes::<WeaponStoreBox>()
                .into_iter()
                .map(|x| x.cast())
                .collect(),
            StoreType::Systems => self
                .active_boxes::<SystemStoreBox>()
                .into_iter()
                .map(|x| x.cast())
                .collect(),
            StoreType::Drones => self
                .active_boxes::<DroneStoreBox>()
                .into_iter()
                .map(|x| x.cast())
                .collect(),
            StoreType::Crew => self
                .active_boxes::<CrewStoreBox>()
                .into_iter()
                .map(|x| x.cast())
                .collect(),
            StoreType::Items => self
                .active_boxes::<ItemStoreBox>()
                .into_iter()
                .map(|x| x.cast())
                .collect(),
            StoreType::None => self
                .active_boxes::<RepairStoreBox>()
                .into_iter()
                .map(|x| x.cast())
                .collect(),
            StoreType::Total => vec![],
        }
    }
    pub fn active_boxes<T: StoreBoxTrait>(&self) -> Vec<*mut T> {
        #[allow(clippy::never_loop)]
        'a: loop {
            match T::TYPE {
                Either::A(x) => {
                    for (i, t) in self.types.into_iter().enumerate() {
                        if t == x as i32 {
                            break 'a self.v_store_boxes.iter().skip(i * 3).take(3);
                        }
                    }
                    return vec![];
                }
                Either::B(n) => {
                    break self
                        .v_store_boxes
                        .iter()
                        .skip((self.section_count * 3) as usize + n)
                        .take(3)
                }
            }
        }
        .copied()
        .filter(|x| !x.is_null())
        .filter(|x| T::IGNORE_COUNT || unsafe { (**x).count > 0 })
        .map(|x| x.cast())
        .collect()
    }
    pub fn has_type(&self, x: StoreType) -> bool {
        self.types.contains(&(x as i32))
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Button {
    #[test_offset = 0x0]
    pub base: GenericButton,
    #[test_offset = 0x50]
    pub images: [*mut GL_Texture; 3],
    #[test_offset = 0x68]
    pub primitives: [*mut GL_Primitive; 3],
    #[test_offset = 0x80]
    pub image_size: Point,
    #[test_offset = 0x88]
    pub b_mirror: bool,
}

/// Button without b_mirror for alignment
#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ButtonPrime {
    #[test_offset = 0x0]
    pub base: GenericButton,
    #[test_offset = 0x50]
    pub images: [*mut GL_Texture; 3],
    #[test_offset = 0x68]
    pub primitives: [*mut GL_Primitive; 3],
    #[test_offset = 0x80]
    pub image_size: Point,
}

#[vtable]
pub struct VtableWarningMessage {
    pub dtor: Option<fn(*mut WarningMessage)>,
    pub delete_dtor: Option<fn(*mut WarningMessage)>,
    pub render_with_alpha: Option<fn(*mut WarningMessage, c_float)>,
}

#[repr(u32)]
pub enum Centered {
    Centered = 0,
}

#[vtable]
pub struct VtableCachedPrimitive {
    pub create_primitive: Option<fn(*mut CachedPrimitive)>,
    pub dtor: Option<fn(*mut CachedPrimitive)>,
    pub delete_dtor: Option<fn(*mut CachedPrimitive)>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CachedPrimitive {
    #[test_offset = 0x0]
    pub vtable: *const VtableCachedPrimitive,
    #[test_offset = 0x8]
    pub primitive: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CachedImage {
    #[test_offset = 0x0]
    pub base: CachedPrimitive,
    #[test_offset = 0x10]
    pub image_name: StdString,
    #[test_offset = 0x18]
    pub texture: *mut GL_Texture,
    #[test_offset = 0x20]
    pub x: c_int,
    #[test_offset = 0x24]
    pub y: c_int,
    #[test_offset = 0x28]
    pub w_scale: c_float,
    #[test_offset = 0x2c]
    pub h_scale: c_float,
    #[test_offset = 0x30]
    pub x_start: c_float,
    #[test_offset = 0x34]
    pub y_start: c_float,
    #[test_offset = 0x38]
    pub x_size: c_float,
    #[test_offset = 0x3c]
    pub y_size: c_float,
    #[test_offset = 0x40]
    pub rotation: c_float,
    #[test_offset = 0x44]
    pub mirrored: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WarningMessage {
    pub vtable: *const VtableWarningMessage,
    #[test_offset = 0x8]
    pub tracker: AnimationTracker,
    #[test_offset = 0x28]
    pub position: Point,
    #[test_offset = 0x30]
    pub is_image: bool,
    #[test_offset = 0x38]
    pub text: TextString,
    #[test_offset = 0x48]
    pub center_text: bool,
    #[test_offset = 0x4c]
    pub text_color: GL_Color,
    #[test_offset = 0x5c]
    pub use_warning_line: bool,
    #[test_offset = 0x60]
    pub image: CachedImage,
    #[test_offset = 0xa8]
    pub image_name: StdString,
    #[test_offset = 0xb0]
    pub flash: bool,
    #[test_offset = 0xb8]
    pub sound: StdString,
    #[test_offset = 0xc0]
    pub flash_tracker: AnimationTracker,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct FleetShip {
    // offset = 424109
    // DW_AT_decl_file = /media/sf_FTL/Project/src/Gameplay/SpaceManager.h
    // DW_AT_decl_line = 0x37
    #[test_offset = 0x0]
    pub image: *mut GL_Texture,
    // offset = 424121
    // DW_AT_decl_file = /media/sf_FTL/Project/src/Gameplay/SpaceManager.h
    // DW_AT_decl_line = 0x38
    #[test_offset = 0x8]
    pub location: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct NebulaCloud {
    #[test_offset = 0x0]
    pub pos: Point,
    #[test_offset = 0x8]
    pub curr_alpha: c_float,
    #[test_offset = 0xc]
    pub curr_scale: c_float,
    #[test_offset = 0x10]
    pub delta_alpha: c_float,
    #[test_offset = 0x14]
    pub delta_scale: c_float,
    #[test_offset = 0x18]
    pub new_trigger: c_float,
    #[test_offset = 0x1c]
    pub new_cloud: bool,
    #[test_offset = 0x1d]
    pub b_lightning: bool,
    #[test_offset = 0x20]
    pub lightning_flash: AnimationTracker,
    #[test_offset = 0x40]
    pub flash_timer: c_float,
    #[test_offset = 0x44]
    pub lightning_rotation: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Scroller {
    #[test_offset = 0x0]
    pub image_id: *mut GL_Texture,
    #[test_offset = 0x8]
    pub size_x: c_int,
    #[test_offset = 0xc]
    pub size_y: c_int,
    #[test_offset = 0x10]
    pub image_x: c_int,
    #[test_offset = 0x14]
    pub image_y: c_int,
    #[test_offset = 0x18]
    pub f_speed: c_float,
    #[test_offset = 0x1c]
    pub current_x: c_float,
    #[test_offset = 0x20]
    pub b_initialized: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AsteroidGenerator {
    #[test_offset = 0x0]
    pub asteroid_queue: Queue<*mut Projectile>,
    #[test_offset = 0x50]
    pub spawn_rate: [RandomAmount; 3],
    #[test_offset = 0x74]
    pub state_length: [RandomAmount; 3],
    #[test_offset = 0x98]
    pub number_of_ships: c_int,
    #[test_offset = 0x9c]
    pub i_state: c_int,
    #[test_offset = 0xa0]
    pub current_space: c_int,
    #[test_offset = 0xa4]
    pub i_next_direction: c_int,
    #[test_offset = 0xa8]
    pub f_state_timer: c_float,
    #[test_offset = 0xac]
    pub timer: c_float,
    #[test_offset = 0xb0]
    pub b_running: bool,
    #[test_offset = 0xb4]
    pub init_shields: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SpaceManager {
    #[test_offset = 0x0]
    pub projectiles: Vector<*mut Projectile>,
    #[test_offset = 0x18]
    pub asteroid_generator: AsteroidGenerator,
    #[test_offset = 0xd0]
    pub ships: Vector<*mut ShipManager>,
    #[test_offset = 0xe8]
    pub drones: Vector<*mut SpaceDrone>,
    #[test_offset = 0x100]
    pub danger_zone: bool,
    #[test_offset = 0x108]
    pub current_back: *mut GL_Texture,
    #[test_offset = 0x110]
    pub current_planet: ImageDesc,
    #[test_offset = 0x130]
    pub planet_image: CachedImage,
    #[test_offset = 0x178]
    pub fleet_ship: ImageDesc,
    #[test_offset = 0x198]
    pub ship_ids: [*mut GL_Texture; 8],
    #[test_offset = 0x1d8]
    pub fleet_ships: [FleetShip; 9],
    #[test_offset = 0x268]
    pub asteroid_scroller: [Scroller; 3],
    #[test_offset = 0x2e0]
    pub sun_image: *mut GL_Texture,
    #[test_offset = 0x2e8]
    pub sun_glow: *mut GL_Texture,
    #[test_offset = 0x2f0]
    pub sun_glow1: AnimationTracker,
    #[test_offset = 0x310]
    pub sun_glow2: AnimationTracker,
    #[test_offset = 0x330]
    pub sun_glow3: AnimationTracker,
    #[test_offset = 0x350]
    pub sun_level: bool,
    #[test_offset = 0x351]
    pub pulsar_level: bool,
    #[test_offset = 0x358]
    pub pulsar_front: *mut GL_Texture,
    #[test_offset = 0x360]
    pub pulsar_back: *mut GL_Texture,
    #[test_offset = 0x368]
    pub lowend_pulsar: *mut GL_Texture,
    #[test_offset = 0x370]
    pub b_pds: bool,
    #[test_offset = 0x374]
    pub env_target: c_int,
    #[test_offset = 0x378]
    pub ship_position: Point,
    #[test_offset = 0x380]
    pub random_pds_timer: c_float,
    #[test_offset = 0x388]
    pub pds_queue: Vector<*mut Projectile>,
    #[test_offset = 0x3a0]
    pub flash_timer: TimerHelper,
    #[test_offset = 0x3b8]
    pub flash_tracker: AnimationTracker,
    #[test_offset = 0x3d8]
    pub current_beacon: ImageDesc,
    #[test_offset = 0x3f8]
    current_beacon_flash: ImageDesc,
    #[test_offset = 0x418]
    pub beacon_tracker: AnimationTracker,
    #[test_offset = 0x438]
    pub flash_sound: bool,
    #[test_offset = 0x439]
    pub b_nebula: bool,
    #[test_offset = 0x43a]
    pub b_storm: bool,
    #[test_offset = 0x440]
    pub nebula_clouds: Vector<NebulaCloud>,
    #[test_offset = 0x458]
    pub lowend_nebula: *mut GL_Texture,
    #[test_offset = 0x460]
    pub lowend_storm: *mut GL_Texture,
    #[test_offset = 0x468]
    pub lowend_sun: *mut GL_Texture,
    #[test_offset = 0x470]
    pub lowend_asteroids: *mut GL_Texture,
    #[test_offset = 0x478]
    pub ship_health: c_float,
    #[test_offset = 0x47c]
    pub game_paused: bool,
    #[test_offset = 0x480]
    pub pds_fire_timer: TimerHelper,
    #[test_offset = 0x494]
    pub pds_countdown: c_int,
    #[test_offset = 0x498]
    pub pds_smoke_anims: Vector<Animation>,
    #[test_offset = 0x4b0]
    pub queue_screen_shake: bool,
    #[test_offset = 0x4b1]
    pub player_ship_in_front: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ChoiceReq {
    #[test_offset = 0x0]
    pub object: StdString,
    #[test_offset = 0x8]
    pub min_level: c_int,
    #[test_offset = 0xc]
    pub max_level: c_int,
    #[test_offset = 0x10]
    pub max_group: c_int,
    #[test_offset = 0x14]
    pub blue: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Choice {
    #[test_offset = 0x0]
    pub event: *mut LocationEvent,
    #[test_offset = 0x8]
    pub text: TextString,
    #[test_offset = 0x18]
    pub requirement: ChoiceReq,
    #[test_offset = 0x30]
    pub hidden_reward: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BoardingEvent {
    #[test_offset = 0x0]
    pub type_: StdString,
    #[test_offset = 0x8]
    pub min: c_int,
    #[test_offset = 0xc]
    pub max: c_int,
    #[test_offset = 0x10]
    pub amount: c_int,
    #[test_offset = 0x14]
    pub breach: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct StatusEffect {
    #[test_offset = 0x0]
    pub type_: c_int,
    #[test_offset = 0x4]
    #[allow(non_snake_case)]
    pub _sil_do_not_use_system: c_int,
    #[test_offset = 0x8]
    pub amount: c_int,
    #[test_offset = 0xc]
    pub target: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct EventDamage {
    #[test_offset = 0x0]
    #[allow(non_snake_case)]
    pub _sil_do_not_use_system: c_int,
    #[test_offset = 0x4]
    pub amount: c_int,
    #[test_offset = 0x8]
    pub effect: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewDesc {
    #[test_offset = 0x0]
    pub type_: StdString,
    #[test_offset = 0x8]
    pub proportion: c_float,
    #[test_offset = 0xc]
    pub amount: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipEvent {
    #[test_offset = 0x0]
    pub present: bool,
    #[test_offset = 0x8]
    pub name: StdString,
    #[test_offset = 0x10]
    pub blueprint: StdString,
    #[test_offset = 0x18]
    pub auto_blueprint: StdString,
    #[test_offset = 0x20]
    pub blueprint_list: Vector<StdString>,
    #[test_offset = 0x38]
    pub actual_blueprint: ShipBlueprint,
    #[test_offset = 0x288]
    pub hostile: bool,
    #[test_offset = 0x290]
    pub surrender: StdString,
    #[test_offset = 0x298]
    pub escape: StdString,
    #[test_offset = 0x2a0]
    pub destroyed: StdString,
    #[test_offset = 0x2a8]
    pub dead_crew: StdString,
    #[test_offset = 0x2b0]
    pub gotaway: StdString,
    #[test_offset = 0x2b8]
    pub escape_timer: c_int,
    #[test_offset = 0x2bc]
    pub surrender_threshold: RandomAmount,
    #[test_offset = 0x2c8]
    pub escape_threshold: RandomAmount,
    #[test_offset = 0x2d8]
    pub crew_override: Vector<CrewDesc>,
    #[test_offset = 0x2f0]
    pub weapon_override: Vector<StdString>,
    #[test_offset = 0x308]
    pub weapon_over_count: c_int,
    #[test_offset = 0x310]
    pub drone_override: Vector<StdString>,
    #[test_offset = 0x328]
    pub drone_over_count: c_int,
    #[test_offset = 0x32c]
    pub ship_seed: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct LocationEvent {
    #[test_offset = 0x0]
    pub text: TextString,
    #[test_offset = 0x10]
    pub ship: ShipEvent,
    #[test_offset = 0x340]
    pub stuff: ResourceEvent,
    #[test_offset = 0x4b8]
    pub environment: c_int,
    #[test_offset = 0x4bc]
    pub environment_target: c_int,
    #[test_offset = 0x4c0]
    pub store: bool,
    #[test_offset = 0x4c4]
    pub fleet_position: c_int,
    #[test_offset = 0x4c8]
    pub beacon: bool,
    #[test_offset = 0x4c9]
    pub reveal_map: bool,
    #[test_offset = 0x4ca]
    pub distress_beacon: bool,
    #[test_offset = 0x4cb]
    pub repair: bool,
    #[test_offset = 0x4cc]
    pub modify_pursuit: c_int,
    #[test_offset = 0x4d0]
    pub p_store: *mut Store,
    #[test_offset = 0x4d8]
    pub damage: Vector<EventDamage>,
    #[test_offset = 0x4f0]
    pub quest: StdString,
    #[test_offset = 0x4f8]
    pub status_effects: Vector<StatusEffect>,
    #[test_offset = 0x510]
    pub name_definitions: Vector<Pair<StdString, StdString>>,
    #[test_offset = 0x528]
    pub space_image: StdString,
    #[test_offset = 0x530]
    pub planet_image: StdString,
    #[test_offset = 0x538]
    pub event_name: StdString,
    #[test_offset = 0x540]
    pub reward: ResourceEvent,
    #[test_offset = 0x6b8]
    pub boarders: BoardingEvent,
    #[test_offset = 0x6d0]
    pub choices: Vector<Choice>,
    #[test_offset = 0x6e8]
    pub unlock_ship: c_int,
    #[test_offset = 0x6f0]
    pub unlock_ship_text: TextString,
    #[test_offset = 0x700]
    pub secret_sector: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Location {
    #[test_offset = 0x0]
    pub loc: Pointf,
    #[test_offset = 0x8]
    pub connected_locations: Vector<*mut Location>,
    #[test_offset = 0x20]
    pub beacon: bool,
    #[test_offset = 0x21]
    pub known: bool,
    #[test_offset = 0x24]
    pub visited: c_int,
    #[test_offset = 0x28]
    pub danger_zone: bool,
    #[test_offset = 0x29]
    pub new_sector: bool,
    #[test_offset = 0x2a]
    pub nebula: bool,
    #[test_offset = 0x2b]
    pub boss: bool,
    #[test_offset = 0x30]
    pub event: *mut LocationEvent,
    #[test_offset = 0x38]
    pub planet: ImageDesc,
    #[test_offset = 0x58]
    pub space: ImageDesc,
    #[test_offset = 0x78]
    pub beacon_image: ImageDesc,
    #[test_offset = 0x98]
    pub image_id: *mut GL_Texture,
    #[test_offset = 0xa0]
    pub quest_loc: bool,
    #[test_offset = 0xa8]
    pub flash_tracker: AnimationTracker,
    #[test_offset = 0xc8]
    pub fleet_changing: bool,
    #[test_offset = 0xd0]
    pub planet_image: StdString,
    #[test_offset = 0xd8]
    pub space_image: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AugmentEquipBox {
    #[test_offset = 0x0]
    pub base: EquipmentBox,
    #[test_offset = 0xa8]
    pub ship: *mut ShipManager,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponEquipBox {
    #[test_offset = 0x0]
    pub base: EquipmentBox,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneEquipBox {
    #[test_offset = 0x0]
    pub base: EquipmentBox,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Equipment {
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub box_: *mut GL_Texture,
    #[test_offset = 0x28]
    pub store_box: *mut GL_Texture,
    #[test_offset = 0x30]
    pub over_box: DropBox,
    #[test_offset = 0xb8]
    pub over_aug_image: DropBox,
    #[test_offset = 0x140]
    pub sell_box: DropBox,
    #[test_offset = 0x1c8]
    pub b_selling_item: bool,
    #[test_offset = 0x1d0]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x1d8]
    pub v_equipment_boxes: Vector<*mut EquipmentBox>,
    #[test_offset = 0x1f0]
    pub weapons_trash_list: Vector<*mut ProjectileFactory>,
    #[test_offset = 0x208]
    pub overcapacity_box: *mut EquipmentBox,
    #[test_offset = 0x210]
    pub over_aug_box: *mut AugmentEquipBox,
    #[test_offset = 0x218]
    pub selected_equip_box: c_int,
    #[test_offset = 0x21c]
    pub dragging_equip_box: c_int,
    #[test_offset = 0x220]
    pub potential_dragging_box: c_int,
    #[test_offset = 0x224]
    pub b_dragging: bool,
    #[test_offset = 0x228]
    pub first_mouse: Point,
    #[test_offset = 0x230]
    pub current_mouse: Point,
    #[test_offset = 0x238]
    pub drag_box_center: Point,
    #[test_offset = 0x240]
    pub drag_box_offset: Point,
    #[test_offset = 0x248]
    pub info_box: InfoBox,
    #[test_offset = 0x320]
    pub sell_cost_text: StdString,
    #[test_offset = 0x328]
    pub b_over_capacity: bool,
    #[test_offset = 0x329]
    pub b_over_aug_capacity: bool,
    #[test_offset = 0x32a]
    pub b_store_mode: bool,
    #[test_offset = 0x32c]
    pub cargo_id: c_int,
    #[test_offset = 0x330]
    pub info_box_loc: Point,
}

pub trait EquipBoxTrait {
    fn range(bp: &ShipBlueprint) -> Range<c_int>;
}

impl EquipBoxTrait for WeaponEquipBox {
    fn range(bp: &ShipBlueprint) -> Range<c_int> {
        0..bp.weapon_slots
    }
}

impl EquipBoxTrait for DroneEquipBox {
    fn range(bp: &ShipBlueprint) -> Range<c_int> {
        bp.weapon_slots..bp.weapon_slots + bp.drone_slots
    }
}

impl EquipBoxTrait for AugmentEquipBox {
    fn range(bp: &ShipBlueprint) -> Range<c_int> {
        bp.weapon_slots + bp.drone_slots..bp.weapon_slots + bp.drone_slots + 3
    }
}

impl EquipBoxTrait for EquipmentBox {
    fn range(bp: &ShipBlueprint) -> Range<c_int> {
        bp.weapon_slots + bp.drone_slots + 3..bp.weapon_slots + bp.drone_slots + 7
    }
}

// next is overcap, aug overcap

impl Equipment {
    pub fn boxes<T: EquipBoxTrait>(&self) -> Vec<*mut EquipmentBox> {
        let r = T::range(unsafe { &(*self.ship_manager).my_blueprint });
        self.v_equipment_boxes
            .iter()
            .skip(r.start as usize)
            .take(r.len())
            .copied()
            .collect()
    }
    pub fn has_augment(&self, augment: &str) -> bool {
        for b in self.v_equipment_boxes.iter() {
            unsafe {
                let b = &**b;
                if !b.item.augment.is_null() && (*b.item.augment).name.to_str() == augment {
                    return true;
                }
            }
        }
        false
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum AllowedCharType {
    Ascii = 0,
    Language = 1,
    Any = 2,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TextInput {
    #[test_offset = 0x0]
    pub prompt: StdString,
    #[test_offset = 0x8]
    pub text: Vector<c_int>,
    #[test_offset = 0x20]
    pub old_text: Vector<c_int>,
    #[test_offset = 0x38]
    pub pos: c_int,
    #[test_offset = 0x3c]
    pub last_pos: c_int,
    #[test_offset = 0x40]
    pub b_active: bool,
    #[test_offset = 0x44]
    pub allowed_chars: AllowedCharType,
    #[test_offset = 0x48]
    pub max_chars: c_int,
    #[test_offset = 0x4c]
    pub blinker: TimerHelper,
}

#[vtable]
pub struct VtableEquipmentBox {
    pub dtor: Option<fn(*mut EquipmentBox)>,
    pub delete_dtor: Option<fn(*mut EquipmentBox)>,
    pub set_position: Option<fn(*mut EquipmentBox, Point)>,
    pub on_render: Option<fn(*mut EquipmentBox, bool)>,
    pub render_labels: Option<fn(*mut EquipmentBox, bool)>,
    pub render_icon: Option<fn(*mut EquipmentBox)>,
    pub set_ship_manager: Option<fn(*mut EquipmentBox, *mut ShipManager)>,
    pub mouse_move: Option<fn(*mut EquipmentBox, c_int, c_int)>,
    pub on_touch: Option<fn(*mut EquipmentBox, TouchAction, c_int, c_int, c_int, c_int, c_int)>,
    pub update_box_image: Option<fn(*mut EquipmentBox, bool)>,
    pub restart: Option<fn(*mut EquipmentBox)>,
    pub add_item: Option<fn(*mut EquipmentBox, EquipmentBoxItem)>,
    pub remove_item: Option<fn(*mut EquipmentBox)>,
    pub can_hold_weapon: Option<fn(*mut EquipmentBox) -> bool>,
    pub can_hold_drone: Option<fn(*mut EquipmentBox) -> bool>,
    pub can_hold_augment: Option<fn(*mut EquipmentBox) -> bool>,
    pub check_contents: Option<fn(*mut EquipmentBox)>,
    pub get_type: Option<fn(*mut EquipmentBox, bool) -> c_int>,
    pub is_cargo_box: Option<fn(*mut EquipmentBox) -> bool>,
    pub can_hold_crew: Option<fn(*mut EquipmentBox) -> bool>,
    pub can_do_job: Option<fn(*mut EquipmentBox) -> bool>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct EquipmentBoxItem {
    #[test_offset = 0x0]
    pub p_weapon: *mut ProjectileFactory,
    #[test_offset = 0x8]
    pub p_drone: *mut Drone,
    #[test_offset = 0x10]
    pub p_crew: *mut CrewMember,
    #[test_offset = 0x18]
    pub augment: *const AugmentBlueprint,
}

impl EquipmentBoxItem {
    pub fn is_empty(&self) -> bool {
        self.p_weapon.is_null()
            && self.p_drone.is_null()
            && self.p_crew.is_null()
            && self.augment.is_null()
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct EquipmentBox {
    #[test_offset = 0x0]
    pub vtable: *const VtableEquipmentBox,
    #[test_offset = 0x8]
    pub blocked_overlay: *mut GL_Primitive,
    #[test_offset = 0x10]
    pub overlay_color: GL_Color,
    #[test_offset = 0x20]
    pub image_name: StdString,
    #[test_offset = 0x28]
    pub empty: *mut GL_Primitive,
    #[test_offset = 0x30]
    pub full: *mut GL_Primitive,
    #[test_offset = 0x38]
    pub selected_empty: *mut GL_Primitive,
    #[test_offset = 0x40]
    pub selected_full: *mut GL_Primitive,
    #[test_offset = 0x48]
    pub weapon_sys: *mut WeaponSystem,
    #[test_offset = 0x50]
    pub drone_sys: *mut DroneSystem,
    #[test_offset = 0x58]
    pub location: Point,
    #[test_offset = 0x60]
    pub hit_box: Rect,
    #[test_offset = 0x70]
    pub item: EquipmentBoxItem,
    #[test_offset = 0x90]
    pub b_mouse_hovering: bool,
    #[test_offset = 0x91]
    pub b_glow: bool,
    #[test_offset = 0x92]
    pub b_blocked: bool,
    #[test_offset = 0x94]
    pub slot: c_int,
    #[test_offset = 0x98]
    pub b_locked: bool,
    #[test_offset = 0x9c]
    pub value: c_int,
    #[test_offset = 0xa0]
    pub b_permanent_lock: bool,
    #[test_offset = 0xa1]
    pub block_detailed: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewEquipBox {
    #[test_offset = 0x0]
    pub base: EquipmentBox,
    #[test_offset = 0xa8]
    pub ship: *mut ShipManager,
    #[test_offset = 0xb0]
    pub b_dead: bool,
    #[test_offset = 0xb8]
    pub delete_button: TextButton,
    #[test_offset = 0x1b8]
    pub rename_button: TextButton,
    #[test_offset = 0x2b8]
    pub b_show_delete: bool,
    #[test_offset = 0x2b9]
    pub b_show_rename: bool,
    #[test_offset = 0x2ba]
    pub b_quick_renaming: bool,
    #[test_offset = 0x2c0]
    pub name_input: TextInput,
    #[test_offset = 0x320]
    pub box_: *mut GL_Primitive,
    #[test_offset = 0x328]
    pub box_on: *mut GL_Primitive,
    #[test_offset = 0x330]
    pub b_confirm_delete: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DropBox {
    #[test_offset = 0x0]
    pub position: Point,
    #[test_offset = 0x8]
    pub is_sell_box: bool,
    #[test_offset = 0x10]
    pub box_image: [*mut GL_Texture; 2],
    #[test_offset = 0x20]
    pub selected_image: c_int,
    #[test_offset = 0x28]
    pub title_text: TextString,
    #[test_offset = 0x38]
    pub body_text: TextString,
    #[test_offset = 0x48]
    pub body_space: c_int,
    #[test_offset = 0x50]
    pub lower_text: TextString,
    #[test_offset = 0x60]
    pub sell_text: TextString,
    #[test_offset = 0x70]
    pub sell_cost_text: StdString,
    #[test_offset = 0x78]
    pub text_width: c_int,
    #[test_offset = 0x7c]
    pub insert_height: c_int,
    #[test_offset = 0x80]
    pub title_insert: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewManifest {
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub box_: *mut GL_Primitive,
    #[test_offset = 0x28]
    pub over_box: DropBox,
    #[test_offset = 0xb0]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0xb8]
    pub crew_boxes: Vector<*mut CrewEquipBox>,
    #[test_offset = 0xd0]
    pub info_box: InfoBox,
    #[test_offset = 0x1a8]
    pub confirming_delete: c_int,
    #[test_offset = 0x1b0]
    pub delete_dialog: ConfirmWindow,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ReactorButton {
    #[test_offset = 0x0]
    pub base: ButtonPrime,
    #[test_offset = 0x88]
    pub b_mirror: bool,
    #[test_offset = 0x8c]
    pub temp_upgrade: c_int,
    #[test_offset = 0x90]
    pub ship: *mut ShipManager,
    #[test_offset = 0x98]
    pub selected: bool,
}

pub unsafe fn power_manager(ship_id: i32) -> Option<&'static PowerManager> {
    (*super::POWER_MANAGERS).get(ship_id as usize)
}

impl ReactorButton {
    pub fn reactor_cost(&self) -> c_int {
        let Some(power) = (unsafe { power_manager((*self.ship).i_ship_id) }) else {
            return 0;
        };
        let p = power.current_power.second + self.temp_upgrade;
        if p >= 5 {
            5 * (p / 5) + 15
        } else {
            30
        }
    }
    pub fn reactor_refund(&self) -> c_int {
        let Some(power) = (unsafe { power_manager((*self.ship).i_ship_id) }) else {
            return 0;
        };
        let p = self.temp_upgrade + power.current_power.second - 1;
        if p >= 5 {
            5 * (p / 5) + 15
        } else {
            30
        }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct UpgradeBox {
    #[test_offset = 0x0]
    pub _sil_do_not_use_system: *mut ShipSystem,
    #[test_offset = 0x8]
    pub ship: *mut ShipManager,
    #[test_offset = 0x10]
    pub blueprint: *const SystemBlueprint,
    #[test_offset = 0x18]
    pub location: Point,
    #[test_offset = 0x20]
    pub temp_upgrade: c_int,
    #[test_offset = 0x28]
    pub current_button: *mut Button,
    #[test_offset = 0x30]
    pub button_base_name: StdString,
    #[test_offset = 0x38]
    pub max_button: Button,
    #[test_offset = 0xc8]
    pub box_button: Button,
    #[test_offset = 0x158]
    pub subsystem: bool,
    #[test_offset = 0x159]
    pub is_dummy: bool,
    #[test_offset = 0x160]
    pub dummy_box: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Upgrades {
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub box_: *mut GL_Texture,
    #[test_offset = 0x28]
    pub v_upgrade_boxes: Vector<*mut UpgradeBox>,
    #[test_offset = 0x40]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x48]
    pub undo_button: TextButton,
    #[test_offset = 0x148]
    pub reactor_button: ReactorButton,
    #[test_offset = 0x1e8]
    pub info_box: InfoBox,
    #[test_offset = 0x2c0]
    pub info_box_loc: Point,
    #[test_offset = 0x2c8]
    pub system_count: c_int,
    #[test_offset = 0x2cc]
    pub force_system_info_width: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TabbedWindow {
    #[test_offset = 0x0]
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub buttons: Vector<*mut Button>,
    #[test_offset = 0x38]
    pub windows: Vector<*mut FocusWindow>,
    #[test_offset = 0x50]
    pub names: Vector<StdString>,
    #[test_offset = 0x68]
    pub current_tab: c_uint,
    #[test_offset = 0x6c]
    pub button_type: c_int,
    #[test_offset = 0x70]
    pub done_button: TextButton,
    #[test_offset = 0x170]
    pub move_: Point,
    #[test_offset = 0x178]
    pub b_block_close: bool,
    #[test_offset = 0x179]
    pub b_tutorial_mode: bool,
    #[test_offset = 0x17a]
    pub b_window_lock: bool,
}

impl TabbedWindow {
    pub fn close(&mut self) {
        let tab = self
            .windows
            .get(self.current_tab as usize)
            .copied()
            .unwrap();
        unsafe {
            (*(*tab).vtable).close(tab);
        }
        self.base.b_open = false;
    }
    pub fn open(&mut self) {
        if self.base.b_open || self.buttons.is_empty() {
            return;
        }
        self.base.b_open = true;
        unsafe {
            self.set_tab(0);
        }
    }
    pub unsafe fn set_tab(&mut self, tab: c_uint) {
        unsafe {
            (super::SET_TAB.unwrap())(ptr::addr_of_mut!(*self), tab);
        }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct InputBox {
    #[test_offset = 0x0]
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub text_box: *mut WindowFrame,
    #[test_offset = 0x28]
    pub main_text: StdString,
    #[test_offset = 0x30]
    pub b_done: bool,
    #[test_offset = 0x31]
    pub b_invert_caps: bool,
    #[test_offset = 0x38]
    pub input_text: StdString,
    #[test_offset = 0x40]
    pub last_inputs: Vector<StdString>,
    #[test_offset = 0x58]
    pub last_input_index: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct LanguageChooser {
    #[test_offset = 0x0]
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub buttons: Vector<*mut TextButton>,
    #[test_offset = 0x38]
    pub i_choice: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ControlButton {
    #[test_offset = 0x0]
    pub rect: Rect,
    #[test_offset = 0x10]
    pub value: StdString,
    #[test_offset = 0x18]
    pub desc: TextString,
    #[test_offset = 0x28]
    pub key: StdString,
    #[test_offset = 0x30]
    pub state: c_int,
    #[test_offset = 0x34]
    pub desc_length: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ControlsScreen {
    #[test_offset = 0x0]
    pub buttons: [Vector<ControlButton>; 4],
    #[test_offset = 0x60]
    pub selected_button: c_int,
    #[test_offset = 0x68]
    pub default_button: TextButton,
    #[test_offset = 0x168]
    pub reset_dialog: ConfirmWindow,
    #[test_offset = 0x3e8]
    pub page_buttons: [Button; 4],
    #[test_offset = 0x628]
    pub current_page: c_int,
    #[test_offset = 0x630]
    pub custom_box: *mut WindowFrame,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SlideBar {
    #[test_offset = 0x0]
    pub box_: Rect,
    #[test_offset = 0x10]
    pub hovering: bool,
    #[test_offset = 0x11]
    pub holding: bool,
    #[test_offset = 0x14]
    pub marker: Rect,
    #[test_offset = 0x24]
    pub mouse_start: Point,
    #[test_offset = 0x2c]
    pub rect_start: Point,
    #[test_offset = 0x34]
    pub min_max: Pair<c_int, c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct OptionsScreen {
    #[test_offset = 0x0]
    pub base: ChoiceBox,
    #[test_offset = 0x218]
    pub position: Point,
    #[test_offset = 0x220]
    pub wipe_profile_position: Point,
    #[test_offset = 0x228]
    pub sound_volume: SlideBar,
    #[test_offset = 0x264]
    pub music_volume: SlideBar,
    #[test_offset = 0x2a0]
    pub b_customize_controls: bool,
    #[test_offset = 0x2a8]
    pub controls: ControlsScreen,
    #[test_offset = 0x8e0]
    pub close_button: TextButton,
    #[test_offset = 0x9e0]
    pub wipe_profile_button: TextButton,
    #[test_offset = 0xae0]
    pub show_sync_achievements: bool,
    #[test_offset = 0xae8]
    pub sync_achievements_button: TextButton,
    #[test_offset = 0xbe8]
    pub choice_fullscreen: c_int,
    #[test_offset = 0xbec]
    pub choice_vsync: c_int,
    #[test_offset = 0xbf0]
    pub choice_frame_limit: c_int,
    #[test_offset = 0xbf4]
    pub choice_lowend: c_int,
    #[test_offset = 0xbf8]
    pub choice_colorblind: c_int,
    #[test_offset = 0xbfc]
    pub choice_language: c_int,
    #[test_offset = 0xc00]
    pub choice_dialog_keys: c_int,
    #[test_offset = 0xc04]
    pub choice_show_paths: c_int,
    #[test_offset = 0xc08]
    pub choice_achievement_popups: c_int,
    #[test_offset = 0xc0c]
    pub choice_auto_pause: c_int,
    #[test_offset = 0xc10]
    pub choice_touch_auto_pause: c_int,
    #[test_offset = 0xc14]
    pub choice_controls: c_int,
    #[test_offset = 0xc18]
    pub last_full_screen: c_int,
    #[test_offset = 0xc1c]
    pub is_sound_touch: bool,
    #[test_offset = 0xc1d]
    pub is_music_touch: bool,
    #[test_offset = 0xc20]
    pub lang_chooser: LanguageChooser,
    #[test_offset = 0xc60]
    pub show_wipe_button: bool,
    #[test_offset = 0xc68]
    pub wipe_profile_dialog: ConfirmWindow,
    #[test_offset = 0xee8]
    pub restart_required_dialog: ChoiceBox,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CreditScreen {
    #[test_offset = 0x0]
    pub scroll: c_float,
    #[test_offset = 0x4]
    pub scroll_speed: c_float,
    #[test_offset = 0x8]
    pub ship_name: StdString,
    #[test_offset = 0x10]
    pub crew_string: StdString,
    #[test_offset = 0x18]
    pub pausing: c_float,
    #[test_offset = 0x20]
    pub bg: *mut GL_Texture,
    #[test_offset = 0x28]
    pub credit_names: Vector<StdString>,
    #[test_offset = 0x40]
    pub last_valid_credit: c_int,
    #[test_offset = 0x44]
    pub touches_down: c_int,
    #[test_offset = 0x48]
    pub touch_down_time: c_double,
    #[test_offset = 0x50]
    pub skip_message_timer: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct GameOver {
    #[test_offset = 0x0]
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub buttons: Vector<*mut TextButton>,
    #[test_offset = 0x38]
    pub box_: *mut GL_Primitive,
    #[test_offset = 0x40]
    pub box_width: c_int,
    #[test_offset = 0x44]
    pub command: c_int,
    #[test_offset = 0x48]
    pub commands: Vector<c_int>,
    #[test_offset = 0x60]
    pub b_show_stats: bool,
    #[test_offset = 0x64]
    pub position: Point,
    #[test_offset = 0x70]
    pub gameover_text: StdString,
    #[test_offset = 0x78]
    pub b_victory: bool,
    #[test_offset = 0x7c]
    pub opened_timer: c_float,
    #[test_offset = 0x80]
    pub credits: CreditScreen,
    #[test_offset = 0xd8]
    pub b_showing_credits: bool,
}

#[repr(i32)]
pub enum ExpandDir {
    // DW_AT_const_value = 0xffffffffffffffff
    Up = -1,
    // DW_AT_const_value = 0x0
    None = 0,
    // DW_AT_const_value = 0x1
    Down = 1,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct InfoBox {
    #[test_offset = 0x0]
    pub location: Point,
    #[test_offset = 0x8]
    pub blueprint: *const SystemBlueprint,
    #[test_offset = 0x10]
    pub desc: Description,
    #[test_offset = 0x70]
    pub temp_upgrade: c_int,
    #[test_offset = 0x74]
    pub power_level: c_int,
    #[test_offset = 0x78]
    pub max_power: c_int,
    #[test_offset = 0x7c]
    pub system_id: c_int,
    #[test_offset = 0x80]
    pub system_width: c_int,
    #[test_offset = 0x84]
    pub y_shift: c_int,
    #[test_offset = 0x88]
    pub desc_box_size: Point,
    #[test_offset = 0x90]
    pub p_crew_blueprint: *const CrewBlueprint,
    #[test_offset = 0x98]
    pub warning: StdString,
    #[test_offset = 0xa0]
    pub b_detailed: bool,
    #[test_offset = 0xa8]
    pub additional_tip: StdString,
    #[test_offset = 0xb0]
    pub additional_warning: StdString,
    #[test_offset = 0xb8]
    pub primary_box: *mut WindowFrame,
    #[test_offset = 0xc0]
    pub primary_box_offset: c_int,
    #[test_offset = 0xc8]
    pub secondary_box: *mut WindowFrame,
    #[test_offset = 0xd0]
    pub drone_blueprint: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[test_offset = 0x8]
    pub name: StdString,
    /// Inherited from Blueprint
    #[test_offset = 0x10]
    pub desc: Description,
    /// Inherited from Blueprint
    #[test_offset = 0x70]
    pub type_: c_int,
    #[test_offset = 0x74]
    pub max_power: c_int,
    #[test_offset = 0x78]
    pub start_power: c_int,
    #[test_offset = 0x80]
    pub upgrade_costs: Vector<c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CAchievement {
    #[test_offset = 0x0]
    pub name_id: StdString,
    #[test_offset = 0x8]
    pub progress: Pair<c_int, c_int>,
    #[test_offset = 0x10]
    pub unlocked: bool,
    #[test_offset = 0x18]
    pub name: TextString,
    #[test_offset = 0x28]
    pub description: TextString,
    #[test_offset = 0x38]
    pub header: TextString,
    #[test_offset = 0x48]
    pub new_achievement: bool,
    #[test_offset = 0x49]
    pub multi_difficulty: bool,
    #[test_offset = 0x4c]
    pub difficulty: c_int,
    #[test_offset = 0x50]
    pub ship: StdString,
    #[test_offset = 0x58]
    pub ship_difficulties: [c_int; 3],
    #[test_offset = 0x64]
    pub dimension: c_int,
    #[test_offset = 0x68]
    pub icon: CachedImage,
    #[test_offset = 0xb0]
    pub mini_icon: CachedImage,
    #[test_offset = 0xf8]
    pub mini_icon_locked: CachedImage,
    #[test_offset = 0x140]
    pub lock_image: CachedImage,
    #[test_offset = 0x188]
    pub dot_on: CachedImage,
    #[test_offset = 0x1d0]
    pub dot_off: CachedImage,
    #[test_offset = 0x218]
    pub outline: *mut GL_Primitive,
    #[test_offset = 0x220]
    pub mini_outline: *mut GL_Primitive,
    #[test_offset = 0x228]
    pub lock_overlay: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipAchievementInfo {
    #[test_offset = 0x0]
    pub achievement: *mut CAchievement,
    #[test_offset = 0x8]
    pub position: Point,
    #[test_offset = 0x10]
    pub dimension: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct MenuScreen {
    #[test_offset = 0x0]
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub main_image: *mut GL_Texture,
    #[test_offset = 0x28]
    pub menu_primitive: *mut GL_Primitive,
    #[test_offset = 0x30]
    pub menu_width: c_int,
    #[test_offset = 0x38]
    pub buttons: Vector<*mut TextButton>,
    #[test_offset = 0x50]
    pub command: c_int,
    #[test_offset = 0x58]
    pub commands: Vector<c_int>,
    #[test_offset = 0x70]
    pub position: Point,
    #[test_offset = 0x78]
    pub confirm_dialog: ConfirmWindow,
    #[test_offset = 0x2f8]
    pub temp_command: c_int,
    #[test_offset = 0x300]
    pub save_quit: *mut GenericButton,
    #[test_offset = 0x308]
    pub b_show_controls: bool,
    #[test_offset = 0x30c]
    pub status_position: Point,
    #[test_offset = 0x318]
    pub difficulty_box: *mut GL_Texture,
    #[test_offset = 0x320]
    pub difficulty_width: c_int,
    #[test_offset = 0x328]
    pub difficulty_label: StdString,
    #[test_offset = 0x330]
    pub difficulty_text: StdString,
    #[test_offset = 0x338]
    pub dlc_box: *mut GL_Texture,
    #[test_offset = 0x340]
    pub dlc_width: c_int,
    #[test_offset = 0x348]
    pub dlc_label: StdString,
    #[test_offset = 0x350]
    pub dlc_text: StdString,
    #[test_offset = 0x358]
    pub ach_box: *mut GL_Texture,
    #[test_offset = 0x360]
    pub ach_box_primitive: *mut GL_Primitive,
    #[test_offset = 0x368]
    pub ach_width: c_int,
    #[test_offset = 0x370]
    pub ach_label: StdString,
    #[test_offset = 0x378]
    pub ship_achievements: Vector<ShipAchievementInfo>,
    #[test_offset = 0x390]
    pub selected_ach: c_int,
    #[test_offset = 0x398]
    pub info: InfoBox,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct NebulaInfo {
    #[test_offset = 0x0]
    pub primitive: *mut GL_Primitive,
    #[test_offset = 0x8]
    pub x: c_int,
    #[test_offset = 0xc]
    pub y: c_int,
    #[test_offset = 0x10]
    pub w: c_int,
    #[test_offset = 0x14]
    pub h: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct RandomAmount {
    #[test_offset = 0x0]
    pub min: c_int,
    #[test_offset = 0x4]
    pub max: c_int,
    #[test_offset = 0x8]
    pub chance_none: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SectorDescription {
    #[test_offset = 0x0]
    pub event_counts: Vector<Pair<StdString, RandomAmount>>,
    #[test_offset = 0x18]
    pub rarities: Vector<Pair<StdString, c_int>>,
    #[test_offset = 0x30]
    pub unique: bool,
    #[test_offset = 0x38]
    pub names: Vector<TextString>,
    #[test_offset = 0x50]
    pub short_names: Vector<TextString>,
    #[test_offset = 0x68]
    pub music_tracks: Vector<StdString>,
    #[test_offset = 0x80]
    pub type_: StdString,
    #[test_offset = 0x88]
    pub name: TextString,
    #[test_offset = 0x98]
    pub short_name: TextString,
    #[test_offset = 0xa8]
    pub min_sector: c_int,
    #[test_offset = 0xac]
    pub used: bool,
    #[test_offset = 0xb0]
    pub first_event: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Sector {
    #[test_offset = 0x0]
    pub type_: c_int,
    #[test_offset = 0x4]
    pub visited: bool,
    #[test_offset = 0x5]
    pub reachable: bool,
    #[test_offset = 0x8]
    pub neighbors: Vector<*mut Sector>,
    #[test_offset = 0x20]
    pub location: Point,
    #[test_offset = 0x28]
    pub level: c_int,
    #[test_offset = 0x30]
    pub description: SectorDescription,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DistressButton {
    #[test_offset = 0x0]
    pub base: TextButton,
    #[test_offset = 0x100]
    pub labels: [TextString; 2],
    #[test_offset = 0x120]
    pub state: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct StarMap {
    #[test_offset = 0x0]
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub visual_size: c_float,
    #[test_offset = 0x28]
    pub locations: Vector<*mut Location>,
    #[test_offset = 0x40]
    pub locations_grid: Map<Point, Vector<*mut Location>>,
    #[test_offset = 0x70]
    pub temp_path: Vector<*mut Location>,
    #[test_offset = 0x88]
    pub current_loc: *mut Location,
    #[test_offset = 0x90]
    pub potential_loc: *mut Location,
    #[test_offset = 0x98]
    pub hover_loc: *mut Location,
    #[test_offset = 0xa0]
    pub position: Point,
    #[test_offset = 0xa8]
    pub i_populated_tiles: c_int,
    #[test_offset = 0xac]
    pub i_location_count: c_int,
    #[test_offset = 0xb0]
    pub i_empty_tiles: c_int,
    #[test_offset = 0xb4]
    pub b_initialized_display: bool,
    #[test_offset = 0xb8]
    pub translation: Pointf,
    #[test_offset = 0xc0]
    pub ready_to_travel: bool,
    #[test_offset = 0xc4]
    pub danger_zone: Point,
    #[test_offset = 0xcc]
    pub danger_zone_radius: c_float,
    #[test_offset = 0xd0]
    pub ship_rotation: [c_float; 2],
    #[test_offset = 0xd8]
    pub end_button: TextButton,
    #[test_offset = 0x1d8]
    pub wait_button: TextButton,
    #[test_offset = 0x2d8]
    pub distress_button: DistressButton,
    #[test_offset = 0x400]
    pub jump_button: TextButton,
    #[test_offset = 0x500]
    pub world_level: c_int,
    #[test_offset = 0x504]
    pub b_map_revealed: bool,
    #[test_offset = 0x508]
    pub pursuit_delay: c_int,
    #[test_offset = 0x50c]
    pub sector_name_font: c_int,
    #[test_offset = 0x510]
    pub map_border: WindowFrame,
    #[test_offset = 0x538]
    pub map_border_title: *mut GL_Primitive,
    #[test_offset = 0x540]
    pub map_border_title_mask: *mut GL_Primitive,
    #[test_offset = 0x548]
    pub map_border_sector: *mut GL_Texture,
    #[test_offset = 0x550]
    pub map_inset_text_left: *mut GL_Texture,
    #[test_offset = 0x558]
    pub map_inset_text_middle: *mut GL_Texture,
    #[test_offset = 0x560]
    pub map_inset_text_right: *mut GL_Texture,
    #[test_offset = 0x568]
    pub map_inset_text_jump: *mut GL_Texture,
    #[test_offset = 0x570]
    pub map_inset_wait_distress: *mut GL_Texture,
    #[test_offset = 0x578]
    pub red_light: *mut GL_Primitive,
    #[test_offset = 0x580]
    pub fuel_message: *mut GL_Primitive,
    #[test_offset = 0x588]
    pub waiting_message: *mut GL_Primitive,
    #[test_offset = 0x590]
    pub unexplored: *mut GL_Primitive,
    #[test_offset = 0x598]
    pub explored: *mut GL_Primitive,
    #[test_offset = 0x5a0]
    pub danger: *mut GL_Primitive,
    #[test_offset = 0x5a8]
    pub warning: *mut GL_Primitive,
    #[test_offset = 0x5b0]
    pub yellow_warning: *mut GL_Primitive,
    #[test_offset = 0x5b8]
    pub warning_circle: *mut GL_Primitive,
    #[test_offset = 0x5c0]
    pub nebula_circle: *mut GL_Primitive,
    #[test_offset = 0x5c8]
    pub box_green: [*mut GL_Texture; 3],
    #[test_offset = 0x5e0]
    pub box_purple: [*mut GL_Texture; 3],
    #[test_offset = 0x5f8]
    pub box_white: [*mut GL_Texture; 3],
    #[test_offset = 0x610]
    pub ship: *mut GL_Primitive,
    #[test_offset = 0x618]
    pub ship_no_fuel: *mut GL_Primitive,
    #[test_offset = 0x620]
    pub boss_ship: *mut GL_Primitive,
    #[test_offset = 0x628]
    pub danger_zone_edge: *mut GL_Primitive,
    #[test_offset = 0x630]
    pub danger_zone_tile: *mut GL_Texture,
    #[test_offset = 0x638]
    pub danger_zone_advance: *mut GL_Primitive,
    #[test_offset = 0x640]
    pub target_box: *mut GL_Primitive,
    #[test_offset = 0x648]
    pub sector_target_box_green: *mut GL_Primitive,
    #[test_offset = 0x650]
    pub sector_target_box_yellow: *mut GL_Primitive,
    #[test_offset = 0x658]
    pub target_box_timer: AnimationTracker,
    #[test_offset = 0x678]
    pub close_button: TextButton,
    #[test_offset = 0x778]
    pub desc_box: *mut WindowFrame,
    #[test_offset = 0x780]
    pub shadow: *mut GL_Primitive,
    #[test_offset = 0x788]
    pub warning_shadow: *mut GL_Primitive,
    #[test_offset = 0x790]
    pub fuel_overlay: *mut GL_Primitive,
    #[test_offset = 0x798]
    pub danger_flash: *mut GL_Primitive,
    #[test_offset = 0x7a0]
    pub maps_bottom: [*mut GL_Primitive; 3],
    #[test_offset = 0x7b8]
    pub dotted_line: *mut GL_Texture,
    #[test_offset = 0x7c0]
    pub cross: *mut GL_Texture,
    #[test_offset = 0x7c8]
    pub boss_jumps_box: *mut GL_Texture,
    #[test_offset = 0x7d0]
    pub small_nebula: Vector<ImageDesc>,
    #[test_offset = 0x7e8]
    pub large_nebula: Vector<ImageDesc>,
    #[test_offset = 0x800]
    pub current_nebulas: Vector<NebulaInfo>,
    #[test_offset = 0x818]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x820]
    pub out_of_fuel: bool,
    pub waiting: AnimationTracker,
    #[test_offset = 0x848]
    pub danger_wait_start: c_int,
    #[test_offset = 0x850]
    pub distress_anim: AnimationTracker,
    #[test_offset = 0x870]
    pub b_tutorial_generated: bool,
    #[test_offset = 0x878]
    pub delayed_quests: Vector<StdString>,
    #[test_offset = 0x890]
    pub sectors: Vector<*mut Sector>,
    #[test_offset = 0x8a8]
    pub current_sector: *mut Sector,
    #[test_offset = 0x8b0]
    pub secret_sector: *mut Sector,
    #[test_offset = 0x8b8]
    pub b_choosing_new_sector: bool,
    #[test_offset = 0x8b9]
    pub b_secret_sector: bool,
    #[test_offset = 0x8c0]
    pub dummy_new_sector: Location,
    #[test_offset = 0x9a0]
    pub maps_analyzed: c_int,
    #[test_offset = 0x9a4]
    pub locations_created: c_int,
    #[test_offset = 0x9a8]
    pub ships_created: c_int,
    #[test_offset = 0x9b0]
    pub scrap_collected: Map<StdString, c_int>,
    #[test_offset = 0x9e0]
    pub drones_collected: Map<StdString, c_int>,
    #[test_offset = 0xa10]
    pub fuel_collected: Map<StdString, c_int>,
    #[test_offset = 0xa40]
    pub weapon_found: Map<StdString, c_int>,
    #[test_offset = 0xa70]
    pub drone_found: Map<StdString, c_int>,
    #[test_offset = 0xaa0]
    pub boss_loc: c_int,
    #[test_offset = 0xaa4]
    pub arrived_at_base: c_int,
    #[test_offset = 0xaa8]
    pub reversed_path: bool,
    #[test_offset = 0xaa9]
    pub boss_jumping: bool,
    #[test_offset = 0xab0]
    pub boss_path: Vector<*mut Location>,
    #[test_offset = 0xac8]
    pub boss_level: bool,
    #[test_offset = 0xac9]
    pub boss_wait: bool,
    #[test_offset = 0xacc]
    pub boss_position: Pointf,
    #[test_offset = 0xad8]
    pub force_sector_choice: StdString,
    #[test_offset = 0xae0]
    pub b_enemy_ship: bool,
    #[test_offset = 0xae1]
    pub b_nebula_map: bool,
    #[test_offset = 0xae2]
    pub b_infinite_mode: bool,
    #[test_offset = 0xae8]
    pub last_sectors: Vector<*mut Sector>,
    #[test_offset = 0xb00]
    pub close_sector_button: TextButton,
    #[test_offset = 0xc00]
    pub sector_map_seed: c_int,
    #[test_offset = 0xc04]
    pub current_sector_seed: c_int,
    #[test_offset = 0xc08]
    pub fuel_event_seed: c_int,
    #[test_offset = 0xc10]
    pub last_escape_event: StdString,
    #[test_offset = 0xc18]
    pub waited_last: bool,
    #[test_offset = 0xc20]
    pub store_trash: Vector<*mut Store>,
    #[test_offset = 0xc38]
    pub added_quests: Vector<Pair<StdString, c_int>>,
    #[test_offset = 0xc50]
    pub boss_stage: c_int,
    #[test_offset = 0xc58]
    pub boss_message: TextString,
    #[test_offset = 0xc68]
    pub boss_jumping_warning: *mut WarningMessage,
    #[test_offset = 0xc70]
    pub crystal_alien_found: bool,
    #[test_offset = 0xc78]
    pub found_map: Map<*mut Location, bool>,
    #[test_offset = 0xca8]
    pub sector_map_offset: Point,
    #[test_offset = 0xcb0]
    pub potential_sector_choice: c_int,
    #[test_offset = 0xcb4]
    pub final_sector_choice: c_int,
    #[test_offset = 0xcb8]
    pub sector_hit_boxes: Vector<Rect>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SpaceStatus {
    #[test_offset = 0x0]
    pub warning_images: [*mut GL_Primitive; 10],
    #[test_offset = 0x50]
    pub warning_message: *mut WarningMessage,
    #[test_offset = 0x58]
    pub incoming_fire: *mut WarningMessage,
    #[test_offset = 0x60]
    pub hitbox: Rect,
    #[test_offset = 0x70]
    pub hitbox2: Rect,
    #[test_offset = 0x80]
    pub current_effect: c_int,
    #[test_offset = 0x84]
    pub current_effect2: c_int,
    #[test_offset = 0x88]
    pub space: *mut SpaceManager,
    #[test_offset = 0x90]
    pub position: Point,
    #[test_offset = 0x98]
    pub touched_tooltip: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct FTLButton {
    #[test_offset = 0x0]
    pub base: TextButtonPrime,
    #[test_offset = 0xf8]
    pub text_y_offset: c_int,
    #[test_offset = 0xfc]
    pub auto_shrink: bool,
    #[test_offset = 0xfd]
    pub ready: bool,
    #[test_offset = 0x100]
    pub ftl_blink: c_float,
    #[test_offset = 0x104]
    pub ftl_blink_dx: c_float,
    #[test_offset = 0x108]
    pub pullout: c_float,
    #[test_offset = 0x110]
    pub ship: *mut ShipManager,
    #[test_offset = 0x118]
    pub base_image: *mut GL_Primitive,
    #[test_offset = 0x120]
    pub base_image_red: *mut GL_Primitive,
    #[test_offset = 0x128]
    pub pullout_base: *mut GL_Primitive,
    #[test_offset = 0x130]
    pub pullout_base_red: *mut GL_Primitive,
    #[test_offset = 0x138]
    pub pilot_on: *mut GL_Primitive,
    #[test_offset = 0x140]
    pub pilot_off1: *mut GL_Primitive,
    #[test_offset = 0x148]
    pub pilot_off2: *mut GL_Primitive,
    #[test_offset = 0x150]
    pub engines_on: *mut GL_Primitive,
    #[test_offset = 0x158]
    pub engines_off1: *mut GL_Primitive,
    #[test_offset = 0x160]
    pub engines_off2: *mut GL_Primitive,
    #[test_offset = 0x168]
    pub ftl_loadingbars: *mut GL_Texture,
    #[test_offset = 0x170]
    pub ftl_loadingbars_off: *mut GL_Texture,
    #[test_offset = 0x178]
    pub loading_bars: *mut GL_Primitive,
    #[test_offset = 0x180]
    pub loading_bars_off: *mut GL_Primitive,
    #[test_offset = 0x188]
    pub last_bars_width: c_int,
    #[test_offset = 0x190]
    pub engines_down: *mut WarningMessage,
    #[test_offset = 0x198]
    pub b_out_of_fuel: bool,
    #[test_offset = 0x199]
    pub b_boss_fight: bool,
    #[test_offset = 0x19a]
    pub b_hover_raw: bool,
    #[test_offset = 0x19b]
    pub b_hover_pilot: bool,
    #[test_offset = 0x19c]
    pub b_hover_engine: bool,
}

impl FTLButton {
    pub fn mouse_click(&self) -> bool {
        if !self.base.base.b_active {
            return false;
        }
        if !unsafe {
            (*self.ship)
                .system(System::Engines)
                .is_some_and(|x| x.functioning())
        } {
            return false;
        }
        if !unsafe {
            (*self.ship)
                .system(System::Pilot)
                .is_some_and(|x| x.functioning())
        } {
            return false;
        }
        if unsafe { (*self.ship).jump_timer.first < (*self.ship).jump_timer.second } {
            return false;
        }
        true
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct HandAnimation {
    #[test_offset = 0x0]
    pub hand: *mut GL_Texture,
    #[test_offset = 0x8]
    pub start: Point,
    #[test_offset = 0x10]
    pub finish: Point,
    #[test_offset = 0x18]
    pub location: Pointf,
    #[test_offset = 0x20]
    pub b_running: bool,
    #[test_offset = 0x24]
    pub pause: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneControl {
    pub base: ArmamentControl,
    #[test_offset = 0xd0]
    pub drone_message: WarningMessage,
    #[test_offset = 0x1b0]
    pub no_target_message: WarningMessage,
    #[test_offset = 0x290]
    pub system_message: WarningMessage,
}

#[vtable]
pub struct VtableArmamentControl {
    pub dtor: Option<fn(*mut ArmamentControl)>,
    pub delete_dtor: Option<fn(*mut ArmamentControl)>,
    pub on_language_change: Option<fn(*mut ArmamentControl)>,
    pub on_loop: Option<fn(*mut ArmamentControl)>,
    pub on_render: Option<fn(*mut ArmamentControl, bool)>,
    pub render_touch_tooltips: Option<fn(*mut ArmamentControl, bool)>,
    pub render_labels: Option<fn(*mut ArmamentControl)>,
    pub render_warnings: Option<fn(*mut ArmamentControl)>,
    pub render_dragging: Option<fn(*mut ArmamentControl)>,
    pub is_dragging: Option<fn(*mut ArmamentControl) -> bool>,
    pub restart: Option<fn(*mut ArmamentControl)>,
    pub on_cleanup: Option<fn(*mut ArmamentControl)>,
    pub close: Option<fn(*mut ArmamentControl)>,
    pub set_open: Option<fn(*mut ArmamentControl, bool)>,
    pub l_button: Option<fn(*mut ArmamentControl, c_int, c_int, bool) -> bool>,
    pub l_button_up: Option<fn(*mut ArmamentControl, c_int, c_int, bool) -> bool>,
    pub r_button: Option<fn(*mut ArmamentControl, c_int, c_int, bool)>,
    pub mouse_move: Option<fn(*mut ArmamentControl, c_int, c_int)>,
    pub on_touch:
        Option<fn(*mut ArmamentControl, TouchAction, c_int, c_int, c_int, c_int, c_int) -> bool>,
    pub key_down: Option<fn(*mut ArmamentControl, SDLKey) -> bool>,
    pub link_ship: Option<fn(*mut ArmamentControl, *mut ShipManager)>,
    pub create_armament_box: Option<fn(*mut ArmamentControl, Point) -> *mut ArmamentBox>,
    pub num_armament_slots: Option<fn(*mut ArmamentControl) -> c_int>,
    pub armament_box_origin: Option<fn(*mut ArmamentControl) -> Point>,
    pub holder_label: Option<fn(*mut ArmamentControl) -> TextString>,
    pub armament_hotkey: Option<fn(*mut ArmamentControl, c_uint) -> SDLKey>,
    pub select_armament: Option<fn(*mut ArmamentControl, c_uint)>,
    pub deselect_armament: Option<fn(*mut ArmamentControl, c_uint)>,
    pub swap_armaments: Option<fn(*mut ArmamentControl, c_uint, c_uint)>,
}

#[vtable]
pub struct VtableArmamentBox {
    pub dtor: Option<fn(*mut ArmamentBox)>,
    pub delete_dtor: Option<fn(*mut ArmamentBox)>,
    pub empty: Option<fn(*mut ArmamentBox) -> bool>,
    pub name: Option<fn(*mut ArmamentBox) -> StdString>,
    pub powered: Option<fn(*mut ArmamentBox) -> bool>,
    pub set_default_autofire: Option<fn(*mut ArmamentBox, bool)>,
    pub real_required_power: Option<fn(*mut ArmamentBox) -> c_int>,
    pub get_bonus_power: Option<fn(*mut ArmamentBox) -> c_int>,
    pub get_type: Option<fn(*mut ArmamentBox) -> StdString>,
    pub status_color: Option<fn(*mut ArmamentBox) -> GL_Color>,
    pub generate_tooltip: Option<fn(*mut ArmamentBox) -> StdString>,
    pub on_loop: Option<fn(*mut ArmamentBox)>,
    pub render_touch_tooltip: Option<fn(*mut ArmamentBox, c_int, bool)>,
    pub on_render: Option<fn(*mut ArmamentBox, bool, bool)>,
    pub render_box: Option<fn(*mut ArmamentBox, bool, bool)>,
    pub render_labels: Option<fn(*mut ArmamentBox)>,
    pub render_icon: Option<fn(*mut ArmamentBox, *const Point)>,
    pub get_hacked: Option<fn(*mut ArmamentBox) -> bool>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ArmamentBox {
    pub vtable: *const VtableArmamentBox,
    #[test_offset = 0x8]
    pub background: Vector<*mut GL_Primitive>,
    #[test_offset = 0x20]
    pub empty_background: *mut GL_Primitive,
    #[test_offset = 0x28]
    pub hover_highlight: *mut GL_Primitive,
    #[test_offset = 0x30]
    pub outline: *mut GL_Primitive,
    #[test_offset = 0x38]
    pub empty_outline: *mut GL_Primitive,
    #[test_offset = 0x40]
    pub power_bar_glow: [*mut GL_Primitive; 4],
    #[test_offset = 0x60]
    pub icon_background: *mut GL_Primitive,
    #[test_offset = 0x68]
    pub icon_inset_background: *mut GL_Primitive,
    #[test_offset = 0x70]
    pub icon: *mut GL_Primitive,
    #[test_offset = 0x78]
    pub icon_double_size: *mut GL_Primitive,
    #[test_offset = 0x80]
    pub icon_name: StdString,
    #[test_offset = 0x88]
    pub icon_background_name: StdString,
    #[test_offset = 0x90]
    pub last_icon_pos: Point,
    #[test_offset = 0x98]
    pub location: Point,
    #[test_offset = 0xa0]
    pub x_offset: c_int,
    #[test_offset = 0xa4]
    pub large_icon_offset: Point,
    #[test_offset = 0xac]
    pub name_offset: Point,
    #[test_offset = 0xb4]
    pub name_width: c_int,
    #[test_offset = 0xb8]
    pub mouse_hover: bool,
    #[test_offset = 0xb9]
    pub touch_hover: bool,
    #[test_offset = 0xba]
    pub touch_highlight: bool,
    #[test_offset = 0xbb]
    pub selected: bool,
    #[test_offset = 0xbc]
    pub hot_key: c_int,
    #[test_offset = 0xc0]
    pub active_touch: c_int,
    #[test_offset = 0xc8]
    pub touch_tooltip: *mut TouchTooltip,
    #[test_offset = 0xd0]
    pub hack_animation: Animation,
    #[test_offset = 0x190]
    pub touch_button_border: *mut GL_Primitive,
    #[test_offset = 0x198]
    pub touch_button_border_rect: Rect,
    #[test_offset = 0x1a8]
    pub touch_button_slide_pos: c_float,
    #[test_offset = 0x1b0]
    pub touch_buttons: Vector<*mut GenericButton>,
    #[test_offset = 0x1c8]
    pub touch_button_hitbox: Rect,
    #[test_offset = 0x1d8]
    pub icon_color: GL_Color,
    #[test_offset = 0x1e8]
    pub drone_variation: bool,
    #[test_offset = 0x1e9]
    pub b_ioned: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneBox {
    pub base: ArmamentBox,
    pub p_drone: *mut Drone,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponBox {
    #[test_offset = 0x0]
    pub base: ArmamentBox,
    #[test_offset = 0x1f0]
    pub p_weapon: *mut ProjectileFactory,
    #[test_offset = 0x1f8]
    pub armed: bool,
    #[test_offset = 0x1f9]
    pub armed_for_autofire: bool,
    #[test_offset = 0x1fc]
    pub cooldown_max: c_float,
    #[test_offset = 0x200]
    pub cooldown_modifier: c_float,
    #[test_offset = 0x204]
    pub cooldown_point: Point,
    #[test_offset = 0x20c]
    pub cooldown_width: c_int,
    #[test_offset = 0x210]
    pub cooldown_height: c_int,
    #[test_offset = 0x218]
    pub cooldown_box: Vector<*mut GL_Primitive>,
    #[test_offset = 0x230]
    pub cooldown_bar: *mut GL_Primitive,
    #[test_offset = 0x238]
    pub charge_icons: Vector<CachedImage>,
    #[test_offset = 0x250]
    pub default_autofire: bool,
    #[test_offset = 0x251]
    pub was_charged: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ArmamentControl {
    #[test_offset = 0x0]
    pub vtable: *const VtableArmamentControl,
    #[test_offset = 0x8]
    pub system_id: c_int,
    #[test_offset = 0x10]
    pub gui: *mut CommandGui,
    #[test_offset = 0x18]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x20]
    pub boxes: Vector<*mut ArmamentBox>,
    #[test_offset = 0x38]
    pub location: Point,
    #[test_offset = 0x40]
    pub touch_hit_box: Rect,
    #[test_offset = 0x50]
    pub holder_image: *mut GL_Texture,
    #[test_offset = 0x58]
    pub holder: *mut GL_Primitive,
    #[test_offset = 0x60]
    pub holder_tab: *mut GL_Primitive,
    #[test_offset = 0x68]
    pub small_box_holder: Vector<*mut GL_Primitive>,
    #[test_offset = 0x80]
    pub small_box_hack_anim: Vector<Animation>,
    #[test_offset = 0x98]
    pub small_box_holder_top: c_int,
    #[test_offset = 0x9c]
    pub b_open: bool,
    #[test_offset = 0xa0]
    pub last_mouse: Point,
    #[test_offset = 0xa8]
    pub current_mouse: Point,
    #[test_offset = 0xb0]
    pub dragging_box: c_int,
    #[test_offset = 0xb4]
    pub dragging_touch: c_int,
    #[test_offset = 0xb8]
    pub b_dragging: bool,
    #[test_offset = 0xbc]
    pub i_last_swap_slot: c_int,
    #[test_offset = 0xc0]
    pub b_tutorial_flash: bool,
    #[test_offset = 0xc4]
    pub i_flash_slot: c_int,
    #[test_offset = 0xc8]
    pub active_touch: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponControl {
    #[test_offset = 0x0]
    pub base: ArmamentControl,
    #[test_offset = 0xd0]
    pub current_target: *mut Targetable,
    #[test_offset = 0xd8]
    pub armed_weapon: *mut ProjectileFactory,
    #[test_offset = 0xe0]
    pub auto_firing: bool,
    #[test_offset = 0xe8]
    pub auto_fire_button: TextButton,
    #[test_offset = 0x1e8]
    pub auto_fire_base: *mut GL_Primitive,
    #[test_offset = 0x1f0]
    pub target_icon: [*mut GL_Primitive; 4],
    #[test_offset = 0x210]
    pub target_icon_yellow: [*mut GL_Primitive; 4],
    #[test_offset = 0x230]
    pub auto_fire_focus: Pointf,
    #[test_offset = 0x238]
    pub missile_message: WarningMessage,
    #[test_offset = 0x318]
    pub system_message: WarningMessage,
    #[test_offset = 0x3f8]
    pub armed_slot: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CombatControl {
    #[test_offset = 0x0]
    pub gui: *mut CommandGui,
    #[test_offset = 0x8]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x10]
    pub player_ship_position: Point,
    #[test_offset = 0x18]
    pub space: *mut SpaceManager,
    #[test_offset = 0x20]
    pub weap_control: WeaponControl,
    #[test_offset = 0x420]
    pub drone_control: DroneControl,
    #[test_offset = 0x790]
    pub sys_boxes: Vector<*mut SystemBox>,
    #[test_offset = 0x7a8]
    pub enemy_ships: Vector<*mut CompleteShip>,
    #[test_offset = 0x7c0]
    pub current_target: *mut CompleteShip,
    #[test_offset = 0x7c8]
    pub current_drone: *mut SpaceDrone,
    #[test_offset = 0x7d0]
    pub position: Point,
    #[test_offset = 0x7d8]
    pub selected_room: c_int,
    #[test_offset = 0x7dc]
    pub selected_self_room: c_int,
    #[test_offset = 0x7e0]
    pub target_position: Point,
    #[test_offset = 0x7e8]
    pub box_position: Point,
    #[test_offset = 0x7f0]
    pub hostile_box_frame: *mut WindowFrame,
    #[test_offset = 0x7f8]
    pub health_mask: CachedImage,
    #[test_offset = 0x840]
    pub shield_circle_charged: [CachedImage; 5],
    #[test_offset = 0x9a8]
    pub shield_circle_uncharged: [CachedImage; 5],
    #[test_offset = 0xb10]
    pub shield_circle_hacked: [CachedImage; 5],
    #[test_offset = 0xc78]
    pub shield_circle_hacked_charged: [CachedImage; 5],
    #[test_offset = 0xde0]
    pub shield_charge_box: CachedImage,
    #[test_offset = 0xe28]
    pub super_shield_box5: CachedImage,
    #[test_offset = 0xe70]
    pub super_shield_box12: CachedImage,
    #[test_offset = 0xeb8]
    pub open: bool,
    #[test_offset = 0xebc]
    pub ship_icon_size: c_float,
    #[test_offset = 0xec0]
    pub potential_aiming: Pointf,
    #[test_offset = 0xec8]
    pub aiming_points: Vector<Pointf>,
    #[test_offset = 0xee0]
    pub last_mouse: Pointf,
    #[test_offset = 0xee8]
    pub mouse_down: bool,
    #[test_offset = 0xee9]
    pub is_aiming_touch: bool,
    #[test_offset = 0xeea]
    pub moving_beam: bool,
    #[test_offset = 0xeec]
    pub beam_move_last: Point,
    #[test_offset = 0xef4]
    pub invalid_beam_touch: bool,
    #[test_offset = 0xef8]
    pub screen_reposition: Point,
    #[test_offset = 0xf00]
    pub teleport_command: Pair<c_int, c_int>,
    #[test_offset = 0xf08]
    pub i_teleport_armed: c_int,
    #[test_offset = 0xf10]
    pub teleport_target_send: CachedImage,
    #[test_offset = 0xf58]
    pub teleport_target_return: CachedImage,
    #[test_offset = 0xfa0]
    pub hack_target: CachedImage,
    #[test_offset = 0xfe8]
    pub mind_target: CachedImage,
    #[test_offset = 0x1030]
    pub ftl_timer: AnimationTracker,
    #[test_offset = 0x1050]
    pub ftl_warning: WarningMessage,
    #[test_offset = 0x1130]
    pub hacking_timer: AnimationTracker,
    #[test_offset = 0x1150]
    pub hacking_messages: Vector<StdString>,
    #[test_offset = 0x1168]
    boss_visual: bool,
    #[test_offset = 0x1169]
    pub b_teaching_beam: bool,
    #[test_offset = 0x1170]
    pub tip_box: *mut WindowFrame,
    #[test_offset = 0x1178]
    pub hand: HandAnimation,
}

impl CombatControl {
    pub unsafe fn ship_manager(&self) -> &ShipManager {
        unsafe { &*self.ship_manager }
    }
    pub unsafe fn weapons_armed(&self) -> bool {
        self.ship_manager().has_system(System::Teleporter)
            && self.ship_manager().teleport_system().i_armed != 0
            || !self.weap_control.armed_weapon.is_null()
            || self.ship_manager().has_system(System::Mind)
                && self.ship_manager().mind_system().i_armed != 0
            || self.ship_manager().has_system(System::Hacking)
                && self.ship_manager().hacking_system().b_armed
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct PowerBars {
    #[test_offset = 0x0]
    pub normal: [*mut GL_Primitive; 30],
    #[test_offset = 0xf0]
    pub tiny: [*mut GL_Primitive; 30],
    #[test_offset = 0x1e0]
    pub empty: [*mut GL_Primitive; 30],
    #[test_offset = 0x2d0]
    pub damaged: [*mut GL_Primitive; 30],
}

#[vtable]
pub struct VtableSystemBox {
    pub render_touch_tooltips: Option<fn(*mut SystemBox, bool)>,
    pub dtor: Option<fn(*mut SystemBox)>,
    pub delete_dtor: Option<fn(*mut SystemBox)>,
    pub has_button: Option<fn(*mut SystemBox) -> bool>,
    pub get_cooldown_bar_height: Option<fn(*mut SystemBox) -> c_int>,
    pub get_height_modifier: Option<fn(*mut SystemBox) -> c_int>,
    pub on_loop: Option<fn(*mut SystemBox)>,
    pub get_extended_hit_box: Option<fn(*mut SystemBox) -> Rect>,
    pub on_render: Option<fn(*mut SystemBox, bool)>,
    pub render_hit_box: Option<fn(*mut SystemBox)>,
    pub tap_box_system_height: Option<fn(*mut SystemBox, c_int, c_int, bool) -> c_int>,
    pub render_tap_box: Option<fn(*mut SystemBox)>,
    pub set_show_power: Option<fn(*mut SystemBox, bool)>,
    pub set_power_alpha: Option<fn(*mut SystemBox, c_float)>,
    pub get_mouse_hover: Option<fn(*mut SystemBox) -> bool>,
    pub mouse_move: Option<fn(*mut SystemBox, c_int, c_int)>,
    pub mouse_click: Option<fn(*mut SystemBox, bool) -> bool>,
    pub mouse_right_click: Option<fn(*mut SystemBox, bool)>,
    pub on_touch: Option<fn(*mut SystemBox, TouchAction, c_int, c_int, c_int, c_int, c_int)>,
    pub cancel_touch: Option<fn(*mut SystemBox)>,
    pub close_tap_box: Option<fn(*mut SystemBox)>,
    pub is_tapped: Option<fn(*mut SystemBox) -> bool>,
    pub force_tapped: Option<fn(*mut SystemBox, bool)>,
    pub is_touch_tooltip_open: Option<fn(*mut SystemBox) -> bool>,
    pub is_touch_tooltip_active: Option<fn(*mut SystemBox) -> bool>,
    pub close_touch_tooltip: Option<fn(*mut SystemBox, bool)>,
    pub key_down: Option<fn(*mut SystemBox, SDLKey, bool)>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TapBoxFrame {
    #[test_offset = 0x0]
    pub location: Point,
    #[test_offset = 0x8]
    pub use_wide_box: bool,
    #[test_offset = 0xc]
    pub box_height: c_int,
    #[test_offset = 0x10]
    pub button_heights: Vector<c_int>,
    #[test_offset = 0x28]
    pub primitives: Vector<*mut GL_Primitive>,
    #[test_offset = 0x40]
    pub hit_box: Rect,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TouchTooltip {
    #[test_offset = 0x0]
    pub position: Point,
    #[test_offset = 0x8]
    pub tab_offset: Point,
    #[test_offset = 0x10]
    pub mirrored: bool,
    #[test_offset = 0x18]
    pub text: StdString,
    #[test_offset = 0x20]
    pub tray_width: c_int,
    #[test_offset = 0x24]
    pub tray_height: c_int,
    #[test_offset = 0x28]
    pub tab: *mut GL_Primitive,
    #[test_offset = 0x30]
    pub tab_size: Point,
    #[test_offset = 0x38]
    pub tray: *mut GL_Primitive,
    #[test_offset = 0x40]
    pub tab_hit_box: Rect,
    #[test_offset = 0x50]
    pub tray_hit_box: Rect,
    #[test_offset = 0x60]
    pub slide_offset: c_int,
    #[test_offset = 0x64]
    pub is_open: bool,
    #[test_offset = 0x65]
    pub is_snapping: bool,
    #[test_offset = 0x68]
    pub snap_target_offset: c_int,
    #[test_offset = 0x70]
    pub snap_last_timestamp: c_double,
    #[test_offset = 0x78]
    active_touch: c_int,
    #[test_offset = 0x7c]
    pub ignore_touch: bool,
    #[test_offset = 0x80]
    pub initial_slide_offset: c_int,
    #[test_offset = 0x84]
    pub last_touch_delta: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemBox {
    pub vtable: *const VtableSystemBox,
    #[test_offset = 0x8]
    pub location: Point,
    #[test_offset = 0x10]
    pub timer_circle: [*mut GL_Primitive; 10],
    #[test_offset = 0x60]
    pub timer_lines: *mut GL_Primitive,
    #[test_offset = 0x68]
    pub timer_stencil: *mut GL_Primitive,
    #[test_offset = 0x70]
    pub last_timer_stencil_count: c_int,
    #[test_offset = 0x78]
    pub broken_icon: *mut GL_Primitive,
    #[test_offset = 0x80]
    pub lock_icon: *mut GL_Primitive,
    #[test_offset = 0x88]
    pub hack_icon: *mut GL_Primitive,
    #[test_offset = 0x90]
    pub p_system: *mut ShipSystem,
    #[test_offset = 0x98]
    pub b_show_power: bool,
    #[test_offset = 0x9c]
    pub power_alpha: c_float,
    #[test_offset = 0xa0]
    pub mouse_hover: bool,
    #[test_offset = 0xa4]
    pub active_touch: c_int,
    #[test_offset = 0xa8]
    pub touch_initial_offset: Point,
    #[test_offset = 0xb0]
    pub tapped: bool,
    #[test_offset = 0xb1]
    pub dragging_power: bool,
    #[test_offset = 0xb4]
    pub drag_initial_power: c_int,
    #[test_offset = 0xb8]
    pub last_drag_speed: c_float,
    #[test_offset = 0xbc]
    pub last_drag_y: c_int,
    #[test_offset = 0xc0]
    pub last_drag_time: c_double,
    #[test_offset = 0xc8]
    pub warning: WarningMessage,
    #[test_offset = 0x1a8]
    pub top_power: c_int,
    #[test_offset = 0x1ac]
    pub hit_box: Rect,
    #[test_offset = 0x1bc]
    pub hit_box_top: c_int,
    #[test_offset = 0x1c0]
    pub hit_box_top_was_set: bool,
    #[test_offset = 0x1c8]
    pub wire_image: *mut GL_Texture,
    #[test_offset = 0x1d0]
    pub b_simple_power: bool,
    #[test_offset = 0x1d1]
    pub b_player_u_i: bool,
    #[test_offset = 0x1d2]
    pub use_large_tap_icon: bool,
    #[test_offset = 0x1d4]
    pub large_tap_icon_offset: Point,
    #[test_offset = 0x1e0]
    pub tap_button_heights: Vector<c_int>,
    #[test_offset = 0x1f8]
    pub tap_button_offset_y: c_int,
    #[test_offset = 0x1fc]
    pub cooldown_offset_y: c_int,
    #[test_offset = 0x200]
    pub key_pressed: c_float,
    #[test_offset = 0x208]
    pub touch_tooltip: *mut TouchTooltip,
    #[test_offset = 0x210]
    pub tap_box_frame: TapBoxFrame,
    #[test_offset = 0x260]
    pub locked_open: bool,
}

#[vtable]
pub struct VtableCooldownSystemBox {
    pub base: VtableSystemBox,
    pub get_cooldown_level: Option<fn(*mut CooldownSystemBox) -> c_int>,
    pub get_cooldown_fraction: Option<fn(*mut CooldownSystemBox) -> c_float>,
    pub get_cooldown_color: Option<fn(*mut CooldownSystemBox) -> GL_Color>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CooldownSystemBox {
    #[test_offset = 0x0]
    pub base: SystemBox,
    #[test_offset = 0x268]
    pub box_: [*mut GL_Primitive; 5],
    #[test_offset = 0x290]
    pub bar_: [*mut GL_Texture; 5],
    #[test_offset = 0x2b8]
    pub box_position: Point,
    #[test_offset = 0x2c0]
    pub round_down: bool,
    #[test_offset = 0x2c8]
    pub bar_primitive: *mut GL_Primitive,
    #[test_offset = 0x2d0]
    pub last_bar_height: c_int,
    #[test_offset = 0x2d4]
    pub last_bar_top: c_int,
}

impl CooldownSystemBox {
    pub fn vtable(&self) -> *const VtableCooldownSystemBox {
        self.base.vtable.cast()
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct HackBox {
    #[test_offset = 0x0]
    pub base: CooldownSystemBox,
    #[test_offset = 0x2d8]
    pub hack_sys: *mut HackingSystem,
    #[test_offset = 0x2e0]
    pub buttons: Vector<*mut Button>,
    #[test_offset = 0x2f8]
    pub current_button: *mut Button,
    #[test_offset = 0x300]
    pub button_offset: Point,
    #[test_offset = 0x308]
    pub box_: *mut GL_Texture,
    #[test_offset = 0x310]
    pub box2: *mut GL_Texture,
    #[test_offset = 0x318]
    pub hack_button: Button,
    #[test_offset = 0x3a8]
    pub overlay_button: Button,
    #[test_offset = 0x438]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x440]
    pub flash_tracker: AnimationTracker,
    #[test_offset = 0x460]
    pub super_shield_warning: *mut WarningMessage,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BatteryBox {
    #[test_offset = 0x0]
    pub base: CooldownSystemBox,
    #[test_offset = 0x2d8]
    pub battery_system: *mut BatterySystem,
    #[test_offset = 0x2e0]
    pub battery_button: Button,
    #[test_offset = 0x370]
    pub button_offset: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CloakingBox {
    #[test_offset = 0x0]
    pub base: CooldownSystemBox,
    #[test_offset = 0x2d8]
    pub buttons: Vector<*mut Button>,
    #[test_offset = 0x2f0]
    pub current_button: *mut Button,
    #[test_offset = 0x2f8]
    pub cloak_system: *mut CloakingSystem,
    #[test_offset = 0x300]
    pub button_offset: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemControl {
    #[test_offset = 0x0]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x8]
    pub combat_control: *mut CombatControl,
    #[test_offset = 0x10]
    pub sys_boxes: Vector<*mut SystemBox>,
    #[test_offset = 0x28]
    pub _system_power: Rect,
    #[test_offset = 0x38]
    pub b_system_power_hover: bool,
    #[test_offset = 0x3c]
    pub position: Point,
    #[test_offset = 0x44]
    pub system_power_position: Point,
    #[test_offset = 0x4c]
    pub sub_system_position: Point,
    #[test_offset = 0x58]
    pub wires_image: *mut GL_Primitive,
    #[test_offset = 0x60]
    pub wires_mask: *mut GL_Primitive,
    #[test_offset = 0x68]
    pub no_button: *mut GL_Primitive,
    #[test_offset = 0x70]
    pub button: *mut GL_Primitive,
    #[test_offset = 0x78]
    pub no_button_cap: *mut GL_Primitive,
    #[test_offset = 0x80]
    pub button_cap: *mut GL_Primitive,
    #[test_offset = 0x88]
    pub drone: *mut GL_Primitive,
    #[test_offset = 0x90]
    pub drone3: *mut GL_Primitive,
    #[test_offset = 0x98]
    pub drone2: *mut GL_Primitive,
    #[test_offset = 0xa0]
    pub sub_box: *mut GL_Primitive,
    #[test_offset = 0xa8]
    pub sub_spacing: c_int,
    #[test_offset = 0xb0]
    pub not_enough_power: *mut WarningMessage,
    #[test_offset = 0xb8]
    pub flash_battery_power: AnimationTracker,
    #[test_offset = 0xd8]
    pub flash_tracker: AnimationTracker,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewBox {
    #[test_offset = 0x0]
    pub box_: Rect,
    #[test_offset = 0x10]
    pub skill_box: Rect,
    #[test_offset = 0x20]
    pub p_crew: *const CrewMember,
    #[test_offset = 0x28]
    pub mouse_hover: bool,
    #[test_offset = 0x30]
    pub power_button: TextButton,
    #[test_offset = 0x130]
    pub number: c_int,
    #[test_offset = 0x134]
    pub b_selectable: bool,
    #[test_offset = 0x138]
    pub flash_health_tracker: AnimationTracker,
    #[test_offset = 0x158]
    pub box_background: *mut GL_Primitive,
    #[test_offset = 0x160]
    pub box_outline: *mut GL_Primitive,
    #[test_offset = 0x168]
    pub skill_box_background: *mut GL_Primitive,
    #[test_offset = 0x170]
    pub skill_box_outline: *mut GL_Primitive,
    #[test_offset = 0x178]
    pub cooldown_bar: *mut GL_Primitive,
    #[test_offset = 0x180]
    pub health_warning: CachedImage,
    #[test_offset = 0x1c8]
    pub last_cooldown_height: c_int,
    #[test_offset = 0x1d0]
    pub health_bar: *mut GL_Primitive,
    #[test_offset = 0x1d8]
    pub last_health_width: c_int,
    #[test_offset = 0x1e0]
    pub mind_controlled: Animation,
    #[test_offset = 0x2a0]
    pub stunned: Animation,
    #[test_offset = 0x360]
    pub hide_extra: bool,
    #[test_offset = 0x368]
    pub s_tooltip: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewControl {
    #[test_offset = 0x0]
    pub ship_manager: *mut ShipManager,
    #[test_offset = 0x8]
    pub selected_crew: Vector<*mut CrewMember>,
    #[test_offset = 0x20]
    pub potential_selected_crew: Vector<*mut CrewMember>,
    #[test_offset = 0x38]
    pub selected_door: *mut Door,
    #[test_offset = 0x40]
    pub selected_repair: *mut Repairable,
    #[test_offset = 0x48]
    pub selected_grid: Point,
    #[test_offset = 0x50]
    pub selected_room: c_int,
    #[test_offset = 0x54]
    pub selected_player_ship: bool,
    #[test_offset = 0x58]
    pub available_position: Point,
    #[test_offset = 0x60]
    pub crew_boxes: Vector<*mut CrewBox>,
    #[test_offset = 0x78]
    pub first_mouse: Point,
    #[test_offset = 0x80]
    pub current_mouse: Point,
    #[test_offset = 0x88]
    pub world_first_mouse: Point,
    #[test_offset = 0x90]
    pub world_current_mouse: Point,
    #[test_offset = 0x98]
    pub mouse_down: bool,
    #[test_offset = 0x99]
    pub b_updated: bool,
    #[test_offset = 0x9c]
    pub active_touch: c_int,
    #[test_offset = 0xa0]
    pub selecting_crew: bool,
    #[test_offset = 0xa1]
    pub selecting_crew_on_player_ship: bool,
    #[test_offset = 0xa8]
    pub selecting_crew_start_time: c_double,
    #[test_offset = 0xb0]
    pub door_control_mode: bool,
    #[test_offset = 0xb1]
    pub door_control_open: bool,
    #[test_offset = 0xb2]
    pub door_control_open_set: bool,
    #[test_offset = 0xb8]
    pub combat_control: *mut CombatControl,
    #[test_offset = 0xc0]
    pub selected_crew_box: c_uint,
    #[test_offset = 0xc8]
    pub crew_message: AnimationTracker,
    #[test_offset = 0xe8]
    pub message: StdString,
    #[test_offset = 0xf0]
    pub save_stations: Button,
    #[test_offset = 0x180]
    pub return_stations: Button,
    #[test_offset = 0x210]
    pub save_stations_base: *mut GL_Primitive,
    #[test_offset = 0x218]
    pub return_stations_base: *mut GL_Primitive,
    #[test_offset = 0x220]
    pub stations_last_y: c_int,
}

// DW_AT_byte_size = 0x4
// DW_AT_decl_file = /media/sf_FTL/Project/src/Utilities/DamageMessage.h
// DW_AT_decl_line = 0x12
#[repr(i32)]
pub enum MessageType {
    // DW_AT_const_value = 0x0
    Miss = 0,
    // DW_AT_const_value = 0x1
    Resist = 1,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DamageMessage {
    #[test_offset = 0x0]
    pub tracker: AnimationTracker,
    #[test_offset = 0x20]
    pub position: Pointf,
    #[test_offset = 0x28]
    pub color: GL_Color,
    #[test_offset = 0x38]
    pub b_float_down: bool,
    #[test_offset = 0x40]
    pub primitives: Vector<*mut GL_Primitive>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WarningWithLines {
    #[test_offset = 0x0]
    pub base: WarningMessage,
    #[test_offset = 0xe0]
    pub line_primitive: *mut GL_Primitive,
    #[test_offset = 0xe8]
    pub text_origin: Point,
    #[test_offset = 0xf0]
    pub top_text: TextString,
    #[test_offset = 0x100]
    pub bottom_text: TextString,
    #[test_offset = 0x110]
    pub top_text_limit: c_int,
    #[test_offset = 0x114]
    pub bottom_text_limit: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Ellipse {
    #[test_offset = 0x0]
    pub center: Point,
    #[test_offset = 0x8]
    pub a: c_float,
    #[test_offset = 0xc]
    pub b: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipObject {
    pub vtable: *const VtableShipObject,
    #[test_offset = 0x8]
    pub i_ship_id: c_int,
}

#[vtable]
pub struct VtableShipObject {
    pub dtor: Option<fn(*mut ShipObject)>,
    pub delete_dtor: Option<fn(*mut ShipObject)>,
}

#[vtable]
pub struct VtableTargetable {
    pub dtor: Option<fn(*mut Targetable)>,
    pub delete_dtor: Option<fn(*mut Targetable)>,
    pub get_world_center_point: Option<fn(*mut Targetable) -> Pointf>,
    pub get_random_targeting_point: Option<fn(*mut Targetable, bool) -> Pointf>,
    pub get_all_targeting_points: Option<fn(*mut Targetable) -> Vector<Pointf>>,
    pub get_shield_shape: Option<fn(*mut Targetable) -> Ellipse>,
    pub get_shield_power: Option<fn(*mut Targetable) -> ShieldPower>,
    pub get_space_id: Option<fn(*mut Targetable) -> c_int>,
    pub get_speed: Option<fn(*mut Targetable) -> Pointf>,
    pub get_owner_id: Option<fn(*mut Targetable) -> c_int>,
    pub get_self_id: Option<fn(*mut Targetable) -> c_int>,
    pub is_cloaked: Option<fn(*mut Targetable) -> bool>,
    pub damage_target: Option<fn(*mut Targetable, Pointf, Damage)>,
    pub get_is_dying: Option<fn(*mut Targetable) -> bool>,
    pub get_is_jumping: Option<fn(*mut Targetable) -> bool>,
    pub valid_target: Option<fn(*mut Targetable) -> bool>,
    pub get_shape: Option<fn(*mut Targetable) -> Rect>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Targetable {
    pub vtable: *const VtableTargetable,
    #[test_offset = 0x8]
    pub type_: c_int,
    #[test_offset = 0xc]
    pub hostile: bool,
    #[test_offset = 0xd]
    pub targeted: bool,
}

#[vtable]
pub struct VtableCollideable {
    pub dtor: Option<fn(*mut Collideable)>,
    pub delete_dtor: Option<fn(*mut Collideable)>,
    pub collision_moving:
        Option<fn(*mut Collideable, Pointf, Pointf, Damage, bool) -> CollisionResponse>,
    pub damage_beam: Option<fn(*mut Collideable, Pointf, Pointf, Damage) -> bool>,
    pub damage_area: Option<fn(*mut Collideable, Pointf, Damage, bool) -> bool>,
    pub damage_shield: Option<fn(*mut Collideable, Pointf, Damage, bool) -> bool>,
    pub get_dodged: Option<fn(*mut Collideable) -> bool>,
    pub get_super_shield: Option<fn(*mut Collideable) -> Pointf>,
    pub set_temp_vision: Option<fn(*mut Collideable, Pointf)>,
    pub get_space_id: Option<fn(*mut Collideable) -> c_int>,
    pub get_self_id: Option<fn(*mut Collideable) -> c_int>,
    pub get_owner_id: Option<fn(*mut Collideable) -> c_int>,
    pub valid_target_location: Option<fn(*mut Collideable, Pointf) -> bool>,
}

#[repr(C)]
#[derive(Debug)]
pub struct Collideable {
    pub vtable: *const VtableCollideable,
}

#[vtable]
pub struct VtableShipManager {
    pub base: VtableShipObject,
    pub get_super_shield: Option<fn(*mut ShipManager) -> Pointf>,
    pub get_shape: Option<fn(*mut ShipManager) -> Rect>,
    pub is_cloaked: Option<fn(*mut ShipManager) -> bool>,
    pub set_temp_vision: Option<fn(*mut ShipManager, Pointf)>,
    pub collision_moving:
        Option<fn(*mut ShipManager, Pointf, Pointf, Damage, bool) -> CollisionResponse>,
    pub damage_beam: Option<fn(*mut ShipManager, Pointf, Pointf, Damage) -> bool>,
    pub damage_area: Option<fn(*mut ShipManager, Pointf, Damage, bool) -> bool>,
    pub damage_shield: Option<fn(*mut ShipManager, Pointf, Damage, bool) -> bool>,
    pub damage_target: Option<fn(*mut ShipManager, Pointf, Damage)>,
    pub get_dodged: Option<fn(*mut ShipManager) -> bool>,
    pub get_random_targeting_point: Option<fn(*mut ShipManager, bool) -> Pointf>,
    pub get_all_targeting_points: Option<fn(*mut ShipManager) -> Vector<Pointf>>,
    pub get_shield_power: Option<fn(*mut ShipManager) -> ShieldPower>,
    pub get_shield_shape: Option<fn(*mut ShipManager) -> Ellipse>,
    pub get_is_jumping: Option<fn(*mut ShipManager) -> bool>,
    pub get_is_dying: Option<fn(*mut ShipManager) -> bool>,
    pub get_space_id: Option<fn(*mut ShipManager) -> c_int>,
    pub get_owner_id: Option<fn(*mut ShipManager) -> c_int>,
    pub get_self_id: Option<fn(*mut ShipManager) -> c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Particle {
    #[test_offset = 0x0]
    pub position_x: c_float,
    #[test_offset = 0x4]
    pub position_y: c_float,
    #[test_offset = 0x8]
    pub speed_x: c_float,
    #[test_offset = 0xc]
    pub speed_y: c_float,
    #[test_offset = 0x10]
    pub acceleration_x: c_float,
    #[test_offset = 0x14]
    pub acceleration_y: c_float,
    #[test_offset = 0x18]
    pub lifespan: c_float,
    #[test_offset = 0x1c]
    pub alive: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ParticleEmitter {
    #[test_offset = 0x0]
    pub particles: [Particle; 64],
    #[test_offset = 0x800]
    pub birth_rate: c_float,
    #[test_offset = 0x804]
    pub birth_counter: c_float,
    #[test_offset = 0x808]
    pub lifespan: c_float,
    #[test_offset = 0x80c]
    pub speed_mag: c_float,
    #[test_offset = 0x810]
    pub position_x: c_float,
    #[test_offset = 0x814]
    pub position_y: c_float,
    #[test_offset = 0x818]
    pub max_dx: c_float,
    #[test_offset = 0x81c]
    pub min_dx: c_float,
    #[test_offset = 0x820]
    pub max_dy: c_float,
    #[test_offset = 0x824]
    pub min_dy: c_float,
    #[test_offset = 0x828]
    pub image_x: c_int,
    #[test_offset = 0x82c]
    pub image_y: c_int,
    #[test_offset = 0x830]
    pub primitive: *mut GL_Primitive,
    #[test_offset = 0x838]
    pub emit_angle: c_float,
    #[test_offset = 0x83c]
    pub rand_angle: bool,
    #[test_offset = 0x83d]
    pub running: bool,
    #[test_offset = 0x840]
    pub max_alpha: c_float,
    #[test_offset = 0x844]
    pub min_size: c_float,
    #[test_offset = 0x848]
    pub max_size: c_float,
    #[test_offset = 0x84c]
    pub current_count: c_int,
}

// XXX: maps are really annoying to go through so not gonna bother recreating this
#[repr(C)]
#[derive(Debug)]
pub struct Map<K, V> {
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
    pub a4: u64,
    pub a5: u64,
    pub ph: PhantomData<(K, V)>,
}

#[vtable]
pub struct VtableDrone {
    pub dtor: Option<fn(*mut Drone)>,
    pub delete_dtor: Option<fn(*mut Drone)>,
    pub on_init: Option<fn(*mut Drone)>,
    pub on_loop: Option<fn(*mut Drone)>,
    pub on_destroy: Option<fn(*mut Drone)>,
    pub set_powered: Option<fn(*mut Drone, bool)>,
    pub set_instant_powered: Option<fn(*mut Drone)>,
    pub get_powered: Option<fn(*mut Drone) -> bool>,
    pub set_current_ship: Option<fn(*mut Drone, c_int)>,
    pub set_deployed: Option<fn(*mut Drone, bool)>,
    pub set_destroyed: Option<fn(*mut Drone, bool, bool)>,
    pub set_hacked: Option<fn(*mut Drone, c_int)>,
    pub get_deployed: Option<fn(*mut Drone) -> bool>, //
    pub needs_room: Option<fn(*mut Drone) -> bool>,
    pub set_slot: Option<fn(*mut Drone, c_int, c_int)>,
    pub destroyed: Option<fn(*mut Drone) -> bool>,
    pub get_world_location: Option<fn(*mut Drone) -> Point>,
    pub set_world_location: Option<fn(*mut Drone, Point)>,
    pub get_drone_slot: Option<fn(*mut Drone) -> Slot>,
    pub get_drone_health: Option<fn(*mut Drone) -> c_int>,
    pub get_required_power: Option<fn(*mut Drone) -> c_int>,
    pub render_icon: Option<fn(*mut Drone)>,
    pub get_name: Option<fn(*mut Drone) -> StdString>,
    pub can_be_deployed: Option<fn(*mut Drone) -> bool>, //
    pub recall_on_jump: Option<fn(*mut Drone) -> bool>,
    pub can_be_recovered: Option<fn(*mut Drone) -> bool>,
    pub save_state: Option<fn(*mut Drone, c_int)>,
    pub load_state: Option<fn(*mut Drone, c_int)>,
    pub blow_up: Option<fn(*mut Drone, bool)>,
    pub get_stunned: Option<fn(*mut Drone) -> bool>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Drone {
    pub vtable: *const VtableDrone,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[test_offset = 0xc]
    pub self_id: c_int,
    #[test_offset = 0x10]
    pub powered: bool,
    #[test_offset = 0x14]
    pub power_required: c_int,
    #[test_offset = 0x18]
    pub deployed: bool,
    #[test_offset = 0x1c]
    pub type_: c_int,
    #[test_offset = 0x20]
    pub blueprint: *const DroneBlueprint,
    #[test_offset = 0x28]
    pub b_dead: bool,
    #[test_offset = 0x2c]
    pub i_bonus_power: c_int,
    #[test_offset = 0x30]
    pub powered_at_location: bool,
    #[test_offset = 0x34]
    pub destroyed_timer: c_float,
    #[test_offset = 0x38]
    pub i_hack_level: c_int,
    #[test_offset = 0x3c]
    pub hack_time: c_float,
}

impl Drone {
    pub fn required_power(&self) -> c_int {
        self.power_required - self.i_bonus_power
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewDrone {
    #[test_offset = 0x0]
    // XXX: a bunch of extended (with drone funcs) vtable entries are missing, dont want to go through the pain
    pub base: CrewMember,
    // offset = 335093
    #[test_offset = 0x748]
    pub base1: Drone,
    #[test_offset = 0x788]
    pub drone_room: c_int,
    #[test_offset = 0x790]
    pub power_up: Animation,
    #[test_offset = 0x850]
    pub power_down: Animation,
    #[test_offset = 0x910]
    pub light_layer: *mut GL_Texture,
    #[test_offset = 0x918]
    pub base_layer: *mut GL_Texture,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CollisionResponse {
    #[test_offset = 0x0]
    pub collision_type: c_int,
    #[test_offset = 0x4]
    pub point: Pointf,
    #[test_offset = 0xc]
    pub damage: c_int,
    #[test_offset = 0x10]
    pub super_damage: c_int,
}

#[vtable]
pub struct VtableSpaceDrone {
    pub base: VtableDrone,
    pub pick_destination: Option<fn(*mut SpaceDrone)>,
    pub pick_target: Option<fn(*mut SpaceDrone)>,
    pub has_target: Option<fn(*mut SpaceDrone) -> bool>,
    pub valid_target: Option<fn(*mut SpaceDrone) -> bool>,
    pub get_weapon_cooldown: Option<fn(*mut SpaceDrone) -> c_float>,
    pub randomize_starting_position: Option<fn(*mut SpaceDrone)>,
    pub hide_under_owner: Option<fn(*mut SpaceDrone) -> bool>,
    pub get_next_projectile: Option<fn(*mut SpaceDrone) -> *mut Projectile>,
    pub set_movement_target: Option<fn(*mut SpaceDrone, *mut Targetable)>,
    pub set_weapon_target: Option<fn(*mut SpaceDrone, *mut Targetable)>,
    pub valid_target_object: Option<fn(*mut SpaceDrone, *mut Targetable) -> bool>,
    pub on_render: Option<fn(*mut SpaceDrone, c_int)>,
    pub render_drone: Option<fn(*mut SpaceDrone)>,
    pub get_tooltip: Option<fn(*mut SpaceDrone) -> StdString>,
    pub get_world_center_point: Option<fn(*mut SpaceDrone) -> Pointf>,
    pub set_current_location: Option<fn(*mut SpaceDrone, Pointf)>,
    pub mouse_move: Option<fn(*mut SpaceDrone, c_int, c_int)>,
    pub get_random_targeting_point: Option<fn(*mut SpaceDrone, bool) -> Pointf>,
    pub get_shield_shape: Option<fn(*mut SpaceDrone) -> Ellipse>,
    pub get_space_id: Option<fn(*mut SpaceDrone) -> c_int>,
    pub get_speed: Option<fn(*mut SpaceDrone) -> Pointf>,
    pub get_owner_id: Option<fn(*mut SpaceDrone) -> c_int>,
    pub get_self_id: Option<fn(*mut SpaceDrone) -> c_int>,
    pub collision_moving:
        Option<fn(*mut SpaceDrone, Pointf, Pointf, Damage, bool) -> CollisionResponse>,
    pub damage_beam: Option<fn(*mut SpaceDrone, Pointf, Pointf, Damage) -> bool>,
    pub damage_area: Option<fn(*mut SpaceDrone, Pointf, Damage, bool) -> bool>,
    pub get_boarding_drone: Option<fn(*mut SpaceDrone) -> *mut CrewDrone>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SpaceDrone {
    pub base: Drone,
    #[test_offset = 0x40]
    pub base1: Targetable,
    #[test_offset = 0x50]
    pub base2: Collideable,
    #[test_offset = 0x58]
    pub current_space: c_int,
    #[test_offset = 0x5c]
    pub destination_space: c_int,
    #[test_offset = 0x60]
    pub current_location: Pointf,
    #[test_offset = 0x68]
    pub last_location: Pointf,
    #[test_offset = 0x70]
    pub destination_location: Pointf,
    #[test_offset = 0x78]
    pub point_target: Pointf,
    #[test_offset = 0x80]
    pub explosion: Animation,
    #[test_offset = 0x140]
    pub weapon_target: *mut Targetable,
    #[test_offset = 0x148]
    pub target_location: Pointf,
    #[test_offset = 0x150]
    pub target_speed: Pointf,
    #[test_offset = 0x158]
    pub movement_target: *mut Targetable,
    #[test_offset = 0x160]
    pub speed_vector: Pointf,
    #[test_offset = 0x168]
    pub powered_last_frame: bool,
    #[test_offset = 0x169]
    pub deployed_last_frame: bool,
    #[test_offset = 0x16a]
    pub b_fire: bool,
    #[test_offset = 0x16c]
    pub pause: c_float,
    #[test_offset = 0x170]
    pub additional_pause: c_float,
    #[test_offset = 0x174]
    pub weapon_cooldown: c_float,
    #[test_offset = 0x178]
    pub current_angle: c_float,
    #[test_offset = 0x17c]
    pub aiming_angle: c_float,
    #[test_offset = 0x180]
    pub last_aiming_angle: c_float,
    #[test_offset = 0x184]
    pub desired_aiming_angle: c_float,
    #[test_offset = 0x188]
    pub message: *mut DamageMessage,
    #[test_offset = 0x190]
    pub weapon_animation: Animation,
    #[test_offset = 0x250]
    pub weapon_blueprint: *const WeaponBlueprint,
    #[test_offset = 0x258]
    pub lifespan: c_int,
    #[test_offset = 0x25c]
    pub b_loaded_position: bool,
    #[test_offset = 0x25d]
    pub b_disrupted: bool,
    #[test_offset = 0x260]
    pub hack_angle: c_float,
    #[test_offset = 0x264]
    pub ion_stun: c_float,
    #[test_offset = 0x268]
    pub beam_current_target: Pointf,
    #[test_offset = 0x270]
    pub beam_final_target: Pointf,
    #[test_offset = 0x278]
    pub beam_speed: c_float,
    #[test_offset = 0x280]
    pub hack_sparks: Animation,
}

#[vtable]
pub struct VtableProjectile {
    pub base: VtableCollideable,
    pub set_weapon_animation: Option<fn(*mut Projectile, *mut WeaponAnimation)>,
    pub on_render_specific: Option<fn(*mut Projectile, c_int)>,
    pub collision_check: Option<fn(*mut Projectile, *mut Collideable)>,
    pub on_update: Option<fn(*mut Projectile)>,
    pub get_world_center_point: Option<fn(*mut Projectile) -> Pointf>,
    pub get_random_targeting_point: Option<fn(*mut Projectile, bool) -> Pointf>,
    pub compute_heading: Option<fn(*mut Projectile)>,
    pub set_destination_space: Option<fn(*mut Projectile, c_int)>,
    pub enter_destination_space: Option<fn(*mut Projectile)>,
    pub dead: Option<fn(*mut Projectile) -> bool>,
    pub valid_target: Option<fn(*mut Projectile) -> bool>,
    pub kill: Option<fn(*mut Projectile)>,
    pub get_speed: Option<fn(*mut Projectile) -> Pointf>,
    pub set_damage: Option<fn(*mut Projectile, Damage)>,
    pub force_render_layer: Option<fn(*mut Projectile) -> c_int>,
    pub set_spin: Option<fn(*mut Projectile, c_float)>,
    pub save_projectile: Option<fn(*mut Projectile, c_int)>,
    pub load_projectile: Option<fn(*mut Projectile, c_int)>,
    pub get_type: Option<fn(*mut Projectile) -> c_int>,
    pub set_moving_target: Option<fn(*mut Projectile, *mut Targetable)>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Projectile {
    #[test_offset = 0x0]
    pub vtable: *const VtableProjectile,
    // offset = 274913
    #[test_offset = 0x8]
    pub base1: Targetable,
    #[test_offset = 0x18]
    pub position: Pointf,
    #[test_offset = 0x20]
    pub last_position: Pointf,
    #[test_offset = 0x28]
    pub speed_magnitude: c_float,
    #[test_offset = 0x2c]
    pub target: Pointf,
    #[test_offset = 0x34]
    pub heading: c_float,
    #[test_offset = 0x38]
    pub owner_id: c_int,
    #[test_offset = 0x3c]
    pub self_id: c_uint,
    #[test_offset = 0x40]
    pub damage: Damage,
    #[test_offset = 0x74]
    pub lifespan: c_float,
    #[test_offset = 0x78]
    pub destination_space: c_int,
    #[test_offset = 0x7c]
    pub current_space: c_int,
    #[test_offset = 0x80]
    pub target_id: c_int,
    #[test_offset = 0x84]
    pub dead: bool,
    #[test_offset = 0x88]
    pub death_animation: Animation,
    #[test_offset = 0x148]
    pub flight_animation: Animation,
    #[test_offset = 0x208]
    pub speed: Pointf,
    #[test_offset = 0x210]
    pub missed: bool,
    #[test_offset = 0x211]
    pub hit_target: bool,
    #[test_offset = 0x218]
    pub hit_solid_sound: StdString,
    #[test_offset = 0x220]
    pub hit_shield_sound: StdString,
    #[test_offset = 0x228]
    pub miss_sound: StdString,
    #[test_offset = 0x230]
    pub entry_angle: c_float,
    #[test_offset = 0x234]
    pub started_death: bool,
    #[test_offset = 0x235]
    pub passed_target: bool,
    #[test_offset = 0x236]
    pub b_broadcast_target: bool,
    #[test_offset = 0x238]
    pub flash_tracker: AnimationTracker,
    #[test_offset = 0x258]
    pub color: GL_Color,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AnimationDescriptor {
    #[test_offset = 0x0]
    pub num_frames: c_int,
    #[test_offset = 0x4]
    pub image_width: c_int,
    #[test_offset = 0x8]
    pub image_height: c_int,
    #[test_offset = 0xc]
    pub strip_start_y: c_int,
    #[test_offset = 0x10]
    pub strip_start_x: c_int,
    #[test_offset = 0x14]
    pub frame_width: c_int,
    #[test_offset = 0x18]
    pub frame_height: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Animation {
    #[test_offset = 0x0]
    pub animation_strip: *mut GL_Texture,
    #[test_offset = 0x8]
    pub info: AnimationDescriptor,
    #[test_offset = 0x28]
    pub tracker: AnimationTracker,
    #[test_offset = 0x48]
    pub position: Pointf,
    #[test_offset = 0x50]
    pub sound_forward: StdString,
    #[test_offset = 0x58]
    pub sound_reverse: StdString,
    #[test_offset = 0x60]
    pub randomize_frames: bool,
    #[test_offset = 0x64]
    pub f_scale: c_float,
    #[test_offset = 0x68]
    pub f_y_stretch: c_float,
    #[test_offset = 0x6c]
    pub current_frame: c_int,
    #[test_offset = 0x70]
    pub b_always_mirror: bool,
    #[test_offset = 0x78]
    pub sound_queue: Vector<Vector<StdString>>,
    #[test_offset = 0x90]
    pub fade_out: c_float,
    #[test_offset = 0x94]
    pub start_fade_out: c_float,
    #[test_offset = 0x98]
    pub anim_name: StdString,
    #[test_offset = 0xa0]
    pub mask_x_pos: c_int,
    #[test_offset = 0xa4]
    pub mask_x_size: c_int,
    #[test_offset = 0xa8]
    pub mask_y_pos: c_int,
    #[test_offset = 0xac]
    pub mask_y_size: c_int,
    #[test_offset = 0xb0]
    pub primitive: *mut GL_Primitive,
    #[test_offset = 0xb8]
    pub mirrored_primitive: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemTemplate {
    #[test_offset = 0x0]
    pub system_id: c_int,
    #[test_offset = 0x4]
    pub power_level: c_int,
    #[test_offset = 0x8]
    pub location: Vector<c_int>,
    #[test_offset = 0x20]
    pub bp: c_int,
    #[test_offset = 0x24]
    pub max_power: c_int,
    #[test_offset = 0x28]
    pub image: StdString,
    #[test_offset = 0x30]
    pub slot: c_int,
    #[test_offset = 0x34]
    pub direction: c_int,
    #[test_offset = 0x38]
    pub weapon: Vector<StdString>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[test_offset = 0x8]
    pub base_name: StdString,
    /// Inherited from Blueprint
    #[test_offset = 0x10]
    pub base_desc: Description,
    /// Inherited from Blueprint
    #[test_offset = 0x70]
    pub type_: c_int,
    #[test_offset = 0x78]
    pub desc: Description,
    #[test_offset = 0xd8]
    pub blueprint_name: StdString,
    #[test_offset = 0xe0]
    pub name: TextString,
    #[test_offset = 0xf0]
    pub ship_class: TextString,
    #[test_offset = 0x100]
    pub layout_file: StdString,
    #[test_offset = 0x108]
    pub img_file: StdString,
    #[test_offset = 0x110]
    pub cloak_file: StdString,
    #[test_offset = 0x118]
    pub shield_file: StdString,
    #[test_offset = 0x120]
    pub floor_file: StdString,
    #[test_offset = 0x128]
    pub system_info: Map<c_int, SystemTemplate>,
    #[test_offset = 0x158]
    pub systems: Vector<c_int>,
    #[test_offset = 0x170]
    pub drone_count: c_int,
    #[test_offset = 0x174]
    pub original_drone_count: c_int,
    #[test_offset = 0x178]
    pub drone_slots: c_int,
    #[test_offset = 0x180]
    pub load_drones: StdString,
    #[test_offset = 0x188]
    pub drones: Vector<StdString>,
    #[test_offset = 0x1a0]
    pub augments: Vector<StdString>,
    #[test_offset = 0x1b8]
    pub weapon_count: c_int,
    #[test_offset = 0x1bc]
    pub original_weapon_count: c_int,
    #[test_offset = 0x1c0]
    pub weapon_slots: c_int,
    #[test_offset = 0x1c8]
    pub load_weapons: StdString,
    #[test_offset = 0x1d0]
    pub weapons: Vector<StdString>,
    #[test_offset = 0x1e8]
    pub missiles: c_int,
    #[test_offset = 0x1ec]
    pub drone_count_1: c_int,
    #[test_offset = 0x1f0]
    pub health: c_int,
    #[test_offset = 0x1f4]
    pub original_crew_count: c_int,
    #[test_offset = 0x1f8]
    pub default_crew: Vector<StdString>,
    #[test_offset = 0x210]
    pub custom_crew: Vector<CrewBlueprint>,
    #[test_offset = 0x228]
    pub max_power: c_int,
    #[test_offset = 0x22c]
    pub boarding_a_i: c_int,
    #[test_offset = 0x230]
    pub bp_count: c_int,
    #[test_offset = 0x234]
    pub max_crew: c_int,
    #[test_offset = 0x238]
    pub max_sector: c_int,
    #[test_offset = 0x23c]
    pub min_sector: c_int,
    #[test_offset = 0x240]
    pub unlock: TextString,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum DoorStateEnum {
    Closed = 0,
    Open = 1,
    OpenForced = 2,
    Hit = 3,
    Animating = 4,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DoorState {
    #[test_offset = 0x0]
    pub state: DoorStateEnum,
    #[test_offset = 0x4]
    pub hacked: bool,
    #[test_offset = 0x8]
    pub level: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct LockdownShard {
    #[test_offset = 0x0]
    pub shard: Animation,
    #[test_offset = 0xc0]
    pub position: Pointf,
    #[test_offset = 0xc8]
    pub goal: Pointf,
    #[test_offset = 0xd0]
    pub speed: c_float,
    #[test_offset = 0xd4]
    pub b_arrived: bool,
    #[test_offset = 0xd5]
    pub b_done: bool,
    #[test_offset = 0xd8]
    pub life_time: c_float,
    #[test_offset = 0xdc]
    pub super_freeze: bool,
    #[test_offset = 0xe0]
    pub locking_room: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ExplosionAnimation {
    // XXX: this vtable includes a bunch of other stuff but who cares
    #[test_offset = 0x0]
    pub base: AnimationTracker,
    // offset = 371421
    #[test_offset = 0x20]
    pub base1: ShipObject,
    #[test_offset = 0x30]
    pub explosions: Vector<Animation>,
    #[test_offset = 0x48]
    pub pieces: Vector<*mut GL_Texture>,
    #[test_offset = 0x60]
    pub piece_names: Vector<StdString>,
    #[test_offset = 0x78]
    pub rotation_speed: Vector<c_float>,
    #[test_offset = 0x90]
    pub rotation: Vector<c_float>,
    #[test_offset = 0xa8]
    pub rotation_speed_min_max: Vector<Pair<c_float, c_float>>,
    #[test_offset = 0xc0]
    pub movement_vector: Vector<Pointf>,
    #[test_offset = 0xd8]
    pub position: Vector<Pointf>,
    #[test_offset = 0xf0]
    pub starting_position: Vector<Pointf>,
    #[test_offset = 0x108]
    pub explosion_timer: c_float,
    #[test_offset = 0x10c]
    pub sound_timer: c_float,
    #[test_offset = 0x110]
    pub b_final_boom: bool,
    #[test_offset = 0x111]
    pub b_jump_out: bool,
    #[test_offset = 0x118]
    pub weapon_anims: Vector<*mut WeaponAnimation>,
    #[test_offset = 0x130]
    pub pos: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ImageDesc {
    #[test_offset = 0x0]
    pub tex: *mut GL_Texture,
    #[test_offset = 0x8]
    pub res_id: c_int,
    #[test_offset = 0xc]
    pub w: c_int,
    #[test_offset = 0x10]
    pub h: c_int,
    #[test_offset = 0x14]
    pub x: c_int,
    #[test_offset = 0x18]
    pub y: c_int,
    #[test_offset = 0x1c]
    pub rot: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct OuterHull {
    #[test_offset = 0x0]
    pub base: Repairable,
    #[test_offset = 0x40]
    pub breach: Animation,
    #[test_offset = 0x100]
    pub heal: Animation,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Room {
    #[test_offset = 0x0]
    pub base: Selectable,
    /// Inherited from ShipObject
    pub base1_vtable: *const VtableShipObject,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[test_offset = 0x1c]
    pub rect: Rect,
    #[test_offset = 0x2c]
    pub i_room_id: c_int,
    #[test_offset = 0x30]
    pub b_blacked_out: bool,
    #[test_offset = 0x38]
    pub filled_slots: Vector<c_int>,
    #[test_offset = 0x50]
    pub slots: Vector<VectorBool>,
    #[test_offset = 0x68]
    pub b_warning_light: bool,
    #[test_offset = 0x70]
    pub light_tracker: AnimationTracker,
    #[test_offset = 0x90]
    pub i_fire_count: c_int,
    #[test_offset = 0x98]
    pub fires: Vector<Animation>,
    #[test_offset = 0xb0]
    pub primary_slot: c_int,
    #[test_offset = 0xb4]
    pub primary_direction: c_int,
    #[test_offset = 0xb8]
    pub last_o2: c_float,
    #[test_offset = 0xc0]
    pub floor_primitive: *mut GL_Primitive,
    #[test_offset = 0xc8]
    pub blackout_primitive: *mut GL_Primitive,
    #[test_offset = 0xd0]
    pub highlight_primitive: *mut GL_Primitive,
    #[test_offset = 0xd8]
    pub highlight_primitive2: *mut GL_Primitive,
    #[test_offset = 0xe0]
    pub o2_low_primitive: *mut GL_Primitive,
    #[test_offset = 0xe8]
    pub computer_primitive: *mut GL_Primitive,
    #[test_offset = 0xf0]
    pub computer_glow_primitive: *mut GL_Primitive,
    #[test_offset = 0xf8]
    pub computer_glow_yellow_primitive: *mut GL_Primitive,
    #[test_offset = 0x100]
    pub light_primitive: *mut GL_Primitive,
    #[test_offset = 0x108]
    pub light_glow_primitive: *mut GL_Primitive,
    #[test_offset = 0x110]
    pub stun_sparks: Animation,
    #[test_offset = 0x1d0]
    pub console_sparks: Animation,
    #[test_offset = 0x290]
    pub b_stunning: bool,
    #[test_offset = 0x294]
    pub f_hacked: c_float,
    #[test_offset = 0x298]
    pub current_spark_rotation: c_int,
    #[test_offset = 0x2a0]
    pub sparks: Vector<Animation>,
    #[test_offset = 0x2b8]
    pub spark_timer: c_float,
    #[test_offset = 0x2bc]
    pub spark_count: c_int,
    #[test_offset = 0x2c0]
    pub i_hack_level: c_int,
    #[test_offset = 0x2c8]
    pub room_tapped: Animation,
}

impl Room {
    pub fn slot_count(&self) -> c_int {
        // normally you're supposed to get self.slots[intruder].len() but i'd rather not implement
        // std::vector<bool> for two platforms
        (self.rect.h / 35) * (self.rect.w / 35)
    }
    pub fn empty(&self, intruder: bool) -> bool {
        *self.filled_slots.get(usize::from(intruder)).unwrap() == 0
    }
    pub fn available_slots(&self, intruder: bool) -> c_int {
        self.slot_count() - *self.filled_slots.get(usize::from(intruder)).unwrap()
    }
    pub fn full(&self, intruder: bool) -> bool {
        self.slot_count() == *self.filled_slots.get(usize::from(intruder)).unwrap()
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Ship {
    #[test_offset = 0x0]
    pub base: ShipObject,
    #[test_offset = 0x10]
    pub v_room_list: Vector<*mut Room>,
    #[test_offset = 0x28]
    pub v_door_list: Vector<*mut Door>,
    #[test_offset = 0x40]
    pub v_outer_walls: Vector<*mut OuterHull>,
    #[test_offset = 0x58]
    pub v_outer_airlocks: Vector<*mut Door>,
    #[test_offset = 0x70]
    pub hull_integrity: Pair<c_int, c_int>,
    #[test_offset = 0x78]
    pub weapon_mounts: Vector<WeaponMount>,
    #[test_offset = 0x90]
    pub floor_image_name: StdString,
    #[test_offset = 0x98]
    pub ship_floor: ImageDesc,
    #[test_offset = 0xb8]
    pub floor_primitive: *mut GL_Primitive,
    #[test_offset = 0xc0]
    pub ship_image_name: StdString,
    #[test_offset = 0xc8]
    pub ship_image: ImageDesc,
    #[test_offset = 0xe8]
    pub glow_offset: Point,
    #[test_offset = 0xf0]
    pub ship_image_primitive: *mut GL_Primitive,
    #[test_offset = 0xf8]
    pub cloak_image_name: StdString,
    #[test_offset = 0x100]
    pub ship_image_cloak: ImageDesc,
    #[test_offset = 0x120]
    pub cloak_primitive: *mut GL_Primitive,
    #[test_offset = 0x128]
    pub grid_primitive: *mut GL_Primitive,
    #[test_offset = 0x130]
    pub walls_primitive: *mut GL_Primitive,
    #[test_offset = 0x138]
    pub doors_primitive: *mut GL_Primitive,
    #[test_offset = 0x140]
    pub door_state: Vector<DoorState>,
    #[test_offset = 0x158]
    pub last_door_control_mode: bool,
    #[test_offset = 0x160]
    pub thrusters_image: *mut GL_Texture,
    #[test_offset = 0x168]
    pub jump_glare: *mut GL_Texture,
    #[test_offset = 0x170]
    pub vertical_shift: c_int,
    #[test_offset = 0x174]
    pub horizontal_shift: c_int,
    #[test_offset = 0x178]
    pub ship_name: StdString,
    #[test_offset = 0x180]
    pub explosion: ExplosionAnimation,
    #[test_offset = 0x2b8]
    pub b_destroyed: bool,
    #[test_offset = 0x2bc]
    pub base_ellipse: Ellipse,
    #[test_offset = 0x2d0]
    pub engine_anim: [Animation; 2],
    #[test_offset = 0x450]
    pub cloaking_tracker: AnimationTracker,
    #[test_offset = 0x470]
    pub b_cloaked: bool,
    #[test_offset = 0x471]
    pub b_experiment: bool,
    #[test_offset = 0x472]
    pub b_show_engines: bool,
    #[test_offset = 0x478]
    pub lockdowns: Vector<LockdownShard>,
}

impl Ship {
    pub unsafe fn get_room_blackout(&self, room_id: usize) -> bool {
        if let Some(room) = self.v_room_list.get(room_id) {
            !(**room).filled_slots.is_empty()
        } else {
            false
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Spreader<T> {
    /// Inherited from ShipObject
    pub base_vtable: *const VtableShipObject,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    // #[test_offset = 0xc]
    pub count: c_int,
    // #[test_offset = 0x10]
    pub room_count: Vector<c_int>,
    // #[test_offset = 0x28]
    pub grid: Vector<Vector<T>>,
}

#[vtable]
pub struct VtableSelectable {
    pub dtor: Option<fn(*mut Selectable)>,
    pub delete_dtor: Option<fn(*mut Selectable)>,
    pub set_selected: Option<fn(*mut Selectable, c_int)>,
    pub get_selected: Option<fn(*mut Selectable) -> c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Selectable {
    #[test_offset = 0x0]
    pub vtable: *const VtableSelectable,
    #[test_offset = 0x8]
    pub selected_state: c_int,
}

#[vtable]
pub struct VtableRepairable {
    pub base: VtableSelectable,
    // 4
    pub completely_destroyed: Option<fn(*mut Repairable) -> bool>,
    pub get_name: Option<fn(*mut Repairable) -> StdString>,
    pub set_name: Option<fn(*mut Repairable, StdString)>,
    pub repair: Option<fn(*mut Repairable)>,
    pub partial_repair: Option<fn(*mut Repairable, c_float, bool) -> bool>,
    pub partial_damage: Option<fn(*mut Repairable, c_float) -> bool>,
    pub needs_repairing: Option<fn(*mut Repairable) -> bool>,
    pub functioning: Option<fn(*mut Repairable) -> bool>,
    pub can_be_sabotaged: Option<fn(*mut Repairable) -> bool>,
    pub get_damage: Option<fn(*mut Repairable) -> c_float>,
    pub get_location: Option<fn(*mut Repairable) -> Point>,
    pub get_grid_location: Option<fn(*mut Repairable) -> Point>,
    pub set_damage: Option<fn(*mut Repairable, c_float)>,
    pub set_max_damage: Option<fn(*mut Repairable, c_float)>,
    pub set_location: Option<fn(*mut Repairable, Point)>,
    pub on_render_highlight: Option<fn(*mut Repairable)>,
    pub get_id: Option<fn(*mut Repairable) -> c_int>,
    pub is_room_based: Option<fn(*mut Repairable) -> bool>,
    pub get_room_id: Option<fn(*mut Repairable) -> c_int>,
    pub ioned: Option<fn(*mut Repairable, c_int) -> bool>,
    pub set_room_id: Option<fn(*mut Repairable)>,
}

#[vtable]
pub struct VtableSpreadable {
    pub base: VtableRepairable,
    pub present: Option<fn(*mut Spreadable) -> bool>,
    pub update_death_timer: Option<fn(*mut Spreadable, c_int)>,
    pub update_start_timer: Option<fn(*mut Spreadable, c_int)>,
    pub reset_start_timer: Option<fn(*mut Spreadable)>,
    pub spread: Option<fn(*mut Spreadable)>,
    pub on_loop: Option<fn(*mut Spreadable)>,
    pub on_init: Option<fn(*mut Spreadable)>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Spreadable {
    #[test_offset = 0x0]
    pub vtable: *const VtableSpreadable,
    #[test_offset = 0x8]
    /// Inherited from Repairable
    pub selected_state: c_int,
    /// Inherited from ShipObject
    pub base1_vtable: *const VtableShipObject,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[test_offset = 0x1c]
    /// Inherited from Repairable
    pub f_damage: c_float,
    #[test_offset = 0x20]
    /// Inherited from Repairable
    pub p_loc: Point,
    #[test_offset = 0x28]
    /// Inherited from Repairable
    pub f_max_damage: c_float,
    #[test_offset = 0x30]
    /// Inherited from Repairable
    pub name: StdString,
    #[test_offset = 0x38]
    /// Inherited from Repairable
    pub room_id: c_int,
    #[test_offset = 0x3c]
    /// Inherited from Repairable
    pub i_repair_count: c_int,
    #[test_offset = 0x40]
    pub sound_name: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Repairable {
    #[test_offset = 0x0]
    pub vtable: *const VtableRepairable,
    #[test_offset = 0x8]
    /// Inherited from Selectable
    pub selected_state: c_int,
    /// Inherited from ShipObject
    #[test_offset = 0x10]
    pub base1_vtable: *const VtableShipObject,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[test_offset = 0x1c]
    pub f_damage: c_float,
    #[test_offset = 0x20]
    pub p_loc: Point,
    #[test_offset = 0x28]
    pub f_max_damage: c_float,
    #[test_offset = 0x30]
    pub name: StdString,
    #[test_offset = 0x38]
    pub room_id: c_int,
    #[test_offset = 0x3c]
    pub i_repair_count: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Fire {
    #[test_offset = 0x0]
    pub base: Spreadable,
    #[test_offset = 0x48]
    pub f_death_timer: c_float,
    #[test_offset = 0x4c]
    pub f_start_timer: c_float,
    #[test_offset = 0x50]
    pub f_oxygen: c_float,
    #[test_offset = 0x58]
    pub fire_animation: Animation,
    #[test_offset = 0x118]
    pub smoke_animation: Animation,
    #[test_offset = 0x1d8]
    pub b_was_on_fire: bool,
}

#[vtable]
pub struct VtableCrewTarget {
    pub base: VtableShipObject,
    pub get_position: Option<fn(*mut CrewTarget) -> Point>,
    pub position_shift: Option<fn(*mut CrewTarget) -> c_float>,
    pub inside_room: Option<fn(*mut CrewTarget, c_int) -> bool>,
    pub apply_damage: Option<fn(*mut CrewTarget, c_float) -> bool>,
    pub get_priority: Option<fn(*mut CrewTarget) -> c_int>,
    pub valid_target: Option<fn(*mut CrewTarget, c_int) -> bool>,
    pub multi_shots: Option<fn(*mut CrewTarget) -> bool>,
    pub exact_target: Option<fn(*mut CrewTarget) -> bool>,
    pub is_crew: Option<fn(*mut CrewTarget) -> bool>,
    pub is_cloned: Option<fn(*mut CrewTarget) -> bool>,
    // 12
    pub is_drone: Option<fn(*mut CrewTarget) -> bool>,
}

#[vtable]
pub struct VtableCrewMember {
    pub base: VtableCrewTarget,
    // 13
    pub jump: Option<fn(*mut CrewMember)>,
    pub get_intruder: Option<fn(*mut CrewMember) -> bool>,
    pub save_state: Option<fn(*mut CrewMember, c_int)>,
    pub load_state: Option<fn(*mut CrewMember, c_int)>,
    pub on_loop: Option<fn(*mut CrewMember)>,
    pub on_render: Option<fn(*mut CrewMember, bool)>,
    pub out_of_game: Option<fn(*mut CrewMember) -> bool>,
    // 20
    pub set_out_of_game: Option<fn(*mut CrewMember)>,
    pub functional: Option<fn(*mut CrewMember) -> bool>,
    pub count_for_victory: Option<fn(*mut CrewMember) -> bool>,
    pub get_controllable: Option<fn(*mut CrewMember) -> bool>,
    pub ready_to_fight: Option<fn(*mut CrewMember) -> bool>,
    pub can_fight: Option<fn(*mut CrewMember) -> bool>,
    pub can_repair: Option<fn(*mut CrewMember) -> bool>,
    pub can_sabotage: Option<fn(*mut CrewMember) -> bool>,
    pub can_man: Option<fn(*mut CrewMember) -> bool>,
    pub can_teleport: Option<fn(*mut CrewMember) -> bool>,
    // 30
    pub can_heal: Option<fn(*mut CrewMember) -> bool>,
    pub can_suffocate: Option<fn(*mut CrewMember) -> bool>,
    pub can_burn: Option<fn(*mut CrewMember) -> bool>,
    pub get_max_health: Option<fn(*mut CrewMember) -> c_int>,
    pub is_dead: Option<fn(*mut CrewMember) -> bool>,
    pub permanent_death: Option<fn(*mut CrewMember) -> bool>,
    pub ship_damage: Option<fn(*mut CrewMember, c_float) -> bool>,
    pub fire_fighting_sound_effect: Option<fn(*mut CrewMember) -> bool>,
    pub get_unique_repairing: Option<fn(*mut CrewMember) -> StdString>,
    pub provides_vision: Option<fn(*mut CrewMember) -> bool>,
    // 40
    pub get_move_speed_multipler: Option<fn(*mut CrewMember) -> c_float>,
    pub get_repair_speed: Option<fn(*mut CrewMember) -> c_float>,
    pub get_damage_multiplier: Option<fn(*mut CrewMember) -> c_float>,
    pub provides_power: Option<fn(*mut CrewMember) -> bool>,
    pub get_species: Option<fn(*mut CrewMember) -> StdString>,
    pub get_fire_repair_multiplier: Option<fn(*mut CrewMember) -> c_float>,
    pub is_telepathic: Option<fn(*mut CrewMember) -> bool>,
    pub get_power_cooldown: Option<fn(*mut CrewMember) -> Pair<c_float, c_float>>,
    pub power_ready: Option<fn(*mut CrewMember) -> bool>,
    pub activate_power: Option<fn(*mut CrewMember)>,
    // 50
    pub has_special_power: Option<fn(*mut CrewMember) -> bool>,
    pub reset_power: Option<fn(*mut CrewMember)>,
    pub get_suffocation_modifier: Option<fn(*mut CrewMember) -> c_float>,
    pub block_room: Option<fn(*mut CrewMember) -> c_int>,
    pub get_room_damage: Option<fn(*mut CrewMember) -> Damage>,
    pub is_anaerobic: Option<fn(*mut CrewMember) -> bool>,
    pub update_repair: Option<fn(*mut CrewMember)>,
    pub can_stim: Option<fn(*mut CrewMember) -> bool>,
}
#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewTarget {
    #[test_offset = 0x0]
    pub vtable: *const VtableCrewMember,
    pub i_ship_id: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Slot {
    #[test_offset = 0x0]
    pub room_id: c_int,
    #[test_offset = 0x4]
    pub slot_id: c_int,
    #[test_offset = 0x8]
    pub world_location: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SCrewStats {
    #[test_offset = 0x0]
    pub stat: Vector<c_int>,
    #[test_offset = 0x18]
    pub species: StdString,
    #[test_offset = 0x20]
    pub name: StdString,
    #[test_offset = 0x28]
    pub male: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Door {
    #[test_offset = 0x0]
    pub base: CrewTarget,
    /// Inherited from Selectable
    #[test_offset = 0x10]
    pub base1_vtable: *const VtableSelectable,
    /// Inherited from Selectable
    pub base1_selected_state: c_int,
    #[test_offset = 0x1c]
    pub i_room1: c_int,
    #[test_offset = 0x20]
    pub i_room2: c_int,
    #[test_offset = 0x24]
    pub b_open: bool,
    #[test_offset = 0x28]
    pub i_blast: c_int,
    #[test_offset = 0x2c]
    pub b_fake_open: bool,
    #[test_offset = 0x30]
    pub width: c_int,
    #[test_offset = 0x34]
    pub height: c_int,
    #[test_offset = 0x38]
    pub outline_primitive: *mut GL_Primitive,
    #[test_offset = 0x40]
    pub highlight_primitive: *mut GL_Primitive,
    #[test_offset = 0x48]
    pub door_anim: Animation,
    #[test_offset = 0x108]
    pub door_anim_large: Animation,
    #[test_offset = 0x1c8]
    pub i_door_id: c_int,
    #[test_offset = 0x1cc]
    pub base_health: c_int,
    #[test_offset = 0x1d0]
    pub health: c_int,
    #[test_offset = 0x1d8]
    pub forced_open: AnimationTracker,
    #[test_offset = 0x1f8]
    pub got_hit: AnimationTracker,
    #[test_offset = 0x218]
    pub door_level: c_int,
    #[test_offset = 0x21c]
    pub b_ioned: bool,
    #[test_offset = 0x220]
    pub fake_open_timer: c_float,
    #[test_offset = 0x228]
    pub locked_down: AnimationTracker,
    #[test_offset = 0x248]
    pub lastbase: c_float,
    #[test_offset = 0x24c]
    pub i_hacked: c_int,
    #[test_offset = 0x250]
    pub x: c_int,
    #[test_offset = 0x254]
    pub y: c_int,
    #[test_offset = 0x258]
    pub b_vertical: bool,
}

impl Door {
    pub fn close(&mut self) {
        unsafe { (super::DOOR_CLOSE.unwrap())(ptr::addr_of_mut!(*self)) }
    }
    pub fn open(&mut self) {
        unsafe { (super::DOOR_OPEN.unwrap())(ptr::addr_of_mut!(*self)) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewTask {
    #[test_offset = 0x0]
    pub task_id: c_int,
    #[test_offset = 0x4]
    pub room: c_int,
    #[test_offset = 0x8]
    #[allow(non_snake_case)]
    pub _sil_do_not_use_system: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BoardingGoal {
    #[test_offset = 0x0]
    pub f_health_limit: c_float,
    #[test_offset = 0x4]
    pub caused_damage: c_int,
    #[test_offset = 0x8]
    pub targets_destroyed: c_int,
    #[test_offset = 0xc]
    pub target: c_int,
    #[test_offset = 0x10]
    pub damage_type: c_int,
}

#[vtable]
pub struct VtableCrewAnimation {
    pub base: VtableShipObject,
    pub on_render: Option<fn(*mut CrewAnimation, c_float, c_int, bool)>,
    pub on_render_props: Option<fn(*mut CrewAnimation)>,
    pub on_update_effects: Option<fn(*mut CrewAnimation)>,
    pub update_firing: Option<fn(*mut CrewAnimation)>,
    pub update_shooting: Option<fn(*mut CrewAnimation)>,
    pub fire_shot: Option<fn(*mut CrewAnimation) -> bool>,
    pub get_firing_frame: Option<fn(*mut CrewAnimation) -> c_int>,
    pub get_shooting_sound: Option<fn(*mut CrewAnimation) -> StdString>,
    pub get_death_sound: Option<fn(*mut CrewAnimation) -> StdString>,
    pub restart: Option<fn(*mut CrewAnimation)>,
    pub custom_death: Option<fn(*mut CrewAnimation) -> bool>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewLaser {
    #[test_offset = 0x0]
    pub base: Projectile,
    #[test_offset = 0x268]
    pub r: c_int,
    #[test_offset = 0x26c]
    pub g: c_int,
    #[test_offset = 0x270]
    pub b: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewAnimation {
    #[test_offset = 0x0]
    pub vtable: *const VtableCrewAnimation,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[test_offset = 0x10]
    pub anims: Vector<Vector<Animation>>,
    #[test_offset = 0x28]
    pub base_strip: *mut GL_Texture,
    #[test_offset = 0x30]
    pub color_strip: *mut GL_Texture,
    #[test_offset = 0x38]
    pub layer_strips: Vector<*mut GL_Texture>,
    #[test_offset = 0x50]
    pub last_position: Pointf,
    #[test_offset = 0x58]
    pub direction: c_int,
    #[test_offset = 0x5c]
    pub sub_direction: c_int,
    #[test_offset = 0x60]
    pub status: c_int,
    #[test_offset = 0x64]
    pub move_direction: c_int,
    #[test_offset = 0x68]
    pub smoke_emitter: ParticleEmitter,
    #[test_offset = 0x8b8]
    pub b_shared_spot: bool,
    #[test_offset = 0x8c0]
    pub shots: Vector<CrewLaser>,
    #[test_offset = 0x8d8]
    pub shoot_timer: TimerHelper,
    #[test_offset = 0x8ec]
    pub punch_timer: TimerHelper,
    #[test_offset = 0x900]
    pub target: Pointf,
    #[test_offset = 0x908]
    pub f_damage_done: c_float,
    #[test_offset = 0x90c]
    pub b_player: bool,
    #[test_offset = 0x90d]
    pub b_frozen: bool,
    #[test_offset = 0x90e]
    pub b_drone: bool,
    #[test_offset = 0x90f]
    pub b_ghost: bool,
    #[test_offset = 0x910]
    pub b_exact_shooting: bool,
    #[test_offset = 0x918]
    pub projectile: Animation,
    #[test_offset = 0x9d8]
    pub b_typing: bool,
    #[test_offset = 0x9e0]
    pub race: StdString,
    #[test_offset = 0x9e8]
    pub current_ship: c_int,
    #[test_offset = 0x9ec]
    pub b_male: bool,
    #[test_offset = 0x9ed]
    pub colorblind: bool,
    #[test_offset = 0x9f0]
    pub layer_colors: Vector<GL_Color>,
    #[test_offset = 0xa08]
    pub forced_animation: c_int,
    #[test_offset = 0xa0c]
    pub forced_direction: c_int,
    #[test_offset = 0xa10]
    pub projectile_color: GL_Color,
    #[test_offset = 0xa20]
    pub b_stunned: bool,
    #[test_offset = 0xa21]
    pub b_door_target: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Path {
    #[test_offset = 0x0]
    pub start: Point,
    #[test_offset = 0x8]
    pub doors: Vector<*mut Door>,
    #[test_offset = 0x20]
    pub finish: Point,
    #[test_offset = 0x28]
    pub distance: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewMember {
    pub vtable: *const VtableCrewMember,
    /// Inherited from CrewTarget
    pub i_ship_id: c_int,
    #[test_offset = 0xc]
    pub x: c_float,
    #[test_offset = 0x10]
    pub y: c_float,
    #[test_offset = 0x14]
    pub size: c_float,
    #[test_offset = 0x18]
    pub scale: c_float,
    #[test_offset = 0x1c]
    pub goal_x: c_float,
    #[test_offset = 0x20]
    pub goal_y: c_float,
    #[test_offset = 0x24]
    pub width: c_int,
    #[test_offset = 0x28]
    pub height: c_int,
    #[test_offset = 0x2c]
    pub health: Pair<c_float, c_float>,
    #[test_offset = 0x34]
    pub speed_x: c_float,
    #[test_offset = 0x38]
    pub speed_y: c_float,
    #[test_offset = 0x40]
    pub path: Path,
    #[test_offset = 0x70]
    pub new_path: bool,
    #[test_offset = 0x74]
    pub x_destination: c_float,
    #[test_offset = 0x78]
    pub y_destination: c_float,
    #[test_offset = 0x80]
    pub last_door: *mut Door,
    #[test_offset = 0x88]
    pub current_repair: *mut Repairable,
    #[test_offset = 0x90]
    pub b_suffocating: bool,
    #[test_offset = 0x94]
    pub move_goal: c_int,
    #[test_offset = 0x98]
    pub selection_state: c_int,
    #[test_offset = 0x9c]
    pub i_room_id: c_int,
    #[test_offset = 0xa0]
    pub i_manning_id: c_int,
    #[test_offset = 0xa4]
    pub i_repair_id: c_int,
    #[test_offset = 0xa8]
    pub i_stack_id: c_int,
    #[test_offset = 0xac]
    pub current_slot: Slot,
    #[test_offset = 0xbc]
    pub intruder: bool,
    #[test_offset = 0xbd]
    pub b_fighting: bool,
    #[test_offset = 0xbe]
    pub b_shared_spot: bool,
    #[test_offset = 0xc0]
    pub crew_anim: *mut CrewAnimation,
    #[test_offset = 0xc8]
    pub selection_image: *mut GL_Texture,
    #[test_offset = 0xd0]
    pub health_box: CachedImage,
    #[test_offset = 0x118]
    pub health_box_red: CachedImage,
    #[test_offset = 0x160]
    pub health_bar: CachedRect,
    #[test_offset = 0x180]
    pub f_medbay: c_float,
    #[test_offset = 0x184]
    pub last_damage_timer: c_float,
    #[test_offset = 0x188]
    pub last_health_change: c_float,
    #[test_offset = 0x18c]
    pub current_ship_id: c_int,
    #[test_offset = 0x190]
    pub flash_health_tracker: AnimationTracker,
    #[test_offset = 0x1b0]
    pub current_target: Pointf,
    #[test_offset = 0x1b8]
    pub crew_target: *mut CrewTarget,
    #[test_offset = 0x1c0]
    pub boarding_goal: BoardingGoal,
    #[test_offset = 0x1d4]
    pub b_frozen: bool,
    #[test_offset = 0x1d5]
    pub b_frozen_location: bool,
    #[test_offset = 0x1d8]
    pub task: CrewTask,
    #[test_offset = 0x1e8]
    pub type_: StdString,
    #[test_offset = 0x1f0]
    pub ship: *mut Ship,
    #[test_offset = 0x1f8]
    pub final_goal: Slot,
    #[test_offset = 0x208]
    pub blocking_door: *mut Door,
    #[test_offset = 0x210]
    pub b_out_of_game: bool,
    #[test_offset = 0x218]
    pub species: StdString,
    #[test_offset = 0x220]
    pub b_dead: bool,
    #[test_offset = 0x224]
    pub i_on_fire: c_int,
    #[test_offset = 0x228]
    pub b_active_manning: bool,
    #[test_offset = 0x230]
    pub current_system: *mut ShipSystem,
    #[test_offset = 0x238]
    pub using_skill: c_int,
    #[test_offset = 0x240]
    pub blueprint: CrewBlueprint,
    #[test_offset = 0x340]
    pub healing: Animation,
    #[test_offset = 0x400]
    pub stunned: Animation,
    #[test_offset = 0x4c0]
    pub level_up: AnimationTracker,
    #[test_offset = 0x4e0]
    pub last_level_up: c_int,
    #[test_offset = 0x4e8]
    pub stats: SCrewStats,
    #[test_offset = 0x518]
    pub skills_earned: Vector<VectorBool>,
    #[test_offset = 0x530]
    pub clone_ready: bool,
    #[test_offset = 0x531]
    pub b_mind_controlled: bool,
    #[test_offset = 0x534]
    pub i_death_number: c_int,
    #[test_offset = 0x538]
    pub mind_controlled: Animation,
    #[test_offset = 0x5f8]
    pub stun_icon: Animation,
    #[test_offset = 0x6b8]
    pub skill_up: Vector<Vector<AnimationTracker>>,
    #[test_offset = 0x6d0]
    pub health_boost: c_int,
    #[test_offset = 0x6d4]
    pub f_mind_damage_boost: c_float,
    #[test_offset = 0x6d8]
    pub f_clone_dying: c_float,
    #[test_offset = 0x6dc]
    pub b_resisted: bool,
    #[test_offset = 0x6e0]
    pub saved_position: Slot,
    #[test_offset = 0x6f0]
    pub f_stun_time: c_float,
    #[test_offset = 0x6f8]
    pub movement_target: CachedImage,
    #[test_offset = 0x740]
    pub b_cloned: bool,
}

impl CrewMember {
    pub fn intruder(&self) -> bool {
        if self.b_mind_controlled {
            self.current_ship_id == self.i_ship_id
        } else {
            self.current_ship_id != self.i_ship_id
        }
    }
    pub fn move_to_room(&mut self, room_id: c_int, slot_id: c_int, force: bool) -> bool {
        unsafe { (super::MOVE_CREW.unwrap())(ptr::addr_of_mut!(*self), room_id, slot_id, force) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ArtillerySystem {
    pub base: ShipSystem,
    #[test_offset = 0x248]
    pub projectile_factory: *mut ProjectileFactory,
    #[test_offset = 0x250]
    pub target: *mut Targetable,
    #[test_offset = 0x258]
    pub b_cloaked: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct MedbaySystem {
    #[test_offset = 0x0]
    pub base: ShipSystem,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct EngineSystem {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub b_boost_ftl: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneSystem {
    #[test_offset = 0x0]
    pub base: ShipSystem,
    #[test_offset = 0x248]
    pub drones: Vector<*mut Drone>,
    #[test_offset = 0x260]
    pub drone_count: c_int,
    #[test_offset = 0x264]
    pub drone_start: c_int,
    #[test_offset = 0x268]
    pub target_ship: *mut Targetable,
    #[test_offset = 0x270]
    pub user_powered: VectorBool,
    #[test_offset = 0x298]
    pub slot_count: c_int,
    #[test_offset = 0x29c]
    pub i_starting_battery_power: c_int,
    #[test_offset = 0x2a0]
    pub repower_list: VectorBool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponMount {
    #[test_offset = 0x0]
    pub position: Point,
    #[test_offset = 0x8]
    pub mirror: bool,
    #[test_offset = 0x9]
    pub rotate: bool,
    #[test_offset = 0xc]
    pub slide: c_int,
    #[test_offset = 0x10]
    pub gib: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponAnimation {
    #[test_offset = 0x0]
    pub anim: Animation,
    #[test_offset = 0xc0]
    pub b_fire_shot: bool,
    #[test_offset = 0xc1]
    pub b_firing: bool,
    #[test_offset = 0xc4]
    pub f_charge_level: c_float,
    #[test_offset = 0xc8]
    pub i_charged_frame: c_int,
    #[test_offset = 0xcc]
    pub i_fire_frame: c_int,
    #[test_offset = 0xd0]
    pub b_mirrored: bool,
    #[test_offset = 0xd1]
    pub b_rotation: bool,
    #[test_offset = 0xd4]
    pub fire_location: Point,
    #[test_offset = 0xdc]
    pub b_powered: bool,
    #[test_offset = 0xe0]
    pub mount_point: Point,
    #[test_offset = 0xe8]
    pub render_point: Point,
    #[test_offset = 0xf0]
    pub fire_mount_vector: Point,
    #[test_offset = 0xf8]
    pub slide_tracker: AnimationTracker,
    #[test_offset = 0x118]
    pub slide_direction: c_int,
    #[test_offset = 0x120]
    pub i_charge_image: CachedImage,
    #[test_offset = 0x168]
    pub explosion_anim: Animation,
    #[test_offset = 0x228]
    pub mount: WeaponMount,
    #[test_offset = 0x23c]
    pub f_delay_charge_time: c_float,
    #[test_offset = 0x240]
    pub boost_anim: Animation,
    #[test_offset = 0x300]
    pub boost_level: c_int,
    #[test_offset = 0x304]
    pub b_show_charge: bool,
    #[test_offset = 0x308]
    pub f_actual_charge_level: c_float,
    #[test_offset = 0x30c]
    pub i_charge_offset: c_int,
    #[test_offset = 0x310]
    pub i_charge_levels: c_int,
    #[test_offset = 0x314]
    pub current_offset: c_int,
    #[test_offset = 0x318]
    pub charge_box: CachedImage,
    #[test_offset = 0x360]
    pub charge_bar: CachedImage,
    #[test_offset = 0x3a8]
    pub i_hack_level: c_int,
    #[test_offset = 0x3b0]
    pub hack_sparks: Animation,
    #[test_offset = 0x470]
    pub player_ship: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ProjectileFactory {
    /// Inherited from ShipObject
    pub base_vtable: *const VtableShipObject,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[test_offset = 0xc]
    pub cooldown: Pair<c_float, c_float>,
    #[test_offset = 0x14]
    pub sub_cooldown: Pair<c_float, c_float>,
    #[test_offset = 0x1c]
    pub base_cooldown: c_float,
    #[test_offset = 0x20]
    pub blueprint: *const WeaponBlueprint,
    #[test_offset = 0x28]
    pub local_position: Point,
    #[test_offset = 0x30]
    pub flight_animation: Animation,
    #[test_offset = 0xf0]
    pub auto_firing: bool,
    #[test_offset = 0xf1]
    pub fire_when_ready: bool,
    #[test_offset = 0xf2]
    pub powered: bool,
    #[test_offset = 0xf4]
    pub required_power: c_int,
    #[test_offset = 0xf8]
    pub targets: Vector<Pointf>,
    #[test_offset = 0x110]
    pub last_targets: Vector<Pointf>,
    #[test_offset = 0x128]
    pub target_id: c_int,
    #[test_offset = 0x12c]
    pub i_ammo: c_int,
    #[test_offset = 0x130]
    pub name: StdString,
    #[test_offset = 0x138]
    pub num_shots: c_int,
    #[test_offset = 0x13c]
    pub current_firing_angle: c_float,
    #[test_offset = 0x140]
    pub current_entry_angle: c_float,
    #[test_offset = 0x148]
    pub current_ship_target: *mut Targetable,
    #[test_offset = 0x150]
    pub cloaking_system: *mut CloakingSystem,
    #[test_offset = 0x158]
    pub weapon_visual: WeaponAnimation,
    #[test_offset = 0x5d0]
    pub mount: WeaponMount,
    #[test_offset = 0x5e8]
    pub queued_projectiles: Vector<*mut Projectile>,
    #[test_offset = 0x600]
    pub i_bonus_power: c_int,
    #[test_offset = 0x604]
    pub b_fired_once: bool,
    #[test_offset = 0x608]
    pub i_spend_missile: c_int,
    #[test_offset = 0x60c]
    pub cooldown_modifier: c_float,
    #[test_offset = 0x610]
    pub shots_fired_at_target: c_int,
    #[test_offset = 0x614]
    pub radius: c_int,
    #[test_offset = 0x618]
    pub boost_level: c_int,
    #[test_offset = 0x61c]
    pub last_projectile_id: c_int,
    #[test_offset = 0x620]
    pub charge_level: c_int,
    #[test_offset = 0x624]
    pub i_hack_level: c_int,
    #[test_offset = 0x628]
    pub goal_charge_level: c_int,
    #[test_offset = 0x62c]
    pub is_artillery: bool,
}

impl ProjectileFactory {
    pub unsafe fn num_targets_required(&self) -> c_int {
        if (*self.blueprint).charge_levels > 1 {
            self.charge_level.max(1)
        } else {
            self.num_shots
        }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipGraph {
    #[test_offset = 0x0]
    pub rooms: Vector<*const Room>,
    #[test_offset = 0x18]
    pub doors: Vector<*const Door>,
    #[test_offset = 0x30]
    pub door_counts: Vector<c_int>,
    #[test_offset = 0x48]
    pub center: Point,
    #[test_offset = 0x50]
    pub world_position: Pointf,
    #[test_offset = 0x58]
    pub world_heading: c_float,
    #[test_offset = 0x5c]
    pub last_world_position: Pointf,
    #[test_offset = 0x64]
    pub last_world_heading: c_float,
    #[test_offset = 0x68]
    pub ship_box: Rect,
    #[test_offset = 0x78]
    pub ship_name: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponSystem {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub target: Pointf,
    #[test_offset = 0x250]
    pub weapons: Vector<*mut ProjectileFactory>,
    #[test_offset = 0x268]
    pub weapons_trash_list: Vector<*mut ProjectileFactory>,
    #[test_offset = 0x280]
    pub shot_timer: c_float,
    #[test_offset = 0x284]
    pub shot_count: c_int,
    #[test_offset = 0x288]
    pub missile_count: c_int,
    #[test_offset = 0x28c]
    pub missile_start: c_int,
    #[test_offset = 0x290]
    pub cloaking_system: *mut CloakingSystem,
    #[test_offset = 0x298]
    pub user_powered: VectorBool,
    #[test_offset = 0x2c0]
    pub slot_count: c_int,
    #[test_offset = 0x2c4]
    pub i_starting_battery_power: c_int,
    #[test_offset = 0x2c8]
    pub repower_list: VectorBool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShieldAnimation {
    #[test_offset = 0x0]
    pub location: Pointf,
    #[test_offset = 0x8]
    pub current_size: c_float,
    #[test_offset = 0xc]
    pub end_size: c_float,
    #[test_offset = 0x10]
    pub current_thickness: c_float,
    #[test_offset = 0x14]
    pub end_thickness: c_float,
    #[test_offset = 0x18]
    pub length: c_float,
    #[test_offset = 0x1c]
    pub dx: c_float,
    #[test_offset = 0x20]
    pub side: c_int,
    #[test_offset = 0x24]
    pub owner_id: c_int,
    #[test_offset = 0x28]
    pub damage: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShieldPower {
    #[test_offset = 0x0]
    pub first: c_int,
    #[test_offset = 0x4]
    pub second: c_int,
    #[test_offset = 0x8]
    pub super_: Pair<c_int, c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Shield {
    #[test_offset = 0x0]
    pub charger: c_float,
    #[test_offset = 0x4]
    pub power: ShieldPower,
    #[test_offset = 0x14]
    pub super_timer: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Shields {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub ellipse_ratio: c_float,
    #[test_offset = 0x248]
    pub center: Point,
    #[test_offset = 0x250]
    pub base_shield: Ellipse,
    #[test_offset = 0x260]
    pub i_highlighted_side: c_int,
    #[test_offset = 0x264]
    pub debug_x: c_float,
    #[test_offset = 0x268]
    pub debug_y: c_float,
    #[test_offset = 0x26c]
    pub shields: Shield,
    #[test_offset = 0x284]
    pub shields_shutdown: bool,
    #[test_offset = 0x288]
    pub shield_hits: Vector<ShieldAnimation>,
    #[test_offset = 0x2a0]
    pub shields_down: AnimationTracker,
    #[test_offset = 0x2c0]
    pub super_shield_down: bool,
    #[test_offset = 0x2c4]
    pub shields_down_point: Pointf,
    #[test_offset = 0x2d0]
    pub shields_up: AnimationTracker,
    #[test_offset = 0x2f0]
    pub shield_image: *mut GL_Texture,
    #[test_offset = 0x2f8]
    pub shield_primitive: *mut GL_Primitive,
    #[test_offset = 0x300]
    pub shield_image_name: StdString,
    #[test_offset = 0x308]
    pub b_enemy_present: bool,
    #[test_offset = 0x310]
    pub dam_messages: Vector<*mut DamageMessage>,
    #[test_offset = 0x328]
    pub b_barrier_mode: bool,
    #[test_offset = 0x32c]
    pub last_hit_timer: c_float,
    #[test_offset = 0x330]
    pub charge_time: c_float,
    #[test_offset = 0x334]
    pub last_hit_shield_level: c_int,
    #[test_offset = 0x338]
    pub super_shield_up: AnimationTracker,
    #[test_offset = 0x358]
    pub super_up_loc: Point,
    #[test_offset = 0x360]
    pub b_excess_charge_hack: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct HackingSystem {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub b_hacking: bool,
    #[test_offset = 0x248]
    pub drone: HackingDrone,
    #[test_offset = 0x820]
    pub b_blocked: bool,
    #[test_offset = 0x821]
    pub b_armed: bool,
    #[test_offset = 0x828]
    pub current_system: *mut ShipSystem,
    #[test_offset = 0x830]
    pub effect_timer: Pair<c_float, c_float>,
    #[test_offset = 0x838]
    pub b_can_hack: bool,
    #[test_offset = 0x840]
    pub queued_system: *mut ShipSystem,
    #[test_offset = 0x848]
    pub spend_drone: c_int,
}

impl HackingSystem {
    pub fn can_hack(&self) -> bool {
        !self.b_hacking && self.base.functioning() && self.b_can_hack
    }
    pub fn can_pulse(&self) -> bool {
        self.b_hacking
            && self.base.effective_power() != 0
            && !self.current_system.is_null()
            && !self.base.completely_destroyed()
            && self.drone.arrived
            && !self.base.locked()
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CloneSystem {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub f_time_to_clone: c_float,
    #[test_offset = 0x248]
    pub clone: *mut CrewMember,
    #[test_offset = 0x250]
    pub f_time_goal: c_float,
    #[test_offset = 0x254]
    pub f_death_time: c_float,
    #[test_offset = 0x258]
    pub bottom: *mut GL_Texture,
    #[test_offset = 0x260]
    pub top: *mut GL_Texture,
    #[test_offset = 0x268]
    pub gas: *mut GL_Texture,
    #[test_offset = 0x270]
    pub slot: c_int,
    #[test_offset = 0x278]
    pub current_clone_animation: *mut Animation,
    #[test_offset = 0x280]
    pub clone_animations: Map<StdString, Animation>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct MindSystem {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub control_timer: Pair<c_float, c_float>,
    #[test_offset = 0x24c]
    pub b_can_use: bool,
    #[test_offset = 0x250]
    pub i_armed: c_int,
    #[test_offset = 0x258]
    pub controlled_crew: Vector<*mut CrewMember>,
    #[test_offset = 0x270]
    pub b_super_shields: bool,
    #[test_offset = 0x271]
    pub b_blocked: bool,
    #[test_offset = 0x274]
    pub i_queued_target: c_int,
    #[test_offset = 0x278]
    pub i_queued_ship: c_int,
    #[test_offset = 0x280]
    pub queued_crew: Vector<*mut CrewMember>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BatterySystem {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub b_turned_on: bool,
    #[test_offset = 0x248]
    pub timer: TimerHelper,
    #[test_offset = 0x260]
    pub soundeffect: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CloakingSystem {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub b_turned_on: bool,
    #[test_offset = 0x248]
    pub timer: TimerHelper,
    #[test_offset = 0x260]
    pub soundeffect: StdString,
    #[test_offset = 0x268]
    pub glow_tracker: AnimationTracker,
    #[test_offset = 0x288]
    pub glow_image: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TeleportSystem {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub charge_level: c_float,
    #[test_offset = 0x248]
    pub b_can_send: bool,
    #[test_offset = 0x249]
    pub b_can_receive: bool,
    #[test_offset = 0x24c]
    pub i_armed: c_int,
    #[test_offset = 0x250]
    pub crew_slots: VectorBool,
    #[test_offset = 0x278]
    pub i_prepared_crew: c_int,
    #[test_offset = 0x27c]
    pub i_num_slots: c_int,
    #[test_offset = 0x280]
    pub b_super_shields: bool,
}

impl TeleportSystem {
    pub fn can_receive(&self) -> bool {
        if !self.b_can_receive && self.base.i_ship_id == 0 || !self.base.functioning() {
            false
        } else {
            !self.base.locked() && self.base.functioning()
        }
    }
    pub fn can_send(&self, gui: &CommandGui) -> bool {
        if self.base.i_ship_id != 0
            && self.b_super_shields
            && !gui.equip_screen.has_augment("ZOLTAN_BYPASS")
        {
            false
        } else {
            (self.b_can_send || self.base.i_ship_id != 0)
                && self.base.functioning()
                && !self.base.locked()
                && self.i_prepared_crew != 0
        }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct OxygenSystem {
    #[test_offset = 0x0]
    pub base: ShipSystemPrime,
    #[test_offset = 0x240]
    pub computer_level: c_int,
    #[test_offset = 0x244]
    pub max_oxygen: c_float,
    #[test_offset = 0x248]
    pub oxygen_levels: Vector<c_float>,
    #[test_offset = 0x260]
    pub f_total_oxygen: c_float,
    #[test_offset = 0x264]
    pub b_leaking_o2: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ComputerGlowInfo {
    #[test_offset = 0x0]
    pub name: StdString,
    #[test_offset = 0x8]
    pub x: c_int,
    #[test_offset = 0xc]
    pub y: c_int,
    #[test_offset = 0x10]
    pub direction: c_int,
}

#[vtable]
pub struct VtableShipSystem {
    pub base: VtableRepairable,
    // 25
    pub set_hacking_level: Option<fn(*mut ShipSystem, c_int)>,
    pub force_battery_power: Option<fn(*mut ShipSystem, c_int)>,
    pub remove_battery_power: Option<fn(*mut ShipSystem)>,
    pub get_weapon_info: Option<fn(*mut ShipSystem) -> *const WeaponBlueprint>,
    pub get_override_tooltip: Option<fn(*mut ShipSystem) -> StdString>,
    pub check_max_power: Option<fn(*mut ShipSystem)>,
    pub set_bonus_power: Option<fn(*mut ShipSystem, c_int, c_int)>,
    pub add_damage: Option<fn(*mut ShipSystem, c_int)>,
    pub force_decrease_power: Option<fn(*mut ShipSystem, c_int) -> bool>,
    pub force_increase_power: Option<fn(*mut ShipSystem, c_int) -> bool>,
    pub jump: Option<fn(*mut ShipSystem)>,
    pub on_render: Option<fn(*mut ShipSystem)>,
    pub on_render_floor: Option<fn(*mut ShipSystem)>,
    pub on_render_effects: Option<fn(*mut ShipSystem)>,
    pub on_loop: Option<fn(*mut ShipSystem)>,
    pub get_needs_power: Option<fn(*mut ShipSystem) -> bool>,
    pub restart: Option<fn(*mut ShipSystem)>,
    pub clickable: Option<fn(*mut ShipSystem) -> bool>,
    pub powered: Option<fn(*mut ShipSystem) -> bool>,
    pub ship_destroyed: Option<fn(*mut ShipSystem)>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CachedRectOutline {
    #[test_offset = 0x0]
    pub base: CachedPrimitive,
    #[test_offset = 0x10]
    pub x: c_int,
    #[test_offset = 0x14]
    pub y: c_int,
    #[test_offset = 0x18]
    pub w: c_int,
    #[test_offset = 0x1c]
    pub h: c_int,
    #[test_offset = 0x20]
    pub thickness: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CachedRect {
    #[test_offset = 0x0]
    pub base: CachedPrimitive,
    #[test_offset = 0x10]
    pub x: c_int,
    #[test_offset = 0x14]
    pub y: c_int,
    #[test_offset = 0x18]
    pub w: c_int,
    #[test_offset = 0x1c]
    pub h: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipSystem {
    pub vtable: *const VtableShipSystem,
    #[test_offset = 0x8]
    /// Inherited from Repairable
    pub selected_state: c_int,
    #[test_offset = 0x10]
    /// Inherited from Repairable
    pub base1_vtable: *const VtableShipObject,
    #[test_offset = 0x18]
    /// Inherited from Repairable
    pub i_ship_id: c_int,
    #[test_offset = 0x1c]
    /// Inherited from Repairable
    pub f_damage: c_float,
    #[test_offset = 0x20]
    /// Inherited from Repairable
    pub p_loc: Point,
    #[test_offset = 0x28]
    /// Inherited from Repairable
    pub f_max_damage: c_float,
    #[test_offset = 0x30]
    /// Inherited from Repairable
    pub name: StdString,
    #[test_offset = 0x38]
    /// Inherited from Repairable
    pub room_id: c_int,
    #[test_offset = 0x3c]
    /// Inherited from Repairable
    pub i_repair_count: c_int,
    /// System type
    #[test_offset = 0x40]
    pub i_system_type: c_int,
    /// Doesn't work without manning
    #[test_offset = 0x44]
    pub b_needs_manned: bool,
    /// Basically never used
    #[test_offset = 0x45]
    pub b_manned: bool,
    /// How many people are manning
    #[test_offset = 0x48]
    pub i_active_manned: c_int,
    /// Whether manning gives bonus power
    #[test_offset = 0x4c]
    pub b_boostable: bool,
    /// Allocated power and upgrade level
    #[test_offset = 0x50]
    pub power_state: Pair<c_int, c_int>,
    /// I feel like this isn't used? idk
    #[test_offset = 0x58]
    pub i_required_power: c_int,
    #[test_offset = 0x60]
    pub image_icon: *mut GL_Texture,
    #[test_offset = 0x68]
    pub icon_primitive: *mut GL_Primitive,
    #[test_offset = 0x70]
    pub icon_border_primitive: *mut GL_Primitive,
    #[test_offset = 0x78]
    pub icon_primitives: [[[*mut GL_Primitive; 5]; 2]; 2],
    #[test_offset = 0x118]
    pub partial_damage_rect: CachedRect,
    #[test_offset = 0x138]
    pub lock_outline: CachedRectOutline,
    #[test_offset = 0x160]
    pub room_shape: Rect,
    /// Obvious
    #[test_offset = 0x170]
    pub b_on_fire: bool,
    /// If the room is breached this can't be repaired until the breach is fixed
    #[test_offset = 0x171]
    pub b_breached: bool,
    /// Current/max HP
    #[test_offset = 0x174]
    pub health_state: Pair<c_int, c_int>,
    #[test_offset = 0x17c]
    pub f_damage_over_time: c_float,
    #[test_offset = 0x180]
    pub f_repair_over_time: c_float,
    #[test_offset = 0x184]
    pub damaged_last_frame: bool,
    #[test_offset = 0x185]
    pub repaired_last_frame: bool,
    #[test_offset = 0x188]
    pub original_power: c_int,
    /// basically, whether this is a subsystem
    #[test_offset = 0x18c]
    pub b_needs_power: bool,
    #[test_offset = 0x190]
    pub i_temp_power_cap: c_int,
    #[test_offset = 0x194]
    pub i_temp_power_loss: c_int,
    #[test_offset = 0x198]
    pub i_temp_divide_power: c_int,
    #[test_offset = 0x19c]
    pub i_lock_count: c_int,
    #[test_offset = 0x1a0]
    pub lock_timer: TimerHelper,
    #[test_offset = 0x1b4]
    pub b_exploded: bool,
    #[test_offset = 0x1b5]
    pub b_occupied: bool,
    #[test_offset = 0x1b6]
    pub b_friendlies: bool,
    #[test_offset = 0x1b8]
    pub interior_image_name: StdString,
    #[test_offset = 0x1c0]
    pub interior_image: *mut GL_Primitive,
    #[test_offset = 0x1c8]
    pub interior_image_on: *mut GL_Primitive,
    #[test_offset = 0x1d0]
    pub interior_image_manned: *mut GL_Primitive,
    #[test_offset = 0x1d8]
    pub interior_image_manned_fancy: *mut GL_Primitive,
    #[test_offset = 0x1e0]
    pub last_user_power: c_int,
    #[test_offset = 0x1e4]
    pub i_bonus_power: c_int,
    #[test_offset = 0x1e8]
    pub i_last_bonus_power: c_int,
    #[test_offset = 0x1ec]
    pub location: Pointf,
    #[test_offset = 0x1f4]
    pub bp_cost: c_int,
    #[test_offset = 0x1f8]
    pub flash_tracker: AnimationTracker,
    #[test_offset = 0x218]
    pub max_level: c_int,
    #[test_offset = 0x21c]
    pub i_battery_power: c_int,
    #[test_offset = 0x220]
    pub i_hack_effect: c_int,
    #[test_offset = 0x224]
    pub b_under_attack: bool,
    #[test_offset = 0x225]
    pub b_level_boostable: bool,
    #[test_offset = 0x226]
    pub b_trigger_ion: bool,
    #[test_offset = 0x228]
    pub damaging_effects: Vector<Animation>,
    #[test_offset = 0x240]
    pub computer_level: c_int,
}

impl ShipSystem {
    pub fn max_power(&self) -> c_int {
        let ret = self.power_state.second
            - (self.health_state.second - self.health_state.first)
            - self.i_temp_power_loss;
        if self.i_temp_power_cap > 7 {
            self.power_state.second
        } else {
            self.i_temp_power_cap
        }
        .max(0)
        .min(ret)
    }
    pub fn effective_power(&self) -> c_int {
        c_int::from(
            self.i_active_manned > 0
                && self.b_boostable
                && self.health_state.first == self.health_state.second
                && self.b_level_boostable,
        ) + self.i_battery_power
            + self.i_bonus_power
            + self.power_state.first
    }
    pub fn available_power(&self) -> c_int {
        self.max_power() - self.effective_power()
    }
    pub fn damage(&self) -> c_int {
        self.health_state.second - self.health_state.first
    }
    pub fn power_max(&self) -> c_int {
        self.power_state.second
    }
    pub fn locked(&self) -> bool {
        self.i_lock_count == -1 || self.i_lock_count > 0 || self.i_hack_effect > 1
    }
    pub fn powered(&self) -> bool {
        self.effective_power() > c_int::from(self.i_system_type == System::Shields as c_int)
    }
    pub fn functioning(&self) -> bool {
        if self.i_system_type == System::Pilot as c_int {
            if !self.b_manned {
                return false;
            }
        } else if self.b_needs_manned
            && (self.i_active_manned <= 0
                || !self.b_boostable
                || self.health_state.first != self.health_state.second)
        {
            return false;
        }
        self.powered()
    }
    pub fn completely_destroyed(&self) -> bool {
        self.health_state.first == 0
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct PowerManager {
    #[test_offset = 0x0]
    pub current_power: Pair<c_int, c_int>,
    #[test_offset = 0x8]
    pub over_powered: c_int,
    #[test_offset = 0xc]
    pub f_fuel: c_float,
    #[test_offset = 0x10]
    pub failed_powerup: bool,
    #[test_offset = 0x14]
    pub i_temp_power_cap: c_int,
    #[test_offset = 0x18]
    pub i_temp_power_loss: c_int,
    #[test_offset = 0x1c]
    pub i_temp_divide_power: c_int,
    #[test_offset = 0x20]
    pub i_hacked: c_int,
    #[test_offset = 0x24]
    pub battery_power: Pair<c_int, c_int>,
}

/// A modified version of ShipSystem without computer_level because of the damn padding
#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipSystemPrime {
    pub vtable: *const VtableShipSystem,
    #[test_offset = 0x8]
    /// Inherited from Repairable
    pub selected_state: c_int,
    #[test_offset = 0x10]
    /// Inherited from Repairable
    pub base1_vtable: *const VtableShipObject,
    /// Inherited from Repairable
    pub i_ship_id: c_int,
    #[test_offset = 0x1c]
    /// Inherited from Repairable
    pub f_damage: c_float,
    #[test_offset = 0x20]
    /// Inherited from Repairable
    pub p_loc: Point,
    #[test_offset = 0x28]
    /// Inherited from Repairable
    pub f_max_damage: c_float,
    #[test_offset = 0x30]
    /// Inherited from Repairable
    pub name: StdString,
    #[test_offset = 0x38]
    /// Inherited from Repairable
    pub room_id: c_int,
    #[test_offset = 0x3c]
    /// Inherited from Repairable
    pub i_repair_count: c_int,
    #[test_offset = 0x40]
    pub i_system_type: c_int,
    #[test_offset = 0x44]
    pub b_needs_manned: bool,
    #[test_offset = 0x45]
    pub b_manned: bool,
    #[test_offset = 0x48]
    pub i_active_manned: c_int,
    #[test_offset = 0x4c]
    pub b_boostable: bool,
    #[test_offset = 0x50]
    pub power_state: Pair<c_int, c_int>,
    #[test_offset = 0x58]
    pub i_required_power: c_int,
    #[test_offset = 0x60]
    pub image_icon: *mut GL_Texture,
    #[test_offset = 0x68]
    pub icon_primitive: *mut GL_Primitive,
    #[test_offset = 0x70]
    pub icon_border_primitive: *mut GL_Primitive,
    #[test_offset = 0x78]
    pub icon_primitives: [[[*mut GL_Primitive; 5]; 2]; 2],
    #[test_offset = 0x118]
    pub partial_damage_rect: CachedRect,
    #[test_offset = 0x138]
    pub lock_outline: CachedRectOutline,
    #[test_offset = 0x160]
    pub room_shape: Rect,
    #[test_offset = 0x170]
    pub b_on_fire: bool,
    #[test_offset = 0x171]
    pub b_breached: bool,
    #[test_offset = 0x174]
    pub health_state: Pair<c_int, c_int>,
    #[test_offset = 0x17c]
    pub f_damage_over_time: c_float,
    #[test_offset = 0x180]
    pub f_repair_over_time: c_float,
    #[test_offset = 0x184]
    pub damaged_last_frame: bool,
    #[test_offset = 0x185]
    pub repaired_last_frame: bool,
    #[test_offset = 0x188]
    pub original_power: c_int,
    #[test_offset = 0x18c]
    pub b_needs_power: bool,
    #[test_offset = 0x190]
    pub i_temp_power_cap: c_int,
    #[test_offset = 0x194]
    pub i_temp_power_loss: c_int,
    #[test_offset = 0x198]
    pub i_temp_divide_power: c_int,
    #[test_offset = 0x19c]
    pub i_lock_count: c_int,
    #[test_offset = 0x1a0]
    pub lock_timer: TimerHelper,
    #[test_offset = 0x1b4]
    pub b_exploded: bool,
    #[test_offset = 0x1b5]
    pub b_occupied: bool,
    #[test_offset = 0x1b6]
    pub b_friendlies: bool,
    #[test_offset = 0x1b8]
    pub interior_image_name: StdString,
    #[test_offset = 0x1c0]
    pub interior_image: *mut GL_Primitive,
    #[test_offset = 0x1c8]
    pub interior_image_on: *mut GL_Primitive,
    #[test_offset = 0x1d0]
    pub interior_image_manned: *mut GL_Primitive,
    #[test_offset = 0x1d8]
    pub interior_image_manned_fancy: *mut GL_Primitive,
    #[test_offset = 0x1e0]
    pub last_user_power: c_int,
    #[test_offset = 0x1e4]
    pub i_bonus_power: c_int,
    #[test_offset = 0x1e8]
    pub i_last_bonus_power: c_int,
    #[test_offset = 0x1ec]
    pub location: Pointf,
    #[test_offset = 0x1f4]
    pub bp_cost: c_int,
    #[test_offset = 0x1f8]
    pub flash_tracker: AnimationTracker,
    #[test_offset = 0x218]
    pub max_level: c_int,
    #[test_offset = 0x21c]
    pub i_battery_power: c_int,
    #[test_offset = 0x220]
    pub i_hack_effect: c_int,
    #[test_offset = 0x224]
    pub b_under_attack: bool,
    #[test_offset = 0x225]
    pub b_level_boostable: bool,
    #[test_offset = 0x226]
    pub b_trigger_ion: bool,
    #[test_offset = 0x228]
    pub damaging_effects: Vector<Animation>,
}

impl Deref for ShipSystemPrime {
    type Target = ShipSystem;
    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute::<&Self, &ShipSystem>(self) }
    }
}

impl DerefMut for ShipSystemPrime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { mem::transmute::<&mut Self, &mut ShipSystem>(self) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipManager {
    #[test_offset = 0x0]
    pub vtable: *const VtableShipManager,
    #[test_offset = 0x8]
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[test_offset = 0x10]
    pub base1: Targetable,
    #[test_offset = 0x20]
    pub base2: Collideable,
    #[test_offset = 0x28]
    pub v_system_list: Vector<*mut ShipSystem>,
    #[test_offset = 0x40]
    pub oxygen_system: *mut OxygenSystem,
    #[test_offset = 0x48]
    pub teleport_system: *mut TeleportSystem,
    #[test_offset = 0x50]
    pub cloak_system: *mut CloakingSystem,
    #[test_offset = 0x58]
    pub battery_system: *mut BatterySystem,
    #[test_offset = 0x60]
    pub mind_system: *mut MindSystem,
    #[test_offset = 0x68]
    pub clone_system: *mut CloneSystem,
    #[test_offset = 0x70]
    pub hacking_system: *mut HackingSystem,
    #[test_offset = 0x78]
    pub show_network: bool,
    #[test_offset = 0x79]
    pub added_system: bool,
    #[test_offset = 0x80]
    pub shield_system: *mut Shields,
    #[test_offset = 0x88]
    pub weapon_system: *mut WeaponSystem,
    #[test_offset = 0x90]
    pub drone_system: *mut DroneSystem,
    #[test_offset = 0x98]
    pub engine_system: *mut EngineSystem,
    #[test_offset = 0xa0]
    pub medbay_system: *mut MedbaySystem,
    #[test_offset = 0xa8]
    pub artillery_systems: Vector<*mut ArtillerySystem>,
    #[test_offset = 0xc0]
    pub v_crew_list: Vector<*mut CrewMember>,
    #[test_offset = 0xd8]
    pub fire_spreader: Spreader<Fire>,
    #[test_offset = 0x118]
    pub ship: Ship,
    #[test_offset = 0x5a8]
    pub status_messages: Queue<String>,
    #[test_offset = 0x5f8]
    pub b_game_over: bool,
    #[test_offset = 0x600]
    pub current_target: *mut ShipManager,
    #[test_offset = 0x608]
    pub jump_timer: Pair<c_float, c_float>,
    #[test_offset = 0x610]
    pub fuel_count: c_int,
    #[test_offset = 0x614]
    pub hostile_ship: bool,
    #[test_offset = 0x615]
    pub b_destroyed: bool,
    #[test_offset = 0x618]
    pub i_last_damage: c_int,
    #[test_offset = 0x620]
    pub jump_animation: AnimationTracker,
    #[test_offset = 0x640]
    pub dam_messages: Vector<*mut DamageMessage>,
    #[test_offset = 0x658]
    pub system_key: Vector<c_int>,
    #[test_offset = 0x670]
    pub current_scrap: c_int,
    #[test_offset = 0x674]
    pub b_jumping: bool,
    #[test_offset = 0x675]
    pub b_automated: bool,
    #[test_offset = 0x678]
    pub ship_level: c_int,
    #[test_offset = 0x680]
    pub my_blueprint: ShipBlueprint,
    #[test_offset = 0x8d0]
    pub last_engine_status: bool,
    #[test_offset = 0x8d1]
    pub last_jump_ready: bool,
    #[test_offset = 0x8d2]
    pub b_contains_player_crew: bool,
    #[test_offset = 0x8d4]
    pub i_intruder_count: c_int,
    #[test_offset = 0x8d8]
    pub crew_counts: Vector<Vector<c_int>>,
    #[test_offset = 0x8f0]
    pub temp_drone_count: c_int,
    #[test_offset = 0x8f4]
    pub temp_missile_count: c_int,
    #[test_offset = 0x8f8]
    pub explosions: Vector<Animation>,
    #[test_offset = 0x910]
    pub temp_vision: VectorBool,
    #[test_offset = 0x938]
    pub b_highlight_crew: bool,
    #[test_offset = 0x940]
    pub drone_trash: Vector<*mut Drone>,
    #[test_offset = 0x958]
    pub space_drones: Vector<*mut SpaceDrone>,
    #[test_offset = 0x970]
    pub new_drone_arrivals: Vector<*mut SpaceDrone>,
    #[test_offset = 0x988]
    pub bp_count: c_int,
    #[test_offset = 0x98c]
    pub i_customize_mode: c_int,
    #[test_offset = 0x990]
    pub b_show_room: bool,
    #[test_offset = 0x998]
    pub super_barrage: Vector<*mut Projectile>,
    #[test_offset = 0x9b0]
    pub b_invincible: bool,
    #[test_offset = 0x9b8]
    pub super_drones: Vector<*mut SpaceDrone>,
    #[test_offset = 0x9d0]
    pub highlight: *mut GL_Primitive,
    #[test_offset = 0x9d8]
    pub failed_dodge_counter: c_int,
    #[test_offset = 0x9e0]
    pub hit_by_beam: Vector<c_float>,
    #[test_offset = 0x9f8]
    pub enemy_damaged_uncloaked: bool,
    #[test_offset = 0x9fc]
    pub damage_cloaked: c_int,
    #[test_offset = 0xa00]
    pub killed_by_beam: Map<c_int, c_int>,
    #[test_offset = 0xa30]
    pub min_beacon_health: c_int,
    #[test_offset = 0xa38]
    pub fire_extinguishers: Vector<*mut ParticleEmitter>,
    #[test_offset = 0xa50]
    pub b_was_safe: bool,
}

impl ShipManager {
    pub fn power_drone(
        &mut self,
        drone: *mut Drone,
        room_id: c_int,
        user_driven: bool,
        force: bool,
    ) -> bool {
        unsafe {
            (super::POWER_DRONE.unwrap())(
                ptr::addr_of_mut!(*self),
                drone,
                room_id,
                user_driven,
                force,
            )
        }
    }
    pub fn power_weapon(
        &mut self,
        weapon: *mut ProjectileFactory,
        user_driven: bool,
        force: bool,
    ) -> bool {
        unsafe {
            (super::POWER_WEAPON.unwrap())(ptr::addr_of_mut!(*self), weapon, user_driven, force)
        }
    }
    pub fn depower_drone(&mut self, drone: *mut Drone, user_driven: bool) -> bool {
        unsafe { (super::DEPOWER_DRONE.unwrap())(ptr::addr_of_mut!(*self), drone, user_driven) }
    }
    pub fn depower_weapon(&mut self, weapon: *mut ProjectileFactory, user_driven: bool) -> bool {
        unsafe { (super::DEPOWER_WEAPON.unwrap())(ptr::addr_of_mut!(*self), weapon, user_driven) }
    }
    pub fn has_system(&self, system: System) -> bool {
        match system {
            System::Reactor => true,
            system => unsafe { *self.system_key.get_ptr(system as usize) != -1 },
        }
    }
    pub unsafe fn shield_system(&self) -> &Shields {
        unsafe { &*self.shield_system }
    }
    pub unsafe fn engine_system(&self) -> &EngineSystem {
        unsafe { &*self.engine_system }
    }
    pub unsafe fn oxygen_system(&self) -> &OxygenSystem {
        unsafe { &*self.oxygen_system }
    }
    pub unsafe fn weapon_system(&self) -> &WeaponSystem {
        unsafe { &*self.weapon_system }
    }
    pub unsafe fn drone_system(&self) -> &DroneSystem {
        unsafe { &*self.drone_system }
    }
    pub unsafe fn medbay_system(&self) -> &MedbaySystem {
        unsafe { &*self.medbay_system }
    }
    pub unsafe fn teleport_system(&self) -> &TeleportSystem {
        unsafe { &*self.teleport_system }
    }
    pub unsafe fn teleport_system_mut(&mut self) -> &mut TeleportSystem {
        unsafe { &mut *self.teleport_system }
    }
    pub unsafe fn cloak_system(&self) -> &CloakingSystem {
        unsafe { &*self.cloak_system }
    }
    pub unsafe fn cloak_system_mut(&mut self) -> &mut CloakingSystem {
        unsafe { &mut *self.cloak_system }
    }
    pub unsafe fn artillery_systems(&self) -> impl Iterator<Item = &ArtillerySystem> {
        self.artillery_systems.iter().map(|x| unsafe { &**x })
    }
    pub unsafe fn battery_system(&self) -> &BatterySystem {
        unsafe { &*self.battery_system }
    }
    pub unsafe fn battery_system_mut(&mut self) -> &mut BatterySystem {
        unsafe { &mut *self.battery_system }
    }
    pub unsafe fn clone_system(&self) -> &CloneSystem {
        unsafe { &*self.clone_system }
    }
    pub unsafe fn mind_system(&self) -> &MindSystem {
        unsafe { &*self.mind_system }
    }
    pub unsafe fn mind_system_mut(&mut self) -> &mut MindSystem {
        unsafe { &mut *self.mind_system }
    }
    pub unsafe fn hacking_system(&self) -> &HackingSystem {
        unsafe { &*self.hacking_system }
    }
    pub unsafe fn hacking_system_mut(&mut self) -> &mut HackingSystem {
        unsafe { &mut *self.hacking_system }
    }
    pub fn system(&self, system: System) -> Option<&ShipSystem> {
        let key = unsafe { *self.system_key.get_ptr(system as usize) };
        (key >= 0)
            .then(|| self.v_system_list.get_ptr(key as usize))
            .and_then(|x| (!x.is_null()).then(|| unsafe { *x }))
            .map(|x| unsafe { &*x })
    }
    pub fn system_mut(&mut self, system: System) -> Option<&mut ShipSystem> {
        let key = unsafe { *self.system_key.get_ptr(system as usize) };
        (key >= 0)
            .then(|| self.v_system_list.get_ptr(key as usize))
            .and_then(|x| (!x.is_null()).then(|| unsafe { *x }))
            .map(|x| unsafe { &mut *x })
    }
    pub fn systems(&self) -> impl Iterator<Item = &ShipSystem> {
        self.v_system_list.iter().map(|x| unsafe { &**x })
    }
    pub fn has_crew(&self, name: &str) -> bool {
        self.v_crew_list
            .iter()
            .any(|x| unsafe { !(**x).b_dead && (**x).blueprint.name.to_str() == name })
    }
    pub fn drone_count(&self) -> c_int {
        if self.has_system(System::Drones) {
            unsafe { self.drone_system().drone_count }
        } else {
            self.temp_drone_count
        }
    }
    pub fn missile_count(&self) -> c_int {
        if self.has_system(System::Weapons) {
            unsafe { self.weapon_system().missile_count }
        } else {
            self.temp_missile_count
        }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipStatus {
    #[test_offset = 0x0]
    pub location: Point,
    #[test_offset = 0x8]
    pub size: c_float,
    #[test_offset = 0x10]
    pub ship: *mut ShipManager,
    #[test_offset = 0x18]
    pub combat: *mut CombatControl,
    #[test_offset = 0x20]
    pub hull_box: *mut GL_Primitive,
    #[test_offset = 0x28]
    pub hull_box_red: *mut GL_Primitive,
    #[test_offset = 0x30]
    pub shield_box_on: *mut GL_Primitive,
    #[test_offset = 0x38]
    pub shield_box_off: *mut GL_Primitive,
    #[test_offset = 0x40]
    pub shield_box_red: *mut GL_Primitive,
    #[test_offset = 0x48]
    pub shield_circle_charged: [*mut GL_Primitive; 4],
    #[test_offset = 0x68]
    pub shield_circle_uncharged: [*mut GL_Primitive; 4],
    #[test_offset = 0x88]
    pub shield_circle_hacked: [*mut GL_Primitive; 4],
    #[test_offset = 0xa8]
    pub shield_circle_hacked_charged: [*mut GL_Primitive; 4],
    #[test_offset = 0xc8]
    pub energy_shield_box: *mut GL_Primitive,
    #[test_offset = 0xd0]
    pub energy_shield_bar: [*mut GL_Primitive; 5],
    #[test_offset = 0xf8]
    pub hull_label: *mut GL_Texture,
    #[test_offset = 0x100]
    pub hull_label_red: *mut GL_Texture,
    #[test_offset = 0x108]
    pub shield_box_purple: *mut GL_Primitive,
    #[test_offset = 0x110]
    pub oxygen_purple: *mut GL_Primitive,
    #[test_offset = 0x118]
    pub evade_purple: *mut GL_Primitive,
    #[test_offset = 0x120]
    pub evade_oxygen_box: *mut GL_Primitive,
    #[test_offset = 0x128]
    pub evade_oxygen_box_top_red: *mut GL_Primitive,
    #[test_offset = 0x130]
    pub evade_oxygen_box_bottom_red: *mut GL_Primitive,
    #[test_offset = 0x138]
    pub evade_oxygen_box_both_red: *mut GL_Primitive,
    #[test_offset = 0x140]
    pub fuel_icon: *mut GL_Primitive,
    #[test_offset = 0x148]
    pub missiles_icon: *mut GL_Primitive,
    #[test_offset = 0x150]
    pub drones_icon: *mut GL_Primitive,
    #[test_offset = 0x158]
    pub scrap_icon: *mut GL_Primitive,
    #[test_offset = 0x160]
    pub fuel_icon_red: *mut GL_Primitive,
    #[test_offset = 0x168]
    pub missiles_icon_red: *mut GL_Primitive,
    #[test_offset = 0x170]
    pub drones_icon_red: *mut GL_Primitive,
    #[test_offset = 0x178]
    pub scrap_icon_red: *mut GL_Primitive,
    #[test_offset = 0x180]
    pub health_mask: *mut GL_Primitive,
    #[test_offset = 0x188]
    pub health_mask_texture: *mut GL_Texture,
    #[test_offset = 0x190]
    pub last_health: c_int,
    #[test_offset = 0x194]
    pub base_shield: Ellipse,
    #[test_offset = 0x1a4]
    pub current_hover: c_int,
    #[test_offset = 0x1a8]
    pub evade_oxygen_box_location: Point,
    #[test_offset = 0x1b0]
    pub last_fuel: c_int,
    #[test_offset = 0x1b4]
    pub last_drones: c_int,
    #[test_offset = 0x1b8]
    pub last_scrap: c_int,
    #[test_offset = 0x1bc]
    pub last_missiles: c_int,
    #[test_offset = 0x1c0]
    pub last_hull: c_int,
    #[test_offset = 0x1c8]
    pub hull_message: *mut WarningWithLines,
    #[test_offset = 0x1d0]
    pub shield_message: *mut WarningMessage,
    #[test_offset = 0x1d8]
    pub oxygen_message: *mut WarningMessage,
    #[test_offset = 0x1e0]
    pub boarding_message: *mut WarningMessage,
    #[test_offset = 0x1e8]
    pub resource_messages: Vector<*mut DamageMessage>,
    #[test_offset = 0x200]
    pub no_money_tracker: AnimationTracker,
    #[test_offset = 0x220]
    pub flash_tracker: AnimationTracker,
    #[test_offset = 0x240]
    pub b_boss_fight: bool,
    #[test_offset = 0x241]
    pub b_enemy_ship: bool,
    #[test_offset = 0x244]
    pub no_ship_shift: Point,
    #[test_offset = 0x24c]
    pub intruder_shift: Point,
    #[test_offset = 0x254]
    pub energy_shield_pos: Point,
    #[test_offset = 0x25c]
    pub intruder_pos: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CommandGui {
    #[test_offset = 0x0]
    pub ship_status: ShipStatus,
    #[test_offset = 0x268]
    pub crew_control: CrewControl,
    #[test_offset = 0x490]
    pub sys_control: SystemControl,
    #[test_offset = 0x588]
    pub combat_control: CombatControl,
    #[test_offset = 0x1728]
    pub ftl_button: FTLButton,
    #[test_offset = 0x18c8]
    pub space_status: SpaceStatus,
    #[test_offset = 0x1968]
    pub star_map: *mut StarMap,
    #[test_offset = 0x1970]
    pub ship_complete: *mut CompleteShip,
    #[test_offset = 0x1978]
    pub focus_windows: Vector<*mut FocusWindow>,
    #[test_offset = 0x1990]
    pub pause_text_loc: Point,
    #[test_offset = 0x1998]
    pub pause_image: *mut GL_Primitive,
    #[test_offset = 0x19a0]
    pub pause_image2: *mut GL_Primitive,
    #[test_offset = 0x19a8]
    pub pause_image_auto: *mut GL_Primitive,
    #[test_offset = 0x19b0]
    pub pause_crew_image: *mut GL_Primitive,
    #[test_offset = 0x19b8]
    pub pause_doors_image: *mut GL_Primitive,
    #[test_offset = 0x19c0]
    pub pause_hacking_image: *mut GL_Primitive,
    #[test_offset = 0x19c8]
    pub pause_mind_image: *mut GL_Primitive,
    #[test_offset = 0x19d0]
    pub pause_room_image: *mut GL_Primitive,
    #[test_offset = 0x19d8]
    pub pause_target_image: *mut GL_Primitive,
    #[test_offset = 0x19e0]
    pub pause_target_beam_image: *mut GL_Primitive,
    #[test_offset = 0x19e8]
    pub pause_teleport_leave_image: *mut GL_Primitive,
    #[test_offset = 0x19f0]
    pub pause_teleport_arrive_image: *mut GL_Primitive,
    #[test_offset = 0x19f8]
    pub flare_image: *mut GL_Primitive,
    #[test_offset = 0x1a00]
    pub ship_position: Point,
    #[test_offset = 0x1a08]
    pub location_text: StdString,
    #[test_offset = 0x1a10]
    pub load_event: StdString,
    #[test_offset = 0x1a18]
    pub load_sector: c_int,
    #[test_offset = 0x1a20]
    pub choice_box: ChoiceBox,
    #[test_offset = 0x1c38]
    pub gameover: bool,
    #[test_offset = 0x1c39]
    pub already_won: bool,
    #[test_offset = 0x1c3a]
    pub out_of_fuel: bool,
    #[test_offset = 0x1c40]
    pub menu_box: MenuScreen,
    #[test_offset = 0x20b0]
    pub game_over_screen: GameOver,
    #[test_offset = 0x2190]
    pub options_box: OptionsScreen,
    #[test_offset = 0x3290]
    pub b_paused: bool,
    #[test_offset = 0x3291]
    pub b_auto_paused: bool,
    #[test_offset = 0x3292]
    pub menu_pause: bool,
    #[test_offset = 0x3293]
    pub event_pause: bool,
    #[test_offset = 0x3294]
    pub touch_pause: bool,
    #[test_offset = 0x3298]
    pub touch_pause_reason: TouchPauseReason,
    #[test_offset = 0x32a0]
    pub input_box: InputBox,
    #[test_offset = 0x3300]
    pub f_shake_timer: c_float,
    #[test_offset = 0x3308]
    pub ship_screens: TabbedWindow,
    #[test_offset = 0x3488]
    pub store_screens: TabbedWindow,
    #[test_offset = 0x3608]
    pub upgrade_screen: Upgrades,
    #[test_offset = 0x38d8]
    pub crew_screen: CrewManifest,
    #[test_offset = 0x3d08]
    pub equip_screen: Equipment,
    #[test_offset = 0x4040]
    pub new_location: *mut Location,
    #[test_offset = 0x4048]
    pub space: *mut SpaceManager,
    #[test_offset = 0x4050]
    pub upgrade_button: Button,
    #[test_offset = 0x40e0]
    pub upgrade_warning: WarningMessage,
    #[test_offset = 0x41c0]
    pub store_button: TextButton,
    #[test_offset = 0x42c0]
    pub options_button: Button,
    #[test_offset = 0x4350]
    pub pause_anim_time: c_float,
    #[test_offset = 0x4354]
    pub pause_animation: c_float,
    #[test_offset = 0x4358]
    pub store_trash: Vector<*mut Store>,
    #[test_offset = 0x4370]
    pub flicker_timer: TimerHelper,
    #[test_offset = 0x4384]
    pub show_timer: TimerHelper,
    #[test_offset = 0x4398]
    pub b_hide_ui: bool,
    #[test_offset = 0x43a0]
    pub enemy_ship: *mut CompleteShip,
    #[test_offset = 0x43a8]
    pub wait_location: bool,
    #[test_offset = 0x43a9]
    pub last_location_wait: bool,
    #[test_offset = 0x43aa]
    pub danger_location: bool,
    #[test_offset = 0x43b0]
    pub command_key: Vector<c_int>,
    #[test_offset = 0x43c8]
    pub jump_complete: bool,
    #[test_offset = 0x43cc]
    pub map_id: c_int,
    #[test_offset = 0x43d0]
    pub leave_crew_dialog: ConfirmWindow,
    #[test_offset = 0x4650]
    pub secret_sector: bool,
    #[test_offset = 0x4654]
    pub active_touch: c_int,
    #[test_offset = 0x4658]
    pub active_touch_is_button: bool,
    #[test_offset = 0x4659]
    pub active_touch_is_crew_box: bool,
    #[test_offset = 0x465a]
    pub active_touch_is_ship: bool,
    #[test_offset = 0x465b]
    pub active_touch_is_null: bool,
    #[test_offset = 0x4660]
    pub extra_touches: Vector<c_int>,
    #[test_offset = 0x4678]
    pub b_tutorial_was_running: bool,
    #[test_offset = 0x4679]
    pub focus_ate_mouse: bool,
    #[test_offset = 0x467a]
    pub choice_box_open: bool,
    #[test_offset = 0x467c]
    pub system_details_width: c_int,
    #[test_offset = 0x4680]
    pub write_error_dialog: ChoiceBox,
    #[test_offset = 0x4898]
    pub suppress_write_error: bool,
}

impl CommandGui {
    pub unsafe fn ship_manager(&self) -> &ShipManager {
        &*self.crew_control.ship_manager
    }
    pub unsafe fn ship_manager_mut(&mut self) -> &mut ShipManager {
        &mut *self.crew_control.ship_manager
    }
    pub unsafe fn target_self_with_mind_control_error(
        &self,
        room_id: usize,
    ) -> Option<&'static str> {
        if !self.ship_manager().has_system(System::Mind) {
            Some("the mind control system is not installed")
        } else if !self.ship_manager().ship.get_room_blackout(room_id)
            || self.ship_manager().has_crew("slug")
            || self.equip_screen.has_augment("LIFE_SCANNER")
        {
            None
        } else {
            // Cannot mind control in rooms you cannot detect life in.
            Some("the sensors don't detect life in the target room")
        }
    }
    pub unsafe fn mind_control_blocked(&self) -> bool {
        if !self.ship_manager().has_system(System::Mind) {
            return false;
        }
        let mind = self.ship_manager().mind_system();
        if !mind.b_blocked {
            return false;
        }
        if self.equip_screen.has_augment("ZOLTAN_BYPASS") {
            return false;
        }
        // Cannot mind control through Super Shields.
        true
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Pointf {
    pub x: f32,
    pub y: f32,
}

#[repr(u32)]
pub enum TouchAction {
    Down = 0x1,
    Move = 0x2,
    Up = 0x3,
    Cancel = 0x4,
}

#[vtable]
pub struct VtableFocusWindow {
    pub dtor: Option<fn(*mut FocusWindow)>,
    pub delete_dtor: Option<fn(*mut FocusWindow)>,
    pub set_open: Option<fn(*mut FocusWindow, bool)>,
    pub open: Option<fn(*mut FocusWindow)>,
    pub close: Option<fn(*mut FocusWindow)>,
    pub set_position: Option<fn(*mut FocusWindow, Point)>,
    pub on_loop: Option<fn(*mut FocusWindow)>,
    pub lock_window: Option<fn(*mut FocusWindow) -> bool>,
    pub on_render: Option<fn(*mut FocusWindow)>,
    pub mouse_move: Option<fn(*mut FocusWindow, c_int, c_int)>,
    pub mouse_click: Option<fn(*mut FocusWindow, c_int, c_int)>,
    pub mouse_up: Option<fn(*mut FocusWindow, c_int, c_int)>,
    pub mouse_right_click: Option<fn(*mut FocusWindow, c_int, c_int)>,
    pub on_touch: Option<fn(*mut FocusWindow, TouchAction, c_int, c_int, c_int, c_int, c_int)>,
    pub key_down: Option<fn(*mut FocusWindow, SDLKey) -> bool>,
    pub key_up: Option<fn(*mut FocusWindow, SDLKey) -> bool>,
    pub priority_popup: Option<fn(*mut FocusWindow) -> bool>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct FocusWindow {
    #[test_offset = 0x0]
    pub vtable: *const VtableFocusWindow,
    #[test_offset = 0x8]
    pub b_open: bool,
    #[test_offset = 0x9]
    pub b_full_focus: bool,
    #[test_offset = 0xc]
    pub close: Point,
    #[test_offset = 0x14]
    pub b_close_button_selected: bool,
    #[test_offset = 0x18]
    pub position: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Rect {
    #[test_offset = 0x0]
    pub x: c_int,
    #[test_offset = 0x4]
    pub y: c_int,
    #[test_offset = 0x8]
    pub w: c_int,
    #[test_offset = 0xc]
    pub h: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ChoiceText {
    #[test_offset = 0x0]
    /// type 0:   text #ffffff
    /// type 1:   text #969696
    /// selected: text #f3ff50
    /// type 2:   text #00c3ff
    pub type_: c_int,
    #[test_offset = 0x8]
    pub text: StdString,
    #[test_offset = 0x10]
    pub rewards: ResourceEvent,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WindowFrame {
    #[test_offset = 0x0]
    pub rect: Rect,
    #[test_offset = 0x10]
    pub outline: *mut GL_Primitive,
    #[test_offset = 0x18]
    pub mask: *mut GL_Primitive,
    #[test_offset = 0x20]
    pub pattern: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ChoiceBox {
    pub base: FocusWindow,
    #[test_offset = 0x20]
    pub text_box: *mut GL_Texture,
    #[test_offset = 0x28]
    pub box_: *mut WindowFrame,
    #[test_offset = 0x30]
    pub main_text: StdString,
    #[test_offset = 0x38]
    pub choices: Vector<ChoiceText>,
    #[test_offset = 0x50]
    pub column_size: c_uint,
    #[test_offset = 0x58]
    pub choice_boxes: Vector<Rect>,
    #[test_offset = 0x70]
    pub potential_choice: c_int,
    #[test_offset = 0x74]
    pub selected_choice: c_int,
    #[test_offset = 0x78]
    pub font_size: c_int,
    #[test_offset = 0x7c]
    pub centered: bool,
    #[test_offset = 0x80]
    pub gap_size: c_int,
    #[test_offset = 0x84]
    pub open_time: c_float,
    #[test_offset = 0x88]
    pub rewards: ResourceEvent,
    #[test_offset = 0x200]
    pub current_text_color: GL_Color,
    #[test_offset = 0x210]
    pub last_choice: Pointf,
}

#[vtable]
pub struct VtableBlueprint {
    pub dtor: Option<fn(*mut Blueprint)>,
    pub delete_dtor: Option<fn(*mut Blueprint)>,
    pub render_icon: Option<fn(*const Blueprint, c_float)>,
    pub get_name_long: Option<fn(*const Blueprint) -> StdString>,
    pub get_name_short: Option<fn(*const Blueprint) -> StdString>,
    pub get_type: Option<fn(*const Blueprint) -> c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Description {
    #[test_offset = 0x0]
    pub title: TextString,
    #[test_offset = 0x10]
    pub short_title: TextString,
    #[test_offset = 0x20]
    pub description: TextString,
    #[test_offset = 0x30]
    pub cost: c_int,
    #[test_offset = 0x34]
    pub rarity: c_int,
    #[test_offset = 0x38]
    pub base_rarity: c_int,
    #[test_offset = 0x3c]
    pub bp: c_int,
    #[test_offset = 0x40]
    pub locked: bool,
    #[test_offset = 0x48]
    pub tooltip: TextString,
    #[test_offset = 0x58]
    pub tip: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Blueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[test_offset = 0x8]
    pub name: StdString,
    /// Inherited from Blueprint
    #[test_offset = 0x10]
    pub desc: Description,
    /// Inherited from Blueprint
    #[test_offset = 0x70]
    pub type_: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TextString {
    #[test_offset = 0x0]
    pub data: StdString,
    #[test_offset = 0x8]
    pub is_literal: bool,
}

impl TextString {
    pub fn to_str(&self) -> Cow<'_, str> {
        if self.is_literal {
            self.data.to_str()
        } else {
            let key = self.data.to_str();
            super::library().text(&key).map(From::from).unwrap_or(key)
        }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct MiniProjectile {
    #[test_offset = 0x0]
    pub image: StdString,
    #[test_offset = 0x8]
    pub fake: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BoostPower {
    #[test_offset = 0x0]
    pub type_: c_int,
    #[test_offset = 0x4]
    pub amount: c_float,
    #[test_offset = 0x8]
    pub count: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Damage {
    #[test_offset = 0x0]
    pub i_damage: c_int,
    #[test_offset = 0x4]
    pub i_shield_piercing: c_int,
    #[test_offset = 0x8]
    pub fire_chance: c_int,
    #[test_offset = 0xc]
    pub breach_chance: c_int,
    #[test_offset = 0x10]
    pub stun_chance: c_int,
    #[test_offset = 0x14]
    pub i_ion_damage: c_int,
    #[test_offset = 0x18]
    pub i_system_damage: c_int,
    #[test_offset = 0x1c]
    pub i_pers_damage: c_int,
    #[test_offset = 0x20]
    pub b_hull_buster: bool,
    #[test_offset = 0x24]
    pub owner_id: c_int,
    #[test_offset = 0x28]
    pub self_id: c_int,
    #[test_offset = 0x2c]
    pub b_lockdown: bool,
    #[test_offset = 0x2d]
    pub crystal_shard: bool,
    #[test_offset = 0x2e]
    pub b_friendly_fire: bool,
    #[test_offset = 0x30]
    pub i_stun: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct EffectsBlueprint {
    #[test_offset = 0x0]
    pub launch_sounds: Vector<StdString>,
    #[test_offset = 0x18]
    pub hit_ship_sounds: Vector<StdString>,
    #[test_offset = 0x30]
    pub hit_shield_sounds: Vector<StdString>,
    #[test_offset = 0x48]
    pub miss_sounds: Vector<StdString>,
    #[test_offset = 0x60]
    pub image: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[test_offset = 0x8]
    pub name: StdString,
    /// Inherited from Blueprint
    #[test_offset = 0x10]
    pub desc: Description,
    /// Inherited from Blueprint
    #[test_offset = 0x70]
    pub type_: c_int,
    #[test_offset = 0x78]
    pub type_name: StdString,
    #[test_offset = 0x80]
    pub damage: Damage,
    #[test_offset = 0xb4]
    pub shots: c_int,
    #[test_offset = 0xb8]
    pub missiles: c_int,
    #[test_offset = 0xbc]
    pub cooldown: c_float,
    #[test_offset = 0xc0]
    pub power: c_int,
    #[test_offset = 0xc4]
    pub length: c_int,
    #[test_offset = 0xc8]
    pub speed: c_float,
    #[test_offset = 0xcc]
    pub mini_count: c_int,
    #[test_offset = 0xd0]
    pub effects: EffectsBlueprint,
    #[test_offset = 0x138]
    pub weapon_art: StdString,
    #[test_offset = 0x140]
    pub combat_icon: StdString,
    #[test_offset = 0x148]
    pub explosion: StdString,
    #[test_offset = 0x150]
    pub radius: c_int,
    #[test_offset = 0x158]
    pub mini_projectiles: Vector<MiniProjectile>,
    #[test_offset = 0x170]
    pub boost_power: BoostPower,
    #[test_offset = 0x17c]
    pub drone_targetable: c_int,
    #[test_offset = 0x180]
    pub spin: c_int,
    #[test_offset = 0x184]
    pub charge_levels: c_int,
    #[test_offset = 0x188]
    pub flavor_type: TextString,
    #[test_offset = 0x198]
    pub color: GL_Color,
}

impl WeaponBlueprint {
    pub fn can_target_self(&self) -> bool {
        self.type_ == crate::xml::WeaponType::Bomb as i32
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[test_offset = 0x8]
    pub name: StdString,
    /// Inherited from Blueprint
    #[test_offset = 0x10]
    pub desc: Description,
    /// Inherited from Blueprint
    #[test_offset = 0x70]
    pub type_: c_int,
    #[test_offset = 0x78]
    pub type_name: StdString,
    #[test_offset = 0x80]
    pub level: c_int,
    #[test_offset = 0x84]
    pub target_type: c_int,
    #[test_offset = 0x88]
    pub power: c_int,
    #[test_offset = 0x8c]
    pub cooldown: c_float,
    #[test_offset = 0x90]
    pub speed: c_int,
    #[test_offset = 0x94]
    pub dodge: c_int,
    #[test_offset = 0x98]
    pub weapon_blueprint: StdString,
    #[test_offset = 0xa0]
    pub drone_image: StdString,
    #[test_offset = 0xa8]
    pub combat_icon: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AugmentBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[test_offset = 0x8]
    pub name: StdString,
    /// Inherited from Blueprint
    #[test_offset = 0x10]
    pub desc: Description,
    /// Inherited from Blueprint
    #[test_offset = 0x70]
    pub type_: c_int,
    #[test_offset = 0x74]
    pub value: c_float,
    #[test_offset = 0x78]
    pub stacking: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Pair<A, B> {
    pub first: A,
    pub second: B,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[test_offset = 0x8]
    pub name: StdString,
    /// Inherited from Blueprint
    #[test_offset = 0x10]
    pub desc: Description,
    /// Inherited from Blueprint
    #[test_offset = 0x70]
    pub type_: c_int,
    #[test_offset = 0x78]
    pub crew_name: TextString,
    #[test_offset = 0x88]
    pub crew_name_long: TextString,
    #[test_offset = 0x98]
    pub powers: Vector<TextString>,
    #[test_offset = 0xb0]
    pub male: bool,
    #[test_offset = 0xb8]
    pub skill_level: Vector<Pair<c_int, c_int>>,
    #[test_offset = 0xd0]
    pub color_layers: Vector<Vector<GL_Color>>,
    #[test_offset = 0xe8]
    pub color_choices: Vector<c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct HackingDrone {
    #[test_offset = 0x0]
    pub base: SpaceDrone,
    #[test_offset = 0x340]
    pub starting_position: Pointf,
    #[test_offset = 0x348]
    pub drone_image_on: *mut GL_Texture,
    #[test_offset = 0x350]
    pub drone_image_off: *mut GL_Texture,
    #[test_offset = 0x358]
    pub light_image: *mut GL_Texture,
    #[test_offset = 0x360]
    pub final_destination: Pointf,
    #[test_offset = 0x368]
    pub arrived: bool,
    #[test_offset = 0x369]
    pub finished_setup: bool,
    #[test_offset = 0x370]
    pub flash_tracker: AnimationTracker,
    #[test_offset = 0x390]
    pub flying: Animation,
    #[test_offset = 0x450]
    pub extending: Animation,
    #[test_offset = 0x510]
    pub explosion: Animation,
    #[test_offset = 0x5d0]
    pub pref_room: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ResourceEvent {
    #[test_offset = 0x0]
    pub missiles: c_int,
    #[test_offset = 0x4]
    pub fuel: c_int,
    #[test_offset = 0x8]
    pub drones: c_int,
    #[test_offset = 0xc]
    pub scrap: c_int,
    #[test_offset = 0x10]
    pub crew: c_int,
    #[test_offset = 0x14]
    pub traitor: bool,
    #[test_offset = 0x15]
    pub cloneable: bool,
    #[test_offset = 0x18]
    pub clone_text: TextString,
    #[test_offset = 0x28]
    pub crew_type: StdString,
    #[test_offset = 0x30]
    pub weapon: *const WeaponBlueprint,
    #[test_offset = 0x38]
    pub drone: *const DroneBlueprint,
    #[test_offset = 0x40]
    pub augment: *const AugmentBlueprint,
    #[test_offset = 0x48]
    pub crew_blue: CrewBlueprint,
    #[test_offset = 0x148]
    pub system_id: c_int,
    #[test_offset = 0x14c]
    pub weapon_count: c_int,
    #[test_offset = 0x150]
    pub drone_count: c_int,
    #[test_offset = 0x154]
    pub steal: bool,
    #[test_offset = 0x155]
    pub intruders: bool,
    #[test_offset = 0x158]
    pub fleet_delay: c_int,
    #[test_offset = 0x15c]
    pub hull_damage: c_int,
    #[test_offset = 0x160]
    pub upgrade_amount: c_int,
    #[test_offset = 0x164]
    pub upgrade_id: c_int,
    #[test_offset = 0x168]
    pub upgrade_success_flag: c_int,
    #[test_offset = 0x170]
    pub remove_item: StdString,
}

#[cfg(test)]
mod test {
    use crate::bindings::{VtableRepairable, VtableSelectable};
    use std::mem;

    #[test]
    fn test() {
        let u = mem::size_of::<usize>();
        assert_eq!(mem::offset_of!(VtableSelectable, dtor), 0);
        assert_eq!(mem::offset_of!(VtableSelectable, delete_dtor), u);
        assert_eq!(mem::offset_of!(VtableSelectable, set_selected), u * 2);
        assert_eq!(mem::offset_of!(VtableSelectable, get_selected), u * 3);
        assert_eq!(
            mem::offset_of!(VtableRepairable, completely_destroyed),
            u * 4
        );
        assert_eq!(mem::offset_of!(VtableRepairable, get_name), u * 5);
        assert_eq!(mem::offset_of!(VtableRepairable, set_name), u * 6);
        assert_eq!(mem::offset_of!(VtableRepairable, repair), u * 7);
        assert_eq!(mem::offset_of!(VtableRepairable, partial_repair), u * 8);
        assert_eq!(mem::offset_of!(VtableRepairable, partial_damage), u * 9);
        assert_eq!(mem::offset_of!(VtableRepairable, needs_repairing), u * 10);
        assert_eq!(mem::offset_of!(VtableRepairable, functioning), u * 11);
        assert_eq!(mem::offset_of!(VtableRepairable, can_be_sabotaged), u * 12);
        assert_eq!(mem::offset_of!(VtableRepairable, get_damage), u * 13);
        assert_eq!(mem::offset_of!(VtableRepairable, get_location), u * 14);
        assert_eq!(mem::offset_of!(VtableRepairable, get_grid_location), u * 15);
        assert_eq!(mem::offset_of!(VtableRepairable, set_damage), u * 16);
        assert_eq!(mem::offset_of!(VtableRepairable, set_max_damage), u * 17);
        assert_eq!(mem::offset_of!(VtableRepairable, set_location), u * 18);
        assert_eq!(
            mem::offset_of!(VtableRepairable, on_render_highlight),
            u * 19
        );
        assert_eq!(mem::offset_of!(VtableRepairable, get_id), u * 20);
        assert_eq!(mem::offset_of!(VtableRepairable, is_room_based), u * 21);
        assert_eq!(mem::offset_of!(VtableRepairable, get_room_id), u * 22);
        assert_eq!(mem::offset_of!(VtableRepairable, ioned), u * 23);
        assert_eq!(mem::offset_of!(VtableRepairable, set_room_id), u * 24);
    }
}
