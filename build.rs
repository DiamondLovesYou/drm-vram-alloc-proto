
use std::env::{var_os};
use std::path::{PathBuf};

pub fn main() {
  let out_dir: PathBuf = From::from(var_os("OUT_DIR").unwrap());

  let bindings = bindgen::Builder::default()
    .header("/usr/include/libdrm/amdgpu.h")
    .header("/usr/include/libdrm/amdgpu_drm.h")
    .rust_target(bindgen::RustTarget::Nightly)
    .clang_arg("-I/usr/include/libdrm")
    .derive_debug(true)
    .derive_copy(true)
    .derive_hash(true)
    .derive_eq(true)
    .derive_partialeq(false)
    .derive_default(true)
    .impl_debug(true)
    .impl_partialeq(true)
    .rustfmt_bindings(true)
    .array_pointers_in_arguments(true)
    //.rustified_non_exhaustive_enum("_HSAKMT_STATUS")
    .generate()
    .unwrap();

  bindings.write_to_file(out_dir.join("bindings.rs"))
    .unwrap();

  println!("cargo:rustc-link-lib=dylib=drm");
  println!("cargo:rustc-link-lib=dylib=drm_amdgpu");
}
