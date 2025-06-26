//! FFI bindings to the plthook library.

#![allow(non_camel_case_types)]

use libc::{c_char, c_int, c_uint, c_void};

pub(crate) type plthook_t = *const c_void;

extern "C" {
    pub(crate) fn plthook_open(object: *mut plthook_t, filename: *const c_char) -> c_int;

    pub(crate) fn plthook_open_by_handle(object: *mut plthook_t, handle: *const c_void) -> c_int;

    pub(crate) fn plthook_close(object: plthook_t) -> c_void;

    #[cfg(not(windows))]
    pub(crate) fn plthook_enum_with_prot(
        object: plthook_t,
        pos: *mut c_uint,
        name_out: *mut *const c_char,
        addr_out: *mut *const *const c_void,
        prot: *mut c_int,
    ) -> c_int;

    pub(crate) fn plthook_replace(
        object: plthook_t,
        funcname: *const c_char,
        funcaddr: *const c_void,
        oldfunc: *mut *const c_void,
    ) -> c_int;

    pub(crate) fn plthook_error() -> *const c_char;
}

#[cfg(windows)]
pub(crate) unsafe fn plthook_enum_with_prot(
    object: plthook_t,
    pos: *mut c_uint,
    name_out: *mut *const c_char,
    addr_out: *mut *const *const c_void,
    _prot: *mut c_int,
) -> c_int {
    extern "C" {
        fn plthook_enum(
            object: plthook_t,
            pos: *mut c_uint,
            name_out: *mut *const c_char,
            addr_out: *mut *const *const c_void,
        ) -> c_int;
    }

    plthook_enum(object, pos, name_out, addr_out)
}

pub(crate) mod exts {
    use super::plthook_t;
    use crate::errors::{Error, ErrorKind, Result};
    use std::ffi::CStr;
    use std::mem::MaybeUninit;

    // Wrapper for the `plthook_open` function.
    //
    // # Safety
    //
    // `filename` has be a `NULL`-terminated string, or `NULL`.
    pub(crate) unsafe fn open_cstr(filename: *const libc::c_char) -> Result<plthook_t> {
        let mut c_object = MaybeUninit::uninit();
        check(super::plthook_open(c_object.as_mut_ptr(), filename))?;
        Ok(c_object.assume_init())
    }

    // Wrapper for the `plthook_open` function.
    //
    // The current implementation of `plthook_open` uses `GetModuleHandleExA`
    // to load the file. To be able to use it, we have to convert the file name
    // in two steps:
    //
    // 1. To a UTF-16 string, with `OsStrExt::encode_wide`.
    // 2. Then, to an ANSI string with `WideCharToMultiByte`.
    //
    // It is possible that final string can't represent the exact same string,
    // depending on system code page. Though this is very unlikely.
    //
    // Instead of using the `plthook_open` from the C library, we reimplement
    // it with `GetModuleHandleExW`, so we can use the UTF-16 string, instead
    // of the ANSI string equivalent.
    #[cfg(windows)]
    pub(crate) fn open_path_win32<S>(filename: S) -> Result<plthook_t>
    where
        S: AsRef<std::ffi::OsStr>,
    {
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::libloaderapi as l;

        let mut filename: Vec<u16> = filename.as_ref().encode_wide().collect();
        filename.push(0);

        let mut handle = MaybeUninit::uninit();

        let success = unsafe {
            l::GetModuleHandleExW(
                l::GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                filename.as_ptr(),
                handle.as_mut_ptr(),
            )
        };

        if success == 0 {
            return Err(Error::new(ErrorKind::FileNotFound, String::new()));
        }

        let mut object = MaybeUninit::uninit();
        unsafe {
            check(super::plthook_open_by_handle(
                object.as_mut_ptr(),
                handle.assume_init() as *const _,
            ))?
        };

        Ok(unsafe { object.assume_init() })
    }

    // Check if the response from a C function succeeded.
    pub(crate) fn check(ret: libc::c_int) -> Result<()> {
        if ret == 0 {
            return Ok(());
        }

        // Copy the error message from the library.
        let msg_ptr = unsafe { CStr::from_ptr(super::plthook_error()) };
        let errmsg = msg_ptr.to_string_lossy().into_owned();

        Err(Error::new(ErrorKind::from(ret), errmsg))
    }
}
