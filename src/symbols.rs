/// Iterator to get symbols with `plthook_enum`.
use crate::ffi::plthook_enum;
use libc::{c_uint, c_void};
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;

/// A symbol found in the PLT section.
///
/// Use [`ObjectFile::symbols`] to get them.
///
/// [`ObjectFile::symbols`]: crate::ObjectFile::symbols
#[derive(Debug)]
pub struct Symbol {
    /// Name of the symbol.
    pub name: CString,

    /// Pointer to the pointer with the address of the symbol.
    pub func_address: *const *const c_void,
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
        let mut func_address = std::ptr::null();

        let ret = unsafe {
            plthook_enum(
                self.object.c_object,
                &mut self.pos,
                name.as_mut_ptr(),
                &mut func_address,
            )
        };

        if ret != 0 {
            return None;
        }

        // The bytes from `name` are copied in an owned CString instance. In
        // most cases, the address can be considered 'static; however, we have
        // no guarantees.
        let name = unsafe { CStr::from_ptr(name.assume_init()).into() };

        Some(Symbol { name, func_address })
    }
}
