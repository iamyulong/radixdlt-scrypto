#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"] // Enables certain tests of deep typed SBOR to function

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// RE addresses.
pub mod address;
/// RE crypto library
pub mod crypto;
/// RE data model.
pub mod data;
/// RE math library.
pub mod math;
/// RE network abstraction.
pub mod network;
/// RE time library.
pub mod time;

mod macros;
pub use macros::*;

// Re-export SBOR derive.
extern crate sbor;
pub use sbor::{Categorize, Decode, Encode, Sbor};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
pub extern crate self as radix_engine_common;
