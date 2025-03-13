//! Module for serialization and deserialization using MessagePack format.
//!
//! This module provides utilities for efficiently serializing and deserializing data
//! using the MessagePack format, which is a binary serialization format that is
//! more compact than JSON and supports more data types.
//!
//! # Example
//! ```
//! use std::io::Cursor;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! let person = Person {
//!     name: "Alice".to_string(),
//!     age: 30,
//! };
//!
//! let mut buffer = Vec::new();
//! serialize(&person, &mut buffer).unwrap();
//!
//! let deserialized: Person = deserialize(Cursor::new(&buffer)).unwrap();
//! assert_eq!(person, deserialized);
//! ```

use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use std::io::{Read, Write};

/// Serializes a value into a MessagePack format using the provided writer.
///
/// # Arguments
/// * `value` - The value to serialize
/// * `writer` - The writer to write the serialized data to
///
/// # Returns
/// A `Result` containing either `()` on success or an error on failure
///
/// # Type Parameters
/// * `T` - The type of the value to serialize (must implement `Serialize`)
/// * `W` - The type of the writer (must implement `Write`)
pub fn serialize<T, W>(value: T, writer: W) -> Result<(), impl Error>
where
    T: Serialize,
    W: Write,
{
    let mut serializer = rmp_serde::Serializer::new(writer).with_struct_map();
    value.serialize(&mut serializer).map(|_| ())
}

/// Deserializes a value from MessagePack format using the provided reader.
///
/// # Arguments
/// * `reader` - The reader to read the serialized data from
///
/// # Returns
/// A `Result` containing either the deserialized value on success or an error on failure
///
/// # Type Parameters
/// * `T` - The type of the value to deserialize (must implement `DeserializeOwned`)
/// * `R` - The type of the reader (must implement `Read`)
pub fn deserialize<T, R>(reader: R) -> Result<T, impl Error>
where
    T: DeserializeOwned,
    R: Read,
{
    let mut deserializer = rmp_serde::Deserializer::new(reader);
    T::deserialize(&mut deserializer)
}
