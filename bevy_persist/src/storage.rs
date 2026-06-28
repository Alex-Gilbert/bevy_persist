//! Storage abstraction for persistence.
//!
//! This module provides a trait-based abstraction over different storage backends,
//! allowing the same persistence logic to work on desktop (filesystem) and web
//! (localStorage/IndexedDB) platforms.

use crate::PersistResult;

/// Trait for abstract storage operations.
///
/// Implement this trait to add support for different storage backends
/// (filesystem, IndexedDB, localStorage, etc.)
pub trait Storage: Send + Sync {
    /// Read content from storage
    fn read_text(&self, path: &str) -> PersistResult<Option<String>>;

    /// Write content to storage
    fn write_text(&self, path: &str, content: &str) -> PersistResult<()>;

    /// Read binary content from storage
    fn read_bytes(&self, path: &str) -> PersistResult<Option<Vec<u8>>>;

    /// Write binary content to storage
    fn write_bytes(&self, path: &str, content: &[u8]) -> PersistResult<()>;

    /// Check if a path exists
    fn exists(&self, path: &str) -> bool;

    /// Delete content from storage
    fn delete(&self, path: &str) -> PersistResult<()>;

    /// Create parent directories if needed
    fn create_dir(&self, path: &str) -> PersistResult<()>;
}

#[cfg(not(all(feature = "wasm", target_arch = "wasm32")))]
mod filesystem;

#[cfg(not(all(feature = "wasm", target_arch = "wasm32")))]
pub use filesystem::FileSystemStorage;

#[cfg(not(all(feature = "wasm", target_arch = "wasm32")))]
pub fn create_storage() -> FileSystemStorage {
    FileSystemStorage::new()
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod wasm_storage;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use wasm_storage::WasmStorage;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub fn create_storage() -> WasmStorage {
    WasmStorage::new()
}
