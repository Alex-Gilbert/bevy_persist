//! WebAssembly storage implementation using browser's localStorage.
//!
//! This module provides a storage backend that uses the browser's localStorage
//! API, which is available when compiling to WebAssembly/WASM for web targets.

use crate::{PersistError, PersistResult, Storage};

/// Storage implementation for WASM targets (browser).
pub struct WasmStorage {
    storage: web_sys::Storage,
}

impl WasmStorage {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap();
        WasmStorage {
            storage: storage.unwrap(),
        }
    }
}

impl Storage for WasmStorage {
    fn read(&self, path: &str) -> PersistResult<Option<String>> {
        // In localStorage, we store as key-value
        // The path becomes the key
        let key = Self::path_to_key(path);

        match self.storage.get_item(&key) {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => Ok(None),
            Err(e) => Err(PersistError::IoError(format!(
                "Failed to read from localStorage: {:?}",
                e
            ))),
        }
    }

    fn write(&self, path: &str, content: &str) -> PersistResult<()> {
        let key = Self::path_to_key(path);

        self.storage.set_item(&key, content).map_err(|e| {
            PersistError::IoError(format!("Failed to write to localStorage: {:?}", e))
        })?;

        Ok(())
    }

    fn read_bytes(&self, path: &str) -> PersistResult<Option<Vec<u8>>> {
        let Some(content) = self.read_text(path)? else {
            return Ok(None);
        };

        #[cfg(feature = "prod")]
        {
            use base64::{engine::general_purpose, Engine as _};

            return general_purpose::STANDARD
                .decode(content.as_bytes())
                .map(Some)
                .map_err(|e| {
                    PersistError::IoError(format!(
                        "Failed to decode binary localStorage payload: {}",
                        e
                    ))
                });
        }

        #[cfg(not(feature = "prod"))]
        {
            Ok(Some(content.into_bytes()))
        }
    }

    fn write_bytes(&self, path: &str, content: &[u8]) -> PersistResult<()> {
        #[cfg(feature = "prod")]
        {
            use base64::{engine::general_purpose, Engine as _};

            return self.write_text(path, &general_purpose::STANDARD.encode(content));
        }

        #[cfg(not(feature = "prod"))]
        {
            let content = String::from_utf8(content.to_vec()).map_err(|e| {
                PersistError::IoError(format!(
                    "Failed to store binary data in localStorage without base64 support: {}",
                    e
                ))
            })?;
            self.write_text(path, &content)
        }
    }

    fn exists(&self, path: &str) -> bool {
        let key = Self::path_to_key(path);
        self.storage.get_item(&key).ok().flatten().is_some()
    }

    fn delete(&self, path: &str) -> PersistResult<()> {
        let key = Self::path_to_key(path);

        self.storage.remove_item(&key).map_err(|e| {
            PersistError::IoError(format!("Failed to delete from localStorage: {:?}", e))
        })?;

        Ok(())
    }

    fn create_dir(&self, _path: &str) -> PersistResult<()> {
        // localStorage doesn't have directories, so this is a no-op
        // The key-based approach handles this automatically
        Ok(())
    }
}

impl WasmStorage {
    /// Converts a file path to a localStorage key.
    ///
    /// Replaces path separators with underscores and sanitizes the key
    /// to be valid for localStorage.
    fn path_to_key(path: &str) -> String {
        // Replace / and \ with _ to make it a valid key
        let sanitized = path.replace(['/', '\\'], "_");
        // Remove any invalid characters
        let sanitized: String = sanitized
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect();

        format!("bevy_persist_{}", sanitized)
    }
}
