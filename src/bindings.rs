#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::HashMap,
    ffi::{c_double, c_float, c_int, c_uint},
    fmt,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut, Range},
    ptr,
    sync::atomic::AtomicI32,
};

use neuro_ftl_derive::{vtable, TestOffsets};

use crate::game::actions::Direction;

pub unsafe fn xb<'a, T>(x: *const T) -> Option<&'a T> {
    (!x.is_null()).then(|| &*x)
}
pub unsafe fn xc<'a, T>(x: *mut T) -> Option<&'a T> {
    (!x.is_null()).then(|| &*x)
}
pub unsafe fn xm<'a, T>(x: *mut T) -> Option<&'a mut T> {
    (!x.is_null()).then(|| &mut *x)
}

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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub device: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub index: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub y: c_float,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct KeyboardInputEvent {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub key: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub system_key: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub modifiers: c_uint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub is_repeat: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct MemoryInputEvent {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub used_bytes: i64,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub free_bytes: i64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct MouseInputEvent {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub scroll: c_float,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct TextInputEvent {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub ch: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, TestOffsets)]
pub struct TouchInputEvent {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub id: c_uint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub initial_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub type_: InputEventType,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub detail: InputEventDetail,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub timestamp: c_double,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
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
    pub vtable: *const VtableCEvent,
}

impl CEvent {
    pub fn vtable(&self) -> &'static VtableCEvent {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct UnlockArrow {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub direction: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub status: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub shape: Rect,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipButton {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub i_ship_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub b_ship_locked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x99)]
    pub b_layout_locked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9a)]
    pub b_no_exist: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub achievements: Vector<*mut CAchievement>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub i_selected_ach: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbc)]
    pub b_selected: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipSelect {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub title_pos: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub ship_list_base: Vector<*mut GL_Primitive>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub ship_buttons: Vector<*mut ShipButton>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub arrows: Vector<UnlockArrow>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub b_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5c)]
    pub selected_ship: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub info_box: InfoBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub current_type: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub type_a: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub type_b: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x340)]
    pub type_c: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x440)]
    pub confirm: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x540)]
    pub b_confirmed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x544)]
    pub active_touch: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x548)]
    pub tutorial: ChoiceBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x760)]
    pub tutorial_page: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemCustomBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: SystemBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x270)]
    pub button: Button,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewCustomizeBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: CrewEquipBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x338)]
    pub customize_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x438)]
    pub b_customizing: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43c)]
    pub customize_location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x448)]
    pub accept_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x548)]
    pub big_rename_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x648)]
    pub left_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6d8)]
    pub right_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x768)]
    pub b_renaming: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x769)]
    pub have_customize_touch: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x76a)]
    pub customize_activated: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x770)]
    pub box_: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x778)]
    pub box_on: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x780)]
    pub big_box: *mut GL_Texture,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipBuilder {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub current_ship: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub name_box_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub enable_advanced_primitive: *mut GL_Primitive,
    #[cfg_attr(target_os = "windows", test_offset = 0xC)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub reset_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0x78)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub clear_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0xE4)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub start_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0x1D4)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x238)]
    pub back_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0x2C4)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x338)]
    pub rename_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0x3B4)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x438)]
    pub left_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0x420)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c8)]
    pub right_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0x48C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x558)]
    pub list_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0x57C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x658)]
    pub show_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0x66C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x758)]
    pub easy_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0x75C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x858)]
    pub normal_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0x84C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x958)]
    pub hard_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0x93C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa58)]
    pub type_a: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0xA2C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb58)]
    pub type_b: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0xB1C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc58)]
    pub type_c: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd58)]
    pub type_a_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd60)]
    pub type_b_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd68)]
    pub type_c_loc: Point,
    #[cfg_attr(target_os = "windows", test_offset = 0xC24)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd70)]
    pub random_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0xD14)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe70)]
    pub advanced_off_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0xE04)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf70)]
    pub advanced_on_button: TextButton,
    #[cfg_attr(target_os = "windows", test_offset = 0xEF4)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1070)]
    pub buttons: Vector<*mut GenericButton>,
    #[cfg_attr(target_os = "windows", test_offset = 0xF00)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1088)]
    pub animations: Vector<Animation>,
    #[cfg_attr(target_os = "windows", test_offset = 0xF0C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10a0)]
    pub v_crew_boxes: Vector<*mut CrewCustomizeBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10b8)]
    pub b_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10c0)]
    pub base_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10c8)]
    pub ship_select_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10d0)]
    pub ship_ach_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10d8)]
    pub ship_equip_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10e0)]
    pub start_button_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10e8)]
    pub advanced_button_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10f0)]
    pub type_a_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10f4)]
    pub type_b_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10f8)]
    pub type_c_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10fc)]
    pub ship_ach_padding: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1100)]
    pub advanced_title_offset: c_int,
    #[cfg_attr(target_os = "windows", test_offset = 0xF48)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1108)]
    pub v_equipment_boxes: Vector<*mut EquipmentBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1120)]
    pub info_box: InfoBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x11f8)]
    pub sys_boxes: Vector<*mut SystemCustomBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1210)]
    pub shopping_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1214)]
    pub current_slot: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1218)]
    pub current_box: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x121c)]
    pub b_done: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1220)]
    pub ships: [[*const ShipBlueprint; 10]; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1310)]
    pub current_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1314)]
    pub store_ids: [c_int; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1324)]
    pub b_renaming: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1328)]
    pub current_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1330)]
    pub b_show_rooms: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1331)]
    pub b_customizing_crew: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1338)]
    pub walking_man: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x13f8)]
    pub walking_man_pos: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1400)]
    pub ship_select: ShipSelect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b68)]
    pub intro_screen: ChoiceBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d80)]
    pub b_showed_intro: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d84)]
    pub current_type: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d88)]
    pub name_input: TextInput,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1de8)]
    pub active_touch: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1dec)]
    pub active_touch_is_ship: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1ded)]
    pub ship_drag_active: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1dee)]
    pub ship_drag_vertical: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1df0)]
    pub ship_drag_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1df8)]
    pub ship_achievements: Vector<ShipAchievementInfo>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e10)]
    pub selected_ach: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e18)]
    pub arrow: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e20)]
    #[cfg_attr(target_os = "windows", test_offset = 0x1C80)]
    pub desc_box: *mut WindowFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e28)]
    pub tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e48)]
    pub encourage_ship_list: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct MainMenu {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub b_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub active_touch: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub background: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub glowy: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub glow_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    #[cfg_attr(target_os = "windows", test_offset = 0x2C)]
    pub continue_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0x98)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub start_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0x104)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub help_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0x170)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e8)]
    pub stat_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0x1DC)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x278)]
    pub options_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0x248)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x308)]
    pub credits_button: Button,
    #[cfg_attr(target_os = "windows", test_offset = 0x2B4)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x398)]
    pub quit_button: Button,
    #[cfg(target_os = "windows")]
    pub itb_button_active: bool,
    #[cfg(target_os = "windows")]
    #[test_offset = 0x324]
    pub itb_button: Button,
    #[cfg(target_os = "windows")]
    pub itb_anim: *mut Animation,
    #[cfg_attr(target_os = "windows", test_offset = 0x394)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x428)]
    pub buttons: Vector<*mut Button>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x440)]
    pub final_choice: c_int,
    #[cfg_attr(target_os = "windows", test_offset = 0x3A4)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x448)]
    pub ship_builder: ShipBuilder,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2298)]
    pub b_score_screen: bool,
    // TODO:
    #[cfg_attr(target_os = "windows", test_offset = 0x204C)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x22a0)]
    pub option_screen: OptionsScreen,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x33a0)]
    pub b_select_save: bool,
    #[cfg_attr(target_os = "windows", test_offset = 0x3054)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x33a8)]
    pub confirm_new_game: ConfirmWindow,
    #[cfg_attr(target_os = "windows", test_offset = 0x32C0)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3628)]
    pub changelog: ChoiceBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3840)]
    pub b_credit_screen: bool,
    #[cfg_attr(target_os = "windows", test_offset = 0x3508)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3848)]
    pub credits: CreditScreen,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38a0)]
    pub b_changed_login: bool,
    #[cfg(target_os = "windows")]
    pub _unk1: c_int,
    #[cfg(target_os = "windows")]
    pub _unk2: c_int,
    #[cfg_attr(target_os = "windows", test_offset = 0x3574)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38a8)]
    pub test_crew: Vector<*mut CrewMember>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38c0)]
    pub b_changed_screen: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38c1)]
    pub b_sync_screen: bool,
    #[cfg_attr(target_os = "windows", test_offset = 0x3584)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38c8)]
    pub error: StdString,
    #[cfg(target_os = "windows")]
    pub _unk3: c_char,
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub player_ship: *mut CompleteShip,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub boss_ship: *mut BossShip,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub space: SpaceManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c8)]
    pub current_difficulty: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4d0)]
    pub ships: Vector<*mut CompleteShip>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4e8)]
    pub star_map: StarMap,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x11b8)]
    pub command_gui: *mut CommandGui,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x11c0)]
    pub base_location_event: *mut LocationEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x11c8)]
    pub last_location_event: *mut LocationEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x11d0)]
    pub current_ship_event: ShipEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1500)]
    pub current_effects: Vector<StatusEffect>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1518)]
    pub starting_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1520)]
    pub new_location: *mut Location,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1528)]
    pub b_started_game: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1529)]
    pub b_loading_game: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x152a)]
    pub v_auto_saved: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x152b)]
    pub b_extra_choice: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1530)]
    pub choice_history: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1548)]
    pub generated_event: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1550)]
    pub last_main_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1560)]
    pub player_crew_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1564)]
    pub killed_crew: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1568)]
    pub player_hull: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1570)]
    pub blue_race_choices: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1588)]
    pub last_selected_crew_seed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158c)]
    pub testing_blueprints: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1590)]
    pub original_choice_list: Vector<Choice>,
}

impl WorldManager {
    pub fn base_location_event(&self) -> Option<&LocationEvent> {
        unsafe { xc(self.base_location_event) }
    }
    pub fn base_location_event_mut(&mut self) -> Option<&mut LocationEvent> {
        unsafe { xm(self.base_location_event) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CApp {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: CEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub running: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9)]
    pub shift_held: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub gui: *mut CommandGui,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub world: *mut WorldManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    #[cfg_attr(target_os = "windows", test_offset = 0x10)]
    pub menu: MainMenu,
    #[cfg_attr(target_os = "windows", test_offset = 0x35B0)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38f0)]
    pub lang_chooser: LanguageChooser,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3930)]
    pub screen_x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3934)]
    pub screen_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3938)]
    pub modifier_x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x393c)]
    pub modifier_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3940)]
    pub full_screen_last_state: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3941)]
    pub minimized: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3942)]
    pub min_last_state: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3943)]
    pub focus: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3944)]
    pub focus_last_state: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3945)]
    pub steam_overlay: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3946)]
    pub steam_overlay_last_state: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3947)]
    pub rendering: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3948)]
    pub game_logic: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x394c)]
    pub mouse_modifier_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3950)]
    pub mouse_modifier_y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3958)]
    pub framebuffer: *mut GL_FrameBuffer,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3960)]
    pub fbo_support: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3964)]
    pub x_bar: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3968)]
    pub y_bar: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x396c)]
    pub l_ctrl: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x396d)]
    pub use_frame_buffer: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x396e)]
    pub manual_resolution_error: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3970)]
    pub manual_res_error_x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3974)]
    pub manual_res_error_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3978)]
    pub native_full_screen_error: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3979)]
    pub fb_stretch_error: bool,
    #[cfg_attr(target_os = "windows", test_offset = 0x3620)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3980)]
    pub last_language: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3988)]
    pub input_focus: bool,
    #[cfg(target_os = "windows")]
    pub use_direct3d: bool,
}

impl CApp {
    pub fn world(&self) -> Option<&WorldManager> {
        unsafe { xc(self.world) }
    }
    pub fn world_mut(&mut self) -> Option<&mut WorldManager> {
        unsafe { xm(self.world) }
    }
    pub fn gui(&self) -> Option<&CommandGui> {
        unsafe { xc(self.gui) }
    }
    pub fn gui_mut(&mut self) -> Option<&mut CommandGui> {
        unsafe { xm(self.gui) }
    }
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
#[derive(Debug)]
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
#[derive(Debug, TestOffsets)]
#[cfg(target_os = "linux")]
pub struct StdStringRep {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub length: usize,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub capacity: usize,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub refc: AtomicI32,
}

#[repr(C)]
#[derive(Debug)]
#[cfg(target_os = "linux")]
pub struct StdString {
    pub data: *const StdStringRep,
}

#[repr(C)]
#[derive(Debug)]
#[cfg(target_os = "windows")]
pub struct StdString {
    // if it equals &res, stack allocation
    // if it's anything else, heap allocation
    pub data: *const c_char,
    pub size: usize,
    pub res: usize,
    pub extra: [u8; 12],
}

impl StdString {
    #[cfg(target_os = "linux")]
    fn rep(&self) -> &StdStringRep {
        unsafe { &*self.data.sub(1) }
    }
    #[cfg(target_os = "linux")]
    pub fn to_str(&self) -> Cow<'_, str> {
        let rep = self.rep();
        // log::trace!("data: {:?}", self.data);
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(self.data.cast(), rep.length) })
    }
    #[cfg(target_os = "windows")]
    pub fn to_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(self.data.cast(), self.size) })
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub loop_: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub current_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub running: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x15)]
    pub reverse: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x16)]
    pub done: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub loop_delay: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub current_delay: c_float,
}

impl AnimationTracker {
    pub fn vtable(&self) -> &'static VtableAnimationTracker {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub hitbox: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub allow_any_touch: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x21)]
    pub touch_selectable: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x22)]
    pub b_render_off: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x23)]
    pub b_render_selected: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub b_flashing: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub flashing: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub b_active: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x49)]
    pub b_hover: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4a)]
    pub b_activated: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4b)]
    pub b_selected: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c)]
    pub active_touch: c_int,
}

impl GenericButton {
    pub fn vtable(&self) -> &'static VtableGenericButton {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TextButton {
    pub base: GenericButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    #[cfg_attr(target_os = "windows", test_offset = 0x48)]
    pub primitives: [*mut GL_Primitive; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub base_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub base_image_offset: Point,
    #[cfg_attr(target_os = "windows", test_offset = 0x60)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub base_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub colors_set: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x84)]
    pub colors: [GL_Color; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub text_color: GL_Color,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc4)]
    pub button_size: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xcc)]
    pub corner_inset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub auto_width: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd4)]
    pub auto_width_margin: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub auto_width_min: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xdc)]
    pub auto_right_align: bool,
    #[cfg_attr(target_os = "windows", test_offset = 0xC4)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe0)]
    pub label: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub font: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf4)]
    pub line_height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf8)]
    pub text_y_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xfc)]
    pub auto_shrink: bool,
}

/// TextButton without the end for better alignment
#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TextButtonPrime {
    pub base: GenericButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub primitives: [*mut GL_Primitive; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub base_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub base_image_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub base_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub colors_set: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x84)]
    pub colors: [GL_Color; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub text_color: GL_Color,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc4)]
    pub button_size: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xcc)]
    pub corner_inset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub auto_width: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd4)]
    pub auto_width_margin: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub auto_width_min: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xdc)]
    pub auto_right_align: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe0)]
    pub label: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub font: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf4)]
    pub line_height: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ConfirmWindow {
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub text_height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub min_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub window_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub yes_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub no_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub auto_center: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub window_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub window: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub yes_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x178)]
    pub no_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x278)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub system_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub allotment: Pair<c_int, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub sub_indices: Vector<c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CombatAI {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub target: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub weapons: Vector<*mut ProjectileFactory>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub drones: Vector<*mut SpaceDrone>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub stance: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub system_targets: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub b_firing_while_cloaked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub self_: *mut ShipManager,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewAI {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub ship: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub b_a_ion: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9)]
    pub b_airlock_requested: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa)]
    pub b_medbay_requested: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb)]
    pub b_hurt_crew: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub b_calm_ship: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub crew_list: Vector<*mut CrewMember>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub intruder_list: Vector<*mut CrewMember>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub hull_breaches: Vector<*mut Repairable>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub desired_task_list: Vector<CrewTask>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub bonus_task_list: Vector<CrewTask>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub breached_rooms: VectorBool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub i_teleport_request: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub b_urgent_teleport: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub starting_crew_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbc)]
    pub b_multiracial_crew: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbd)]
    pub b_override_race: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipAI {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub ship: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub target: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub crew_ai: CrewAI,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub combat_ai: CombatAI,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub player_ship: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x139)]
    pub surrendered: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x13a)]
    pub escaping: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x13b)]
    pub destroyed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x13c)]
    pub surrender_threshold: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub escape_threshold: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x144)]
    pub escape_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub last_max_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x150)]
    pub power_profiles: Map<StdString, PowerProfile>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub boarding_profile: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x184)]
    pub i_teleport_request: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub i_teleport_target: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18c)]
    pub broken_systems: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x190)]
    pub boarding_ai: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x194)]
    pub i_crew_needed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x198)]
    pub b_stalemate_trigger: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19c)]
    pub f_stalemate_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a0)]
    pub last_health: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a4)]
    pub b_boss: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a8)]
    pub i_times_teleported: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CompleteShip {
    pub vtable: *const VtableCompleteShip,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub space_manager: *mut SpaceManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub enemy_ship: *mut CompleteShip,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub b_player_ship: bool,
    #[cfg_attr(target_os = "windows", test_offset = 24)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub ship_ai: ShipAI,
    #[cfg_attr(target_os = "windows", test_offset = 0x118)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e0)]
    pub arriving_party: Vector<*mut CrewMember>,
    #[cfg_attr(target_os = "windows", test_offset = 0x124)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f8)]
    pub leaving_party: Vector<*mut CrewMember>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
    pub tele_target_room: c_int,
}

impl CompleteShip {
    pub fn vtable(&self) -> &'static VtableCompleteShip {
        unsafe { xb(self.vtable).unwrap() }
    }
    pub fn ship_manager(&self) -> Option<&ShipManager> {
        unsafe { xc(self.ship_manager) }
    }
    pub fn ship_manager_mut(&mut self) -> Option<&mut ShipManager> {
        unsafe { xm(self.ship_manager) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TimerHelper {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub max_time: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub min_time: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub curr_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub curr_goal: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub loop_: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x11)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableStoreBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub item_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub item_box: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub button_image: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub symbol: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub desc: Description,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x110)]
    pub count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x114)]
    pub cost_position: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub shopper: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x120)]
    pub equip_screen: *mut Equipment,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x128)]
    pub p_blueprint: *const Blueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x130)]
    pub b_equipment_box: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x134)]
    pub f_icon_scale: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub push_icon: Point,
}

impl StoreBox {
    pub fn vtable(&self) -> &'static VtableStoreBox {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponStoreBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: StoreBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub blueprint: *const WeaponBlueprint,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemStoreBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: StoreBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub blueprint: *const SystemBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14c)]
    pub b_confirming: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x150)]
    pub confirm_string: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub free_blueprint: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub drone_choice: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct RepairStoreBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: StoreBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub repair_all: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x144)]
    pub repair_cost: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub button_text: TextString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ItemBlueprint {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: Blueprint,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ItemStoreBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: StoreBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub blueprint: *const ItemBlueprint,
}

impl ItemStoreBox {
    pub fn blueprint(&self) -> Option<&ItemBlueprint> {
        unsafe { xb(self.blueprint) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneStoreBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: StoreBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub blueprint: *const DroneBlueprint,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewStoreBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: StoreBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub crew_portrait: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x208)]
    pub blueprint: CrewBlueprint,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AugmentStoreBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: StoreBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub box_: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub heading_title: [TextString; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub page1: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf8)]
    pub page2: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub confirm_dialog: ConfirmWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x408)]
    pub current_button: *mut Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x410)]
    pub current_description: Description,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x470)]
    pub unavailable: StdString,
    // 6 elements
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x478)]
    pub v_store_boxes: Vector<*mut StoreBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x490)]
    pub v_item_boxes: Vector<*mut StoreBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4a8)]
    pub shopper: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4b0)]
    pub selected_weapon: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4b4)]
    pub selected_drone: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4b8)]
    pub info_box: InfoBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x590)]
    pub info_box_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x598)]
    pub exit_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x628)]
    pub world_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x62c)]
    pub section_count: c_int,
    // see StoreType
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x630)]
    pub types: [c_int; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x640)]
    pub b_show_page2: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x648)]
    pub confirm_buy: *mut StoreBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x650)]
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
    pub fn shopper(&self) -> Option<&ShipManager> {
        unsafe { xc(self.shopper) }
    }
    pub fn current_button(&self) -> Option<&Button> {
        unsafe { xc(self.current_button) }
    }
    pub fn current_button_mut(&mut self) -> Option<&mut Button> {
        unsafe { xm(self.current_button) }
    }
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
        .filter(|x| T::IGNORE_COUNT || unsafe { xc(*x).unwrap() }.count > 0)
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: GenericButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub images: [*mut GL_Texture; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub primitives: [*mut GL_Primitive; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub image_size: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub b_mirror: bool,
}

/// Button without b_mirror for alignment
#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ButtonPrime {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: GenericButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub images: [*mut GL_Texture; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub primitives: [*mut GL_Primitive; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableCachedPrimitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub primitive: *mut GL_Primitive,
}

impl CachedPrimitive {
    pub fn vtable(&self) -> &'static VtableCachedPrimitive {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CachedImage {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: CachedPrimitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub image_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub texture: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub w_scale: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c)]
    pub h_scale: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub x_start: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub y_start: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub x_size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    pub y_size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub rotation: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x44)]
    pub mirrored: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WarningMessage {
    pub vtable: *const VtableWarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub is_image: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub center_text: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c)]
    pub text_color: GL_Color,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5c)]
    pub use_warning_line: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub image: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub image_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub flash: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub sound: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub flash_tracker: AnimationTracker,
}

impl WarningMessage {
    pub fn vtable(&self) -> &'static VtableWarningMessage {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct FleetShip {
    // offset = 424109
    // DW_AT_decl_file = /media/sf_FTL/Project/src/Gameplay/SpaceManager.h
    // DW_AT_decl_line = 0x37
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub image: *mut GL_Texture,
    // offset = 424121
    // DW_AT_decl_file = /media/sf_FTL/Project/src/Gameplay/SpaceManager.h
    // DW_AT_decl_line = 0x38
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub location: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct NebulaCloud {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub pos: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub curr_alpha: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub curr_scale: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub delta_alpha: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub delta_scale: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub new_trigger: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub new_cloud: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d)]
    pub b_lightning: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub lightning_flash: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub flash_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x44)]
    pub lightning_rotation: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Scroller {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub image_id: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub size_x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub size_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub image_x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub image_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub f_speed: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub current_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub b_initialized: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AsteroidGenerator {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub asteroid_queue: Queue<*mut Projectile>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub spawn_rate: [RandomAmount; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x74)]
    pub state_length: [RandomAmount; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub number_of_ships: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9c)]
    pub i_state: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub current_space: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa4)]
    pub i_next_direction: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub f_state_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xac)]
    pub timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub b_running: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub init_shields: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SpaceManager {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub projectiles: Vector<*mut Projectile>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub asteroid_generator: AsteroidGenerator,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub ships: Vector<*mut ShipManager>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe8)]
    pub drones: Vector<*mut SpaceDrone>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x100)]
    pub danger_zone: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x108)]
    pub current_back: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x110)]
    pub current_planet: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x130)]
    pub planet_image: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x178)]
    pub fleet_ship: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x198)]
    pub ship_ids: [*mut GL_Texture; 8],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub fleet_ships: [FleetShip; 9],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub asteroid_scroller: [Scroller; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2e0)]
    pub sun_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2e8)]
    pub sun_glow: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2f0)]
    pub sun_glow1: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x310)]
    pub sun_glow2: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x330)]
    pub sun_glow3: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x350)]
    pub sun_level: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x351)]
    pub pulsar_level: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x358)]
    pub pulsar_front: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x360)]
    pub pulsar_back: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x368)]
    pub lowend_pulsar: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x370)]
    pub b_pds: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x374)]
    pub env_target: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x378)]
    pub ship_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x380)]
    pub random_pds_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x388)]
    pub pds_queue: Vector<*mut Projectile>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3a0)]
    pub flash_timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3b8)]
    pub flash_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3d8)]
    pub current_beacon: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3f8)]
    current_beacon_flash: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x418)]
    pub beacon_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x438)]
    pub flash_sound: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x439)]
    pub b_nebula: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43a)]
    pub b_storm: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x440)]
    pub nebula_clouds: Vector<NebulaCloud>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x458)]
    pub lowend_nebula: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x460)]
    pub lowend_storm: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x468)]
    pub lowend_sun: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x470)]
    pub lowend_asteroids: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x478)]
    pub ship_health: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x47c)]
    pub game_paused: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x480)]
    pub pds_fire_timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x494)]
    pub pds_countdown: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x498)]
    pub pds_smoke_anims: Vector<Animation>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4b0)]
    pub queue_screen_shake: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4b1)]
    pub player_ship_in_front: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ChoiceReq {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub object: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub min_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub max_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub max_group: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub blue: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Choice {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub event: *mut LocationEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub requirement: ChoiceReq,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub hidden_reward: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BoardingEvent {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub type_: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub min: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub max: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub amount: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub breach: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct StatusEffect {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    #[allow(non_snake_case)]
    pub _sil_do_not_use_system: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub amount: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub target: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct EventDamage {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    #[allow(non_snake_case)]
    pub _sil_do_not_use_system: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub amount: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub effect: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewDesc {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub type_: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub proportion: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub amount: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipEvent {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub present: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub blueprint: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub auto_blueprint: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub blueprint_list: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub actual_blueprint: ShipBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x288)]
    pub hostile: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x290)]
    pub surrender: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x298)]
    pub escape: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2a0)]
    pub destroyed: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2a8)]
    pub dead_crew: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2b0)]
    pub gotaway: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2b8)]
    pub escape_timer: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2bc)]
    pub surrender_threshold: RandomAmount,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c8)]
    pub escape_threshold: RandomAmount,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d8)]
    pub crew_override: Vector<CrewDesc>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2f0)]
    pub weapon_override: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x308)]
    pub weapon_over_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x310)]
    pub drone_override: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x328)]
    pub drone_over_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x32c)]
    pub ship_seed: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct LocationEvent {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub ship: ShipEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x340)]
    pub stuff: ResourceEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4b8)]
    pub environment: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4bc)]
    pub environment_target: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c0)]
    pub store: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c4)]
    pub fleet_position: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c8)]
    pub beacon: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c9)]
    pub reveal_map: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4ca)]
    pub distress_beacon: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4cb)]
    pub repair: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4cc)]
    pub modify_pursuit: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4d0)]
    pub p_store: *mut Store,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4d8)]
    pub damage: Vector<EventDamage>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4f0)]
    pub quest: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4f8)]
    pub status_effects: Vector<StatusEffect>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x510)]
    pub name_definitions: Vector<Pair<StdString, StdString>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x528)]
    pub space_image: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x530)]
    pub planet_image: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x538)]
    pub event_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x540)]
    pub reward: ResourceEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6b8)]
    pub boarders: BoardingEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6d0)]
    pub choices: Vector<Choice>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6e8)]
    pub unlock_ship: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6f0)]
    pub unlock_ship_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x700)]
    pub secret_sector: bool,
}

impl LocationEvent {
    pub fn store(&self) -> Option<&Store> {
        self.store.then(|| unsafe { xc(self.p_store) }).flatten()
    }
    pub fn store_mut(&mut self) -> Option<&mut Store> {
        self.store.then(|| unsafe { xm(self.p_store) }).flatten()
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Location {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub loc: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub connected_locations: Vector<*mut Location>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub beacon: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x21)]
    pub known: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub visited: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub danger_zone: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x29)]
    pub new_sector: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2a)]
    pub nebula: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2b)]
    pub boss: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub event: *mut LocationEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub planet: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub space: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub beacon_image: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub image_id: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub quest_loc: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub flash_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub fleet_changing: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub planet_image: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub space_image: StdString,
}

impl Location {
    pub fn neighbors(&self) -> HashMap<Direction, *mut Self> {
        let mut ret = HashMap::new();
        let x0 = (self.loc.x as u64) / 110;
        let y0 = (self.loc.y as u64) / 110;
        for l in self.connected_locations.iter().copied() {
            let loc = unsafe { xc(l).unwrap() };
            let x1 = (loc.loc.x as u64) / 110;
            let y1 = (loc.loc.y as u64) / 110;
            let dir = match (x1.cmp(&x0), y1.cmp(&y0)) {
                (Ordering::Less, Ordering::Less) => Direction::TopLeft,
                (Ordering::Less, Ordering::Equal) => Direction::Left,
                (Ordering::Less, Ordering::Greater) => Direction::BottomLeft,
                (Ordering::Equal, Ordering::Less) => Direction::Top,
                (Ordering::Equal, Ordering::Equal) => {
                    log::error!("graph loop found, this shouldn't happen");
                    continue;
                }
                (Ordering::Equal, Ordering::Greater) => Direction::Bottom,
                (Ordering::Greater, Ordering::Less) => Direction::TopRight,
                (Ordering::Greater, Ordering::Equal) => Direction::Right,
                (Ordering::Greater, Ordering::Greater) => Direction::BottomRight,
            };
            ret.insert(dir, l);
        }
        ret
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AugmentEquipBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: EquipmentBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub ship: *mut ShipManager,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponEquipBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: EquipmentBox,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneEquipBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: EquipmentBox,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Equipment {
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub box_: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub store_box: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub over_box: DropBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub over_aug_image: DropBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub sell_box: DropBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c8)]
    pub b_selling_item: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub v_equipment_boxes: Vector<*mut EquipmentBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f0)]
    pub weapons_trash_list: Vector<*mut ProjectileFactory>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x208)]
    pub overcapacity_box: *mut EquipmentBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
    pub over_aug_box: *mut AugmentEquipBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub selected_equip_box: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x21c)]
    pub dragging_equip_box: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    pub potential_dragging_box: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x224)]
    pub b_dragging: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    pub first_mouse: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x230)]
    pub current_mouse: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x238)]
    pub drag_box_center: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub drag_box_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub info_box: InfoBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x320)]
    pub sell_cost_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x328)]
    pub b_over_capacity: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x329)]
    pub b_over_aug_capacity: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x32a)]
    pub b_store_mode: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x32c)]
    pub cargo_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x330)]
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
    pub fn ship_manager(&self) -> Option<&ShipManager> {
        unsafe { xc(self.ship_manager) }
    }
    pub fn boxes<T: EquipBoxTrait>(&self) -> Vec<*mut EquipmentBox> {
        let r = T::range(&self.ship_manager().unwrap().my_blueprint);
        self.v_equipment_boxes
            .iter()
            .skip(r.start as usize)
            .take(r.len())
            .copied()
            .collect()
    }
    pub fn has_augment(&self, augment: &str) -> bool {
        for b in self.v_equipment_boxes.iter() {
            let b = unsafe { xc(*b).unwrap() };
            if b.item.augment().is_some_and(|x| x.name.to_str() == augment) {
                return true;
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub prompt: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub text: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub old_text: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub pos: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    pub last_pos: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub b_active: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x44)]
    pub allowed_chars: AllowedCharType,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub max_chars: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c)]
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
#[derive(Debug, TestOffsets)]
pub struct EquipmentBoxItem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub p_weapon: *mut ProjectileFactory,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub p_drone: *mut Drone,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub p_crew: *mut CrewMember,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub augment: *const AugmentBlueprint,
}

impl EquipmentBoxItem {
    pub unsafe fn clone(&self) -> Self {
        Self {
            p_weapon: self.p_weapon,
            p_drone: self.p_drone,
            p_crew: self.p_crew,
            augment: self.augment,
        }
    }
    pub fn weapon(&self) -> Option<&ProjectileFactory> {
        unsafe { xc(self.p_weapon) }
    }
    pub fn drone(&self) -> Option<&Drone> {
        unsafe { xc(self.p_drone) }
    }
    pub fn crew(&self) -> Option<&CrewMember> {
        unsafe { xc(self.p_crew) }
    }
    pub fn augment(&self) -> Option<&AugmentBlueprint> {
        unsafe { xb(self.augment) }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableEquipmentBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub blocked_overlay: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub overlay_color: GL_Color,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub image_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub empty: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub full: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub selected_empty: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub selected_full: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub weapon_sys: *mut WeaponSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub drone_sys: *mut DroneSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub hit_box: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub item: EquipmentBoxItem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub b_mouse_hovering: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x91)]
    pub b_glow: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x92)]
    pub b_blocked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x94)]
    pub slot: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub b_locked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9c)]
    pub value: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub b_permanent_lock: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa1)]
    pub block_detailed: bool,
}

impl EquipmentBox {
    pub fn vtable(&self) -> &'static VtableEquipmentBox {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewEquipBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: EquipmentBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub ship: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub b_dead: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub delete_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b8)]
    pub rename_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2b8)]
    pub b_show_delete: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2b9)]
    pub b_show_rename: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2ba)]
    pub b_quick_renaming: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c0)]
    pub name_input: TextInput,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x320)]
    pub box_: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x328)]
    pub box_on: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x330)]
    pub b_confirm_delete: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DropBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub is_sell_box: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub box_image: [*mut GL_Texture; 2],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub selected_image: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub title_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub body_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub body_space: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub lower_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub sell_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub sell_cost_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub text_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c)]
    pub insert_height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub title_insert: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewManifest {
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub box_: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub over_box: DropBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub crew_boxes: Vector<*mut CrewEquipBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub info_box: InfoBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a8)]
    pub confirming_delete: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b0)]
    pub delete_dialog: ConfirmWindow,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ReactorButton {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ButtonPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub b_mirror: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8c)]
    pub temp_upgrade: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub ship: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub selected: bool,
}

pub fn power_manager(ship_id: i32) -> Option<&'static PowerManager> {
    unsafe { xc(super::POWER_MANAGERS.0).unwrap() }.get(ship_id as usize)
}

impl ReactorButton {
    pub fn ship(&self) -> Option<&ShipManager> {
        unsafe { xc(self.ship) }
    }
    pub fn reactor_cost(&self) -> c_int {
        let Some(power) = power_manager(self.ship().unwrap().i_ship_id) else {
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
        let Some(power) = power_manager(self.ship().unwrap().i_ship_id) else {
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub _sil_do_not_use_system: *mut ShipSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub ship: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub blueprint: *const SystemBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub temp_upgrade: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub current_button: *mut Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub button_base_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub max_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub box_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub subsystem: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x159)]
    pub is_dummy: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub dummy_box: *mut GL_Primitive,
}

impl UpgradeBox {
    pub fn blueprint(&self) -> Option<&SystemBlueprint> {
        (!self._sil_do_not_use_system.is_null())
            .then(|| unsafe { xb(self.blueprint) })
            .flatten()
    }
    pub fn current_button(&self) -> Option<&Button> {
        unsafe { xc(self.current_button) }
    }
    pub fn current_button_mut(&mut self) -> Option<&mut Button> {
        unsafe { xm(self.current_button) }
    }
    pub fn system(&self) -> Option<&ShipSystem> {
        unsafe { xc(self._sil_do_not_use_system) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Upgrades {
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub box_: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub v_upgrade_boxes: Vector<*mut UpgradeBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub undo_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub reactor_button: ReactorButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e8)]
    pub info_box: InfoBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c0)]
    pub info_box_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c8)]
    pub system_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2cc)]
    pub force_system_info_width: c_int,
}

impl Upgrades {
    pub fn ship_manager(&self) -> Option<&ShipManager> {
        unsafe { xc(self.ship_manager) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TabbedWindow {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub buttons: Vector<*mut Button>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub windows: Vector<*mut FocusWindow>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub names: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub current_tab: c_uint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6c)]
    pub button_type: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub done_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub move_: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x178)]
    pub b_block_close: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x179)]
    pub b_tutorial_mode: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x17a)]
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
        super::SET_TAB.call(ptr::addr_of_mut!(*self), tab);
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct InputBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub text_box: *mut WindowFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub main_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub b_done: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x31)]
    pub b_invert_caps: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub input_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub last_inputs: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub last_input_index: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct LanguageChooser {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub buttons: Vector<*mut TextButton>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub i_choice: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ControlButton {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub rect: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub value: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub desc: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub key: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub state: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub desc_length: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ControlsScreen {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub buttons: [Vector<ControlButton>; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub selected_button: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub default_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x168)]
    pub reset_dialog: ConfirmWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3e8)]
    pub page_buttons: [Button; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x628)]
    pub current_page: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x630)]
    pub custom_box: *mut WindowFrame,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SlideBar {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub box_: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub hovering: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x11)]
    pub holding: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub marker: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub mouse_start: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c)]
    pub rect_start: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub min_max: Pair<c_int, c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct OptionsScreen {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ChoiceBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    pub wipe_profile_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    pub sound_volume: SlideBar,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x264)]
    pub music_volume: SlideBar,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2a0)]
    pub b_customize_controls: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2a8)]
    pub controls: ControlsScreen,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8e0)]
    pub close_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9e0)]
    pub wipe_profile_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xae0)]
    pub show_sync_achievements: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xae8)]
    pub sync_achievements_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbe8)]
    pub choice_fullscreen: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbec)]
    pub choice_vsync: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbf0)]
    pub choice_frame_limit: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbf4)]
    pub choice_lowend: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbf8)]
    pub choice_colorblind: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbfc)]
    pub choice_language: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc00)]
    pub choice_dialog_keys: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc04)]
    pub choice_show_paths: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc08)]
    pub choice_achievement_popups: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0c)]
    pub choice_auto_pause: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc10)]
    pub choice_touch_auto_pause: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc14)]
    pub choice_controls: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc18)]
    pub last_full_screen: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc1c)]
    pub is_sound_touch: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc1d)]
    pub is_music_touch: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc20)]
    pub lang_chooser: LanguageChooser,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc60)]
    pub show_wipe_button: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc68)]
    pub wipe_profile_dialog: ConfirmWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xee8)]
    pub restart_required_dialog: ChoiceBox,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CreditScreen {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub scroll: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub scroll_speed: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub ship_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub crew_string: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub pausing: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub bg: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub credit_names: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub last_valid_credit: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x44)]
    pub touches_down: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub touch_down_time: c_double,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub skip_message_timer: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct GameOver {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub buttons: Vector<*mut TextButton>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub box_: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub box_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x44)]
    pub command: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub commands: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub b_show_stats: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x64)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub gameover_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub b_victory: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c)]
    pub opened_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub credits: CreditScreen,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub blueprint: *const SystemBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub desc: Description,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub temp_upgrade: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x74)]
    pub power_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub max_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c)]
    pub system_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub system_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x84)]
    pub y_shift: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub desc_box_size: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub p_crew_blueprint: *const CrewBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub warning: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub b_detailed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub additional_tip: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub additional_warning: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub primary_box: *mut WindowFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub primary_box_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub secondary_box: *mut WindowFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub drone_blueprint: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub name: StdString,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub desc: Description,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x74)]
    pub max_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub start_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub upgrade_costs: Vector<c_int>,
}

impl SystemBlueprint {
    pub fn vtable(&self) -> &'static VtableBlueprint {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CAchievement {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub name_id: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub progress: Pair<c_int, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub unlocked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub name: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub description: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub header: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub new_achievement: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x49)]
    pub multi_difficulty: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c)]
    pub difficulty: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub ship: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub ship_difficulties: [c_int; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x64)]
    pub dimension: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub icon: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub mini_icon: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf8)]
    pub mini_icon_locked: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub lock_image: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub dot_on: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub dot_off: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub outline: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    pub mini_outline: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    pub lock_overlay: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipAchievementInfo {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub achievement: *mut CAchievement,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub dimension: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct MenuScreen {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub main_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub menu_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub menu_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub buttons: Vector<*mut TextButton>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub command: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub commands: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub confirm_dialog: ConfirmWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2f8)]
    pub temp_command: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x300)]
    pub save_quit: *mut GenericButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x308)]
    pub b_show_controls: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30c)]
    pub status_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x318)]
    pub difficulty_box: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x320)]
    pub difficulty_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x328)]
    pub difficulty_label: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x330)]
    pub difficulty_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x338)]
    pub dlc_box: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x340)]
    pub dlc_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x348)]
    pub dlc_label: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x350)]
    pub dlc_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x358)]
    pub ach_box: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x360)]
    pub ach_box_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x368)]
    pub ach_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x370)]
    pub ach_label: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x378)]
    pub ship_achievements: Vector<ShipAchievementInfo>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x390)]
    pub selected_ach: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x398)]
    pub info: InfoBox,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct NebulaInfo {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub w: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub h: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct RandomAmount {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub min: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub max: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub chance_none: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SectorDescription {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub event_counts: Vector<Pair<StdString, RandomAmount>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub rarities: Vector<Pair<StdString, c_int>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub unique: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub names: Vector<TextString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub short_names: Vector<TextString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub music_tracks: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub type_: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub name: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub short_name: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub min_sector: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xac)]
    pub used: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub first_event: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Sector {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub visited: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5)]
    pub reachable: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub neighbors: Vector<*mut Sector>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub description: SectorDescription,
}

impl Sector {
    pub fn neighbors(&self) -> HashMap<Direction, *mut Self> {
        let mut ret = HashMap::new();
        let x0 = self.location.x;
        let y0 = self.location.y;
        for s in self.neighbors.iter().copied() {
            let sec = unsafe { xc(s).unwrap() };
            let x1 = sec.location.x;
            let y1 = sec.location.y;
            let dir = match (x1.cmp(&x0), y1.cmp(&y0)) {
                (Ordering::Less, Ordering::Less) => continue, // Direction::TopLeft,
                (Ordering::Less, Ordering::Equal) => continue, // Direction::Left,
                (Ordering::Less, Ordering::Greater) => continue, // Direction::BottomLeft,
                (Ordering::Equal, Ordering::Less) => Direction::Top,
                (Ordering::Equal, Ordering::Equal) => {
                    log::error!("graph loop found, this shouldn't happen");
                    continue;
                }
                (Ordering::Equal, Ordering::Greater) => Direction::Bottom,
                (Ordering::Greater, Ordering::Less) => Direction::TopRight,
                (Ordering::Greater, Ordering::Equal) => Direction::Right,
                (Ordering::Greater, Ordering::Greater) => Direction::BottomRight,
            };
            ret.insert(dir, s);
        }
        ret
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DistressButton {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x100)]
    pub labels: [TextString; 2],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x120)]
    pub state: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct StarMap {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub visual_size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub locations: Vector<*mut Location>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub locations_grid: Map<Point, Vector<*mut Location>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub temp_path: Vector<*mut Location>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub current_loc: *mut Location,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub potential_loc: *mut Location,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub hover_loc: *mut Location,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub i_populated_tiles: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xac)]
    pub i_location_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub i_empty_tiles: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub b_initialized_display: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub translation: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub ready_to_travel: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc4)]
    pub danger_zone: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xcc)]
    pub danger_zone_radius: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub ship_rotation: [c_float; 2],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub end_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub wait_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d8)]
    pub distress_button: DistressButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x400)]
    pub jump_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x500)]
    pub world_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x504)]
    pub b_map_revealed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x508)]
    pub pursuit_delay: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50c)]
    pub sector_name_font: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x510)]
    pub map_border: WindowFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x538)]
    pub map_border_title: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x540)]
    pub map_border_title_mask: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x548)]
    pub map_border_sector: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x550)]
    pub map_inset_text_left: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x558)]
    pub map_inset_text_middle: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x560)]
    pub map_inset_text_right: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x568)]
    pub map_inset_text_jump: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x570)]
    pub map_inset_wait_distress: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x578)]
    pub red_light: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x580)]
    pub fuel_message: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x588)]
    pub waiting_message: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x590)]
    pub unexplored: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x598)]
    pub explored: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5a0)]
    pub danger: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5a8)]
    pub warning: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5b0)]
    pub yellow_warning: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5b8)]
    pub warning_circle: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5c0)]
    pub nebula_circle: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5c8)]
    pub box_green: [*mut GL_Texture; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5e0)]
    pub box_purple: [*mut GL_Texture; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5f8)]
    pub box_white: [*mut GL_Texture; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x610)]
    pub ship: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x618)]
    pub ship_no_fuel: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x620)]
    pub boss_ship: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x628)]
    pub danger_zone_edge: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x630)]
    pub danger_zone_tile: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x638)]
    pub danger_zone_advance: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x640)]
    pub target_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x648)]
    pub sector_target_box_green: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x650)]
    pub sector_target_box_yellow: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x658)]
    pub target_box_timer: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x678)]
    pub close_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x778)]
    pub desc_box: *mut WindowFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x780)]
    pub shadow: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x788)]
    pub warning_shadow: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x790)]
    pub fuel_overlay: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x798)]
    pub danger_flash: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7a0)]
    pub maps_bottom: [*mut GL_Primitive; 3],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7b8)]
    pub dotted_line: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c0)]
    pub cross: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c8)]
    pub boss_jumps_box: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7d0)]
    pub small_nebula: Vector<ImageDesc>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7e8)]
    pub large_nebula: Vector<ImageDesc>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x800)]
    pub current_nebulas: Vector<NebulaInfo>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x818)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x820)]
    pub out_of_fuel: bool,
    pub waiting: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x848)]
    pub danger_wait_start: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x850)]
    pub distress_anim: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x870)]
    pub b_tutorial_generated: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x878)]
    pub delayed_quests: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x890)]
    pub sectors: Vector<*mut Sector>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8a8)]
    pub current_sector: *mut Sector,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8b0)]
    pub secret_sector: *mut Sector,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8b8)]
    pub b_choosing_new_sector: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8b9)]
    pub b_secret_sector: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8c0)]
    pub dummy_new_sector: Location,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9a0)]
    pub maps_analyzed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9a4)]
    pub locations_created: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9a8)]
    pub ships_created: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9b0)]
    pub scrap_collected: Map<StdString, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9e0)]
    pub drones_collected: Map<StdString, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa10)]
    pub fuel_collected: Map<StdString, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa40)]
    pub weapon_found: Map<StdString, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa70)]
    pub drone_found: Map<StdString, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xaa0)]
    pub boss_loc: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xaa4)]
    pub arrived_at_base: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xaa8)]
    pub reversed_path: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xaa9)]
    pub boss_jumping: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xab0)]
    pub boss_path: Vector<*mut Location>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xac8)]
    pub boss_level: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xac9)]
    pub boss_wait: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xacc)]
    pub boss_position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xad8)]
    pub force_sector_choice: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xae0)]
    pub b_enemy_ship: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xae1)]
    pub b_nebula_map: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xae2)]
    pub b_infinite_mode: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xae8)]
    pub last_sectors: Vector<*mut Sector>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb00)]
    pub close_sector_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc00)]
    pub sector_map_seed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc04)]
    pub current_sector_seed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc08)]
    pub fuel_event_seed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc10)]
    pub last_escape_event: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc18)]
    pub waited_last: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc20)]
    pub store_trash: Vector<*mut Store>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc38)]
    pub added_quests: Vector<Pair<StdString, c_int>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc50)]
    pub boss_stage: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc58)]
    pub boss_message: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc68)]
    pub boss_jumping_warning: *mut WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc70)]
    pub crystal_alien_found: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc78)]
    pub found_map: Map<*mut Location, bool>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xca8)]
    pub sector_map_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xcb0)]
    pub potential_sector_choice: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xcb4)]
    pub final_sector_choice: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xcb8)]
    pub sector_hit_boxes: Vector<Rect>,
}

impl StarMap {
    pub fn current_loc(&self) -> Option<&Location> {
        unsafe { xc(self.current_loc) }
    }
    pub fn current_sector(&self) -> Option<&Sector> {
        unsafe { xc(self.current_sector) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SpaceStatus {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub warning_images: [*mut GL_Primitive; 10],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub warning_message: *mut WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub incoming_fire: *mut WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub hitbox: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub hitbox2: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub current_effect: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x84)]
    pub current_effect2: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub space: *mut SpaceManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub touched_tooltip: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct FTLButton {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: TextButtonPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf8)]
    pub text_y_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xfc)]
    pub auto_shrink: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xfd)]
    pub ready: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x100)]
    pub ftl_blink: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x104)]
    pub ftl_blink_dx: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x108)]
    pub pullout: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x110)]
    pub ship: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub base_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x120)]
    pub base_image_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x128)]
    pub pullout_base: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x130)]
    pub pullout_base_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub pilot_on: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub pilot_off1: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub pilot_off2: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x150)]
    pub engines_on: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub engines_off1: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub engines_off2: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x168)]
    pub ftl_loadingbars: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub ftl_loadingbars_off: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x178)]
    pub loading_bars: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub loading_bars_off: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub last_bars_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x190)]
    pub engines_down: *mut WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x198)]
    pub b_out_of_fuel: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x199)]
    pub b_boss_fight: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19a)]
    pub b_hover_raw: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19b)]
    pub b_hover_pilot: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19c)]
    pub b_hover_engine: bool,
}

impl FTLButton {
    pub fn ship(&self) -> Option<&ShipManager> {
        unsafe { xc(self.ship) }
    }
    pub fn mouse_click(&self) -> bool {
        if !self.base.base.b_active {
            return false;
        }
        if !self
            .ship()
            .unwrap()
            .system(System::Engines)
            .is_some_and(|x| x.functioning())
        {
            return false;
        }
        if !self
            .ship()
            .unwrap()
            .system(System::Pilot)
            .is_some_and(|x| x.functioning())
        {
            return false;
        }
        if self.ship().unwrap().jump_timer.first < self.ship().unwrap().jump_timer.second {
            return false;
        }
        true
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct HandAnimation {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub hand: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub start: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub finish: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub location: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub b_running: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub pause: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneControl {
    pub base: ArmamentControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub drone_message: WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b0)]
    pub no_target_message: WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x290)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub background: Vector<*mut GL_Primitive>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub empty_background: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub hover_highlight: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub outline: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub empty_outline: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub power_bar_glow: [*mut GL_Primitive; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub icon_background: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub icon_inset_background: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub icon: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub icon_double_size: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub icon_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub icon_background_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub last_icon_pos: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub x_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa4)]
    pub large_icon_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xac)]
    pub name_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub name_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub mouse_hover: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb9)]
    pub touch_hover: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xba)]
    pub touch_highlight: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbb)]
    pub selected: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbc)]
    pub hot_key: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub active_touch: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub touch_tooltip: *mut TouchTooltip,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub hack_animation: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x190)]
    pub touch_button_border: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x198)]
    pub touch_button_border_rect: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a8)]
    pub touch_button_slide_pos: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b0)]
    pub touch_buttons: Vector<*mut GenericButton>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c8)]
    pub touch_button_hitbox: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub icon_color: GL_Color,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e8)]
    pub drone_variation: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e9)]
    pub b_ioned: bool,
}

impl ArmamentBox {
    pub fn vtable(&self) -> &'static VtableArmamentBox {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneBox {
    pub base: ArmamentBox,
    pub p_drone: *mut Drone,
}

impl DroneBox {
    pub fn drone(&self) -> Option<&Drone> {
        unsafe { xc(self.p_drone) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ArmamentBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f0)]
    pub p_weapon: *mut ProjectileFactory,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f8)]
    pub armed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f9)]
    pub armed_for_autofire: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1fc)]
    pub cooldown_max: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x200)]
    pub cooldown_modifier: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x204)]
    pub cooldown_point: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20c)]
    pub cooldown_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
    pub cooldown_height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub cooldown_box: Vector<*mut GL_Primitive>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x230)]
    pub cooldown_bar: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x238)]
    pub charge_icons: Vector<CachedImage>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x250)]
    pub default_autofire: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x251)]
    pub was_charged: bool,
}

impl WeaponBox {
    pub fn weapon(&self) -> Option<&ProjectileFactory> {
        unsafe { xc(self.p_weapon) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ArmamentControl {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableArmamentControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub system_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub gui: *mut CommandGui,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub boxes: Vector<*mut ArmamentBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub touch_hit_box: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub holder_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub holder: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub holder_tab: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub small_box_holder: Vector<*mut GL_Primitive>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub small_box_hack_anim: Vector<Animation>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub small_box_holder_top: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9c)]
    pub b_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub last_mouse: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub current_mouse: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub dragging_box: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub dragging_touch: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub b_dragging: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbc)]
    pub i_last_swap_slot: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub b_tutorial_flash: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc4)]
    pub i_flash_slot: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub active_touch: c_int,
}

impl ArmamentControl {
    pub fn vtable(&self) -> &'static VtableArmamentControl {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponControl {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ArmamentControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub current_target: *mut Targetable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub armed_weapon: *mut ProjectileFactory,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe0)]
    pub auto_firing: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe8)]
    pub auto_fire_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e8)]
    pub auto_fire_base: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f0)]
    pub target_icon: [*mut GL_Primitive; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
    pub target_icon_yellow: [*mut GL_Primitive; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x230)]
    pub auto_fire_focus: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x238)]
    pub missile_message: WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x318)]
    pub system_message: WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3f8)]
    pub armed_slot: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CombatControl {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub gui: *mut CommandGui,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub player_ship_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub space: *mut SpaceManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub weap_control: WeaponControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x420)]
    pub drone_control: DroneControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x790)]
    pub sys_boxes: Vector<*mut SystemBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7a8)]
    pub enemy_ships: Vector<*mut CompleteShip>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c0)]
    pub current_target: *mut CompleteShip,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c8)]
    pub current_drone: *mut SpaceDrone,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7d0)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7d8)]
    pub selected_room: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7dc)]
    pub selected_self_room: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7e0)]
    pub target_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7e8)]
    pub box_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7f0)]
    pub hostile_box_frame: *mut WindowFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7f8)]
    pub health_mask: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x840)]
    pub shield_circle_charged: [CachedImage; 5],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9a8)]
    pub shield_circle_uncharged: [CachedImage; 5],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb10)]
    pub shield_circle_hacked: [CachedImage; 5],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc78)]
    pub shield_circle_hacked_charged: [CachedImage; 5],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xde0)]
    pub shield_charge_box: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe28)]
    pub super_shield_box5: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe70)]
    pub super_shield_box12: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xeb8)]
    pub open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xebc)]
    pub ship_icon_size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xec0)]
    pub potential_aiming: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xec8)]
    pub aiming_points: Vector<Pointf>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xee0)]
    pub last_mouse: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xee8)]
    pub mouse_down: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xee9)]
    pub is_aiming_touch: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xeea)]
    pub moving_beam: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xeec)]
    pub beam_move_last: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xef4)]
    pub invalid_beam_touch: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xef8)]
    pub screen_reposition: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf00)]
    pub teleport_command: Pair<c_int, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf08)]
    pub i_teleport_armed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf10)]
    pub teleport_target_send: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf58)]
    pub teleport_target_return: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xfa0)]
    pub hack_target: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xfe8)]
    pub mind_target: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1030)]
    pub ftl_timer: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1050)]
    pub ftl_warning: WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1130)]
    pub hacking_timer: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1150)]
    pub hacking_messages: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1168)]
    boss_visual: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1169)]
    pub b_teaching_beam: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1170)]
    pub tip_box: *mut WindowFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1178)]
    pub hand: HandAnimation,
}

impl CombatControl {
    pub fn current_target(&self) -> Option<&CompleteShip> {
        unsafe { xc(self.current_target) }
    }
    pub fn current_target_mut(&mut self) -> Option<&mut CompleteShip> {
        unsafe { xm(self.current_target) }
    }
    pub fn ship_manager(&self) -> Option<&ShipManager> {
        unsafe { xc(self.ship_manager) }
    }
    pub fn weapons_armed(&self) -> bool {
        self.ship_manager().unwrap().has_system(System::Teleporter)
            && self
                .ship_manager()
                .unwrap()
                .teleport_system()
                .unwrap()
                .i_armed
                != 0
            || !self.weap_control.armed_weapon.is_null()
            || self.ship_manager().unwrap().has_system(System::Mind)
                && self.ship_manager().unwrap().mind_system().unwrap().i_armed != 0
            || self.ship_manager().unwrap().has_system(System::Hacking)
                && self
                    .ship_manager()
                    .unwrap()
                    .hacking_system()
                    .unwrap()
                    .b_armed
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct PowerBars {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub normal: [*mut GL_Primitive; 30],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub tiny: [*mut GL_Primitive; 30],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e0)]
    pub empty: [*mut GL_Primitive; 30],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d0)]
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
    pub on_render: Option<fn(*mut SystemBox, bool)>,
    pub get_mouse_hover: Option<fn(*mut SystemBox) -> bool>,
    pub mouse_move: Option<fn(*mut SystemBox, c_int, c_int)>,
    pub mouse_click: Option<fn(*mut SystemBox, bool) -> bool>,
    pub mouse_right_click: Option<fn(*mut SystemBox, bool)>,
    pub on_touch: Option<fn(*mut SystemBox, TouchAction, c_int, c_int, c_int, c_int, c_int)>,
    pub cancel_touch: Option<fn(*mut SystemBox)>,
    pub close_tap_box: Option<fn(*mut SystemBox)>,
    pub is_touch_tooltip_open: Option<fn(*mut SystemBox) -> bool>,
    pub is_touch_tooltip_active: Option<fn(*mut SystemBox) -> bool>,
    pub close_touch_tooltip: Option<fn(*mut SystemBox, bool)>,
    pub key_down: Option<fn(*mut SystemBox, SDLKey, bool)>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TapBoxFrame {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub use_wide_box: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub box_height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub button_heights: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub primitives: Vector<*mut GL_Primitive>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub hit_box: Rect,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TouchTooltip {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub tab_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub mirrored: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub tray_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub tray_height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub tab: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub tab_size: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub tray: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub tab_hit_box: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub tray_hit_box: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub slide_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x64)]
    pub is_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x65)]
    pub is_snapping: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub snap_target_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub snap_last_timestamp: c_double,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    active_touch: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c)]
    pub ignore_touch: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub initial_slide_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x84)]
    pub last_touch_delta: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemBox {
    pub vtable: *const VtableSystemBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub timer_circle: [*mut GL_Primitive; 10],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub timer_lines: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub timer_stencil: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub last_timer_stencil_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub broken_icon: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub lock_icon: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub hack_icon: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub p_system: *mut ShipSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub b_show_power: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9c)]
    pub power_alpha: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub mouse_hover: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa4)]
    pub active_touch: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub touch_initial_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub tapped: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb1)]
    pub dragging_power: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub drag_initial_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub last_drag_speed: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbc)]
    pub last_drag_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub last_drag_time: c_double,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub warning: WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a8)]
    pub top_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1ac)]
    pub hit_box: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1bc)]
    pub hit_box_top: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c0)]
    pub hit_box_top_was_set: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c8)]
    pub wire_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub b_simple_power: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d1)]
    pub b_player_u_i: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d2)]
    pub use_large_tap_icon: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d4)]
    pub large_tap_icon_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e0)]
    pub tap_button_heights: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f8)]
    pub tap_button_offset_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1fc)]
    pub cooldown_offset_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x200)]
    pub key_pressed: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x208)]
    pub touch_tooltip: *mut TouchTooltip,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
    pub tap_box_frame: TapBoxFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x260)]
    pub locked_open: bool,
}

impl SystemBox {
    pub fn vtable(&self) -> &'static VtableSystemBox {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: SystemBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub box_: [*mut GL_Primitive; 5],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x290)]
    pub bar_: [*mut GL_Texture; 5],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2b8)]
    pub box_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c0)]
    pub round_down: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c8)]
    pub bar_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d0)]
    pub last_bar_height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d4)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: CooldownSystemBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d8)]
    pub hack_sys: *mut HackingSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2e0)]
    pub buttons: Vector<*mut Button>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2f8)]
    pub current_button: *mut Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x300)]
    pub button_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x308)]
    pub box_: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x310)]
    pub box2: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x318)]
    pub hack_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3a8)]
    pub overlay_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x438)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x440)]
    pub flash_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x460)]
    pub super_shield_warning: *mut WarningMessage,
}

impl HackBox {
    pub fn current_button(&self) -> Option<&Button> {
        unsafe { xc(self.current_button) }
    }
    pub fn current_button_mut(&mut self) -> Option<&mut Button> {
        unsafe { xm(self.current_button) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BatteryBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: CooldownSystemBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d8)]
    pub battery_system: *mut BatterySystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2e0)]
    pub battery_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x370)]
    pub button_offset: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CloakingBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: CooldownSystemBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d8)]
    pub buttons: Vector<*mut Button>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2f0)]
    pub current_button: *mut Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2f8)]
    pub cloak_system: *mut CloakingSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x300)]
    pub button_offset: Point,
}

impl CloakingBox {
    pub fn current_button(&self) -> Option<&Button> {
        unsafe { xc(self.current_button) }
    }
    pub fn current_button_mut(&mut self) -> Option<&mut Button> {
        unsafe { xm(self.current_button) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemControl {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub combat_control: *mut CombatControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub sys_boxes: Vector<*mut SystemBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub _system_power: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub b_system_power_hover: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x44)]
    pub system_power_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c)]
    pub sub_system_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub wires_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub wires_mask: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub no_button: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub button: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub no_button_cap: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub button_cap: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub drone: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub drone3: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub drone2: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub sub_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub sub_spacing: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub not_enough_power: *mut WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub flash_battery_power: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub flash_tracker: AnimationTracker,
}

impl SystemControl {
    pub fn ship_manager(&self) -> Option<&ShipManager> {
        unsafe { xc(self.ship_manager) }
    }
    pub fn ship_manager_mut(&mut self) -> Option<&mut ShipManager> {
        unsafe { xm(self.ship_manager) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewBox {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub box_: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub skill_box: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub p_crew: *const CrewMember,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub mouse_hover: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub power_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x130)]
    pub number: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x134)]
    pub b_selectable: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub flash_health_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub box_background: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub box_outline: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x168)]
    pub skill_box_background: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub skill_box_outline: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x178)]
    pub cooldown_bar: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub health_warning: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c8)]
    pub last_cooldown_height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub health_bar: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub last_health_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e0)]
    pub mind_controlled: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2a0)]
    pub stunned: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x360)]
    pub hide_extra: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x368)]
    pub s_tooltip: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewControl {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub ship_manager: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub selected_crew: Vector<*mut CrewMember>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub potential_selected_crew: Vector<*mut CrewMember>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub selected_door: *mut Door,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub selected_repair: *mut Repairable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub selected_grid: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub selected_room: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x54)]
    pub selected_player_ship: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub available_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub crew_boxes: Vector<*mut CrewBox>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub first_mouse: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub current_mouse: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub world_first_mouse: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub world_current_mouse: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub mouse_down: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x99)]
    pub b_updated: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9c)]
    pub active_touch: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub selecting_crew: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa1)]
    pub selecting_crew_on_player_ship: bool,
    #[cfg(target_os = "windows")]
    pub _unk1: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub selecting_crew_start_time: c_double,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub door_control_mode: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb1)]
    pub door_control_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb2)]
    pub door_control_open_set: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub combat_control: *mut CombatControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub selected_crew_box: c_uint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub crew_message: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe8)]
    pub message: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub save_stations: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub return_stations: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
    pub save_stations_base: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub return_stations_base: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    pub stations_last_y: c_int,
    #[cfg(target_os = "windows")]
    pub _unk2: c_int,
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub color: GL_Color,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub b_float_down: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub primitives: Vector<*mut GL_Primitive>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WarningWithLines {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe0)]
    pub line_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe8)]
    pub text_origin: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub top_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x100)]
    pub bottom_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x110)]
    pub top_text_limit: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x114)]
    pub bottom_text_limit: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Ellipse {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub center: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub a: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub b: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipObject {
    pub vtable: *const VtableShipObject,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub i_ship_id: c_int,
}

impl ShipObject {
    pub fn vtable(&self) -> &'static VtableShipObject {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub hostile: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd)]
    pub targeted: bool,
}

impl Targetable {
    pub fn vtable(&self) -> &'static VtableTargetable {
        unsafe { xb(self.vtable).unwrap() }
    }
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

impl Collideable {
    pub fn vtable(&self) -> &'static VtableCollideable {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub position_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub position_y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub speed_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub speed_y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub acceleration_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub acceleration_y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub lifespan: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub alive: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ParticleEmitter {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub particles: [Particle; 64],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x800)]
    pub birth_rate: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x804)]
    pub birth_counter: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x808)]
    pub lifespan: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80c)]
    pub speed_mag: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x810)]
    pub position_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x814)]
    pub position_y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x818)]
    pub max_dx: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x81c)]
    pub min_dx: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x820)]
    pub max_dy: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x824)]
    pub min_dy: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x828)]
    pub image_x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x82c)]
    pub image_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x830)]
    pub primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x838)]
    pub emit_angle: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x83c)]
    pub rand_angle: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x83d)]
    pub running: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x840)]
    pub max_alpha: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x844)]
    pub min_size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x848)]
    pub max_size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x84c)]
    pub current_count: c_int,
}

// XXX: maps are really annoying to go through so not gonna bother recreating this
#[repr(C)]
#[derive(Debug)]
pub struct Map<K, V> {
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub self_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub powered: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub power_required: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub deployed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub blueprint: *const DroneBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub b_dead: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c)]
    pub i_bonus_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub powered_at_location: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub destroyed_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub i_hack_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    pub hack_time: c_float,
}

impl Drone {
    pub fn vtable(&self) -> &'static VtableDrone {
        unsafe { xb(self.vtable).unwrap() }
    }
}

impl Drone {
    pub fn required_power(&self) -> c_int {
        self.power_required - self.i_bonus_power
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewDrone {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    // XXX: a bunch of extended (with drone funcs) vtable entries are missing, dont want to go through the pain
    pub base: CrewMember,
    // offset = 335093
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x748)]
    pub base1: Drone,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x788)]
    pub drone_room: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x790)]
    pub power_up: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x850)]
    pub power_down: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x910)]
    pub light_layer: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x918)]
    pub base_layer: *mut GL_Texture,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CollisionResponse {
    // 1: hit (but don't damage anymore), 2: shield, 3: miss
    // 0: proper hit (caller should also do some damage)
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub collision_type: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub point: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub damage: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub base1: Targetable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub base2: Collideable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub current_space: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5c)]
    pub destination_space: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub current_location: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub last_location: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub destination_location: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub point_target: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub explosion: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub weapon_target: *mut Targetable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub target_location: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x150)]
    pub target_speed: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub movement_target: *mut Targetable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub speed_vector: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x168)]
    pub powered_last_frame: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x169)]
    pub deployed_last_frame: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x16a)]
    pub b_fire: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x16c)]
    pub pause: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub additional_pause: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x174)]
    pub weapon_cooldown: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x178)]
    pub current_angle: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x17c)]
    pub aiming_angle: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub last_aiming_angle: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x184)]
    pub desired_aiming_angle: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub message: *mut DamageMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x190)]
    pub weapon_animation: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x250)]
    pub weapon_blueprint: *const WeaponBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x258)]
    pub lifespan: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x25c)]
    pub b_loaded_position: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x25d)]
    pub b_disrupted: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x260)]
    pub hack_angle: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x264)]
    pub ion_stun: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub beam_current_target: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x270)]
    pub beam_final_target: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x278)]
    pub beam_speed: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x280)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableProjectile,
    // offset = 274913
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub base1: Targetable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub last_position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub speed_magnitude: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c)]
    pub target: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub heading: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub owner_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    pub self_id: c_uint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub damage: Damage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x74)]
    pub lifespan: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub destination_space: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c)]
    pub current_space: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub target_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x84)]
    pub dead: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub death_animation: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub flight_animation: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x208)]
    pub speed: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
    pub missed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x211)]
    pub hit_target: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub hit_solid_sound: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    pub hit_shield_sound: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    pub miss_sound: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x230)]
    pub entry_angle: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x234)]
    pub started_death: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x235)]
    pub passed_target: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x236)]
    pub b_broadcast_target: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x238)]
    pub flash_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x258)]
    pub color: GL_Color,
}

impl Projectile {
    pub fn vtable(&self) -> &'static VtableProjectile {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AnimationDescriptor {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub num_frames: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub image_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub image_height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub strip_start_y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub strip_start_x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub frame_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub frame_height: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Animation {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub animation_strip: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub info: AnimationDescriptor,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub sound_forward: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub sound_reverse: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub randomize_frames: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x64)]
    pub f_scale: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub f_y_stretch: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6c)]
    pub current_frame: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub b_always_mirror: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub sound_queue: Vector<Vector<StdString>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub fade_out: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x94)]
    pub start_fade_out: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub anim_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub mask_x_pos: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa4)]
    pub mask_x_size: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub mask_y_pos: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xac)]
    pub mask_y_size: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub mirrored_primitive: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SystemTemplate {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub system_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub power_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub location: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub bp: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub max_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub image: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub slot: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub direction: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub weapon: Vector<StdString>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub base_name: StdString,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub base_desc: Description,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub desc: Description,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub blueprint_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe0)]
    pub name: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub ship_class: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x100)]
    pub layout_file: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x108)]
    pub img_file: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x110)]
    pub cloak_file: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub shield_file: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x120)]
    pub floor_file: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x128)]
    pub system_info: Map<c_int, SystemTemplate>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub systems: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub drone_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x174)]
    pub original_drone_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x178)]
    pub drone_slots: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub load_drones: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub drones: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a0)]
    pub augments: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b8)]
    pub weapon_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1bc)]
    pub original_weapon_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c0)]
    pub weapon_slots: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c8)]
    pub load_weapons: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub weapons: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e8)]
    pub missiles: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1ec)]
    pub drone_count_1: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f0)]
    pub health: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f4)]
    pub original_crew_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f8)]
    pub default_crew: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
    pub custom_crew: Vector<CrewBlueprint>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    pub max_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x22c)]
    pub boarding_a_i: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x230)]
    pub bp_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x234)]
    pub max_crew: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x238)]
    pub max_sector: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x23c)]
    pub min_sector: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub unlock: TextString,
}

impl ShipBlueprint {
    pub fn vtable(&self) -> &'static VtableBlueprint {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub state: DoorStateEnum,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub hacked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub level: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct LockdownShard {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub shard: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub goal: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub speed: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd4)]
    pub b_arrived: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd5)]
    pub b_done: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub life_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xdc)]
    pub super_freeze: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe0)]
    pub locking_room: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ExplosionAnimation {
    // XXX: this vtable includes a bunch of other stuff but who cares
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: AnimationTracker,
    // offset = 371421
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub base1: ShipObject,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub explosions: Vector<Animation>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub pieces: Vector<*mut GL_Texture>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub piece_names: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub rotation_speed: Vector<c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub rotation: Vector<c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub rotation_speed_min_max: Vector<Pair<c_float, c_float>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub movement_vector: Vector<Pointf>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub position: Vector<Pointf>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub starting_position: Vector<Pointf>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x108)]
    pub explosion_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10c)]
    pub sound_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x110)]
    pub b_final_boom: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x111)]
    pub b_jump_out: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub weapon_anims: Vector<*mut WeaponAnimation>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x130)]
    pub pos: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ImageDesc {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub tex: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub res_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub w: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub h: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub rot: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct OuterHull {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: Repairable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub breach: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x100)]
    pub heal: Animation,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Room {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: Selectable,
    /// Inherited from ShipObject
    pub base1_vtable: *const VtableShipObject,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub rect: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c)]
    pub i_room_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub b_blacked_out: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub filled_slots: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub slots: Vector<VectorBool>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub b_warning_light: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub light_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub i_fire_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub fires: Vector<Animation>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub primary_slot: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub primary_direction: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub last_o2: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub floor_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub blackout_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub highlight_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub highlight_primitive2: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe0)]
    pub o2_low_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe8)]
    pub computer_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub computer_glow_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf8)]
    pub computer_glow_yellow_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x100)]
    pub light_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x108)]
    pub light_glow_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x110)]
    pub stun_sparks: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub console_sparks: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x290)]
    pub b_stunning: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x294)]
    pub f_hacked: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x298)]
    pub current_spark_rotation: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2a0)]
    pub sparks: Vector<Animation>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2b8)]
    pub spark_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2bc)]
    pub spark_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c0)]
    pub i_hack_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c8)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipObject,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub v_room_list: Vector<*mut Room>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub v_door_list: Vector<*mut Door>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub v_outer_walls: Vector<*mut OuterHull>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub v_outer_airlocks: Vector<*mut Door>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub hull_integrity: Pair<c_int, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub weapon_mounts: Vector<WeaponMount>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub floor_image_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub ship_floor: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub floor_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub ship_image_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub ship_image: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe8)]
    pub glow_offset: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub ship_image_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf8)]
    pub cloak_image_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x100)]
    pub ship_image_cloak: ImageDesc,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x120)]
    pub cloak_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x128)]
    pub grid_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x130)]
    pub walls_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub doors_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub door_state: Vector<DoorState>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub last_door_control_mode: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub thrusters_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x168)]
    pub jump_glare: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub vertical_shift: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x174)]
    pub horizontal_shift: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x178)]
    pub ship_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub explosion: ExplosionAnimation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2b8)]
    pub b_destroyed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2bc)]
    pub base_ellipse: Ellipse,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d0)]
    pub engine_anim: [Animation; 2],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x450)]
    pub cloaking_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x470)]
    pub b_cloaked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x471)]
    pub b_experiment: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x472)]
    pub b_show_engines: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x478)]
    pub lockdowns: Vector<LockdownShard>,
}

impl Ship {
    pub fn get_room_blackout(&self, room_id: c_int) -> bool {
        if let Some(room) = self
            .v_room_list
            .iter()
            .map(|x| unsafe { xc(*x).unwrap() })
            .find(|room| room.i_room_id == room_id)
        {
            !room.filled_slots.is_empty()
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
    // #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub count: c_int,
    // #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub room_count: Vector<c_int>,
    // #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableSelectable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub selected_state: c_int,
}

impl Selectable {
    pub fn vtable(&self) -> &'static VtableSelectable {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    // 10
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
    // 20
    pub get_id: Option<fn(*mut Repairable) -> c_int>,
    pub is_room_based: Option<fn(*mut Repairable) -> bool>,
    pub get_room_id: Option<fn(*mut Repairable) -> c_int>,
    pub ioned: Option<fn(*mut Repairable, c_int) -> bool>,
    // 24
    pub set_room_id: Option<fn(*mut Repairable)>,
}

#[vtable]
pub struct VtableSpreadable {
    pub base: VtableRepairable,
    // 25
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableSpreadable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    /// Inherited from Repairable
    pub selected_state: c_int,
    /// Inherited from ShipObject
    pub base1_vtable: *const VtableShipObject,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    /// Inherited from Repairable
    pub f_damage: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    /// Inherited from Repairable
    pub p_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    /// Inherited from Repairable
    pub f_max_damage: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    /// Inherited from Repairable
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    /// Inherited from Repairable
    pub room_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    /// Inherited from Repairable
    pub i_repair_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub sound_name: StdString,
}

impl Spreadable {
    pub fn vtable(&self) -> &'static VtableSpreadable {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Repairable {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableRepairable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    /// Inherited from Selectable
    pub selected_state: c_int,
    /// Inherited from ShipObject
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub base1_vtable: *const VtableShipObject,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub f_damage: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub p_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub f_max_damage: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub room_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    pub i_repair_count: c_int,
}

impl Repairable {
    pub fn vtable(&self) -> &'static VtableRepairable {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Fire {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: Spreadable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub f_death_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c)]
    pub f_start_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub f_oxygen: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub fire_animation: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub smoke_animation: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableCrewMember,
    pub i_ship_id: c_int,
}

impl CrewTarget {
    pub fn vtable(&self) -> &'static VtableCrewMember {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Slot {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub room_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub slot_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub world_location: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SCrewStats {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub stat: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub species: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub male: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Door {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: CrewTarget,
    /// Inherited from Selectable
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub base1_vtable: *const VtableSelectable,
    /// Inherited from Selectable
    pub base1_selected_state: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub i_room1: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub i_room2: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub b_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub i_blast: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c)]
    pub b_fake_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub outline_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub highlight_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub door_anim: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x108)]
    pub door_anim_large: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c8)]
    pub i_door_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1cc)]
    pub base_health: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub health: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub forced_open: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f8)]
    pub got_hit: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub door_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x21c)]
    pub b_ioned: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    pub fake_open_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    pub locked_down: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub lastbase: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24c)]
    pub i_hacked: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x250)]
    pub x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x254)]
    pub y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x258)]
    pub b_vertical: bool,
}

impl Door {
    pub fn close(&mut self) {
        unsafe { super::DOOR_CLOSE.call(ptr::addr_of_mut!(*self)) }
    }
    pub fn open(&mut self) {
        unsafe { super::DOOR_OPEN.call(ptr::addr_of_mut!(*self)) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewTask {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub task_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub room: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    #[allow(non_snake_case)]
    pub _sil_do_not_use_system: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BoardingGoal {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub f_health_limit: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub caused_damage: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub targets_destroyed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub target: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: Projectile,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub r: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x26c)]
    pub g: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x270)]
    pub b: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewAnimation {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableCrewAnimation,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub anims: Vector<Vector<Animation>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub base_strip: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub color_strip: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub layer_strips: Vector<*mut GL_Texture>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub last_position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub direction: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5c)]
    pub sub_direction: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub status: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x64)]
    pub move_direction: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub smoke_emitter: ParticleEmitter,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8b8)]
    pub b_shared_spot: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8c0)]
    pub shots: Vector<CrewLaser>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8d8)]
    pub shoot_timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8ec)]
    pub punch_timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x900)]
    pub target: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x908)]
    pub f_damage_done: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90c)]
    pub b_player: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90d)]
    pub b_frozen: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90e)]
    pub b_drone: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90f)]
    pub b_ghost: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x910)]
    pub b_exact_shooting: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x918)]
    pub projectile: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9d8)]
    pub b_typing: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9e0)]
    pub race: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9e8)]
    pub current_ship: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9ec)]
    pub b_male: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9ed)]
    pub colorblind: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9f0)]
    pub layer_colors: Vector<GL_Color>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa08)]
    pub forced_animation: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0c)]
    pub forced_direction: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa10)]
    pub projectile_color: GL_Color,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa20)]
    pub b_stunned: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa21)]
    pub b_door_target: bool,
}

impl CrewAnimation {
    pub fn vtable(&self) -> &'static VtableCrewAnimation {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Path {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub start: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub doors: Vector<*mut Door>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub finish: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub distance: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CrewMember {
    pub vtable: *const VtableCrewMember,
    /// Inherited from CrewTarget
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub scale: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub goal_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub goal_y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub height: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c)]
    pub health: Pair<c_float, c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub speed_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub speed_y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub path: Path,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub new_path: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x74)]
    pub x_destination: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub y_destination: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub last_door: *mut Door,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub current_repair: *mut Repairable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub b_suffocating: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x94)]
    pub move_goal: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub selection_state: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9c)]
    pub i_room_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub i_manning_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa4)]
    pub i_repair_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub i_stack_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xac)]
    pub current_slot: Slot,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbc)]
    pub intruder: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbd)]
    pub b_fighting: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbe)]
    pub b_shared_spot: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub crew_anim: *mut CrewAnimation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub selection_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub health_box: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub health_box_red: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub health_bar: CachedRect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub f_medbay: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x184)]
    pub last_damage_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub last_health_change: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18c)]
    pub current_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x190)]
    pub flash_health_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b0)]
    pub current_target: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b8)]
    pub crew_target: *mut CrewTarget,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c0)]
    pub boarding_goal: BoardingGoal,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d4)]
    pub b_frozen: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d5)]
    pub b_frozen_location: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub task: CrewTask,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e8)]
    pub type_: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f0)]
    pub ship: *mut Ship,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f8)]
    pub final_goal: Slot,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x208)]
    pub blocking_door: *mut Door,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
    pub b_out_of_game: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub species: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    pub b_dead: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x224)]
    pub i_on_fire: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    pub b_active_manning: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x230)]
    pub current_system: *mut ShipSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x238)]
    pub using_skill: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub blueprint: CrewBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x340)]
    pub healing: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x400)]
    pub stunned: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c0)]
    pub level_up: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4e0)]
    pub last_level_up: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4e8)]
    pub stats: SCrewStats,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x518)]
    pub skills_earned: Vector<VectorBool>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x530)]
    pub clone_ready: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x531)]
    pub b_mind_controlled: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x534)]
    pub i_death_number: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x538)]
    pub mind_controlled: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5f8)]
    pub stun_icon: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6b8)]
    pub skill_up: Vector<Vector<AnimationTracker>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6d0)]
    pub health_boost: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6d4)]
    pub f_mind_damage_boost: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6d8)]
    pub f_clone_dying: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6dc)]
    pub b_resisted: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6e0)]
    pub saved_position: Slot,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6f0)]
    pub f_stun_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x6f8)]
    pub movement_target: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x740)]
    pub b_cloned: bool,
}

impl CrewMember {
    pub fn vtable(&self) -> &'static VtableCrewMember {
        unsafe { xb(self.vtable).unwrap() }
    }
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
        unsafe { super::MOVE_CREW.call(ptr::addr_of_mut!(*self), room_id, slot_id, force) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ArtillerySystem {
    pub base: ShipSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub projectile_factory: *mut ProjectileFactory,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x250)]
    pub target: *mut Targetable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x258)]
    pub b_cloaked: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct MedbaySystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystem,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct EngineSystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub b_boost_ftl: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct DroneSystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    #[cfg_attr(target_os = "windows", test_offset = 0x1C0)]
    pub drones: Vector<*mut Drone>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x260)]
    #[cfg_attr(target_os = "windows", test_offset = 0x1CC)]
    pub drone_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x264)]
    #[cfg_attr(target_os = "windows", test_offset = 0x1D0)]
    pub drone_start: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    #[cfg_attr(target_os = "windows", test_offset = 0x1D4)]
    pub target_ship: *mut Targetable,
    #[cfg_attr(target_os = "windows", test_offset = 0x1D8)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x270)]
    pub user_powered: VectorBool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x298)]
    #[cfg_attr(target_os = "windows", test_offset = 0x1EC)]
    pub slot_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x29c)]
    pub i_starting_battery_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2a0)]
    pub repower_list: VectorBool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct HotkeyDesc {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub key: SDLKey,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Array<T, const N: usize> {
    pub data: [T; N],
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct SettingValues {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub fullscreen: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub current_fullscreen: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub last_fullscreen: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub sound: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub music: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub difficulty: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub command_console: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19)]
    pub alt_pause: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a)]
    pub touch_auto_pause: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b)]
    pub lowend: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub fb_error: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub language: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub language_set: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c)]
    pub screen_resolution: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub dialog_keys: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub logging: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x39)]
    pub b_show_changelog: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3a)]
    pub b_show_sync_achievements: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    pub loading_save_version: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub ach_popups: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x41)]
    pub vsync: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x42)]
    pub frame_limit: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43)]
    pub manual_resolution: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x44)]
    pub manual_windowed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x45)]
    pub manual_stretched: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x46)]
    pub show_paths: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x47)]
    pub swap_texture_type: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub colorblind: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub hotkeys: Array<Vector<HotkeyDesc>, 4>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub holding_modifier: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb1)]
    pub b_dlc_enabled: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub opened_list: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub beam_tutorial: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponMount {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub mirror: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9)]
    pub rotate: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub slide: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub gib: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponAnimation {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub anim: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub b_fire_shot: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc1)]
    pub b_firing: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc4)]
    pub f_charge_level: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub i_charged_frame: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xcc)]
    pub i_fire_frame: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub b_mirrored: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd1)]
    pub b_rotation: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd4)]
    pub fire_location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xdc)]
    pub b_powered: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe0)]
    pub mount_point: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe8)]
    pub render_point: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub fire_mount_vector: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf8)]
    pub slide_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub slide_direction: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x120)]
    pub i_charge_image: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x168)]
    pub explosion_anim: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    pub mount: WeaponMount,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x23c)]
    pub f_delay_charge_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub boost_anim: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x300)]
    pub boost_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x304)]
    pub b_show_charge: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x308)]
    pub f_actual_charge_level: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30c)]
    pub i_charge_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x310)]
    pub i_charge_levels: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x314)]
    pub current_offset: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x318)]
    pub charge_box: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x360)]
    pub charge_bar: CachedImage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3a8)]
    pub i_hack_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3b0)]
    pub hack_sparks: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x470)]
    pub player_ship: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ProjectileFactory {
    /// Inherited from ShipObject
    pub base_vtable: *const VtableShipObject,
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub cooldown: Pair<c_float, c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub sub_cooldown: Pair<c_float, c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub base_cooldown: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub blueprint: *const WeaponBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub local_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub flight_animation: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf0)]
    pub auto_firing: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf1)]
    pub fire_when_ready: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf2)]
    pub powered: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf4)]
    pub required_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf8)]
    pub targets: Vector<Pointf>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x110)]
    pub last_targets: Vector<Pointf>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x128)]
    pub target_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x12c)]
    pub i_ammo: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x130)]
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub num_shots: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x13c)]
    pub current_firing_angle: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub current_entry_angle: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub current_ship_target: *mut Targetable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x150)]
    pub cloaking_system: *mut CloakingSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub weapon_visual: WeaponAnimation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5d0)]
    pub mount: WeaponMount,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5e8)]
    pub queued_projectiles: Vector<*mut Projectile>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x600)]
    pub i_bonus_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x604)]
    pub b_fired_once: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x608)]
    pub i_spend_missile: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60c)]
    pub cooldown_modifier: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x610)]
    pub shots_fired_at_target: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x614)]
    pub radius: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x618)]
    pub boost_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x61c)]
    pub last_projectile_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x620)]
    pub charge_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x624)]
    pub i_hack_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x628)]
    pub goal_charge_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x62c)]
    pub is_artillery: bool,
}

impl ProjectileFactory {
    pub fn blueprint(&self) -> Option<&WeaponBlueprint> {
        unsafe { xb(self.blueprint) }
    }
    pub fn num_targets_required(&self) -> c_int {
        if self.blueprint().unwrap().charge_levels > 1 {
            self.charge_level.max(1)
        } else {
            self.num_shots
        }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipGraph {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub rooms: Vector<*const Room>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub doors: Vector<*const Door>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub door_counts: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub center: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub world_position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub world_heading: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5c)]
    pub last_world_position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x64)]
    pub last_world_heading: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub ship_box: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub ship_name: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponSystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub target: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x250)]
    pub weapons: Vector<*mut ProjectileFactory>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub weapons_trash_list: Vector<*mut ProjectileFactory>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x280)]
    pub shot_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x284)]
    pub shot_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x288)]
    pub missile_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28c)]
    pub missile_start: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x290)]
    pub cloaking_system: *mut CloakingSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x298)]
    pub user_powered: VectorBool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c0)]
    pub slot_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c4)]
    pub i_starting_battery_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c8)]
    pub repower_list: VectorBool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShieldAnimation {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub location: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub current_size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub end_size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub current_thickness: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub end_thickness: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub length: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub dx: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub side: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub owner_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub damage: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShieldPower {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub first: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub second: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub super_: Pair<c_int, c_int>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Shield {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub charger: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub power: ShieldPower,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub super_timer: c_float,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Shields {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub ellipse_ratio: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub center: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x250)]
    pub base_shield: Ellipse,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x260)]
    pub i_highlighted_side: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x264)]
    pub debug_x: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub debug_y: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x26c)]
    pub shields: Shield,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x284)]
    pub shields_shutdown: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x288)]
    pub shield_hits: Vector<ShieldAnimation>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2a0)]
    pub shields_down: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c0)]
    pub super_shield_down: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c4)]
    pub shields_down_point: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d0)]
    pub shields_up: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2f0)]
    pub shield_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2f8)]
    pub shield_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x300)]
    pub shield_image_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x308)]
    pub b_enemy_present: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x310)]
    pub dam_messages: Vector<*mut DamageMessage>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x328)]
    pub b_barrier_mode: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x32c)]
    pub last_hit_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x330)]
    pub charge_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x334)]
    pub last_hit_shield_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x338)]
    pub super_shield_up: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x358)]
    pub super_up_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x360)]
    pub b_excess_charge_hack: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct HackingSystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub b_hacking: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub drone: HackingDrone,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x820)]
    pub b_blocked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x821)]
    pub b_armed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x828)]
    pub current_system: *mut ShipSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x830)]
    pub effect_timer: Pair<c_float, c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x838)]
    pub b_can_hack: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x840)]
    pub queued_system: *mut ShipSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x848)]
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
    pub fn current_system(&self) -> Option<&ShipSystem> {
        unsafe { xc(self.current_system) }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CloneSystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub f_time_to_clone: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub clone: *mut CrewMember,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x250)]
    pub f_time_goal: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x254)]
    pub f_death_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x258)]
    pub bottom: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x260)]
    pub top: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub gas: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x270)]
    pub slot: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x278)]
    pub current_clone_animation: *mut Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x280)]
    pub clone_animations: Map<StdString, Animation>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct MindSystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub control_timer: Pair<c_float, c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24c)]
    pub b_can_use: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x250)]
    pub i_armed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x258)]
    pub controlled_crew: Vector<*mut CrewMember>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x270)]
    pub b_super_shields: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x271)]
    pub b_blocked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x274)]
    pub i_queued_target: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x278)]
    pub i_queued_ship: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x280)]
    pub queued_crew: Vector<*mut CrewMember>,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BatterySystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub b_turned_on: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x260)]
    pub soundeffect: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CloakingSystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub b_turned_on: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x260)]
    pub soundeffect: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub glow_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x288)]
    pub glow_image: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TeleportSystem {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub charge_level: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub b_can_send: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x249)]
    pub b_can_receive: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24c)]
    pub i_armed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x250)]
    pub crew_slots: VectorBool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x278)]
    pub i_prepared_crew: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x27c)]
    pub i_num_slots: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x280)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: ShipSystemPrime,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub max_oxygen: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x248)]
    pub oxygen_levels: Vector<c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x260)]
    pub f_total_oxygen: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x264)]
    pub b_leaking_o2: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ComputerGlowInfo {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: CachedPrimitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub w: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub h: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub thickness: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CachedRect {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: CachedPrimitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub w: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub h: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipSystem {
    pub vtable: *const VtableShipSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    /// Inherited from Repairable
    pub selected_state: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    /// Inherited from Repairable
    pub base1_vtable: *const VtableShipObject,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    /// Inherited from Repairable
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    /// Inherited from Repairable
    pub f_damage: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    /// Inherited from Repairable
    pub p_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    /// Inherited from Repairable
    pub f_max_damage: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    /// Inherited from Repairable
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    /// Inherited from Repairable
    pub room_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    /// Inherited from Repairable
    pub i_repair_count: c_int,
    /// System type
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub i_system_type: c_int,
    /// Doesn't work without manning
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x44)]
    pub b_needs_manned: bool,
    /// Basically never used
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x45)]
    pub b_manned: bool,
    /// How many people are manning
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub i_active_manned: c_int,
    /// Whether manning gives bonus power
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c)]
    pub b_boostable: bool,
    /// Allocated power and upgrade level
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub power_state: Pair<c_int, c_int>,
    /// I feel like this isn't used? idk
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub i_required_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub image_icon: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub icon_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub icon_border_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub icon_primitives: [[[*mut GL_Primitive; 5]; 2]; 2],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub partial_damage_rect: CachedRect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub lock_outline: CachedRectOutline,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub room_shape: Rect,
    /// Obvious
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub b_on_fire: bool,
    /// If the room is breached this can't be repaired until the breach is fixed
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x171)]
    pub b_breached: bool,
    /// Current/max HP
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x174)]
    pub health_state: Pair<c_int, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x17c)]
    pub f_damage_over_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub f_repair_over_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x184)]
    pub damaged_last_frame: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x185)]
    pub repaired_last_frame: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub original_power: c_int,
    /// basically, whether this is a subsystem
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18c)]
    pub b_needs_power: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x190)]
    pub i_temp_power_cap: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x194)]
    pub i_temp_power_loss: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x198)]
    pub i_temp_divide_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19c)]
    pub i_lock_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a0)]
    pub lock_timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b4)]
    pub b_exploded: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b5)]
    pub b_occupied: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b6)]
    pub b_friendlies: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b8)]
    pub interior_image_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c0)]
    pub interior_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c8)]
    pub interior_image_on: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub interior_image_manned: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub interior_image_manned_fancy: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e0)]
    pub last_user_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e4)]
    pub i_bonus_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e8)]
    pub i_last_bonus_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1ec)]
    pub location: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f4)]
    pub bp_cost: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f8)]
    pub flash_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub max_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x21c)]
    pub i_battery_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    #[cfg_attr(target_os = "windows", test_offset = 0x1A8)]
    pub i_hack_effect: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x224)]
    #[cfg_attr(target_os = "windows", test_offset = 0x1AC)]
    pub b_under_attack: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x225)]
    pub b_level_boostable: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x226)]
    pub b_trigger_ion: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    #[cfg_attr(target_os = "windows", test_offset = 0x1B0)]
    pub damaging_effects: Vector<Animation>,
    #[cfg_attr(target_os = "windows", test_offset = 0x1BC)]
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub computer_level: c_int,
}

impl ShipSystem {
    pub fn vtable(&self) -> &'static VtableShipSystem {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub current_power: Pair<c_int, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub over_powered: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub f_fuel: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub failed_powerup: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub i_temp_power_cap: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub i_temp_power_loss: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub i_temp_divide_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub i_hacked: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub battery_power: Pair<c_int, c_int>,
}

/// A modified version of ShipSystem without computer_level because of the damn padding
#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipSystemPrime {
    pub vtable: *const VtableShipSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    /// Inherited from Repairable
    pub selected_state: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    /// Inherited from Repairable
    pub base1_vtable: *const VtableShipObject,
    /// Inherited from Repairable
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    /// Inherited from Repairable
    pub f_damage: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    /// Inherited from Repairable
    pub p_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    /// Inherited from Repairable
    pub f_max_damage: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    /// Inherited from Repairable
    pub name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    /// Inherited from Repairable
    pub room_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    /// Inherited from Repairable
    pub i_repair_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub i_system_type: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x44)]
    pub b_needs_manned: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x45)]
    pub b_manned: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub i_active_manned: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4c)]
    pub b_boostable: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub power_state: Pair<c_int, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub i_required_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub image_icon: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub icon_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub icon_border_primitive: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub icon_primitives: [[[*mut GL_Primitive; 5]; 2]; 2],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub partial_damage_rect: CachedRect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub lock_outline: CachedRectOutline,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub room_shape: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub b_on_fire: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x171)]
    pub b_breached: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x174)]
    pub health_state: Pair<c_int, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x17c)]
    pub f_damage_over_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub f_repair_over_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x184)]
    pub damaged_last_frame: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x185)]
    pub repaired_last_frame: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub original_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18c)]
    pub b_needs_power: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x190)]
    pub i_temp_power_cap: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x194)]
    pub i_temp_power_loss: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x198)]
    pub i_temp_divide_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19c)]
    pub i_lock_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a0)]
    pub lock_timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b4)]
    pub b_exploded: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b5)]
    pub b_occupied: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b6)]
    pub b_friendlies: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b8)]
    pub interior_image_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c0)]
    pub interior_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c8)]
    pub interior_image_on: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub interior_image_manned: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub interior_image_manned_fancy: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e0)]
    pub last_user_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e4)]
    pub i_bonus_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e8)]
    pub i_last_bonus_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1ec)]
    pub location: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f4)]
    pub bp_cost: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1f8)]
    pub flash_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x218)]
    pub max_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x21c)]
    pub i_battery_power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    pub i_hack_effect: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x224)]
    pub b_under_attack: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x225)]
    pub b_level_boostable: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x226)]
    pub b_trigger_ion: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x228)]
    pub damaging_effects: Vector<Animation>,
}

impl ShipSystemPrime {
    pub fn vtable(&self) -> &'static VtableShipSystem {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    /// Inherited from ShipObject
    pub i_ship_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub base1: Targetable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub base2: Collideable,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub v_system_list: Vector<*mut ShipSystem>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub oxygen_system: *mut OxygenSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub teleport_system: *mut TeleportSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub cloak_system: *mut CloakingSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub battery_system: *mut BatterySystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub mind_system: *mut MindSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub clone_system: *mut CloneSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub hacking_system: *mut HackingSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub show_network: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x79)]
    pub added_system: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub shield_system: *mut Shields,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub weapon_system: *mut WeaponSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub drone_system: *mut DroneSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub engine_system: *mut EngineSystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub medbay_system: *mut MedbaySystem,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub artillery_systems: Vector<*mut ArtillerySystem>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub v_crew_list: Vector<*mut CrewMember>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd8)]
    pub fire_spreader: Spreader<Fire>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub ship: Ship,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5a8)]
    pub status_messages: Queue<String>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5f8)]
    pub b_game_over: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x600)]
    pub current_target: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x608)]
    pub jump_timer: Pair<c_float, c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x610)]
    pub fuel_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x614)]
    pub hostile_ship: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x615)]
    pub b_destroyed: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x618)]
    pub i_last_damage: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x620)]
    pub jump_animation: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x640)]
    pub dam_messages: Vector<*mut DamageMessage>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x658)]
    pub system_key: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x670)]
    pub current_scrap: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x674)]
    pub b_jumping: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x675)]
    pub b_automated: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x678)]
    pub ship_level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x680)]
    pub my_blueprint: ShipBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8d0)]
    pub last_engine_status: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8d1)]
    pub last_jump_ready: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8d2)]
    pub b_contains_player_crew: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8d4)]
    pub i_intruder_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8d8)]
    pub crew_counts: Vector<Vector<c_int>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8f0)]
    pub temp_drone_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8f4)]
    pub temp_missile_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8f8)]
    pub explosions: Vector<Animation>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x910)]
    pub temp_vision: VectorBool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x938)]
    pub b_highlight_crew: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x940)]
    pub drone_trash: Vector<*mut Drone>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x958)]
    pub space_drones: Vector<*mut SpaceDrone>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x970)]
    pub new_drone_arrivals: Vector<*mut SpaceDrone>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x988)]
    pub bp_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98c)]
    pub i_customize_mode: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x990)]
    pub b_show_room: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x998)]
    pub super_barrage: Vector<*mut Projectile>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9b0)]
    pub b_invincible: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9b8)]
    pub super_drones: Vector<*mut SpaceDrone>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9d0)]
    pub highlight: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9d8)]
    pub failed_dodge_counter: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9e0)]
    pub hit_by_beam: Vector<c_float>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9f8)]
    pub enemy_damaged_uncloaked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9fc)]
    pub damage_cloaked: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa00)]
    pub killed_by_beam: Map<c_int, c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa30)]
    pub min_beacon_health: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa38)]
    pub fire_extinguishers: Vector<*mut ParticleEmitter>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa50)]
    pub b_was_safe: bool,
}

impl ShipManager {
    pub fn vtable(&self) -> &'static VtableShipManager {
        unsafe { xb(self.vtable).unwrap() }
    }
}

impl ShipManager {
    pub unsafe fn power_drone(
        &mut self,
        drone: *mut Drone,
        room_id: c_int,
        user_driven: bool,
        force: bool,
    ) -> bool {
        super::POWER_DRONE.call(ptr::addr_of_mut!(*self), drone, room_id, user_driven, force)
    }
    pub unsafe fn power_weapon(
        &mut self,
        weapon: *mut ProjectileFactory,
        user_driven: bool,
        force: bool,
    ) -> bool {
        super::POWER_WEAPON.call(ptr::addr_of_mut!(*self), weapon, user_driven, force)
    }
    pub unsafe fn depower_drone(&mut self, drone: *mut Drone, user_driven: bool) -> bool {
        super::DEPOWER_DRONE.call(ptr::addr_of_mut!(*self), drone, user_driven)
    }
    pub unsafe fn depower_weapon(
        &mut self,
        weapon: *mut ProjectileFactory,
        user_driven: bool,
    ) -> bool {
        super::DEPOWER_WEAPON.call(ptr::addr_of_mut!(*self), weapon, user_driven)
    }
    pub fn has_system(&self, system: System) -> bool {
        match system {
            System::Reactor => true,
            system => *self.system_key.get(system as usize).unwrap() != -1,
        }
    }
    pub fn shield_system(&self) -> Option<&Shields> {
        unsafe { xc(self.shield_system) }
    }
    pub fn engine_system(&self) -> Option<&EngineSystem> {
        unsafe { xc(self.engine_system) }
    }
    pub fn oxygen_system(&self) -> Option<&OxygenSystem> {
        unsafe { xc(self.oxygen_system) }
    }
    pub fn weapon_system(&self) -> Option<&WeaponSystem> {
        unsafe { xc(self.weapon_system) }
    }
    pub fn drone_system(&self) -> Option<&DroneSystem> {
        unsafe { xc(self.drone_system) }
    }
    pub fn medbay_system(&self) -> Option<&MedbaySystem> {
        unsafe { xc(self.medbay_system) }
    }
    pub fn teleport_system(&self) -> Option<&TeleportSystem> {
        unsafe { xc(self.teleport_system) }
    }
    pub fn teleport_system_mut(&mut self) -> Option<&mut TeleportSystem> {
        unsafe { xm(self.teleport_system) }
    }
    pub fn cloak_system(&self) -> Option<&CloakingSystem> {
        unsafe { xc(self.cloak_system) }
    }
    pub fn cloak_system_mut(&mut self) -> Option<&mut CloakingSystem> {
        unsafe { xm(self.cloak_system) }
    }
    pub fn artillery_systems(&self) -> impl Iterator<Item = &ArtillerySystem> {
        self.artillery_systems
            .iter()
            .map(|x| unsafe { xc(*x).unwrap() })
    }
    pub fn battery_system(&self) -> Option<&BatterySystem> {
        unsafe { xc(self.battery_system) }
    }
    pub fn battery_system_mut(&mut self) -> Option<&mut BatterySystem> {
        unsafe { xm(self.battery_system) }
    }
    pub fn clone_system(&self) -> Option<&CloneSystem> {
        unsafe { xc(self.clone_system) }
    }
    pub fn mind_system(&self) -> Option<&MindSystem> {
        unsafe { xc(self.mind_system) }
    }
    pub fn mind_system_mut(&mut self) -> Option<&mut MindSystem> {
        unsafe { xm(self.mind_system) }
    }
    pub fn hacking_system(&self) -> Option<&HackingSystem> {
        unsafe { xc(self.hacking_system) }
    }
    pub fn hacking_system_mut(&mut self) -> Option<&mut HackingSystem> {
        unsafe { xm(self.hacking_system) }
    }
    pub fn system(&self, system: System) -> Option<&ShipSystem> {
        let key = *self.system_key.get(system as usize).unwrap();
        (key >= 0).then(|| unsafe { xc(*self.v_system_list.get(key as usize).unwrap()).unwrap() })
    }
    pub fn system_mut(&mut self, system: System) -> Option<&mut ShipSystem> {
        let key = *self.system_key.get(system as usize).unwrap();
        (key >= 0).then(|| unsafe { xm(*self.v_system_list.get(key as usize).unwrap()).unwrap() })
    }
    pub fn systems(&self) -> impl Iterator<Item = &ShipSystem> {
        self.v_system_list
            .iter()
            .map(|x| unsafe { xc(*x).unwrap() })
    }
    pub fn has_crew(&self, name: &str) -> bool {
        self.v_crew_list
            .iter()
            .map(|x| unsafe { xc(*x).unwrap() })
            .any(|x| !x.b_dead && x.blueprint.name.to_str() == name)
    }
    pub fn drone_count(&self) -> c_int {
        if self.has_system(System::Drones) {
            self.drone_system().unwrap().drone_count
        } else {
            self.temp_drone_count
        }
    }
    pub fn missile_count(&self) -> c_int {
        if self.has_system(System::Weapons) {
            self.weapon_system().unwrap().missile_count
        } else {
            self.temp_missile_count
        }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ShipStatus {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub size: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub ship: *mut ShipManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub combat: *mut CombatControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub hull_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub hull_box_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub shield_box_on: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub shield_box_off: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub shield_box_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub shield_circle_charged: [*mut GL_Primitive; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x68)]
    pub shield_circle_uncharged: [*mut GL_Primitive; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub shield_circle_hacked: [*mut GL_Primitive; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub shield_circle_hacked_charged: [*mut GL_Primitive; 4],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub energy_shield_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub energy_shield_bar: [*mut GL_Primitive; 5],
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xf8)]
    pub hull_label: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x100)]
    pub hull_label_red: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x108)]
    pub shield_box_purple: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x110)]
    pub oxygen_purple: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x118)]
    pub evade_purple: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x120)]
    pub evade_oxygen_box: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x128)]
    pub evade_oxygen_box_top_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x130)]
    pub evade_oxygen_box_bottom_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub evade_oxygen_box_both_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub fuel_icon: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub missiles_icon: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x150)]
    pub drones_icon: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub scrap_icon: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub fuel_icon_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x168)]
    pub missiles_icon_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub drones_icon_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x178)]
    pub scrap_icon_red: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub health_mask: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub health_mask_texture: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x190)]
    pub last_health: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x194)]
    pub base_shield: Ellipse,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a4)]
    pub current_hover: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a8)]
    pub evade_oxygen_box_location: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b0)]
    pub last_fuel: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b4)]
    pub last_drones: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1b8)]
    pub last_scrap: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1bc)]
    pub last_missiles: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c0)]
    pub last_hull: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c8)]
    pub hull_message: *mut WarningWithLines,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d0)]
    pub shield_message: *mut WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1d8)]
    pub oxygen_message: *mut WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e0)]
    pub boarding_message: *mut WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1e8)]
    pub resource_messages: Vector<*mut DamageMessage>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x200)]
    pub no_money_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x220)]
    pub flash_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x240)]
    pub b_boss_fight: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x241)]
    pub b_enemy_ship: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x244)]
    pub no_ship_shift: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24c)]
    pub intruder_shift: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x254)]
    pub energy_shield_pos: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x25c)]
    pub intruder_pos: Point,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct CommandGui {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub ship_status: ShipStatus,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x268)]
    pub crew_control: CrewControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x490)]
    pub sys_control: SystemControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x588)]
    pub combat_control: CombatControl,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1728)]
    pub ftl_button: FTLButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18c8)]
    pub space_status: SpaceStatus,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1968)]
    pub star_map: *mut StarMap,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1970)]
    pub ship_complete: *mut CompleteShip,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1978)]
    pub focus_windows: Vector<*mut FocusWindow>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1990)]
    pub pause_text_loc: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1998)]
    pub pause_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19a0)]
    pub pause_image2: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19a8)]
    pub pause_image_auto: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19b0)]
    pub pause_crew_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19b8)]
    pub pause_doors_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19c0)]
    pub pause_hacking_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19c8)]
    pub pause_mind_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19d0)]
    pub pause_room_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19d8)]
    pub pause_target_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19e0)]
    pub pause_target_beam_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19e8)]
    pub pause_teleport_leave_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19f0)]
    pub pause_teleport_arrive_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x19f8)]
    pub flare_image: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a00)]
    pub ship_position: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a08)]
    pub location_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a10)]
    pub load_event: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a18)]
    pub load_sector: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1a20)]
    pub choice_box: ChoiceBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c38)]
    pub gameover: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c39)]
    pub already_won: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c3a)]
    pub out_of_fuel: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c40)]
    pub menu_box: MenuScreen,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20b0)]
    pub game_over_screen: GameOver,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2190)]
    pub options_box: OptionsScreen,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3290)]
    pub b_paused: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3291)]
    pub b_auto_paused: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3292)]
    pub menu_pause: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3293)]
    pub event_pause: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3294)]
    pub touch_pause: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3298)]
    pub touch_pause_reason: TouchPauseReason,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x32a0)]
    pub input_box: InputBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3300)]
    pub f_shake_timer: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3308)]
    pub ship_screens: TabbedWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3488)]
    pub store_screens: TabbedWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3608)]
    pub upgrade_screen: Upgrades,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38d8)]
    pub crew_screen: CrewManifest,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3d08)]
    pub equip_screen: Equipment,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4040)]
    pub new_location: *mut Location,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4048)]
    pub space: *mut SpaceManager,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4050)]
    pub upgrade_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40e0)]
    pub upgrade_warning: WarningMessage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x41c0)]
    pub store_button: TextButton,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x42c0)]
    pub options_button: Button,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4350)]
    pub pause_anim_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4354)]
    pub pause_animation: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4358)]
    pub store_trash: Vector<*mut Store>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4370)]
    pub flicker_timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4384)]
    pub show_timer: TimerHelper,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4398)]
    pub b_hide_ui: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43a0)]
    pub enemy_ship: *mut CompleteShip,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43a8)]
    pub wait_location: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43a9)]
    pub last_location_wait: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43aa)]
    pub danger_location: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43b0)]
    pub command_key: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43c8)]
    pub jump_complete: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43cc)]
    pub map_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x43d0)]
    pub leave_crew_dialog: ConfirmWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4650)]
    pub secret_sector: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4654)]
    pub active_touch: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4658)]
    pub active_touch_is_button: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4659)]
    pub active_touch_is_crew_box: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x465a)]
    pub active_touch_is_ship: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x465b)]
    pub active_touch_is_null: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4660)]
    pub extra_touches: Vector<c_int>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4678)]
    pub b_tutorial_was_running: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4679)]
    pub focus_ate_mouse: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x467a)]
    pub choice_box_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x467c)]
    pub system_details_width: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4680)]
    pub write_error_dialog: ChoiceBox,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4898)]
    pub suppress_write_error: bool,
}

impl CommandGui {
    pub fn enemy_ship(&self) -> Option<&CompleteShip> {
        unsafe { xc(self.enemy_ship) }
    }
    pub fn star_map(&self) -> Option<&StarMap> {
        unsafe { xc(self.star_map) }
    }
    pub fn star_map_mut(&mut self) -> Option<&mut StarMap> {
        unsafe { xm(self.star_map) }
    }
    pub fn ship_manager(&self) -> Option<&ShipManager> {
        unsafe { xc(self.crew_control.ship_manager) }
    }
    pub fn ship_manager_mut(&mut self) -> Option<&mut ShipManager> {
        unsafe { xm(self.crew_control.ship_manager) }
    }
    pub fn target_self_with_mind_control_error(&self, room_id: i32) -> Option<&'static str> {
        if !self.ship_manager().unwrap().has_system(System::Mind) {
            Some("the mind control system is not installed")
        } else if !self.ship_manager().unwrap().ship.get_room_blackout(room_id)
            || self.ship_manager().unwrap().has_crew("slug")
            || self.equip_screen.has_augment("LIFE_SCANNER")
        {
            None
        } else {
            // Cannot mind control in rooms you cannot detect life in.
            Some("the sensors don't detect life in the target room")
        }
    }
    pub fn mind_control_blocked(&self) -> bool {
        if !self.ship_manager().unwrap().has_system(System::Mind) {
            return false;
        }
        let mind = self.ship_manager().unwrap().mind_system().unwrap();
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub vtable: *const VtableFocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub b_open: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x9)]
    pub b_full_focus: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub close: Point,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub b_close_button_selected: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub position: Point,
}

impl FocusWindow {
    pub fn vtable(&self) -> &'static VtableFocusWindow {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Rect {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub x: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub y: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub w: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub h: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ChoiceText {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    /// type 0:   text #ffffff
    /// type 1:   text #969696
    /// selected: text #f3ff50
    /// type 2:   text #00c3ff
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub rewards: ResourceEvent,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WindowFrame {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub rect: Rect,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub outline: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub mask: *mut GL_Primitive,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub pattern: *mut GL_Primitive,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ChoiceBox {
    pub base: FocusWindow,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub text_box: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub box_: *mut WindowFrame,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub main_text: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub choices: Vector<ChoiceText>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x50)]
    pub column_size: c_uint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub choice_boxes: Vector<Rect>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub potential_choice: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x74)]
    pub selected_choice: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub font_size: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x7c)]
    pub centered: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub gap_size: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x84)]
    pub open_time: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub rewards: ResourceEvent,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x200)]
    pub current_text_color: GL_Color,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x210)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub title: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub short_title: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub description: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub cost: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x34)]
    pub rarity: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub base_rarity: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x3c)]
    pub bp: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub locked: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub tooltip: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x58)]
    pub tip: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Blueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub name: StdString,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub desc: Description,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub type_: c_int,
}

impl Blueprint {
    pub fn vtable(&self) -> &'static VtableBlueprint {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct TextString {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub data: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub image: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub fake: bool,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct BoostPower {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub amount: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub count: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct Damage {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub i_damage: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub i_shield_piercing: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub fire_chance: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub breach_chance: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub stun_chance: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub i_ion_damage: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub i_system_damage: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x1c)]
    pub i_pers_damage: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x20)]
    pub b_hull_buster: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x24)]
    pub owner_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub self_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2c)]
    pub b_lockdown: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2d)]
    pub crystal_shard: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x2e)]
    pub b_friendly_fire: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub i_stun: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct EffectsBlueprint {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub launch_sounds: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub hit_ship_sounds: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub hit_shield_sounds: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub miss_sounds: Vector<StdString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x60)]
    pub image: StdString,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct WeaponBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub name: StdString,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub desc: Description,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub type_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub damage: Damage,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb4)]
    pub shots: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub missiles: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xbc)]
    pub cooldown: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc0)]
    pub power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc4)]
    pub length: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc8)]
    pub speed: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xcc)]
    pub mini_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub effects: EffectsBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x138)]
    pub weapon_art: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x140)]
    pub combat_icon: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub explosion: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x150)]
    pub radius: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub mini_projectiles: Vector<MiniProjectile>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub boost_power: BoostPower,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x17c)]
    pub drone_targetable: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x180)]
    pub spin: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x184)]
    pub charge_levels: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x188)]
    pub flavor_type: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x198)]
    pub color: GL_Color,
}

impl WeaponBlueprint {
    pub fn vtable(&self) -> &'static VtableBlueprint {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub name: StdString,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub desc: Description,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub type_name: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x80)]
    pub level: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x84)]
    pub target_type: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub power: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8c)]
    pub cooldown: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x90)]
    pub speed: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x94)]
    pub dodge: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub weapon_blueprint: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa0)]
    pub drone_image: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xa8)]
    pub combat_icon: StdString,
}

impl DroneBlueprint {
    pub fn vtable(&self) -> &'static VtableBlueprint {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct AugmentBlueprint {
    /// Inherited from Blueprint
    pub vtable: *const VtableBlueprint,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub name: StdString,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub desc: Description,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x74)]
    pub value: c_float,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub stacking: bool,
}

impl AugmentBlueprint {
    pub fn vtable(&self) -> &'static VtableBlueprint {
        unsafe { xb(self.vtable).unwrap() }
    }
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
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub name: StdString,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub desc: Description,
    /// Inherited from Blueprint
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x70)]
    pub type_: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x78)]
    pub crew_name: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x88)]
    pub crew_name_long: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x98)]
    pub powers: Vector<TextString>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb0)]
    pub male: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xb8)]
    pub skill_level: Vector<Pair<c_int, c_int>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xd0)]
    pub color_layers: Vector<Vector<GL_Color>>,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xe8)]
    pub color_choices: Vector<c_int>,
}

impl CrewBlueprint {
    pub fn vtable(&self) -> &'static VtableBlueprint {
        unsafe { xb(self.vtable).unwrap() }
    }
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct HackingDrone {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub base: SpaceDrone,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x340)]
    pub starting_position: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x348)]
    pub drone_image_on: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x350)]
    pub drone_image_off: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x358)]
    pub light_image: *mut GL_Texture,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x360)]
    pub final_destination: Pointf,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x368)]
    pub arrived: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x369)]
    pub finished_setup: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x370)]
    pub flash_tracker: AnimationTracker,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x390)]
    pub flying: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x450)]
    pub extending: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x510)]
    pub explosion: Animation,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x5d0)]
    pub pref_room: c_int,
}

#[repr(C)]
#[derive(Debug, TestOffsets)]
pub struct ResourceEvent {
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x0)]
    pub missiles: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x4)]
    pub fuel: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x8)]
    pub drones: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0xc)]
    pub scrap: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x10)]
    pub crew: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14)]
    pub traitor: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x15)]
    pub cloneable: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x18)]
    pub clone_text: TextString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x28)]
    pub crew_type: StdString,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x30)]
    pub weapon: *const WeaponBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x38)]
    pub drone: *const DroneBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x40)]
    pub augment: *const AugmentBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x48)]
    pub crew_blue: CrewBlueprint,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x148)]
    pub system_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x14c)]
    pub weapon_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x150)]
    pub drone_count: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x154)]
    pub steal: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x155)]
    pub intruders: bool,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x158)]
    pub fleet_delay: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x15c)]
    pub hull_damage: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x160)]
    pub upgrade_amount: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x164)]
    pub upgrade_id: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x168)]
    pub upgrade_success_flag: c_int,
    #[cfg_attr(target_pointer_width = "64", test_offset = 0x170)]
    pub remove_item: StdString,
}

impl ResourceEvent {
    pub fn weapon(&self) -> Option<&WeaponBlueprint> {
        unsafe { xb(self.weapon) }
    }
    pub fn drone(&self) -> Option<&DroneBlueprint> {
        unsafe { xb(self.drone) }
    }
    pub fn augment(&self) -> Option<&AugmentBlueprint> {
        unsafe { xb(self.augment) }
    }
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
