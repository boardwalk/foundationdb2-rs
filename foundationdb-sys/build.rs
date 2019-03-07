use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[cfg(feature = "fdb-5_1")]
const API_VERSION: i32 = 510;

#[cfg(feature = "fdb-5_2")]
const API_VERSION: i32 = 520;

#[cfg(feature = "fdb-6_0")]
const API_VERSION: i32 = 600;

#[cfg(feature = "fdb-6_1")]
const API_VERSION: i32 = 610;

fn main() {
    let mut clang_args = Vec::new();

    println!("cargo:rustc-link-lib=fdb_c");

    if let Ok(lib_path) = env::var("FDB_LIB_PATH") {
        println!("cargo:rustc-link-search={}", lib_path);
    }

    if let Ok(include_path) = env::var("FDB_INCLUDE_PATH") {
        clang_args.push(format!("-I{}", include_path));
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wrapper_path = out_dir.join("wrapper.h").to_string_lossy().into_owned();
    File::create(&wrapper_path)
        .expect("Unable to create wrapper.h")
        .write_all(
            format!(
                "#define FDB_API_VERSION {}\n#include <fdb_c.h>\n#include <fdb_c_options.g.h>\n",
                API_VERSION
            )
            .as_bytes(),
        )
        .expect("Unable to write wrapper.h");

    bindgen::Builder::default()
        .clang_args(clang_args)
        .header(wrapper_path)
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Unable to write bindings");
}
