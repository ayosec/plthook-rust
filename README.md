# Rust bindings for plthook

This crates provides Rust bindings for the [plthook] library.

Please see the [API documentation] and the description in the [plthook] library
for more details.

[plthook]: https://github.com/kubo/plthook
[API documentation]: https://docs.rs/plthook

## Examples

To print symbols in the current process:

```rust
use plthook::ObjectFile;

fn main() {
    let object = ObjectFile::open_main_program().unwrap();

    for symbol in object.symbols() {
        println!("{:?} {:?}", symbol.func_address, symbol.name);
    }
}
```

To replace a symbol:

```rust
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
        let atoi_entry = object.replace("atoi", neg_atoi as *const _).unwrap();
        ATOI_FN = MaybeUninit::new(mem::transmute(atoi_entry.original_address()));
        atoi_entry.forget();
    };

    let i = unsafe { libc::atoi(b"100\0".as_ptr().cast()) };
    assert_eq!(i, -100);
}
```
