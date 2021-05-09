//! Serialize values into a human-readable encoding that preserves lexicographic sort order.
//!
//! The encoded format is useful for key-value stores/databases that maintains keys in sorted order.
//!
//! Example:
//!
//! ```rust
//! # fn main() -> Result<(), strkey::Error> {
//! let serialized = strkey::to_vec(&("account", 1234u32))?;
//!
//! assert_eq!(&serialized, b"account:000004d2");
//!
//! let deserialized = strkey::from_slice::<(&str, u32)>(&serialized)?;
//!
//! assert_eq!(deserialized.0, "account");
//! assert_eq!(deserialized.1, 1234);
//! # Ok(())
//! # }
//! ```
//!
//! For details, see the [`ser`] module.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod de;
pub mod error;
pub mod ser;

pub use crate::de::{from_reader, from_slice, Deserializer};
pub use crate::error::{Error, Result};
pub use crate::ser::{to_vec, to_writer, Serializer};
