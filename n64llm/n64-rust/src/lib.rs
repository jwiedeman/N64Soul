#![cfg_attr(not(any(test, feature = "host")), no_std)]

extern crate alloc;

pub mod config;
#[cfg(target_arch = "mips")]
pub mod ipl3;
pub mod n64_sys;
pub mod platform;
pub mod weights;
pub mod weights_manifest;
pub mod weights_manifest_find;
pub mod manifest;
pub mod model;
pub mod stream;
pub mod util;
pub mod io;
