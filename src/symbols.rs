//! Iterator to get symbols with `plthook_enum`.

use crate::ffi::plthook_enum;
use libc::c_uint;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;

/// A symbol found in the PLT section.
///
/// Use [`ObjectFile::symbols`] to get them.
///
/// [`ObjectFile::symbols`]: crate::ObjectFile::symbols
///
/// # Using function addresses
///
/// The function address in [`Symbol`] can be used to invoke functions.
///
/// You have to cast the address to the correct function type.
///
/// ```
/// # #[cfg(target_os = "linux")] {
/// use plthook::ObjectFile;
///
/// let pid = std::process::id();
///
/// let object = ObjectFile::open_main_program().unwrap();
/// let getpid_fn = object
///     .symbols()
///     .find(|sym| sym.name.to_str() == Ok("getpid"))
///     .unwrap()
///     .func_address as *const fn() -> libc::pid_t;
///
/// assert_eq!(pid, unsafe { (*getpid_fn)() as u32 });
/// # }
/// ```
#[derive(Debug)]
pub struct Symbol {
    /// Name of the symbol.
    pub name: CString,

    /// Pointer to the address of the symbol.
    pub func_address: *const fn(),
}

pub(crate) fn iterator(object: &crate::ObjectFile) -> SymbolIterator {
    SymbolIterator { pos: 0, object }
}

pub(crate) struct SymbolIterator<'a> {
    pos: c_uint,
    object: &'a crate::ObjectFile,
}

impl<'a> Iterator for SymbolIterator<'a> {
    type Item = Symbol;

    fn next(&mut self) -> Option<Symbol> {
        let mut name = MaybeUninit::uninit();
        let mut func_address = MaybeUninit::uninit();

        let ret = unsafe {
            plthook_enum(
                self.object.c_object,
                &mut self.pos,
                name.as_mut_ptr(),
                func_address.as_mut_ptr() as *mut _,
            )
        };

        if ret != 0 {
            return None;
        }

        // The bytes from `name` are copied in an owned CString instance. In
        // most cases, the address can be considered 'static; however, we have
        // no guarantees.
        let name = unsafe { CStr::from_ptr(name.assume_init()).into() };
        let func_address = unsafe { func_address.assume_init() };

        Some(Symbol { name, func_address })
    }
}
