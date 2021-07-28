//! Bindings for the [plthook] library.
//!
//! [plthook]: https://github.com/kubo/plthook

mod errors;
mod ffi;

use std::ffi::CString;
use std::mem::MaybeUninit;

pub use errors::{Error, Result};

/// An [object file] loaded in memory.
///
/// [object file]: https://en.wikipedia.org/wiki/Object_file
pub struct ObjectFile {
    object: ffi::plthook_t,
}

impl ObjectFile {
    /// Load the object for the main program.
    pub fn open_main_program() -> Result<Self> {
        plthook_open(std::ptr::null())
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
    pub unsafe fn open_by_handle(handle: *const libc::c_void) -> Result<Self> {
        let object = unsafe {
            let mut object = MaybeUninit::uninit();
            match ffi::plthook_open_by_handle(object.as_mut_ptr(), handle) {
                0 => (),
                e => return Err(Error::from(e)),
            }

            object.assume_init()
        };

        Ok(ObjectFile { object })
    }
}

// Wrapper for the `plthook_open` function.
fn plthook_open(filename: *const libc::c_char) -> Result<ObjectFile> {
    let object = unsafe {
        let mut object = MaybeUninit::uninit();
        match ffi::plthook_open(object.as_mut_ptr(), filename) {
            0 => (),
            e => return Err(errors::Error::from(e)),
        }

        object.assume_init()
    };

    Ok(ObjectFile { object })
}
