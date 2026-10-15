//! Windows only: embed the app icon and version info (VERSIONINFO) into `brahma.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `BRAHMA_REQUIRE_WINRES=1` (set by the
//! release workflow) turns it into an error.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/brahma.ico");
    println!("cargo:rerun-if-env-changed=BRAHMA_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/brahma.ico")
        .set("ProductName", "brahma")
        .set("FileDescription", "brahma image editor")
        .set("CompanyName", "Learning Machines LLC")
        .set("LegalCopyright", "Copyright (c) the brahma authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "brahma.exe")
        .set("InternalName", "brahma");
    if let Err(e) = res.compile() {
        if std::env::var_os("BRAHMA_REQUIRE_WINRES").is_some() {
            panic!("embedding Windows resources failed: {e}");
        }
        println!("cargo:warning=brahma.exe built without icon/version resources: {e}");
    }
}
