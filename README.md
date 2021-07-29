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
use libc::{c_char, c_int};
use plthook::ObjectFile;
use std::mem::MaybeUninit;

static mut ATOI_FN: MaybeUninit<fn(*const c_char) -> c_int> = MaybeUninit::uninit();

fn neg_atoi(nptr: *const c_char) -> c_int {
    let i = unsafe { (ATOI_FN.assume_init())(nptr) };
    -i
}

fn main() {
    let object = ObjectFile::open_main_program().unwrap();

    unsafe {
        let atoi_fn = ATOI_FN.as_mut_ptr() as *mut _;
        *atoi_fn = object.replace("atoi", neg_atoi as *const _).unwrap();
    };

    let i = unsafe { libc::atoi(b"100\0".as_ptr().cast()) };
    assert_eq!(i, -100);
}
```
