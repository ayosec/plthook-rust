//! Print all symbols in object files.
//!
//! It accepts shared objects as command line arguments.

use plthook::ObjectFile;
use std::env;

fn main() {
    let object = ObjectFile::open_main_program().expect("Failed to open main program");
    print_symbols(object);

    for path in env::args_os().skip(1) {
        println!("\n{:?}", path);
        match ObjectFile::open_file(path) {
            Ok(o) => print_symbols(o),
            Err(e) => eprintln!(
                "{}\nlast_error_message = {}",
                e,
                plthook::last_error_message()
            ),
        };
    }
}

fn print_symbols(object: ObjectFile) {
    for symbol in object.symbols() {
        println!("{:?} {:?}", symbol.func_address, symbol.name);
    }
}
