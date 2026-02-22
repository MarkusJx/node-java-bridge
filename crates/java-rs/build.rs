use build_target::{Arch, Os};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let java_home_path = java_locator::locate_java_home().unwrap();
    let java_base_path = Path::new(java_home_path.as_str());
    let java_include_path_buf = java_base_path.join("include");
    let java_home = java_include_path_buf.as_path();

    #[cfg(target_os = "macos")]
    let os_dir = "darwin";
    #[cfg(target_os = "linux")]
    let os_dir = "linux";
    #[cfg(target_os = "windows")]
    let os_dir = "win32";

    let mut bindings = bindgen::Builder::default()
        .header(java_home.join("jni.h").as_path().to_str().unwrap())
        .clang_arg(format!("-I{}", java_home.to_str().unwrap()).as_str())
        .clang_arg(format!("-I{}", java_home.join(os_dir).as_path().to_str().unwrap()).as_str())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if build_target::target_os() == Os::Linux && build_target::target_arch() == Arch::AArch64 {
        bindings = bindings.size_t_is_usize(false);
    }

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
