#[cfg(feature = "blas")]
pub mod hipblas {
    #[doc(inline)]
    pub use hipblas_sys::*;
}
#[cfg(feature = "rtc")]
pub mod hiprtc {
    #[doc(inline)]
    pub use hiprtc_sys::*;
}
pub mod hiprt {
    #[doc(inline)]
    pub use hip_runtime_sys::*;
}
