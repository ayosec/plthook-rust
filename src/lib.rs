//! Bindings for the [plthook] library.
//!
//! This crates allows hooking library function calls in a running process.
//! Please see the description of the [plthook] library for more details.
//!
//! # Usage
//!
//! The main item in this crate is [`ObjectFile`]. Using its `open_*` functions
//! you can access to the PLT (Unix) or IAT (Windows) entries in the loaded
//! object files.
//!
//! ## Symbols in object files
//!
//! Use [`ObjectFile::symbols`] to get all symbols in the object file.
//!
//! ```
//! # let _fn = || -> Result<(), plthook::Error> {
//! # use plthook::ObjectFile;
//! let object = ObjectFile::open_main_program()?;
//! for symbol in object.symbols() {
//!     println!("{:?} {:?} {}", symbol.func_address, symbol.name, symbol.protection);
//! }
//! # Ok(()) };
//! ```
//!
//! ## Invoking functions
//!
//! The addresses yielded by [`ObjectFile::symbols`] can be used to invoke
//! functions directly.
//!
//! You have to cast the address to the correct function type.
//!
//! ```
//! # #[cfg(target_os = "linux")] {
//! # use plthook::ObjectFile;
//! let pid = std::process::id();
//!
//! let object = ObjectFile::open_main_program().unwrap();
//! let getpid_fn = object
//!     .symbols()
//!     .find(|sym| sym.name.to_str() == Ok("getpid"))
//!     .unwrap()
//!     .func_address as *const fn() -> libc::pid_t;
//!
//! assert_eq!(pid, unsafe { (*getpid_fn)() as u32 });
//! # }
//! ```
//!
//! ## Replacing functions
//!
//! [`ObjectFile::replace`] replaces an entry in the PLT table, and returns a
//! reference to the previous value.
//!
//! # Errors
//!
//! Errors are wrapped by the [`Error`] type. When an error is returned from
//! any [plthook] function, the message from the `plthook_error` function is
//! included in the [`Error`] instance.
//!
//! [plthook]: https://github.com/kubo/plthook
//! [`Symbol`]: crate::Symbol
//! [`ObjectFile`]: crate::ObjectFile
//! [`ObjectFile::symbols`]: crate::ObjectFile::symbols
//! [`ObjectFile::replace`]: crate::ObjectFile::replace
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
use std::rc::Rc;

use libc::c_void;

pub use errors::{Error, ErrorKind, Result};
pub use symbols::Symbol;

/// An [object file] loaded in memory.
///
/// Please see the [top-level documentation](crate) for more details.
///
/// [object file]: https://en.wikipedia.org/wiki/Object_file
pub struct ObjectFile(Rc<ObjectFileInner>);

/// Wrapper for the C object.
struct ObjectFileInner {
    c_object: ffi::plthook_t,
}

impl ObjectFile {
    /// New instance from the raw C object.
    fn new(c_object: ffi::plthook_t) -> ObjectFile {
        ObjectFile(Rc::new(ObjectFileInner { c_object }))
    }

    /// Load the object for the main program.
    pub fn open_main_program() -> Result<Self> {
        let res = unsafe { ffi::exts::open_cstr(ptr::null()) };
        res.map(ObjectFile::new)
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
        res.map(ObjectFile::new)
    }

    /// Load an object from a file.
    #[cfg(windows)]
    pub fn open_file<P: AsRef<Path>>(filename: P) -> Result<Self> {
        let res = ffi::exts::open_path_win32(filename.as_ref());
        res.map(ObjectFile::new)
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

        Ok(ObjectFile::new(object.assume_init()))
    }

    /// Replace the address of a symbol in the PLT section, and returns a
    /// reference to the previous entry. When this reference is dropped, the
    /// entry is restored to the previous value.
    ///
    /// The reference to the previous entry can be used to invoke the original
    /// function.
    ///
    /// # Safety
    ///
    /// The caller has to verify that the new address for the symbol is
    /// valid.
    ///
    /// The function is not thread-safe.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(target_os = "linux")] {
    /// use plthook::ObjectFile;
    /// use std::process;
    ///
    /// let pid = process::id();
    ///
    /// extern "C" fn broken_getpid() -> libc::pid_t {
    ///     -1
    /// }
    ///
    /// let replacement = unsafe {
    ///     ObjectFile::open_main_program()
    ///         .unwrap()
    ///         .replace("getpid", broken_getpid as *const _)
    ///         .unwrap()
    /// };
    ///
    /// assert_eq!(process::id(), u32::MAX);
    ///
    /// drop(replacement);
    /// assert_eq!(process::id(), pid);
    /// # }
    /// ```
    pub unsafe fn replace(
        &self,
        symbol_name: &str,
        func_address: *const c_void,
    ) -> Result<Replacement> {
        let symbol_name = match CString::new(symbol_name) {
            Ok(s) => s,
            Err(_) => {
                // If the name is not a valid C string, we assume that
                // there is no symbol with that name.
                return Err(Error::new(ErrorKind::FunctionNotFound, String::new()));
            }
        };

        let mut old_addr = MaybeUninit::uninit();
        ffi::exts::check(ffi::plthook_replace(
            self.0.c_object,
            symbol_name.as_ptr(),
            func_address,
            old_addr.as_mut_ptr(),
        ))?;

        Ok(Replacement {
            restore_ref: Some(RestoreRef {
                object: Rc::clone(&self.0),
                symbol_name,
            }),
            address: old_addr.assume_init(),
        })
    }

    /// Returns an iterator to get all symbols in the PLT section.
    ///
    /// # Example
    ///
    /// ```
    /// # let _fn = || -> Result<(), plthook::Error> {
    /// use plthook::ObjectFile;
    ///
    /// let object = ObjectFile::open_main_program()?;
    /// for symbol in object.symbols() {
    ///     println!("{:?} {:?}", symbol.func_address, symbol.name);
    /// }
    /// # Ok(()) };
    /// ```
    pub fn symbols(&self) -> impl Iterator<Item = Symbol> + '_ {
        symbols::iterator(self)
    }
}

impl Drop for ObjectFileInner {
    fn drop(&mut self) {
        unsafe {
            ffi::plthook_close(self.c_object);
        }
    }
}

/// A replacement of an entry in the PLT section.
///
/// The address in the PLT entry is restored when this value is dropped.
pub struct Replacement {
    restore_ref: Option<RestoreRef>,
    address: *const c_void,
}

/// Reference to restore a symbol when `Replacement` is dropped.
struct RestoreRef {
    object: Rc<ObjectFileInner>,
    symbol_name: CString,
}

impl Replacement {
    /// Returns the original address of the PLT entry.
    ///
    /// This address can be used to invoke the function replaced by
    /// [`ObjectFile::replace`].
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(target_os = "linux")] {
    /// use plthook::ObjectFile;
    /// use std::mem;
    ///
    /// extern "C" fn broken_getpid() -> libc::pid_t {
    ///     -1
    /// }
    ///
    /// let program = ObjectFile::open_main_program().unwrap();
    ///
    /// let pid = unsafe { libc::getpid() };
    /// assert_ne!(pid, -1);
    ///
    /// // Replace getpid with our broken function.
    ///
    /// let replacement = unsafe {
    ///     program.replace("getpid", broken_getpid as *const _).unwrap()
    /// };
    ///
    /// let libc_getpid: extern "C" fn() -> libc::pid_t = unsafe {
    ///     mem::transmute(replacement.original_address())
    /// };
    ///
    /// assert_eq!(unsafe { libc::getpid() }, -1);
    /// assert_eq!(unsafe { (libc_getpid)() }, pid);
    ///
    /// drop(replacement);
    /// assert_eq!(unsafe { libc::getpid() }, pid);
    /// # }
    /// ```
    pub fn original_address(&self) -> *const c_void {
        self.address
    }

    /// Discard this replacement, so the original address will not be restored
    /// when this replacement is dropped.
    pub fn discard(&mut self) {
        self.restore_ref = None;
    }
}

impl Drop for Replacement {
    fn drop(&mut self) {
        unsafe {
            if let Some(restore_ref) = self.restore_ref.take() {
                let _ = ffi::exts::check(ffi::plthook_replace(
                    restore_ref.object.c_object,
                    restore_ref.symbol_name.as_ptr(),
                    self.address,
                    ptr::null_mut(),
                ));
            }
        };
    }
}
