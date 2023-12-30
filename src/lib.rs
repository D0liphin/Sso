#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "nightly", feature(allocator_api))]

mod impl_macros;
mod sso_string;
pub mod unified_alloc;
pub mod unsafe_field;
use sso_string::{SsoString, SsoStr};

#[cfg(test)]
mod tests;

#[cfg(all(target_endian = "little", target_pointer_width = "64"))]
pub type String = SsoString;

#[cfg(all(not(target_endian = "little"), not(target_pointer_width = "64")))]
pub type String = std::string::String;

#[cfg(all(target_endian = "little", target_pointer_width = "64"))]
pub type Str = SsoStr;

#[cfg(all(not(target_endian = "little"), not(target_pointer_width = "64")))]
pub type Str = str;
