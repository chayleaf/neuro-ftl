use std::sync::OnceLock;

use retour::GenericDetour;

#[macro_export]
macro_rules! cross_fn {
    {
        $vis:vis unsafe fn $name:ident($($arg:ident : $ty:ty $(,)?)?) $(-> $ret:ty)? $body:block
    } => {
        #[cfg(target_os = "linux")]
        $vis unsafe extern "C" fn $name($($arg: $ty)?) $(-> $ret)? {
            $body
        }
        #[cfg(target_os = "windows")]
        $vis unsafe extern "fastcall" fn $name($($arg: $ty)?) $(-> $ret)? {
            $body
        }
    };
    {
        $vis:vis unsafe fn $name:ident($arg0:ident : $ty0:ty $(, $arg:ident : $ty:ty)+ $(,)?) $(-> $ret:ty)? $body:block
    } => {
        #[cfg(target_os = "linux")]
        $vis unsafe extern "C" fn $name($arg0: $ty0, $($arg: $ty),+) $(-> $ret)? {
            $body
        }
        #[cfg(target_os = "windows")]
        $vis unsafe extern "fastcall" fn $name($arg0: $ty0, _: std::ffi::c_int, $($arg: $ty),+) $(-> $ret)? {
            $body
        }
    };
}

#[repr(transparent)]
pub struct Ptr<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, T>(pub *mut T);

impl<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, T>
    Ptr<WINDOWS_OFFSET, LINUX_OFFSET, T>
{
    #[cfg(target_os = "windows")]
    pub const OFFSET: usize = WINDOWS_OFFSET - 0x400000;
    #[cfg(target_os = "linux")]
    pub const OFFSET: usize = LINUX_OFFSET;
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(std::ptr::null_mut())
    }
    pub unsafe fn init(&mut self, base: *mut std::ffi::c_void) {
        self.0 = base.byte_add(Self::OFFSET).cast();
    }
}

macro_rules! impl_fns {
    ($(($x:ident, $y:ident, $name0:ident: $arg0: ident $(, $name:ident: $arg:ident)*),)*) => {
        $(
            #[repr(transparent)]
            #[cfg(target_os = "linux")]
            pub struct $x<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0, $($arg,)* R>(pub Option<unsafe extern "C" fn($arg0, $($arg),*) -> R>);

            #[repr(transparent)]
            #[cfg(target_os = "windows")]
            pub struct $x<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0, $($arg,)* R>(pub Option<unsafe extern "fastcall" fn($arg0, std::ffi::c_int, $($arg),*) -> R>);
            impl<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0, $($arg,)* R> $x<WINDOWS_OFFSET, LINUX_OFFSET, $arg0, $($arg,)* R> {
                #[cfg(target_os = "windows")]
                pub const OFFSET: usize = WINDOWS_OFFSET - 0x400000;
                #[cfg(target_os = "linux")]
                pub const OFFSET: usize = LINUX_OFFSET;
                #[allow(clippy::new_without_default)]
                pub const fn new() -> Self {
                    Self(None)
                }
                #[allow(clippy::missing_transmute_annotations)]
                pub unsafe fn init(&mut self, base: *mut std::ffi::c_void) {
                    self.0 = std::mem::transmute(base.byte_add(Self::OFFSET));
                }
                #[allow(clippy::too_many_arguments)]
                pub unsafe fn call(&self, $name0: $arg0, $($name: $arg),*) -> R {
                    #[cfg(target_os = "linux")]
                    {
                        (self.0.unwrap())($name0, $($name),*)
                    }
                    #[cfg(target_os = "windows")]
                    {
                        (self.0.unwrap())($name0, 0, $($name),*)
                    }
                }
            }

            #[repr(transparent)]
            #[cfg(target_os = "linux")]
            pub struct $y<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0: 'static, $($arg: 'static,)* R: 'static>(
                pub OnceLock<GenericDetour<unsafe extern "C" fn($arg0, $($arg),*) -> R>>,
            );

            #[repr(transparent)]
            #[cfg(target_os = "windows")]
            pub struct $y<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0: 'static, $($arg: 'static,)* R: 'static>(
                pub OnceLock<GenericDetour<unsafe extern "fastcall" fn($arg0, std::ffi::c_int, $($arg),*) -> R>>,
            );

            impl<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0: 'static, $($arg: 'static,)* R: 'static>
                $y<WINDOWS_OFFSET, LINUX_OFFSET, $arg0, $($arg,)* R>
            {
                #[cfg(target_os = "windows")]
                pub const OFFSET: usize = WINDOWS_OFFSET - 0x400000;
                #[cfg(target_os = "linux")]
                pub const OFFSET: usize = LINUX_OFFSET;
                #[allow(clippy::new_without_default)]
                pub const fn new() -> Self {
                    Self(OnceLock::new())
                }
                #[allow(clippy::missing_transmute_annotations)]
                pub unsafe fn init(
                    &mut self,
                    base: *mut std::ffi::c_void,
                    #[cfg(target_os = "windows")] hook: unsafe extern "fastcall" fn($arg0, std::ffi::c_int, $($arg),*) -> R,
                    #[cfg(target_os = "linux")] hook: unsafe extern "C" fn($arg0, $($arg),*) -> R,
                ) {
                    self.0.get_or_init(|| {
                        let mut func = $x::<WINDOWS_OFFSET, LINUX_OFFSET, $arg0, $($arg,)* R>::new();
                        log::info!("hooking {:x} at {:?}", Self::OFFSET, base.byte_add(Self::OFFSET));
                        func.init(base);
                        let detour = GenericDetour::new(func.0.unwrap(), hook)
                            .map_err(|err| format!("failed to hook {:x}: {err}", Self::OFFSET))
                            .unwrap();
                        detour.enable().map_err(|err| format!("failed to enable hook {:x}: {err}", Self::OFFSET)).unwrap();
                        detour
                    });
                }
                #[allow(clippy::too_many_arguments)]
                pub unsafe fn call(&self, $name0: $arg0, $($name: $arg),*) -> R {
                    #[cfg(target_os = "linux")]
                    {
                        self.0.get().unwrap().call($name0, $($name),*)
                    }
                    #[cfg(target_os = "windows")]
                    {
                        self.0.get().unwrap().call($name0, 0, $($name),*)
                    }
                }
            }
        )*
    };
}

#[repr(transparent)]
#[cfg(target_os = "linux")]
pub struct Fn1<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, T, R>(
    pub Option<unsafe extern "C" fn(T) -> R>,
);

#[repr(transparent)]
#[cfg(target_os = "windows")]
pub struct Fn1<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, T, R>(
    pub Option<unsafe extern "fastcall" fn(T) -> R>,
);
impl<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, A, R>
    Fn1<WINDOWS_OFFSET, LINUX_OFFSET, A, R>
{
    #[cfg(target_os = "windows")]
    pub const OFFSET: usize = WINDOWS_OFFSET - 0x400000;
    #[cfg(target_os = "linux")]
    pub const OFFSET: usize = LINUX_OFFSET;
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(None)
    }
    #[allow(clippy::missing_transmute_annotations)]
    pub unsafe fn init(&mut self, base: *mut std::ffi::c_void) {
        self.0 = std::mem::transmute(base.byte_add(Self::OFFSET));
    }
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn call(&self, a: A) -> R {
        (self.0.unwrap())(a)
    }
}

#[repr(transparent)]
#[cfg(target_os = "linux")]
pub struct Hook1<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, A: 'static, R: 'static>(
    pub OnceLock<GenericDetour<unsafe extern "C" fn(A) -> R>>,
);

#[repr(transparent)]
#[cfg(target_os = "windows")]
pub struct Hook1<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, A: 'static, R: 'static>(
    pub OnceLock<GenericDetour<unsafe extern "fastcall" fn(A) -> R>>,
);

impl<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, A: 'static, R: 'static>
    Hook1<WINDOWS_OFFSET, LINUX_OFFSET, A, R>
{
    #[cfg(target_os = "windows")]
    pub const OFFSET: usize = WINDOWS_OFFSET - 0x400000;
    #[cfg(target_os = "linux")]
    pub const OFFSET: usize = LINUX_OFFSET;
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }
    #[allow(clippy::missing_transmute_annotations)]
    pub unsafe fn init(
        &mut self,
        base: *mut std::ffi::c_void,
        #[cfg(target_os = "windows")] hook: unsafe extern "fastcall" fn(A) -> R,
        #[cfg(target_os = "linux")] hook: unsafe extern "C" fn(A) -> R,
    ) {
        self.0.get_or_init(|| {
            let mut func = Fn1::<WINDOWS_OFFSET, LINUX_OFFSET, A, R>::new();
            log::info!(
                "hooking {:x} at {:?}",
                Self::OFFSET,
                base.byte_add(Self::OFFSET)
            );
            func.init(base);
            let detour = GenericDetour::new(func.0.unwrap(), hook)
                .map_err(|err| format!("failed to hook {:x}: {err}", Self::OFFSET))
                .unwrap();
            detour
                .enable()
                .map_err(|err| format!("failed to enable hook {:x}: {err}", Self::OFFSET))
                .unwrap();
            detour
        });
    }
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn call(&self, a: A) -> R {
        self.0.get().unwrap().call(a)
    }
}

impl_fns!(
    // (Fn1, a: A),
    (Fn2, Hook2, a: A, b: B),
    (Fn3, Hook3, a: A, b: B, c: C),
    (Fn4, Hook4, a: A, b: B, c: C, d: D),
    (Fn5, Hook5, a: A, b: B, c: C, d: D, e: E),
    (Fn6, Hook6, a: A, b: B, c: C, d: D, e: E, f: F),
    (Fn7, Hook7, a: A, b: B, c: C, d: D, e: E, f: F, g: G),
    (Fn8, Hook8, a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H),
    (Fn9, Hook9, a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I),
);
