//! Print all symbols in object files.
//!
//! It accepts shared objects as command line arguments.

use std::{env, ffi::c_int, fmt, mem};

use plthook::ObjectFile;

fn main() {
    let object = ObjectFile::open_main_program().expect("Failed to open main program");
    print_symbols(object);

    for path in env::args_os().skip(1) {
        println!("\n{:?}", path);
        match ObjectFile::open_file(path) {
            Ok(o) => print_symbols(o),
            Err(e) => eprintln!("{}", e),
        };
    }
}

fn print_symbols(object: ObjectFile) {
    for symbol in object.symbols() {
        println!(
            "{:?} {:?} ({:?})",
            symbol.func_address,
            symbol.name,
            Prot(symbol.protection)
        );
    }
}

struct Prot(i32);

impl fmt::Debug for Prot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(not(windows))]
        const PROTS: [(c_int, &str); 3] = [
            (libc::PROT_READ, "PROT_READ"),
            (libc::PROT_WRITE, "PROT_WRITE"),
            (libc::PROT_EXEC, "PROT_EXEC"),
        ];

        #[cfg(windows)]
        const PROTS: [(c_int, &str); 0] = [];

        let mut first = true;
        let mut flags = self.0;

        for (prot, label) in PROTS {
            if flags & prot != 0 {
                flags &= !prot;

                if !mem::take(&mut first) {
                    f.write_str(" | ")?;
                }

                f.write_str(label)?;
            }
        }

        if flags != 0 {
            write!(f, "{}{}", if first { "" } else { " | " }, flags)?;
        }

        Ok(())
    }
}
