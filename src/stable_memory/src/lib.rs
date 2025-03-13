//! Module for managing stable memory in the Internet Computer context.
//!
//! This module provides utilities for efficiently reading and writing to stable memory
//! using buffers, and allows tracking memory usage.
//!
//! # Example
//! ```
//! use ic_stable_structures::DefaultMemoryImpl;
//!
//! let memory = DefaultMemoryImpl::default();
//! let mut writer = get_writer(&mut memory);
//! writer.write_all(b"Hello, World!").unwrap();
//! ```

use ic_cdk::api::stable::stable_size;
use ic_cdk::api::stable::WASM_PAGE_SIZE_IN_BYTES;
use ic_stable_structures::reader::{BufferedReader, Reader};
use ic_stable_structures::writer::{BufferedWriter, Writer};
use ic_stable_structures::Memory;
use std::cmp::min;
use std::io::{Read, Write};

const MAX_READER_WRITER_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

/// Creates a new buffered reader for stable memory.
///
/// # Arguments
/// * `memory` - The stable memory to use
///
/// # Returns
/// A `Read` implementation that allows reading data from stable memory
pub fn get_reader<M: Memory>(memory: &M) -> impl Read + '_ {
    BufferedReader::new(buffer_size(memory), Reader::new(memory, 0))
}

/// Creates a new buffered writer for stable memory.
///
/// # Arguments
/// * `memory` - The stable memory to use
///
/// # Returns
/// A `Write` implementation that allows writing data to stable memory
pub fn get_writer<M: Memory>(memory: &mut M) -> impl Write + '_ {
    BufferedWriter::new(MAX_READER_WRITER_BUFFER_SIZE, Writer::new(memory, 0))
}

/// Calculates the optimal buffer size based on memory size.
///
/// # Arguments
/// * `memory` - The stable memory to use
///
/// # Returns
/// The buffer size in bytes, limited to `MAX_READER_WRITER_BUFFER_SIZE`
fn buffer_size<M: Memory>(memory: &M) -> usize {
    let memory_size = memory.size() * (WASM_PAGE_SIZE_IN_BYTES as u64);

    match usize::try_from(memory_size) {
        Ok(size) => min(size / 4, MAX_READER_WRITER_BUFFER_SIZE),
        Err(_) => MAX_READER_WRITER_BUFFER_SIZE,
    }
}

/// Returns the total amount of stable memory used in bytes.
///
/// # Returns
/// The number of bytes of stable memory used
pub fn used() -> u64 {
    (stable_size() as u64) * (WASM_PAGE_SIZE_IN_BYTES as u64)
}
