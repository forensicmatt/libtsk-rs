extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!(r"cargo:rerun-if-changed=wrapper.h");
    println!(r"cargo:rustc-link-lib=libtsk");

    // TODO: Make more dynamic and cross platform (might need help here)
    // On Windows Ole32 is required
    println!(r"cargo:rustc-link-lib=Ole32");
    println!(r"cargo:rustc-link-search=D:\libraries\sleuthkit-4.10.0\win32\x64\Release_NoLibs");
    // Ole32 can be found in Windows Kits
    println!(r"cargo:rustc-link-search=C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64");
    println!(r"cargo:rustc-link-arg=/NODEFAULTLIB:libtsk");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // .clang_args(&["-x", "c++"])
        .clang_args(&["-I", "sleuthkit"])
        .whitelist_function("tsk_error_get")
        
        .whitelist_function("tsk_img_open_utf8_sing")
        .whitelist_function("tsk_img_close")

        .whitelist_function("tsk_vs_open")
        .whitelist_function("tsk_vs_close")

        .whitelist_function("tsk_vs_part_get")

        .whitelist_function("tsk_fs_open_img")
        .whitelist_function("tsk_fs_close")
        
        .whitelist_function("tsk_fs_file_open")
        .whitelist_function("tsk_fs_file_open_meta")
        .whitelist_function("tsk_fs_file_close")
        
        .whitelist_function("tsk_fs_file_read")
        .whitelist_type("TSK_FS_TYPE_ENUM")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}