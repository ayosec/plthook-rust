//! Build script to compile the plthook sources located in `vendor/plthook`
//! directory.

use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    let suffix;

    if target.contains("-windows-") {
        println!("cargo:rustc-link-lib=dbghelp");
        suffix = "win32";
    } else if target.contains("-darwin") {
        suffix = "osx";
    } else {
        suffix = "elf";
    };

    cc::Build::new()
        .file(format!("vendor/plthook/plthook_{}.c", suffix))
        .compile("plthook-sys");
}
