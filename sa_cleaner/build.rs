use std::env;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let is_win = target.contains("windows");

    let mut b = cc::Build::new();
    b.cpp(true);
    b.file("src/cpp/engine.cpp");
    b.include("src/cpp");
    b.flag_if_supported("-static-libgcc");
    b.flag_if_supported("-static-libstdc++");
    b.flag_if_supported("-static");
    b.flag_if_supported("-O2");
    b.define("UNICODE", None);
    b.define("_UNICODE", None);

    if !is_win {
        b.target("x86_64-pc-windows-gnu");
        b.compiler("x86_64-w64-mingw32-g++");
    }

    b.compile("sacleaner_cpp");

    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=kernel32");
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=psapi");
    println!("cargo:rustc-link-lib=ole32");

    println!("cargo:rustc-link-arg=-static-libgcc");
    println!("cargo:rustc-link-arg=-static-libstdc++");
    println!("cargo:rustc-link-arg=-static");

    println!("cargo:rerun-if-changed=src/cpp/engine.cpp");
    println!("cargo:rerun-if-changed=src/cpp/engine.h");
}
