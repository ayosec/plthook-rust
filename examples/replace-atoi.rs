//! Replace atoi to return the negative value of the real atoi().

#![allow(clippy::missing_transmute_annotations)]

use plthook::ObjectFile;
use std::mem::{self, MaybeUninit};
use std::os::raw::{c_char, c_int};

static mut ATOI_FN: MaybeUninit<fn(*const c_char) -> c_int> = MaybeUninit::uninit();

extern "C" fn neg_atoi(nptr: *const c_char) -> c_int {
    let i = unsafe { (ATOI_FN.assume_init())(nptr) };
    -i
}

fn main() {
    let object = ObjectFile::open_main_program().expect("Failed to open main program");

    unsafe {
        let mut atoi_entry = object.replace("atoi", neg_atoi as *const _).unwrap();
        ATOI_FN = MaybeUninit::new(mem::transmute(atoi_entry.original_address()));
        atoi_entry.discard();
    };

    let i = unsafe { libc::atoi(b"100\0".as_ptr().cast()) };
    assert_eq!(i, -100);
}
