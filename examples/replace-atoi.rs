//! Replace atoi to return the negative value of the real atoi().

use libc::{c_char, c_int};
use plthook::ObjectFile;
use std::mem::MaybeUninit;

static mut ATOI_FN: MaybeUninit<fn(*const c_char) -> c_int> = MaybeUninit::uninit();

fn neg_atoi(nptr: *const c_char) -> c_int {
    let i = unsafe { (ATOI_FN.assume_init())(nptr) };
    -i
}

fn main() {
    let object = ObjectFile::open_main_program().expect("Failed to open main program");

    unsafe {
        let atoi_fn = ATOI_FN.as_mut_ptr() as *mut _;
        *atoi_fn = object.replace("atoi", neg_atoi as *const _).unwrap();
    };

    let i = unsafe { libc::atoi(b"100\0".as_ptr().cast()) };
    assert_eq!(i, -100);
}
