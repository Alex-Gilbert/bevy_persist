//! Automatic persistence for Bevy resources with change detection.
//!
//! This crate provides automatic saving and loading of Bevy resources,
//! with support for multiple serialization formats and change detection.
//!
//! # Features
//!
//! - **Automatic Save/Load**: Resources are automatically saved when modified and loaded on startup
//! - **Multiple Formats**: Support for JSON and RON serialization formats
//! - **Change Detection**: Only saves when resources actually change, minimizing disk I/O
//! - **Derive Macro**: Simple `#[derive(Persist)]` to make any resource persistent
//! - **Flexible Configuration**: Customize save paths, formats, and save strategies per resource
//!
//! # Quick Start
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy_persist::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Resource, Default, Serialize, Deserialize, Persist)]
//! struct Settings {
//!     volume: f32,
//!     difficulty: String,
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(PersistPlugin::default())
//!         .init_resource::<Settings>()
//!         .run();
//! }
//! ```

use bevy::prelude::*;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Re-export the derive macro
pub use bevy_persist_derive::Persist;

// For auto-registration
pub use inventory;

pub mod prelude {
    pub use crate::{
        Persist, PersistData, PersistError, PersistFile, PersistManager, PersistPlugin,
        PersistResult, Persistable,
    };
}

/// Result type for persistence operations
pub type PersistResult<T> = Result<T, PersistError>;

/// Errors that can occur during persistence operations
#[derive(Debug, Clone)]
pub enum PersistError {
    /// Failed to read/write file
    IoError(String),
    /// Failed to serialize/deserialize
    SerializationError(String),
    /// Resource not found
    ResourceNotFound(String),
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Self::ResourceNotFound(e) => write!(f, "Resource not found: {}", e),
        }
    }
}

impl std::error::Error for PersistError {}

/// Data structure for persisting parameter values.
///
/// This is used internally to store serialized resource data
/// in a generic format that can be saved to JSON or RON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistData {
    pub values: HashMap<String, serde_json::Value>,
}

impl PersistData {
    /// Creates a new, empty PersistData instance.
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Inserts a serializable value with the given key.
    pub fn insert<T: serde::Serialize>(&mut self, key: impl Into<String>, value: T) {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.values.insert(key.into(), json_value);
        }
    }

    /// Retrieves and deserializes a value by key.
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.values
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

impl Default for PersistData {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete persistence file format.
///
/// This represents the entire contents of a persistence file,
/// including all persisted resources, metadata, and versioning information.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PersistFile {
    #[serde(flatten)]
    pub type_data: HashMap<String, PersistData>,
    pub last_saved: String,
    pub version: String,
}

impl PersistFile {
    /// Creates a new PersistFile with current timestamp and version.
    pub fn new() -> Self {
        Self {
            type_data: HashMap::new(),
            last_saved: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Loads a PersistFile from disk. Creates a new one if the file doesn't exist.
    /// Automatically detects format based on file extension (.ron or .json).
    pub fn load_from_file(path: impl AsRef<Path>) -> PersistResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| PersistError::IoError(format!("Failed to read file: {}", e)))?;

        // Try RON first, fallback to JSON
        if path.extension().is_some_and(|ext| ext == "ron") {
            ron::from_str(&content)
                .map_err(|e| PersistError::SerializationError(format!("RON parse error: {}", e)))
        } else {
            serde_json::from_str(&content).map_err(|e| {
                PersistError::SerializationError(format!("JSON parse error: {}", e))
            })
        }
    }

    /// Saves the PersistFile to disk.
    /// Format is determined by file extension (.ron for RON, .json for JSON).
    pub fn save_to_file(&mut self, path: impl AsRef<Path>) -> PersistResult<()> {
        let path = path.as_ref();

        // Update timestamp
        self.last_saved = chrono::Utc::now().to_rfc3339();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PersistError::IoError(format!("Failed to create directory: {}", e)))?;
        }

        let content = if path.extension().is_some_and(|ext| ext == "ron") {
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).map_err(|e| {
                PersistError::SerializationError(format!("RON serialization error: {}", e))
            })?
        } else {
            serde_json::to_string_pretty(self).map_err(|e| {
                PersistError::SerializationError(format!("JSON serialization error: {}", e))
            })?
        };

        fs::write(path, content)
            .map_err(|e| PersistError::IoError(format!("Failed to write file: {}", e)))?;

        debug!("Saved settings to {}", path.display());
        Ok(())
    }

    /// Gets the persistence data for a specific type.
    pub fn get_type_data(&self, type_name: &str) -> Option<&PersistData> {
        self.type_data.get(type_name)
    }

    /// Sets the persistence data for a specific type.
    pub fn set_type_data(&mut self, type_name: String, data: PersistData) {
        self.type_data.insert(type_name, data);
    }
}

/// Trait for types that can be persisted.
///
/// This trait is typically implemented automatically by the `#[derive(Persist)]` macro.
/// Manual implementation is possible but not recommended.
pub trait Persistable: Resource + Serialize + for<'de> Deserialize<'de> {
    /// Get the type name for persistence
    fn type_name() -> &'static str;

    /// Convert to persistence data
    fn to_persist_data(&self) -> PersistData;

    /// Load from persistence data
    fn load_from_persist_data(&mut self, data: &PersistData);
}

/// Registration data for auto-discovered Persist types.
///
/// Used internally by the derive macro for automatic registration.
#[derive(Debug)]
pub struct PersistRegistration {
    pub type_name: &'static str,
    pub auto_save: bool,
    pub register_fn: fn(&mut App),
}

inventory::collect!(PersistRegistration);

/// Resource that manages persistence.
///
/// This resource is automatically added by `PersistPlugin` and handles
/// all saving and loading operations for persistent resources.
#[derive(Resource)]
pub struct PersistManager {
    /// Path to the persistence file
    pub file_path: PathBuf,
    /// Cached persist file
    persist_file: PersistFile,
    /// Whether auto-save is enabled globally
    pub auto_save: bool,
    /// Track which types have auto-save enabled
    auto_save_types: HashMap<String, bool>,
}

impl PersistManager {
    /// Creates a new PersistManager with the specified file path.
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        let file_path = file_path.into();
        let persist_file = PersistFile::load_from_file(&file_path).unwrap_or_else(|e| {
            error!("Failed to load persist file: {}", e);
            PersistFile::new()
        });

        Self {
            file_path,
            persist_file,
            auto_save: true,
            auto_save_types: HashMap::new(),
        }
    }

    /// Saves all persistent data to the file.
    pub fn save(&mut self) -> PersistResult<()> {
        self.persist_file.save_to_file(&self.file_path)
    }

    /// Reloads persistent data from the file.
    pub fn load(&mut self) -> PersistResult<()> {
        self.persist_file = PersistFile::load_from_file(&self.file_path)?;
        Ok(())
    }

    /// Gets a reference to the underlying persist file.
    pub fn get_persist_file(&self) -> &PersistFile {
        &self.persist_file
    }

    /// Gets a mutable reference to the underlying persist file.
    pub fn get_persist_file_mut(&mut self) -> &mut PersistFile {
        &mut self.persist_file
    }

    /// Checks if auto-save is enabled for a specific type.
    pub fn is_auto_save_enabled(&self, type_name: &str) -> bool {
        self.auto_save && self.auto_save_types.get(type_name).copied().unwrap_or(true)
    }

    /// Sets whether auto-save is enabled for a specific type.
    pub fn set_type_auto_save(&mut self, type_name: String, enabled: bool) {
        self.auto_save_types.insert(type_name, enabled);
    }
}

/// Plugin for automatic persistence.
///
/// Add this plugin to your Bevy app to enable automatic persistence
/// for resources marked with `#[derive(Persist)]`.
///
/// # Example
///
/// ```ignore
/// app.add_plugins(PersistPlugin::default());
/// // Or with custom file path:
/// app.add_plugins(PersistPlugin::new("save_data.ron"));
/// ```
pub struct PersistPlugin {
    /// Path to the persistence file
    pub file_path: String,
    /// Whether to enable auto-save on changes
    pub auto_save: bool,
}

impl Default for PersistPlugin {
    fn default() -> Self {
        Self {
            file_path: "settings.ron".to_string(),
            auto_save: true,
        }
    }
}

impl PersistPlugin {
    /// Creates a new PersistPlugin with the specified file path.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            auto_save: true,
        }
    }

    /// Sets whether auto-save is enabled globally.
    pub fn with_auto_save(mut self, enabled: bool) -> Self {
        self.auto_save = enabled;
        self
    }
}

impl Plugin for PersistPlugin {
    fn build(&self, app: &mut App) {
        let mut manager = PersistManager::new(self.file_path.clone());
        manager.auto_save = self.auto_save;

        app.insert_resource(manager);

        // Auto-register all Persist types that have been defined
        for registration in inventory::iter::<PersistRegistration> {
            info!("Auto-registering persist type: {}", registration.type_name);
            (registration.register_fn)(app);
        }
    }
}

/// Register a Persist type with the system.
///
/// This is called automatically by the derive macro and typically
/// doesn't need to be called manually.
pub fn register_persist_type<T: Resource + Persistable + Default>(
    app: &mut App,
    auto_save: bool,
) {
    let type_name = T::type_name();

    let world = app.world_mut();

    // Ensure resource exists
    if !world.contains_resource::<T>() {
        world.init_resource::<T>();
    }

    // Set auto-save preference for this type
    if let Some(mut manager) = world.get_resource_mut::<PersistManager>() {
        manager.set_type_auto_save(type_name.to_string(), auto_save);
    }

    // Add systems for this type
    app.add_systems(Startup, load_persisted::<T>);
    app.add_systems(Update, persist_system::<T>);
}

/// Generic system to persist a resource when it changes
pub fn persist_system<T: Persistable>(mut manager: ResMut<PersistManager>, resource: Res<T>) {
    if resource.is_changed() && !resource.is_added() {
        let type_name = T::type_name();

        if manager.is_auto_save_enabled(type_name) {
            let data = resource.to_persist_data();
            manager
                .get_persist_file_mut()
                .set_type_data(type_name.to_string(), data);

            if let Err(e) = manager.save() {
                error!("Failed to auto-save {}: {}", type_name, e);
            } else {
                debug!("Auto-saved {}", type_name);
            }
        }
    }
}

/// Load persisted values on startup
pub fn load_persisted<T: Persistable>(manager: Res<PersistManager>, mut resource: ResMut<T>) {
    if let Some(data) = manager.get_persist_file().get_type_data(T::type_name()) {
        resource.load_from_persist_data(data);
        info!("Loaded persisted data for {}", T::type_name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_persist_data_insert_and_get() {
        let mut data = PersistData::new();
        
        // Test inserting and retrieving different types
        data.insert("number", 42i32);
        data.insert("text", "hello");
        data.insert("float", 3.14f64);
        
        assert_eq!(data.get::<i32>("number"), Some(42));
        assert_eq!(data.get::<String>("text"), Some("hello".to_string()));
        assert_eq!(data.get::<f64>("float"), Some(3.14));
        assert_eq!(data.get::<i32>("nonexistent"), None);
    }

    #[test]
    fn test_persist_data_complex_types() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let mut data = PersistData::new();
        let test_struct = TestStruct {
            name: "test".to_string(),
            value: 100,
        };
        
        data.insert("struct", &test_struct);
        
        let retrieved = data.get::<TestStruct>("struct");
        assert_eq!(retrieved, Some(test_struct));
    }

    #[test]
    fn test_persist_file_new() {
        let file = PersistFile::new();
        
        assert!(file.type_data.is_empty());
        assert!(!file.last_saved.is_empty());
        assert_eq!(file.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_persist_file_type_data() {
        let mut file = PersistFile::new();
        let mut data = PersistData::new();
        data.insert("test_key", "test_value");
        
        file.set_type_data("TestType".to_string(), data.clone());
        
        let retrieved = file.get_type_data("TestType");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().get::<String>("test_key"), Some("test_value".to_string()));
        
        assert!(file.get_type_data("NonExistent").is_none());
    }

    #[test]
    fn test_persist_file_save_and_load_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        
        // Create and save a file
        let mut file = PersistFile::new();
        let mut data = PersistData::new();
        data.insert("key1", "value1");
        data.insert("key2", 42);
        file.set_type_data("TestResource".to_string(), data);
        
        file.save_to_file(&file_path).unwrap();
        
        // Load the file back
        let loaded = PersistFile::load_from_file(&file_path).unwrap();
        
        assert_eq!(loaded.type_data.len(), 1);
        let loaded_data = loaded.get_type_data("TestResource").unwrap();
        assert_eq!(loaded_data.get::<String>("key1"), Some("value1".to_string()));
        assert_eq!(loaded_data.get::<i32>("key2"), Some(42));
    }

    #[test]
    fn test_persist_file_save_and_load_ron() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ron");
        
        // Create and save a file
        let mut file = PersistFile::new();
        let mut data = PersistData::new();
        data.insert("name", "Ron Test");
        data.insert("count", 100);
        file.set_type_data("RonResource".to_string(), data);
        
        file.save_to_file(&file_path).unwrap();
        
        // Load the file back
        let loaded = PersistFile::load_from_file(&file_path).unwrap();
        
        assert_eq!(loaded.type_data.len(), 1);
        let loaded_data = loaded.get_type_data("RonResource").unwrap();
        assert_eq!(loaded_data.get::<String>("name"), Some("Ron Test".to_string()));
        assert_eq!(loaded_data.get::<i32>("count"), Some(100));
    }

    #[test]
    fn test_persist_file_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.json");
        
        // Should return a new file when loading nonexistent
        let file = PersistFile::load_from_file(&file_path).unwrap();
        assert!(file.type_data.is_empty());
    }

    #[test]
    fn test_persist_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("manager_test.json");
        
        let manager = PersistManager::new(&file_path);
        
        assert_eq!(manager.file_path, file_path);
        assert!(manager.auto_save);
        assert!(manager.auto_save_types.is_empty());
    }

    #[test]
    fn test_persist_manager_auto_save_settings() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("auto_save_test.json");
        
        let mut manager = PersistManager::new(&file_path);
        
        // Test default auto-save
        assert!(manager.is_auto_save_enabled("AnyType"));
        
        // Disable auto-save for specific type
        manager.set_type_auto_save("DisabledType".to_string(), false);
        assert!(!manager.is_auto_save_enabled("DisabledType"));
        assert!(manager.is_auto_save_enabled("EnabledType"));
        
        // Disable global auto-save
        manager.auto_save = false;
        assert!(!manager.is_auto_save_enabled("AnyType"));
    }

    #[test]
    fn test_persist_manager_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("manager_save_load.json");
        
        // Create manager and add data
        let mut manager = PersistManager::new(&file_path);
        let mut data = PersistData::new();
        data.insert("test", "data");
        manager.get_persist_file_mut().set_type_data("TestType".to_string(), data);
        
        // Save
        manager.save().unwrap();
        
        // Create new manager and verify data persists
        let manager2 = PersistManager::new(&file_path);
        let loaded_data = manager2.get_persist_file().get_type_data("TestType");
        assert!(loaded_data.is_some());
        assert_eq!(loaded_data.unwrap().get::<String>("test"), Some("data".to_string()));
    }

    #[test]
    fn test_persist_error_display() {
        let io_error = PersistError::IoError("file not found".to_string());
        assert_eq!(format!("{}", io_error), "IO error: file not found");
        
        let ser_error = PersistError::SerializationError("invalid JSON".to_string());
        assert_eq!(format!("{}", ser_error), "Serialization error: invalid JSON");
        
        let res_error = PersistError::ResourceNotFound("MyResource".to_string());
        assert_eq!(format!("{}", res_error), "Resource not found: MyResource");
    }

    #[test]
    fn test_persist_plugin_default() {
        let plugin = PersistPlugin::default();
        assert_eq!(plugin.file_path, "settings.ron");
        assert!(plugin.auto_save);
    }

    #[test]
    fn test_persist_plugin_custom() {
        let plugin = PersistPlugin::new("custom.json")
            .with_auto_save(false);
        assert_eq!(plugin.file_path, "custom.json");
        assert!(!plugin.auto_save);
    }

    #[test]
    fn test_persist_data_default() {
        let data = PersistData::default();
        assert!(data.values.is_empty());
    }

    #[test]
    fn test_persist_file_format_detection() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test JSON format
        let json_path = temp_dir.path().join("test.json");
        let mut json_file = PersistFile::new();
        let mut data = PersistData::new();
        data.insert("test_key", "test_value");
        json_file.set_type_data("TestType".to_string(), data.clone());
        json_file.save_to_file(&json_path).unwrap();
        let content = fs::read_to_string(&json_path).unwrap();
        assert!(content.starts_with('{'), "JSON should start with {{");
        assert!(content.contains("\"TestType\""), "JSON should contain TestType");
        
        // Test RON format
        let ron_path = temp_dir.path().join("test.ron");
        let mut ron_file = PersistFile::new();
        ron_file.set_type_data("TestType".to_string(), data);
        ron_file.save_to_file(&ron_path).unwrap();
        let ron_content = fs::read_to_string(&ron_path).unwrap();
        
        // RON and JSON will have different formatting
        // Just verify both can be loaded back correctly
        let loaded_json = PersistFile::load_from_file(&json_path).unwrap();
        let loaded_ron = PersistFile::load_from_file(&ron_path).unwrap();
        
        assert!(loaded_json.get_type_data("TestType").is_some());
        assert!(loaded_ron.get_type_data("TestType").is_some());
        
        let json_data = loaded_json.get_type_data("TestType").unwrap();
        let ron_data = loaded_ron.get_type_data("TestType").unwrap();
        
        assert_eq!(json_data.get::<String>("test_key"), Some("test_value".to_string()));
        assert_eq!(ron_data.get::<String>("test_key"), Some("test_value".to_string()));
    }
}