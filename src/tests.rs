use crate::ffi::*;
use crate::ObjectFile;
use libc::{c_char, c_double, c_int};
use std::mem::MaybeUninit;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref MUTEX: Mutex<()> = Mutex::new(());
}

#[test]
fn replace_atof() {
    fn other_atof(_: *const c_char) -> c_double {
        42.0
    }

    let lock = MUTEX.lock().unwrap();

    let param = b"100\0".as_ptr().cast();

    let object = ObjectFile::open_main_program().unwrap();

    assert_eq!(unsafe { libc::atof(param) as u64 }, 100);

    let initial_atof = unsafe { object.replace("atof", other_atof as *const _).unwrap() };

    assert_eq!(unsafe { libc::atof(param) as u64 }, 42);

    unsafe { object.replace("atof", initial_atof as *const _).unwrap() };

    assert_eq!(unsafe { libc::atof(param) as u64 }, 100);

    drop(lock);
}

#[test]
fn use_c_api() {
    let lock = MUTEX.lock().unwrap();

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

    drop(lock);
}

#[cfg(not(target_os = "macos"))]
#[test]
fn open_shared_object() {
    #[cfg(unix)]
    let soname = "libc.so.6";

    #[cfg(windows)]
    let soname = "kernel32.dll";

    let object = ObjectFile::open_file(soname).unwrap();
    assert!(object.symbols().next().is_some());
}
