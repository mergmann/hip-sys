use std::{
    env::{var, VarError},
    path::PathBuf,
};

const DEFAULT_HIP_PATH: &str = "/opt/rocm/";

fn main() {
    println!("cargo:rerun-if-env-changed=HIP_PATH");

    // Link to hip. If HIP_PATH isn't supplied, use a default.
    let hip_path = match var("HIP_PATH") {
        Ok(hip_path) => {
            let pb = PathBuf::from(hip_path);
            // Check that the path exists.
            if !pb.exists() {
                panic!(
                    "{}: HIP_PATH is set to '{}', but that path doesn't exist",
                    env!("CARGO_PKG_NAME"),
                    pb.display()
                );
            }
            pb
        }
        Err(VarError::NotPresent) => PathBuf::from(DEFAULT_HIP_PATH),
        Err(e @ VarError::NotUnicode(_)) => panic!("{}: HIP_PATH: {}", env!("CARGO_PKG_NAME"), e),
    };

    println!(
        "cargo:warning={}: Using '{}' as HIP_PATH",
        env!("CARGO_PKG_NAME"),
        hip_path.display()
    );

    let hip_lib = hip_path.join("lib");
    println!("cargo:rustc-link-search=native={}", hip_lib.display());
    println!("cargo:rustc-link-lib=dylib=hiprtc");

    #[cfg(feature = "bindgen")]
    {
        // The bindgen::Builder is the main entry point to bindgen, and lets you
        // build up options for the resulting bindings.
        println!("cargo:rerun-if-changed=wrapper.h");
        let bindings = bindgen::Builder::default()
            .raw_line("#![allow(non_camel_case_types)]")
            .raw_line("#![allow(non_upper_case_globals)]")
            .raw_line("#![allow(non_snake_case)]")
            // The input header we would like to generate bindings for.
            .header("wrapper.h")
            .clang_arg(format!("-I{}", hip_path.join("include").display()))
            .rustified_non_exhaustive_enum("hip.*")
            .generate_block(false)
            .size_t_is_usize(true)
            .ctypes_prefix("::libc")
            .derive_default(true)
            .derive_eq(true)
            .derive_ord(true)
            .derive_hash(true)
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");
        bindings
            .write_to_file("src/bindings.rs")
            .expect("Couldn't write bindings!");
    }
}
