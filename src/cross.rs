pub struct Ptr<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, T>(pub *mut T);

impl<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, T>
    Ptr<WINDOWS_OFFSET, LINUX_OFFSET, T>
{
    #[cfg(target_os = "windows")]
    pub const OFFSET: usize = WINDOWS_OFFSET;
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
    ($(($x:ident, $name0:ident: $arg0: ident $(, $name:ident: $arg:ident)*),)*) => {
        $(
            #[repr(transparent)]
            #[cfg(target_os = "linux")]
            pub struct $x<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0, $($arg,)* R>(pub Option<unsafe extern "C" fn($arg0, $($arg),*) -> R>);

            #[cfg(target_os = "windows")]
            pub struct $x<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0, $($arg,)* R>(pub Option<unsafe extern "fastcall" fn($arg0, std::ffi::c_int, $($arg),*) -> R>);
            impl<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0, $($arg,)* R> $x<WINDOWS_OFFSET, LINUX_OFFSET, $arg0, $($arg,)* R> {
                #[cfg(target_os = "windows")]
                pub const OFFSET: usize = WINDOWS_OFFSET;
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
                        (self.0.unwrap_unchecked())($name0, $($name),*)
                    }
                    #[cfg(target_os = "windows")]
                    {
                        (self.0.unwrap_unchecked())($name0, 0, $($name),*)
                    }
                }
            }
        )*
    };
}

/*macro_rules! impl_hooks {
    ($(($x:ident, $name0:ident: $arg0: ident $(, $name:ident: $arg:ident)*),)*) => {
        $(
            #[repr(transparent)]
            #[cfg(target_os = "linux")]
            pub struct $x<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0, $($arg,)* R, T: Fn(&Self, $arg0 $(, $arg)*) -> R>(pub OnceLock<GenericDetour<unsafe extern "C" fn($arg0, $($arg),*) -> R>>);

            #[cfg(target_os = "windows")]
            pub struct $x<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0, $($arg,)* R, T: Fn(&Self, $arg0 $(, $arg)*) -> R>(pub OnceLock<GenericDetour<unsafe extern "fastcall" fn($arg0, c_int, $($arg),*) -> R>>);
            impl<const WINDOWS_OFFSET: usize, const LINUX_OFFSET: usize, $arg0, $($arg,)* R, T: Fn(&Self, $arg0 $(, $arg)*) -> R> $x<WINDOWS_OFFSET, LINUX_OFFSET, $arg0, $($arg,)* R> {
                #[cfg(target_os = "windows")]
                pub const OFFSET: usize = WINDOWS_OFFSET;
                #[cfg(target_os = "linux")]
                pub const OFFSET: usize = LINUX_OFFSET;
                #[cfg(target_os = "windows")]
                pub unsafe extern "C" fn($arg0, $($arg),*) -> R {
                    T
                }
                #[cfg(target_os = "linux")]
                pub const OFFSET: usize = LINUX_OFFSET;
                pub const fn new() -> Self {
                    Self(None)
                }
                pub unsafe fn init(&mut self, base: *mut std::ffi::c_void) {
                    self.0 = std::mem::transmute(base.byte_add(Self::OFFSET));
                }
            }
            #[allow(clippy::too_many_arguments)]
            pub unsafe fn call(&self, $name0: $arg0, $($name: $arg),*) -> R {
                #[cfg(target_os = "linux")]
                {
                    self.0.get().unwrap_unchecked().call($name0, $($name),*)
                }
                #[cfg(target_os = "windows")]
                {
                    self.0.get().unwrap_unchecked().call($name0, 0, $($name),*)
                }
            }
        )*
    };
}*/
#[repr(transparent)]
#[cfg(target_os = "linux")]
pub struct Fn0<R>(pub Option<unsafe extern "C" fn() -> R>);
#[cfg(target_os = "windows")]
pub struct Fn0<R>(pub Option<unsafe extern "fastcall" fn() -> R>);
impl<R> Fn0<R> {
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn call(&self) -> R {
        (self.0.unwrap_unchecked())()
    }
}

impl_fns!(
    (Fn1, a: A),
    (Fn2, a: A, b: B),
    (Fn3, a: A, b: B, c: C),
    (Fn4, a: A, b: B, c: C, d: D),
    (Fn5, a: A, b: B, c: C, d: D, e: E),
    (Fn6, a: A, b: B, c: C, d: D, e: E, f: F),
    (Fn7, a: A, b: B, c: C, d: D, e: E, f: F, g: G),
    (Fn8, a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H),
    (Fn9, a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I),
);
