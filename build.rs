extern crate bindgen;
use std::env;
use std::path::PathBuf;


fn main() {
    println!(r"cargo:rerun-if-changed=wrapper.h");
    println!(r"cargo:rustc-link-lib=libtsk");

    println!(r"cargo:rustc-link-search=D:\libraries\sleuthkit-4.10.0\win32\x64\Release_NoLibs");

    #[cfg(target_os = "windows")]
    windows_setup();

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
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


#[cfg(target_os = "windows")]
fn windows_setup() {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let sdk_key = hklm.open_subkey(r"SOFTWARE\Wow6432Node\Microsoft\Microsoft SDKs\Windows\v10.0")
        .expect("Error getting SDK key");

    let installation_folder: String = sdk_key.get_value("InstallationFolder").expect("Cant get InstallationFolder");
    let product_version: String = sdk_key.get_value("ProductVersion").expect("Cant get ProductVersion");
    let sdk_path = format!(r"{}Lib\{}\um\x64", &installation_folder, &product_version);

    println!(r"cargo:rustc-link-search={}", sdk_path);
    println!(r"cargo:rustc-link-lib=Ole32");
    println!(r"cargo:rustc-link-arg=/NODEFAULTLIB:libtsk");
}