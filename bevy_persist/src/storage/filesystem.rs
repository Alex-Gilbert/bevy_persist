//! Filesystem-based storage implementation for desktop platforms.

use crate::{PersistError, PersistResult, Storage};
use std::fs;
use std::path::Path;

/// Storage implementation that uses the filesystem (desktop platforms).
pub struct FileSystemStorage;

impl FileSystemStorage {
    pub fn new() -> Self {
        Self
    }
}

impl Storage for FileSystemStorage {
    fn read_text(&self, path: &str) -> PersistResult<Option<String>> {
        let path = Path::new(path);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)
            .map_err(|e| PersistError::IoError(format!("Failed to read file: {}", e)))?;

        Ok(Some(content))
    }

    fn write_text(&self, path: &str, content: &str) -> PersistResult<()> {
        self.write_bytes(path, content.as_bytes())
    }

    fn read_bytes(&self, path: &str) -> PersistResult<Option<Vec<u8>>> {
        let path = Path::new(path);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read(path)
            .map_err(|e| PersistError::IoError(format!("Failed to read file: {}", e)))?;

        Ok(Some(content))
    }

    fn write_bytes(&self, path: &str, content: &[u8]) -> PersistResult<()> {
        let path = Path::new(path);

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PersistError::IoError(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(path, content)
            .map_err(|e| PersistError::IoError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn delete(&self, path: &str) -> PersistResult<()> {
        let path = Path::new(path);

        if path.exists() {
            fs::remove_file(path)
                .map_err(|e| PersistError::IoError(format!("Failed to delete file: {}", e)))?;
        }

        Ok(())
    }

    fn create_dir(&self, path: &str) -> PersistResult<()> {
        let path = Path::new(path);

        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| PersistError::IoError(format!("Failed to create directory: {}", e)))?;
        }

        Ok(())
    }
}

impl Default for FileSystemStorage {
    fn default() -> Self {
        Self::new()
    }
}
