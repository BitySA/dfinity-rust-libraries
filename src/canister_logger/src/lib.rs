//! Module for logging in Internet Computer canisters.
//!
//! This module provides a logging system for canisters that supports both regular logs
//! and traces, with JSON formatting and circular buffer storage. It integrates with
//! the `tracing` ecosystem for structured logging.
//!
//! # Example
//! ```
//! use bity_dfinity_library::canister_logger::{init, export_logs};
//! use tracing::info;
//!
//! // Initialize the logger
//! init(true);  // Enable tracing
//!
//! // Log some messages
//! info!("Application started");
//! tracing::trace!("Detailed trace message");
//!
//! // Export logs
//! let logs = export_logs();
//! ```

use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io::Write;
use tracing::Level;
use tracing_subscriber::fmt::format::{FmtSpan, Writer};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

thread_local! {
    static INITIALIZED: Cell<bool> = Cell::default();
    static LOG: RefCell<LogBuffer> = RefCell::new(LogBuffer::default());
    static TRACE: RefCell<LogBuffer> = RefCell::new(LogBuffer::default());
}

/// Initializes the logging system.
///
/// This function sets up the logging infrastructure with JSON formatting,
/// file and line number information, and optional tracing support.
///
/// # Arguments
/// * `enable_trace` - Whether to enable trace-level logging
///
/// # Panics
/// Panics if the logger has already been initialized
pub fn init(enable_trace: bool) {
    if INITIALIZED.with(|i| i.replace(true)) {
        panic!("Logger already initialized");
    }

    let log_layer = Layer::default()
        .with_writer((|| LogWriter::new(false)).with_max_level(Level::INFO))
        .json()
        .with_timer(Timer {})
        .with_file(true)
        .with_line_number(true)
        .with_current_span(false)
        .with_span_list(false);

    if enable_trace {
        let trace_layer = Layer::default()
            .with_writer(|| LogWriter::new(true))
            .json()
            .with_timer(Timer {})
            .with_file(true)
            .with_line_number(true)
            .with_current_span(false)
            .with_span_events(FmtSpan::ENTER);

        Registry::default().with(log_layer).with(trace_layer).init();
    } else {
        Registry::default().with(log_layer).init();
    }
}

/// Initializes the logging system with pre-existing logs.
///
/// This function initializes the logger and populates it with existing log entries.
///
/// # Arguments
/// * `enable_trace` - Whether to enable trace-level logging
/// * `logs` - Pre-existing log entries to add
/// * `traces` - Pre-existing trace entries to add
pub fn init_with_logs(enable_trace: bool, logs: Vec<LogEntry>, traces: Vec<LogEntry>) {
    init(enable_trace);

    for log in logs {
        LOG.with_borrow_mut(|l| l.append(log));
    }
    for trace in traces {
        TRACE.with_borrow_mut(|t| t.append(trace));
    }
}

/// A circular buffer for storing log messages.
///
/// This struct implements a fixed-size circular buffer that automatically
/// evicts the oldest entries when full.
///
/// # Examples
/// ```
/// use bity_dfinity_library::canister_logger::LogBuffer;
///
/// let mut buffer = LogBuffer::with_capacity(10);
/// buffer.append(LogEntry {
///     timestamp: 1000,
///     message: "Test message".to_string(),
/// });
/// ```
pub struct LogBuffer {
    max_capacity: usize,
    entries: VecDeque<LogEntry>,
}

impl LogBuffer {
    /// Creates a new buffer with the specified maximum capacity.
    ///
    /// # Arguments
    /// * `max_capacity` - The maximum number of entries the buffer can hold
    ///
    /// # Returns
    /// A new `LogBuffer` instance
    pub fn with_capacity(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            entries: VecDeque::with_capacity(max_capacity),
        }
    }

    /// Adds a new entry to the buffer.
    ///
    /// If the buffer is at capacity, the oldest entry is removed before adding
    /// the new one.
    ///
    /// # Arguments
    /// * `entry` - The log entry to add
    pub fn append(&mut self, entry: LogEntry) {
        while self.entries.len() >= self.max_capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Returns an iterator over the entries in insertion order.
    ///
    /// # Returns
    /// An iterator yielding references to `LogEntry`
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter()
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        LogBuffer {
            max_capacity: 100,
            entries: VecDeque::new(),
        }
    }
}

/// Exports all current log entries.
///
/// # Returns
/// A vector containing all log entries
pub fn export_logs() -> Vec<LogEntry> {
    LOG.with_borrow(|l| l.iter().cloned().collect())
}

/// Exports all current trace entries.
///
/// # Returns
/// A vector containing all trace entries
pub fn export_traces() -> Vec<LogEntry> {
    TRACE.with_borrow(|t| t.iter().cloned().collect())
}

/// Represents a single log entry with timestamp and message.
///
/// This struct is used to store individual log messages with their
/// associated timestamps.
#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    /// The timestamp when the log entry was created (in milliseconds)
    pub timestamp: u64,
    /// The log message content
    pub message: String,
}

/// A writer implementation for the logging system.
///
/// This struct handles the actual writing of log messages to the appropriate
/// buffer based on whether it's handling traces or regular logs.
struct LogWriter {
    trace: bool,
    buffer: Vec<u8>,
}

impl LogWriter {
    /// Creates a new log writer.
    ///
    /// # Arguments
    /// * `trace` - Whether this writer handles trace messages
    ///
    /// # Returns
    /// A new `LogWriter` instance
    fn new(trace: bool) -> LogWriter {
        LogWriter {
            trace,
            buffer: Vec::new(),
        }
    }
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let buffer = std::mem::take(&mut self.buffer);
        let json = String::from_utf8(buffer).unwrap();

        let log_entry = LogEntry {
            timestamp: bity_ic_canister_time::timestamp_millis(),
            message: json,
        };

        let sink = if self.trace { &TRACE } else { &LOG };
        sink.with_borrow_mut(|s| s.append(log_entry));
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.write(buf).and_then(|_| self.flush())
    }
}

/// A timer implementation for log timestamps.
///
/// This struct provides timestamp formatting for log entries.
struct Timer;

impl FormatTime for Timer {
    fn format_time(&self, w: &mut Writer) -> std::fmt::Result {
        let now = bity_ic_canister_time::timestamp_millis();
        w.write_str(&format!("{now}"))
    }
}
