#![allow(dead_code)]
//! A test crate for output coherence testing.
//!
//! This crate provides various Rust item types to verify that the docsrs
//! tool produces consistent, well-formed output across all query modes.

/// A generic container with public fields and methods.
///
/// # Examples
///
/// ```
/// let c = Container::new("hello".into(), 42);
/// ```
pub struct Container<T> {
    /// The stored value.
    pub value: T,
    /// An optional label for the container.
    pub label: String,
}

impl<T> Container<T> {
    /// Creates a new `Container` with the given value and label.
    pub fn new(value: T, label: String) -> Self {
        Self { value, label }
    }

    /// Returns a reference to the stored value.
    pub fn get(&self) -> &T {
        &self.value
    }
}

/// Represents the status of an operation.
pub enum Status {
    /// The operation has not started.
    Pending,
    /// The operation is running with a progress percentage.
    Running(u8),
    /// The operation completed with a result message.
    Done {
        /// The result message.
        message: String,
    },
}

/// A trait for processing items.
///
/// Implementors define how items of type [`Self::Input`] are transformed
/// into [`Self::Output`].
pub trait Processor {
    /// The input type.
    type Input;

    /// The output type.
    type Output;

    /// The default batch size.
    const DEFAULT_BATCH_SIZE: usize;

    /// Processes a single item.
    fn process(&self, input: Self::Input) -> Self::Output;

    /// Processes a batch of items using the default implementation.
    fn process_batch(&self, items: Vec<Self::Input>) -> Vec<Self::Output> {
        items.into_iter().map(|i| self.process(i)).collect()
    }
}

/// Processes the input value, applying the given transformation.
///
/// This function accepts any type that implements `Into<String>`.
pub fn process<T>(input: T) -> String
where
    T: Into<String>,
{
    input.into()
}

/// Utility functions for common operations.
pub mod utils {
    /// Formats a value as a debug string.
    pub fn format_debug<T: std::fmt::Debug>(value: &T) -> String {
        format!("{value:?}")
    }

    /// The default buffer size.
    pub const DEFAULT_BUFFER_SIZE: usize = 1024;

    /// Helper functions for advanced use cases.
    pub mod helpers {
        /// A helper function that returns a greeting.
        pub fn helper_fn(name: &str) -> String {
            format!("Hello, {name}!")
        }
    }
}

/// The maximum allowed size for a container.
pub const MAX_SIZE: usize = 256;

/// A type alias for results with [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// An error type for this crate.
///
/// Wraps a human-readable error message.
pub struct Error {
    /// The error message.
    pub message: String,
}
