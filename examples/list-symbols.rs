//! Print all symbols in the object file.

fn main() {
    let object = plthook::ObjectFile::open_main_program().unwrap();

    for symbol in object.symbols() {
        println!("{:?} {:?}", symbol.func_address, symbol.name);
    }
}
