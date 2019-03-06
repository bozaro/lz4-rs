extern crate cc;

use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=LZ4_API_STATIC");
    let want_static = env::var("LZ4_API_STATIC").is_ok();
    if !want_static && pkg_config::probe_library("liblz4").is_ok() {
        return;
    }

    let mut compiler = cc::Build::new();
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
}
