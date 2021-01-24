extern crate bindgen;
use std::process::Command;
use std::env;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

fn main() {
    println!(r"cargo:rerun-if-changed=wrapper.h");
    println!(r"cargo:rustc-link-lib=libtsk");

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
        .whitelist_function("tsk_fs_file_read_type")
        .whitelist_function("tsk_fs_file_attr_getsize")
        .whitelist_function("tsk_fs_file_attr_get_idx")
        .whitelist_function("tsk_fs_file_attr_get")

        .whitelist_function("tsk_fs_attr_read")

        .whitelist_function("tsk_fs_dir_open_meta")
        .whitelist_function("tsk_fs_dir_close")
        .whitelist_function("tsk_fs_dir_get_name")
        

        .whitelist_type("TSK_FS_TYPE_ENUM")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(target_os = "windows")]
fn windows_compile_tsk() {
    // First we need the x86 program path
    let program_path = PathBuf::from(
        std::env::var("ProgramFiles(x86)")
            .expect("NO ProgramFiles(x86) ENV.")
    );
    // Now we need to compile the vswhere.exe path
    let vs_where_path = PathBuf::from(&program_path)
        .join(r"Microsoft Visual Studio\Installer\vswhere.exe");
    // Run vswhere.exe to get install path
    let output = Command::new(&vs_where_path)
        .args(&["-latest", "-property", "installationPath"])
        .output()
        .expect(&format!(r"Error executing vswhere: {}", vs_where_path.display()));
    
    // Get the installation location
    let install_path = PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim_end()
    );

    // Append msbuild.exe to install path
    let msbuild_path = install_path.join(r"MSBuild\Current\Bin\MSBuild.exe");
    eprintln!("msbuild_path -> {:?}\n", msbuild_path);

    let output = Command::new(&msbuild_path)
        .args(&[
            r"-target:libtsk",
            r"/p:PlatformToolset=v142",
            r"/p:Platform=x64",
            r"/p:Configuration=Release_NoLibs",
            r"sleuthkit\win32\tsk-win.sln"
        ])
        .output()
        .expect("failed to build");
    eprintln!("{}", String::from_utf8_lossy(&output.stdout).trim_end());
}


#[cfg(target_os = "windows")]
fn windows_setup() {
    // windows_compile_tsk();
    println!(r"cargo:rustc-link-search=sleuthkit\win32\x64\Release_NoLibs");

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let sdk_key = hklm.open_subkey(r"SOFTWARE\Wow6432Node\Microsoft\Microsoft SDKs\Windows\v10.0")
        .expect("Microsoft SDK v10 is required.");
    eprintln!("sdk_key: {:?}", sdk_key);

    let installation_folder: String = sdk_key.get_value("InstallationFolder")
        .expect("Cant get InstallationFolder");
    eprintln!("installation_folder: {}", installation_folder);

    let product_version: String = sdk_key.get_value("ProductVersion")
        .expect("Cant get ProductVersion");
    eprintln!("product_version: {}", product_version);

    let sdk_path = format!(r"{}Lib\{}.0\um\x64", &installation_folder, &product_version);
    eprintln!("sdk_path: {}", &sdk_path);

    println!(r"cargo:rustc-link-search={}", sdk_path);
    println!(r"cargo:rustc-link-lib=Ole32");
    println!(r"cargo:rustc-link-arg=/NODEFAULTLIB:libtsk");
}