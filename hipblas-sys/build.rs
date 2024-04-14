use std::{
    env::{var, VarError},
    path::PathBuf,
};

const DEFAULT_HIP_PATH: &str = "/opt/rocm/";

fn try_get_path_from_var(v: &str) -> Option<PathBuf> {
    match var(v) {
        Ok(v) => {
            // Check that the path exists.
            let pb = PathBuf::from(&v);
            if !pb.exists() {
                panic!(
                    "{}: {v} is set to '{}', but that path doesn't exist",
                    env!("CARGO_PKG_NAME"),
                    pb.display()
                );
            }
            Some(pb)
        }
        Err(VarError::NotPresent) => None,
        Err(e @ VarError::NotUnicode(_)) => panic!("{}: {v}: {}", env!("CARGO_PKG_NAME"), e),
    }
}

fn main() {
    // Link to hipblas. It appears there's no variable for specifying where
    // hipblas is; search both HIP_BLAS_PATH and HIPBLAS_PATH. If neither of
    // these are defined, then use HIP_PATH, failing that, a default hipblas
    // directory.

    println!("cargo:rerun-if-env-changed=HIP_PATH");
    println!("cargo:rerun-if-env-changed=HIP_BLAS_PATH");
    println!("cargo:rerun-if-env-changed=HIPBLAS_PATH");

    // Try to get HIP_PATH.
    let hip_path =
        try_get_path_from_var("HIP_PATH").unwrap_or_else(|| PathBuf::from(DEFAULT_HIP_PATH));

    println!(
        "cargo:warning={}: Using '{}' as HIP_PATH",
        env!("CARGO_PKG_NAME"),
        hip_path.display()
    );

    let hip_blas_path = {
        // Now try to get the hipblas path.
        let mut hip_blas_path = None;
        for hip_blas_path_var in ["HIP_BLAS_PATH", "HIPBLAS_PATH"] {
            if let Some(v) = try_get_path_from_var(hip_blas_path_var) {
                println!(
                    "cargo:warning={}: Using {hip_blas_path_var}",
                    env!("CARGO_PKG_NAME")
                );
                hip_blas_path = Some(v);
                break;
            }
            println!(
                "cargo:warning={}: {hip_blas_path_var} wasn't defined",
                env!("CARGO_PKG_NAME")
            );
        }

        let hip_blas_path = match hip_blas_path {
            Some(p) => p,
            None => {
                let pb = hip_path.join("..").join("hipblas");
                println!("cargo:warning={}: Assuming HIP_PATH is related to hipblas; checking if '{}' exists", env!("CARGO_PKG_NAME"), pb.display());
                pb.canonicalize().unwrap()
            }
        };

        hip_blas_path
    };

    println!(
        "cargo:warning={}: Using '{}' as HIP_BLAS_PATH",
        env!("CARGO_PKG_NAME"),
        hip_blas_path.display()
    );

    let hip_blas_lib = hip_blas_path.join("lib");
    println!("cargo:rustc-link-search=native={}", hip_blas_lib.display());
    println!("cargo:rustc-link-lib=dylib=hipblas");

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
            .clang_arg(format!("-I{}", hip_blas_path.join("include").display()))
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
