//! Build script to compile the plthook sources located in `vendor/plthook`
//! directory.

fn main() {
    cc::Build::new()
        .warnings_into_errors(true)
        .file("vendor/plthook/plthook_elf.c")
        .compile("plthook-sys");
}
