//! Bindings for the [plthook] library.
//!
//! # Errors
//!
//! Errors are wrapped by the [`Error`] type. When an error is returned from
//! any [plthook] function, the message from the `plthook_error` function is
//! included in the [`Error`] instance.
//!
//! [plthook]: https://github.com/kubo/plthook
//! [`Error`]: crate::Error

mod errors;
mod ffi;
mod symbols;

#[cfg(test)]
mod tests;

use std::ffi::CString;
use std::mem::MaybeUninit;
use std::path::Path;
use std::ptr;

use libc::c_void;

pub use errors::{Error, ErrorKind, Result};
pub use symbols::Symbol;

/// An [object file] loaded in memory.
///
/// [object file]: https://en.wikipedia.org/wiki/Object_file
pub struct ObjectFile {
    c_object: ffi::plthook_t,
}

impl ObjectFile {
    /// Load the object for the main program.
    pub fn open_main_program() -> Result<Self> {
        let res = unsafe { ffi::exts::open_cstr(ptr::null()) };
        res.map(|c_object| ObjectFile { c_object })
    }

    /// Load an object from a file.
    #[cfg(unix)]
    pub fn open_file<P: AsRef<Path>>(filename: P) -> Result<Self> {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let filename_bytes = AsRef::<OsStr>::as_ref(filename.as_ref()).as_bytes();
        let filename = match CString::new(filename_bytes) {
            Ok(f) => f,
            Err(_) => {
                // If the string in filename can't be converted to a C string
                // we assume that it can't be possible to create a file with
                // that name.
                return Err(Error::new(ErrorKind::FileNotFound, String::new()));
            }
        };

        let res = unsafe { ffi::exts::open_cstr(filename.as_ptr()) };
        res.map(|c_object| ObjectFile { c_object })
    }

    /// Load an object from a file.
    #[cfg(windows)]
    pub fn open_file<P: AsRef<Path>>(filename: P) -> Result<Self> {
        let res = ffi::exts::open_path_win32(filename.as_ref());
        res.map(|c_object| ObjectFile { c_object })
    }

    /// Load a dynamic loaded shared object.
    ///
    /// `handle` is the address of the shared object. This value can
    /// be obtained by a function like [`dlopen`].
    ///
    /// # Safety
    ///
    /// This constructor is unsafe because we don't check that the
    /// `handle` is a valid address.
    ///
    /// [`dlopen`]: https://docs.rs/libc/*/libc/fn.dlopen.html
    pub unsafe fn open_by_handle(handle: *const c_void) -> Result<Self> {
        let mut object = MaybeUninit::uninit();
        ffi::exts::check(ffi::plthook_open_by_handle(object.as_mut_ptr(), handle))?;

        Ok(ObjectFile {
            c_object: object.assume_init(),
        })
    }

    /// Replace the entry for a symbol in the PLT section, and returns
    /// the previous value.
    ///
    /// # Safety
    ///
    /// The caller has to verify that the new address for the symbol is
    /// valid.
    ///
    /// The function is not thread-safe.
    pub unsafe fn replace(
        &self,
        symbol_name: &str,
        func_address: *const c_void,
    ) -> Result<*const c_void> {
        let symbol_name = match CString::new(symbol_name) {
            Ok(s) => s,
            Err(_) => {
                // If the name is not a valid C string, we assume that
                // there is no symbol with that name.
                return Err(Error::new(ErrorKind::FunctionNotFound, String::new()));
            }
        };

        let mut old_addr = ptr::null();
        ffi::exts::check(ffi::plthook_replace(
            self.c_object,
            symbol_name.as_ptr(),
            func_address,
            &mut old_addr as *mut _,
        ))?;

        Ok(old_addr)
    }

    /// Returns an iterator to get all symbols in the PLT section.
    pub fn symbols(&self) -> impl Iterator<Item = Symbol> + '_ {
        symbols::iterator(self)
    }
}

impl Drop for ObjectFile {
    fn drop(&mut self) {
        unsafe {
            ffi::plthook_close(self.c_object);
        }
    }
}
