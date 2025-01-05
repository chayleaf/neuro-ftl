#![allow(clippy::missing_safety_doc)]
use std::{
    collections::HashMap,
    ffi::{c_int, c_uint, c_void, CStr},
    mem, ptr,
    sync::OnceLock,
};

use bindings::{CApp, CrewMember, Door, Drone, ProjectileFactory, ShipManager, TabbedWindow};
use ctor::ctor;
use retour::GenericDetour;

pub mod bindings;
pub mod game;
pub mod pak;
pub mod xml;

#[allow(clippy::large_enum_variant)]
enum Blueprint {
    Drone(xml::DroneBlueprint),
    Weapon(xml::WeaponBlueprint),
    Augment(xml::AugBlueprint),
    System(xml::SystemBlueprint),
    Crew(xml::CrewBlueprint),
    Ship(xml::ShipBlueprint),
    Item(xml::ItemBlueprint),
}

struct TextLibrary {
    text: HashMap<String, String>,
    blueprints: HashMap<String, Blueprint>,
}

impl TextLibrary {
    pub fn new() -> Self {
        let exe = std::env::current_exe().unwrap();
        let dat_path = exe.parent().unwrap().join("ftl.dat");
        let mut file = pak::Data::open(dat_path).unwrap();
        let mut blueprints = HashMap::new();
        let mut text = HashMap::new();
        for k in file.file_list() {
            if k.starts_with("data/text_") {
                let contents = file.read(&k).unwrap();
                let contents = std::str::from_utf8(&contents)
                    .unwrap()
                    .replace("</event>-", "</event>")
                    .replace("/>.\r", "/>\r")
                    .replace(">1.f<", ">1.0<");
                let data: xml::XmlText = quick_xml::de::from_str(&contents).unwrap();
                for t in data.text {
                    if t.language.is_none() {
                        text.insert(t.name, t.value);
                    }
                }
            } else if k.contains("luepr") {
                let contents = file.read(&k).unwrap();
                let contents = std::str::from_utf8(&contents)
                    .unwrap()
                    .replace("</event>-", "</event>")
                    .replace("/>.\r", "/>\r")
                    .replace(">1.f<", ">1.0<");
                let data: xml::XmlBlueprints = quick_xml::de::from_str(&contents).unwrap();
                let xml::XmlBlueprints {
                    blueprint_lists: _,
                    aug_blueprints,
                    crew_blueprints,
                    item_blueprints,
                    ship_blueprints,
                    drone_blueprints,
                    system_blueprints,
                    weapon_blueprints,
                } = data;
                for b in aug_blueprints {
                    blueprints.insert(b.name.clone(), Blueprint::Augment(b));
                }
                for b in crew_blueprints {
                    blueprints.insert(b.name.clone(), Blueprint::Crew(b));
                }
                for b in item_blueprints {
                    blueprints.insert(b.name.clone(), Blueprint::Item(b));
                }
                for b in ship_blueprints {
                    blueprints.insert(b.name.clone(), Blueprint::Ship(b));
                }
                for b in drone_blueprints {
                    blueprints.insert(b.name.clone(), Blueprint::Drone(b));
                }
                for b in system_blueprints {
                    blueprints.insert(b.name.clone(), Blueprint::System(b));
                }
                for b in weapon_blueprints {
                    blueprints.insert(b.name.clone(), Blueprint::Weapon(b));
                }
            }
        }
        TextLibrary { text, blueprints }
    }
    fn text_str<'a>(&'a self, s: &'a xml::TextString) -> Option<&'a str> {
        if let Some(ret) = &s.contents {
            Some(ret)
        } else if let Some(ret) = s.id.as_ref().and_then(|s| self.text(s)) {
            Some(ret)
        } else if let Some(ret) = s.load.as_ref().and_then(|s| self.text(s)) {
            Some(ret)
        } else {
            None
        }
    }
    pub fn blueprint_name(&self, name: &str) -> Option<&str> {
        if let Some(blueprint) = self.blueprints.get(name) {
            match blueprint {
                Blueprint::Drone(b) => self.text_str(&b.title),
                Blueprint::Weapon(b) => self.text_str(&b.title),
                Blueprint::Augment(b) => self.text_str(&b.title),
                Blueprint::System(b) => self.text_str(&b.title),
                Blueprint::Crew(b) => self.text_str(&b.title),
                Blueprint::Ship(b) => b.title.as_ref().and_then(|t| self.text_str(t)),
                Blueprint::Item(b) => self.text_str(&b.title),
            }
        } else {
            None
        }
    }
    pub fn text(&self, text: &str) -> Option<&str> {
        self.text.get(text).map(|x| x.as_str())
    }
}

fn library() -> &'static TextLibrary {
    TEXT.get_or_init(TextLibrary::new)
}

static TEXT: OnceLock<TextLibrary> = OnceLock::new();

//fn

// hook input_update

/*

CApp
langChooser.bOpen -> we're in the lang chooser
!gameLogic -> can't do shit besides the lang chooser
menu.bOpen -> in main menu
menu.finalChoice
    2 => exit
    9 => continue (if save exists)
    1 => new game
    anything else => new game without tutorial?

menu.bCreditScreen => credits open
menu.bSelectSave => menu.confirmNewGame
menu.changelog.bOpen => menu.changelog
menu.optionScreen.bOpen => menu.optionScreen
menu.shipBuilder.bOpen => menu.shipBuilder
menu.bScoreScreen => ScoreKeeper::Keeper

continueButton.bActive => can continue
startButton helpButton statButton optionsButton creditsButton quitButton

otherwise, CommandGui
gui->bPaused || gui->menu_pause is IsPaused
0 => RESTART_EASY
1 => RESTART_TUTORIAL
2 => QUIT
// 3 => OPTIONS
// 4 => CONTINUE
5 => MAIN_MENU
6 => HANGAR
// 7 => STATS
8 => SAVE_QUIT
// 9 => LOAD_GAME
// 10 => CONTROLS

 */

static mut GEN_INPUT_EVENTS: OnceLock<GenericDetour<unsafe extern "C" fn(*mut CApp)>> =
    OnceLock::new();

/*static mut POPULATE_GRID: OnceLock<GenericDetour<unsafe extern "C" fn(*mut StarMap, Point)>> =
OnceLock::new();*/

/*static mut RANDOM: OnceLock<GenericDetour<unsafe extern "C" fn() -> gg>> =
OnceLock::new();*/

pub unsafe extern "C" fn gen_input_events_hook(app: *mut CApp) {
    GEN_INPUT_EVENTS.get().unwrap_unchecked().call(app);
    game::loop_hook(app);
}

/*pub unsafe extern "C" fn populate_grid_hook(star_map: *mut StarMap, point: Point) {
    GEN_INPUT_EVENTS.get().unwrap_unchecked().call(app);
    game::loop_hook(app);
}*/

static mut CHEATS: *mut u8 = ptr::null_mut();

static mut POWER_MANAGERS: *mut bindings::Vector<bindings::PowerManager> = ptr::null_mut();

static mut SHIP_GRAPHS: *mut bindings::Vector<bindings::ShipGraph> = ptr::null_mut();

static mut POWER_DRONE: Option<
    extern "C" fn(*mut ShipManager, *mut Drone, c_int, bool, bool) -> bool,
> = None;
static mut DEPOWER_DRONE: Option<extern "C" fn(*mut ShipManager, *mut Drone, bool) -> bool> = None;
static mut POWER_WEAPON: Option<
    extern "C" fn(*mut ShipManager, *mut ProjectileFactory, bool, bool) -> bool,
> = None;
static mut DEPOWER_WEAPON: Option<
    extern "C" fn(*mut ShipManager, *mut ProjectileFactory, bool) -> bool,
> = None;
static mut DOOR_CLOSE: Option<extern "C" fn(*mut Door)> = None;
static mut DOOR_OPEN: Option<extern "C" fn(*mut Door)> = None;
static mut MOVE_CREW: Option<extern "C" fn(*mut CrewMember, c_int, c_int, bool) -> bool> = None;
static mut SET_TAB: Option<extern "C" fn(*mut TabbedWindow, c_uint)> = None;

unsafe fn hook(base: *mut c_void) {
    // enable cheats
    CHEATS = base.byte_add(0xA434C0).byte_add(0x18).cast::<u8>();
    *CHEATS = 1;
    POWER_MANAGERS = base.byte_add(0xA47C70).cast();
    SHIP_GRAPHS = base.byte_add(0xA3BB10).cast();
    #[allow(clippy::missing_transmute_annotations)]
    {
        POWER_DRONE = mem::transmute(base.byte_add(0x4C78B0));
        DEPOWER_DRONE = mem::transmute(base.byte_add(0x4BC5E0));
        POWER_WEAPON = mem::transmute(base.byte_add(0x4BB730));
        DEPOWER_WEAPON = mem::transmute(base.byte_add(0x4BB770));
        DOOR_OPEN = mem::transmute(base.byte_add(0x498C70));
        DOOR_CLOSE = mem::transmute(base.byte_add(0x498BB0));
        MOVE_CREW = mem::transmute(base.byte_add(0x4855E0));
        SET_TAB = mem::transmute(base.byte_add(0x585BE0));
    }
    let gen_input_events = base.byte_add(0x41C490);
    if std::slice::from_raw_parts(gen_input_events as *const u8, 8) != b"USH\x89\xfbH\x83\xec" {
        log::error!(
            "mismatch: {:?}",
            std::slice::from_raw_parts(gen_input_events as *const u8, 8)
        );
        return;
    }
    log::debug!("hooking at {gen_input_events:?}");
    // void __fastcall CApp__GenInputEvents(CApp *const this);
    GEN_INPUT_EVENTS.get_or_init(|| {
        let hook = match GenericDetour::new(
            std::mem::transmute::<*mut std::ffi::c_void, Option<unsafe extern "C" fn(*mut CApp)>>(
                gen_input_events,
            )
            .unwrap(),
            gen_input_events_hook,
        ) {
            Ok(hook) => hook,
            Err(err) => {
                panic!("hook creation error: {err}");
            }
        };
        match hook.enable() {
            Ok(()) => log::info!("hook enabld"),
            Err(err) => {
                log::error!("hook error: {err}");
            }
        }
        hook
    });
}

#[ctor]
unsafe fn init() {
    env_logger::init();
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            log::error!("failed to get current exe: {err}");
            return;
        }
    };
    let Some(stem) = exe.file_stem() else {
        log::debug!("no file stem");
        return;
    };
    let Some(stem) = stem.to_str() else {
        log::debug!("stem not utf-8");
        return;
    };
    if stem != "FTL" {
        // log::debug!("stem is {stem}, not FTL");
        return;
    }
    unsafe extern "C" fn callback(
        info: *mut libc::dl_phdr_info,
        _size: usize,
        _data: *mut libc::c_void,
    ) -> i32 {
        if (*info).dlpi_name.is_null() {
            return 0;
        }
        let s = CStr::from_ptr((*info).dlpi_name);
        if !s.is_empty() {
            return 0;
        }
        let addr = (*info).dlpi_addr as *mut c_void;
        hook(addr);
        0
    }
    if libc::dl_iterate_phdr(Some(callback), ptr::null_mut()) != 0 {
        log::error!("dl_iterate_phdr error: {}", std::io::Error::last_os_error());
    }
}
