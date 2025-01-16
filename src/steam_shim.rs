/*SteamAPI_RegisterCallback
SteamAPI_GetHSteamUser
SteamAPI_UnregisterCallback
SteamAPI_RunCallbacks
SteamInternal_FindOrCreateUserInterface
SteamAPI_Init
SteamInternal_CreateInterface
SteamAPI_Shutdown
SteamInternal_ContextInit*/
use std::{
    os::raw::{c_char, c_int, c_void},
    sync::OnceLock,
};
use steamworks_sys::{CCallbackBase, HSteamUser};

macro_rules! reexport {
    (fn $name:ident($( $arg:ident : $type:ty ),*) $(-> $ret:ty)?) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name($( $arg : $type),*) $(-> $ret)? {
            static CELL: OnceLock<libloading::Symbol<unsafe extern "C" fn($($type),*) $(-> $ret)?>> = OnceLock::new();
            let sym = CELL.get_or_init(|| {
                sym(stringify!($name)).expect(&format!("failed to load symbol: {}", stringify!($name)))
            });
            sym($( $arg ),*)
        }
    };
}

reexport!(fn SteamAPI_RegisterCallback(pCallback : * mut CCallbackBase, iCallback : c_int));
reexport!(fn SteamAPI_GetHSteamUser() -> HSteamUser);
reexport!(fn SteamAPI_UnregisterCallback(pCallback : * mut CCallbackBase));
reexport!(fn SteamAPI_RunCallbacks());
reexport!(fn SteamInternal_FindOrCreateUserInterface(hSteamUser : HSteamUser, pszVersion : * const c_char) -> * mut c_void);
reexport!(fn SteamAPI_Init() -> bool);
reexport!(fn SteamInternal_CreateInterface(ver : * const c_char) -> * mut c_void);
reexport!(fn SteamAPI_Shutdown());
reexport!(fn SteamInternal_ContextInit(pContextInitData : * mut c_void) -> * mut c_void);

unsafe fn lib() -> &'static libloading::Library {
    static CELL: OnceLock<libloading::Library> = OnceLock::new();
    CELL.get_or_init(|| {
        super::init();
        unsafe {
            #[cfg(target_os = "windows")]
            {
                libloading::Library::new("./steam_api.orig.dll")
                    .or_else(|_| libloading::Library::new("./steam_api_orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api.orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api_orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api.orig"))
                    .or_else(|_| libloading::Library::new("steam_api_orig"))
            }
            #[cfg(target_os = "linux")]
            {
                libloading::Library::new("./libsteam_api.orig.so")
                    .or_else(|_| libloading::Library::new("./libsteam_api_orig.so"))
                    .or_else(|_| libloading::Library::new("libsteam_api.orig.so"))
                    .or_else(|_| libloading::Library::new("libsteam_api_orig.so"))
                    .or_else(|_| libloading::Library::new("steam_api.orig.so"))
                    .or_else(|_| libloading::Library::new("steam_api_orig.so"))
                    .or_else(|_| libloading::Library::new("steam_api.orig"))
                    .or_else(|_| libloading::Library::new("steam_api_orig"))
            }
        }
        .expect("failed to load steam api lib")
    })
}

unsafe fn sym<T>(name: &str) -> Result<libloading::Symbol<T>, libloading::Error> {
    lib().get(name.as_bytes())
}
