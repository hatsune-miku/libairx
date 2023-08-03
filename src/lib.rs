pub mod network;
pub mod service;
pub mod packet;
pub mod util;
pub mod compatibility;
pub mod proto;
pub mod extension;

pub mod lib_util;
pub mod lib_generic;

/// cbindgen:ignore
#[cfg(feature = "jni")]
#[allow(non_snake_case)]
pub mod lib_android;
