#![allow(clippy::missing_safety_doc)]
use std::{
    collections::HashMap,
    ffi::{c_int, c_uint, c_void, CStr},
    ptr,
    sync::OnceLock,
};

use bindings::{
    AchievementTracker, CApp, Collideable, CollisionResponse, CrewMember, Damage, Door, Drone,
    Pointf, Projectile, ProjectileFactory, ScoreKeeper, SettingValues, ShipBuilder, ShipManager,
    SpaceDrone, TabbedWindow, WeaponBlueprint,
};
use ctor::ctor;
use game::{activated, deactivate};

pub mod bindings;
pub mod cross;
pub mod game;
mod logger;
pub mod pak;
#[cfg(target_os = "windows")]
pub mod steam_shim;
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

fn keeper() -> &'static ScoreKeeper {
    unsafe { &*KEEPER.0 }
}

static TEXT: OnceLock<TextLibrary> = OnceLock::new();

static mut ACHIEVEMENTS: cross::Ptr<0x913780, 0xA37A20, AchievementTracker> = cross::Ptr::new();
static mut KEEPER: cross::Ptr<0x913980, 0xA38CA0, ScoreKeeper> = cross::Ptr::new();

// win: 916D20
static mut SETTINGS: cross::Ptr<0x916D20, 0xA434C0, SettingValues> = cross::Ptr::new();

// win: 91AB20
static mut POWER_MANAGERS: cross::Ptr<
    0x91AB20,
    0xA47C70,
    bindings::Vector<bindings::PowerManager>,
> = cross::Ptr::new();

// static mut SHIP_GRAPHS: *mut bindings::Vector<bindings::ShipGraph> = ptr::null_mut();

static mut POWER_DRONE: cross::Fn5<
    0x4ABA00,
    0x4C78B0,
    *mut ShipManager,
    *mut Drone,
    c_int,
    bool,
    bool,
    bool,
> = cross::Fn5::new();
// win:
static mut DEPOWER_DRONE: cross::Fn3<0x4A08F0, 0x4BC5E0, *mut ShipManager, *mut Drone, bool, bool> =
    cross::Fn3::new();
// win:
static mut POWER_WEAPON: cross::Fn4<
    0x49F010,
    0x4BB730,
    *mut ShipManager,
    *mut ProjectileFactory,
    bool,
    bool,
    bool,
> = cross::Fn4::new();
// win: 49F080
static mut DEPOWER_WEAPON: cross::Fn3<
    0x49F080,
    0x4BB770,
    *mut ShipManager,
    *mut ProjectileFactory,
    bool,
    bool,
> = cross::Fn3::new();
// win: 470DA0
static mut DOOR_CLOSE: cross::Fn1<0x470DA0, 0x498BB0, *mut Door, ()> = cross::Fn1::new();
// win: 470E70
static mut DOOR_OPEN: cross::Fn1<0x470E70, 0x498C70, *mut Door, ()> = cross::Fn1::new();
// win: 4809B0
static mut MOVE_CREW: cross::Fn4<0x4809B0, 0x4855E0, *mut CrewMember, c_int, c_int, bool, bool> =
    cross::Fn4::new();
// win: 517680
static mut SET_TAB: cross::Fn2<0x517680, 0x585BE0, *mut TabbedWindow, c_uint, ()> =
    cross::Fn2::new();
// win: 4EFA80
static mut SWITCH_SHIP: cross::Fn3<0x4EFA80, 0x54FD00, *mut ShipBuilder, c_int, c_int, ()> =
    cross::Fn3::new();

unsafe fn hook(base: *mut c_void) {
    ACHIEVEMENTS.init(base);
    KEEPER.init(base);
    SETTINGS.init(base);
    POWER_MANAGERS.init(base);
    POWER_DRONE.init(base);
    DEPOWER_DRONE.init(base);
    POWER_WEAPON.init(base);
    DEPOWER_WEAPON.init(base);
    DOOR_OPEN.init(base);
    DOOR_CLOSE.init(base);
    MOVE_CREW.init(base);
    SET_TAB.init(base);
    SWITCH_SHIP.init(base);

    // quick sanity check
    {
        let mut gen_input_events = cross::Ptr::<0x402AA0, 0x41C490, c_void>::new();
        gen_input_events.init(base);
        let gen_input_events = gen_input_events.0;
        if std::slice::from_raw_parts(gen_input_events as *const u8, 8) != {
            #[cfg(all(target_os = "linux", target_pointer_width = "64"))]
            {
                b"USH\x89\xfbH\x83\xec"
            }
            #[cfg(target_os = "windows")]
            b"W\x8d|$\x08\x83\xe4\xf0"
        } {
            log::error!(
                "mismatch: {:?}",
                std::slice::from_raw_parts(gen_input_events as *const u8, 8)
            );
            return;
        }
    }

    #[cfg(target_os = "linux")]
    {
        static mut CRIT_ERR_HDLR: cross::Hook3<
            0,
            0x422140,
            c_int,
            *mut libc::siginfo_t,
            *mut c_void,
            (),
        > = cross::Hook3::new();
        cross_fn! {
            unsafe fn crit_err_hdlr_hook(
                sig_num: c_int,
                info: *mut libc::siginfo_t,
                ucontext: *mut c_void
            ) {
                if activated() {
                    deactivate();
                } else {
                    CRIT_ERR_HDLR.call(sig_num, info, ucontext);
                }
            }
        }
        CRIT_ERR_HDLR.init(base, crit_err_hdlr_hook);
    }

    #[cfg(target_os = "windows")]
    {
        static mut EXC_FILTER: OnceLock<
            retour::GenericDetour<unsafe extern "system" fn(exceptioninfo: *const c_void) -> i32>,
        > = OnceLock::new();
        unsafe extern "system" fn exc_filter_hook(exceptioninfo: *const c_void) -> i32 {
            // EXCEPTION_POINTERS
            if activated() {
                deactivate();
                // EXCEPTION_CONTINUE_EXECUTION
                // -1
                // EXCEPTION_CONTINUE_SEARCH
                0
            } else {
                EXC_FILTER.get().unwrap().call(exceptioninfo)
            }
        }
        EXC_FILTER.get_or_init(|| {
            let addr = base.byte_add(0x4B5A00 - 0x400000);
            let func = std::mem::transmute::<
                *mut c_void,
                Option<unsafe extern "system" fn(exceptioninfo: *const c_void) -> i32>,
            >(addr);
            let detour = retour::GenericDetour::new(func.unwrap(), exc_filter_hook)
                .map_err(|err| format!("failed to hook 0x4B5A00: {err}"))
                .unwrap();
            detour
                .enable()
                .map_err(|err| format!("failed to enable hook 0x4B5A00: {err}"))
                .unwrap();
            detour
        });
    }

    static mut GEN_INPUT_EVENTS: cross::Hook1<0x402AA0, 0x41C490, *mut CApp, ()> =
        cross::Hook1::new();
    cross_fn! {
        unsafe fn gen_input_events_hook(app: *mut CApp) {
            GEN_INPUT_EVENTS.call(app);
            game::loop_hook(app);
        }
    }
    GEN_INPUT_EVENTS.init(base, gen_input_events_hook);

    static mut PROJECTILE_INIT: cross::Hook2<
        0x45A430,
        0x448F10,
        *mut Projectile,
        *const WeaponBlueprint,
        (),
    > = cross::Hook2::new();
    cross_fn! {
        unsafe fn projectile_init_hook(proj: *mut Projectile, bp: *const WeaponBlueprint) {
            PROJECTILE_INIT.call(proj, bp);
            game::projectile_post_init(proj, bp);
        }
    }
    PROJECTILE_INIT.init(base, projectile_init_hook);

    macro_rules! hook_proj_dtors {
        ($(($a:expr, $b:expr, $c:expr, $d:expr),)+) => {
            $({
                static mut PROJECTILE_DTOR: cross::Hook1<
                    $a,
                    $c,
                    *mut Projectile,
                    (),
                > = cross::Hook1::new();
                cross_fn! {
                    unsafe fn projectile_dtor_hook(proj: *mut Projectile) {
                        game::projectile_pre_dtor(proj);
                        PROJECTILE_DTOR.call(proj);
                    }
                }
                PROJECTILE_DTOR.init(base, projectile_dtor_hook);
                static mut PROJECTILE_DTOR2: cross::Hook1<
                    $b,
                    $d,
                    *mut Projectile,
                    (),
                > = cross::Hook1::new();
                cross_fn! {
                    unsafe fn projectile_dtor_hook2(proj: *mut Projectile) {
                        let proj1 = proj.byte_sub(std::mem::offset_of!(Projectile, base1));
                        game::projectile_pre_dtor(proj1);
                        PROJECTILE_DTOR2.call(proj);
                    }
                }
                PROJECTILE_DTOR2.init(base, projectile_dtor_hook2);
            })+
        };
    }
    hook_proj_dtors!(
        // format: windows, windows+8, linux, linux+8
        // Projectile
        (0x7562A0, 0x85D550, 0x44C1C0, 0x44C6D0),
        (0x755FF0, 0x85D290, 0x44CC00, 0x44D120),
        // BeamWeapon
        (0x7543E0, 0x85C6D0, 0x431220, 0x431CB0),
        (0x753DF0, 0x85C0E0, 0x430770, 0x431210),
        // BombProjectile
        (0x450D90, 0x4507E0, 0x433C20, 0x434160),
        (0x450AB0, 0x450510, 0x434170, 0x4346C0),
        // Missile
        (0x76AC60, 0x85DAC0, 0x4453C0, 0x4458D0),
        (0x76A9B0, 0x85D800, 0x4458E0, 0x445E00),
        // LaserBlast
        (0x755BE0, 0x85CF70, 0x44C6E0, 0x44CBF0),
        (0x755930, 0x85CCB0, 0x44E5D0, 0x44EAF0),
        // Asteroid
        (0x76B9C0, 0x85E7B0, 0x44D130, 0x44D640),
        (0x76B710, 0x85E4F0, 0x44DB70, 0x44E090),
        // CrewLaser
        (0x7764A0, 0x85ED10, 0x44D650, 0x44DB60),
        (0x7761F0, 0x85EA50, 0x44E0A0, 0x44E5C0),
        // PDSFire
        (0x76B340, 0x85E130, 0x44F240, 0x44F960),
        (0x76AF70, 0x85DD60, 0x44EB00, 0x44F230),
    );

    {
        static mut SHIP_MGR_COLL: cross::Hook5<
            0x4A5250,
            0x4BFE80,
            *mut ShipManager,
            Pointf,
            Pointf,
            Damage,
            bool,
            CollisionResponse,
        > = cross::Hook5::new();
        cross_fn! {
            unsafe fn ship_mgr_coll_hook(this: *mut ShipManager, start: Pointf, finish: Pointf, damage: Damage, raytrace: bool) -> CollisionResponse {
                let ret = SHIP_MGR_COLL.call(this, start, finish, damage, raytrace);
                game::ship_mgr_post_coll(this, start, finish, damage, raytrace, &ret);
                ret
            }
        }
        SHIP_MGR_COLL.init(base, ship_mgr_coll_hook);
    }
    {
        static mut DRONE_COLL: cross::Hook5<
            0x420C90,
            0x462610,
            *mut SpaceDrone,
            Pointf,
            Pointf,
            Damage,
            bool,
            CollisionResponse,
        > = cross::Hook5::new();
        cross_fn! {
            unsafe fn drone_coll_hook(this: *mut SpaceDrone, start: Pointf, finish: Pointf, damage: Damage, raytrace: bool) -> CollisionResponse {
                let ret = DRONE_COLL.call(this, start, finish, damage, raytrace);
                game::drone_post_coll(this, start, finish, damage, raytrace, &ret);
                ret
            }
        }
        DRONE_COLL.init(base, drone_coll_hook);
    }

    macro_rules! hook_proj_collchk {
        ($(($a:expr, $b:expr),)+) => {
            $({
                static mut PROJECTILE_COLLCHK: cross::Hook2<
                    $a,
                    $b,
                    *mut Projectile,
                    *mut Collideable,
                    (),
                > = cross::Hook2::new();
                cross_fn! {
                    unsafe fn projectile_collchk_hook(proj: *mut Projectile, obj: *mut Collideable) {
                        game::projectile_pre_collchk(proj, obj);
                        PROJECTILE_COLLCHK.call(proj, obj);
                        game::projectile_post_collchk(proj, obj);
                    }
                }
                PROJECTILE_COLLCHK.init(base, projectile_collchk_hook);
            })+
        };
    }
    hook_proj_collchk!(
        // Projectile
        (0x459550, 0x446D80),
        // BeamWeapon
        (0x438C30, 0x42F340),
        // BombProjectile
        (0x450080, 0x4335E0),
        // PDSFire
        (0x456EA0, 0x446730),
    );
    // sun: 4A4E80/4C0C70
    // pulsar: 4A36B0/4BD750
    {
        static mut SUN_DAMAGE: cross::Hook1<0x4A4E80, 0x4C0C70, *mut ShipManager, ()> =
            cross::Hook1::new();
        cross_fn! {
            unsafe fn sundmg_hook(mgr: *mut ShipManager) {
                game::sun_damage(mgr);
                SUN_DAMAGE.call(mgr);
                game::sun_damage(mgr);
            }
        }
        SUN_DAMAGE.init(base, sundmg_hook);
        static mut PULSAR_DAMAGE: cross::Hook1<0x4A36B0, 0x4BD750, *mut ShipManager, ()> =
            cross::Hook1::new();
        cross_fn! {
            unsafe fn pulsardmg_hook(mgr: *mut ShipManager) {
                game::pulsar_damage(mgr);
                PULSAR_DAMAGE.call(mgr);
                game::pulsar_damage(mgr);
            }
        }
        PULSAR_DAMAGE.init(base, pulsardmg_hook);
    }
}

#[cfg_attr(target_os = "linux", ctor)]
unsafe fn init() {
    logger::init();
    //println!("[MOD] stdout test");
    //eprintln!("[MOD] stdout test");
    // env_logger::init();
    #[cfg(target_os = "linux")]
    {
        // on Linux, do LD_PRELOAD stuff
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
        log::error!("logger test");
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
    #[cfg(target_os = "windows")]
    {
        log::error!("logger test");
        // let _ = konigsberg::SteamAPI_SteamApps_v009;
        let base: *mut c_void;
        std::arch::asm! {
            "mov eax, fs:[30h]",
            "mov {base}, [eax+8]",
            base = out(reg) base,
        }
        hook(base);
    }
}
