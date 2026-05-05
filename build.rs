use std::env;
use std::fs;
#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
extern crate bindgen;

fn main() {
    link_libraries();
}

/// On Mac OS and Windows targets, links the XPLM libraries
fn link_libraries() {
    println!("cargo:rerun-if-changed=src/wrapper.h");

    // Get the absolute path to this crate, so that linking will work when done in another folder
    let crate_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    if target.contains("-apple-") {
        let library_path = crate_path.join("SDK/Libraries/Mac");
        fix_macos(&library_path);
        println!(
            "cargo:rustc-link-search=framework={}",
            library_path.to_str().unwrap()
        );
        println!("cargo:rustc-link-lib=framework=XPLM");
        println!("cargo:rustc-link-lib=framework=XPWidgets");
    } else if target.contains("-linux-") {
        // Do nothing for Linux
    } else if target.contains("-windows-") {
        let library_path = crate_path.join("SDK/Libraries/Win");
        println!("cargo:rustc-link-search={}", library_path.to_str().unwrap());
        if target.contains("x86_64") {
            println!("cargo:rustc-link-lib=XPLM_64");
            println!("cargo:rustc-link-lib=XPWidgets_64");
        } else {
            println!("cargo:rustc-link-lib=XPLM");
            println!("cargo:rustc-link-lib=XPWidgets");
        }
    } else {
        panic!("Target operating system not Mac OS, Linux, or Windows")
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.h")
        .clang_args([
            // Parse all comments as documentation
            "-fparse-all-comments",
            // define versions for X-Plane
            "-DXPLM200",
            "-DXPLM210",
            "-DXPLM300",
            "-DXPLM301",
            "-DXPLM303",
            "-DXPLM400",
            "-DXPLM410",
            "-DXPLM411",
            "-DXPLM420",
            // Platform doesn't matter for Rust but must be set - we use LIN here
            "-DLIN=1",
            "-ISDK/CHeaders/XPLM",
            "-ISDK/CHeaders/Widgets",
        ])
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(
            PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR env not set, how?"))
                .join("bindgen.rs"),
        )
        .expect("Couldn't write bindings!");
}

fn fix_macos(p: &Path) {
    let xplm = p.join("XPLM.framework");
    let xpwidgets = p.join("XPWidgets.framework");
    repair_framework(&xplm, "XPLM");
    repair_framework(&xpwidgets, "XPWidgets");
}

fn repair_framework(p: &Path, b: &str) {
    let versions = p.join("Versions");
    let current = versions.join("Current");
    text_ptr_to_symlink(&current, "C");
    text_ptr_to_symlink(&p.join(b), &format!("Versions/Current/{b}"));
    text_ptr_to_symlink(&p.join("Resources"), "Versions/Current/Resources");
}

fn text_ptr_to_symlink(p: &Path, lt: &str) {
    let Ok(m) = fs::symlink_metadata(p) else {
        return;
    };
    if m.is_symlink() || !m.is_file() {
        return;
    }
    let Ok(s) = fs::read_to_string(p) else {
        return;
    };
    let norm = s.trim();
    if norm != lt {
        return;
    }
    if fs::remove_file(p).is_ok() {
        #[cfg(target_family = "unix")]
        let _ = symlink(lt, p);
    }
}
