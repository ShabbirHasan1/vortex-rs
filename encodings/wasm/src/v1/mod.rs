//! This module contains the implementation of the WASM Vortex encoding version 1.
//!
//! The V1 WASM integration provides support _only_ for returning a canonicalized array, with no
//! support for push-down of any kind.
mod array;
mod encoding;
pub mod host;
pub mod module;

pub use array::*;
pub use encoding::*;
