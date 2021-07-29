//! FFI bindings to the plthook library.

#![allow(non_camel_case_types)]

use libc::{c_char, c_int, c_uint, c_void};

pub(crate) type plthook_t = *const c_void;

extern "C" {
    pub(crate) fn plthook_open(object: *mut plthook_t, filename: *const c_char) -> c_int;

    pub(crate) fn plthook_open_by_handle(object: *mut plthook_t, handle: *const c_void) -> c_int;

    pub(crate) fn plthook_close(object: plthook_t) -> c_void;

    pub(crate) fn plthook_enum(
        object: plthook_t,
        pos: *mut c_uint,
        name_out: *mut *const c_char,
        addr_out: *mut *const *const c_void,
    ) -> c_int;

    pub(crate) fn plthook_replace(
        object: plthook_t,
        funcname: *const c_char,
        funcaddr: *const c_void,
        oldfunc: *mut *const c_void,
    ) -> c_int;

    pub(crate) fn plthook_error() -> *const c_char;
}

pub(crate) mod exts {
    use super::plthook_t;
    use crate::errors::{Error, Result};
    use std::mem::MaybeUninit;

    // Wrapper for the `plthook_open` function.
    //
    // # Safety
    //
    // `filename` has be a `NULL`-terminated string, or `NULL`.
    pub(crate) unsafe fn open_cstr(filename: *const libc::c_char) -> Result<plthook_t> {
        let mut c_object = MaybeUninit::uninit();
        match super::plthook_open(c_object.as_mut_ptr(), filename) {
            0 => Ok(c_object.assume_init()),
            e => Err(Error::from(e)),
        }
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
            return Err(Error::FileNotFound);
        }

        let mut object = MaybeUninit::uninit();
        let ret = unsafe {
            super::plthook_open_by_handle(object.as_mut_ptr(), handle.assume_init() as *const _)
        };

        match ret {
            0 => Ok(unsafe { object.assume_init() }),
            e => Err(Error::from(e)),
        }
    }
}
