//! Packed bitmaps that can be used to store boolean values.
//!
//! This module provides a wrapper on top of the `Buffer` type to store mutable and immutable
//! bitsets. The bitsets are stored in little-endian order, meaning that the least significant bit
//! of the first byte is the first bit in the bitset.
#[cfg(feature = "arrow")]
mod arrow;
mod buf;
mod buf_mut;

pub use buf::*;
pub use buf_mut::*;
