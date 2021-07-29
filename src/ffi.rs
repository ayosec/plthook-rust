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

#[test]
fn replace_fn_in_main_program() {
    use std::mem::MaybeUninit;

    let object = unsafe {
        let mut object = MaybeUninit::uninit();

        let ret = plthook_open(object.as_mut_ptr(), std::ptr::null());
        assert_eq!(ret, 0);

        object.assume_init()
    };

    // Replace atoi with a function that always return 42.

    fn other_atoi(_: *const c_char) -> c_int {
        42
    }

    let original_func = unsafe {
        let mut fnaddr = MaybeUninit::<fn(*const c_char) -> c_int>::uninit();
        let ret = plthook_replace(
            object,
            b"atoi\0".as_ptr().cast(),
            other_atoi as *const _,
            fnaddr.as_mut_ptr() as *mut _,
        );

        assert_eq!(ret, 0);

        fnaddr.assume_init()
    };

    let param = b"123\0".as_ptr().cast();

    assert_eq!(unsafe { libc::atoi(param) }, 42);
    assert_eq!((original_func)(param), 123);

    // Restore original atoi function.

    unsafe {
        let ret = plthook_replace(
            object,
            b"atoi\0".as_ptr().cast(),
            original_func as *const _,
            std::ptr::null_mut(),
        );

        assert_eq!(ret, 0);
    };

    assert_eq!(unsafe { libc::atoi(param) }, 123);

    unsafe { plthook_close(object) };
}
