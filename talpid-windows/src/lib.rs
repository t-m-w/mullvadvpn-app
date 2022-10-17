//! Interface with low-level windows specific bits.

#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

/// Logging fixtures to be used with C++ libraries.
#[cfg(windows)]
pub mod logging;
/// Nicer interfaces with Windows networking code.
#[cfg(windows)]
pub mod net;
/// Handling Windows specific String things.
#[cfg(windows)]
pub mod string;
