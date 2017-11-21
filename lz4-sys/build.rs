extern crate gcc;

use std::{env, fs};
use std::path::PathBuf;

fn main() {
    let mut compiler = gcc::Build::new();
    compiler
        .file("liblz4/lib/lz4.c")
        .file("liblz4/lib/lz4frame.c")
        .file("liblz4/lib/lz4hc.c")
        .file("liblz4/lib/xxhash.c")
        // We always compile the C with optimization, because otherwise it is 20x slower.
        .opt_level(3);
    match env::var("TARGET").unwrap().as_str()
    {
      "i686-pc-windows-gnu" => {
        compiler
            .flag("-fno-tree-vectorize");
      },
      _ => {}
    }
    compiler.compile("liblz4.a");

    let src = env::current_dir().unwrap().join("liblz4").join("lib");
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let include = dst.join("include");
    fs::create_dir_all(&include).unwrap();
    for e in fs::read_dir(&src).unwrap() {
        let e = e.unwrap();
        if e.file_name().into_string().unwrap().ends_with(".h") {
            fs::copy(e.path(), include.join(e.file_name())).unwrap();
        }
    }
    println!("cargo:root={}", dst.display());
}
