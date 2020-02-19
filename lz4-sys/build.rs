extern crate cc;

use std::{env, fs, process};
use std::error::Error;
use std::path::PathBuf;

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut compiler = cc::Build::new();
    compiler
        .file("liblz4/lib/lz4.c")
        .file("liblz4/lib/lz4frame.c")
        .file("liblz4/lib/lz4hc.c")
        .file("liblz4/lib/xxhash.c")
        .define("XXH_NAMESPACE", "LZ4_")
        // We always compile the C with optimization, because otherwise it is 20x slower.
        .opt_level(3);
    match env::var("TARGET")
        .map_err(|err| format!("reading TARGET environment variable: {}", err))?
        .as_str()
    {
      "i686-pc-windows-gnu" => {
        compiler
            .flag("-fno-tree-vectorize");
      },
      _ => {}
    }
    compiler.compile("liblz4.a");

    let src = env::current_dir()?.join("liblz4").join("lib");
    let dst = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing OUT_DIR environment variable")?);
    let include = dst.join("include");
    fs::create_dir_all(&include)
        .map_err(|err| format!("creating directory {}: {}", include.display(), err))?;
    for e in fs::read_dir(&src)? {
        let e = e?;
        let utf8_file_name = e.file_name().into_string()
            .map_err(|_| format!("unable to convert file name {:?} to UTF-8", e.file_name()))?;
        if utf8_file_name.ends_with(".h") {
            let from = e.path();
            let to = include.join(e.file_name());
            fs::copy(&from, &to)
                .map_err(|err| format!("copying {} to {}: {}", from.display(), to.display(), err))?;
        }
    }
    println!("cargo:root={}", dst.display());

    Ok(())
}
