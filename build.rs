//! Compiles the Blueprint UI files and bundles them, the stylesheet and
//! the app icon into a GResource that `main.rs` embeds in the binary.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ui_out = out_dir.join("ui");
    std::fs::create_dir_all(&ui_out).unwrap();

    let blueprints: Vec<PathBuf> = std::fs::read_dir("data/ui")
        .expect("data/ui missing")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "blp"))
        .collect();

    let status = Command::new("blueprint-compiler")
        .arg("batch-compile")
        .arg(&ui_out)
        .arg("data/ui")
        .args(&blueprints)
        .status()
        .expect("blueprint-compiler not found - install it (Arch: pacman -S blueprint-compiler)");
    assert!(status.success(), "blueprint-compiler failed");

    glib_build_tools::compile_resources(
        &[PathBuf::from("data"), out_dir.clone()],
        "data/resources.gresource.xml",
        "channex.gresource",
    );

    println!("cargo:rerun-if-changed=data");
}
