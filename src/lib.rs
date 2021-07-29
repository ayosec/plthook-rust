//! Bindings for the [plthook] library.
//!
//! [plthook]: https://github.com/kubo/plthook

mod errors;
mod ffi;
mod symbols;

#[cfg(test)]
mod tests;

use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::ptr;

use libc::c_void;

pub use errors::{Error, Result};
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
        plthook_open(ptr::null())
    }

    /// Load an object from a file.
    #[cfg(unix)]
    pub fn open_file<P: AsRef<std::path::Path>>(filename: P) -> Result<Self> {
        // This function is not available in Windows because the standard
        // library does not provide a way to convert `Path` to `CString`.

        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let filename_bytes = AsRef::<OsStr>::as_ref(filename.as_ref()).as_bytes();
        let filename = match CString::new(filename_bytes) {
            Ok(f) => f,
            Err(_) => {
                // If the string in filename can't be converted to a C string
                // we assume that it can't be possible to create a file with
                // that name.
                return Err(Error::FileNotFound);
            }
        };

        plthook_open(filename.as_ptr())
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
        let c_object = {
            let mut object = MaybeUninit::uninit();
            match ffi::plthook_open_by_handle(object.as_mut_ptr(), handle) {
                0 => (),
                e => return Err(Error::from(e)),
            }

            object.assume_init()
        };

        Ok(ObjectFile { c_object })
    }

    /// Replace the entry for a symbol in the PLT section, and returns
    /// the previous value.
    ///
    /// # Safety
    ///
    /// The caller has to verify that the new address for the symbol is
    /// valid.
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
                return Err(Error::FunctionNotFound);
            }
        };

        let mut old_addr = ptr::null();
        let ret = ffi::plthook_replace(
            self.c_object,
            symbol_name.as_ptr(),
            func_address,
            &mut old_addr as *mut _,
        );

        match ret {
            0 => Ok(old_addr),
            e => Err(Error::from(e)),
        }
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

// Wrapper for the `plthook_open` function.
fn plthook_open(filename: *const libc::c_char) -> Result<ObjectFile> {
    let c_object = unsafe {
        let mut object = MaybeUninit::uninit();
        match ffi::plthook_open(object.as_mut_ptr(), filename) {
            0 => (),
            e => return Err(Error::from(e)),
        }

        object.assume_init()
    };

    Ok(ObjectFile { c_object })
}

/// Return the last error message from `plthook`, if any.
pub fn last_error_message() -> String {
    let errmsg = unsafe { CStr::from_ptr(ffi::plthook_error()) };
    errmsg.to_string_lossy().into_owned()
}
