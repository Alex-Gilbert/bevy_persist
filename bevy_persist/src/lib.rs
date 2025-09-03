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

#[cfg(feature = "prod")]
use directories::ProjectDirs;
#[cfg(feature = "secure")]
use sha2::{Digest, Sha256};

// Re-export the derive macro
pub use bevy_persist_derive::Persist;

// For auto-registration
pub use inventory;

pub mod prelude {
    pub use crate::{
        Persist, PersistData, PersistError, PersistFile, PersistManager, PersistMode,
        PersistPlugin, PersistResult, Persistable,
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
            serde_json::from_str(&content)
                .map_err(|e| PersistError::SerializationError(format!("JSON parse error: {}", e)))
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

/// Persistence mode for a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistMode {
    /// Development mode - saves to local files for tweaking
    Dev,
    /// Embed mode - values are compiled into the binary
    Embed,
    /// Dynamic mode - user settings that persist across runs
    Dynamic,
    /// Secure mode - encrypted/obfuscated save data
    Secure,
}

/// Trait for types that can be persisted.
///
/// This trait is typically implemented automatically by the `#[derive(Persist)]` macro.
/// Manual implementation is possible but not recommended.
pub trait Persistable: Resource + Serialize + for<'de> Deserialize<'de> {
    /// Get the type name for persistence
    fn type_name() -> &'static str;

    /// Get the persistence mode
    fn persist_mode() -> PersistMode {
        PersistMode::Dev
    }

    /// Get embedded data if available
    fn embedded_data() -> Option<&'static str> {
        None
    }

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
    pub persist_mode: &'static str,
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
    /// Development file path (only used when not in production mode)
    #[cfg(not(feature = "prod"))]
    pub dev_file: PathBuf,
    /// Application info for platform-specific paths
    pub app_name: String,
    pub organization: String,
    /// Cached persist file
    persist_file: PersistFile,
    /// Whether auto-save is enabled globally
    pub auto_save: bool,
    /// Track which types have auto-save enabled
    auto_save_types: HashMap<String, bool>,
    /// Track persistence modes for types
    persist_modes: HashMap<String, PersistMode>,
}

impl PersistManager {
    /// Creates a new PersistManager.
    pub fn new(organization: impl Into<String>, app_name: impl Into<String>) -> Self {
        let organization = organization.into();
        let app_name = app_name.into();
        
        // In dev mode, load from the dev file if it exists
        #[cfg(not(feature = "prod"))]
        let dev_file = PathBuf::from(format!("{}_dev.ron", app_name.to_lowercase().replace(" ", "_")));
        
        #[cfg(not(feature = "prod"))]
        let persist_file = PersistFile::load_from_file(&dev_file).unwrap_or_else(|e| {
            debug!("No existing dev file found: {}", e);
            PersistFile::new()
        });
        
        #[cfg(feature = "prod")]
        let persist_file = PersistFile::new();

        Self {
            #[cfg(not(feature = "prod"))]
            dev_file,
            app_name,
            organization,
            persist_file,
            auto_save: true,
            auto_save_types: HashMap::new(),
            persist_modes: HashMap::new(),
        }
    }

    /// Get the appropriate path for a resource based on its mode
    pub fn get_resource_path(&self, type_name: &str, mode: PersistMode) -> PathBuf {
        #[cfg(feature = "prod")]
        {
            match mode {
                PersistMode::Dev => {
                    // In production, dev mode resources shouldn't exist
                    // But if they do, save to a local file as fallback
                    PathBuf::from(format!("{}_dev.ron", self.app_name.to_lowercase().replace(" ", "_")))
                }
                PersistMode::Dynamic => {
                    if let Some(proj_dirs) = ProjectDirs::from("", &self.organization, &self.app_name) {
                        let config_dir = proj_dirs.config_dir();
                        fs::create_dir_all(config_dir).ok();
                        config_dir.join(format!("{}.ron", type_name.to_lowercase()))
                    } else {
                        // Fallback to current directory if platform dirs unavailable
                        PathBuf::from(format!("{}.ron", type_name.to_lowercase()))
                    }
                }
                PersistMode::Secure => {
                    if let Some(proj_dirs) = ProjectDirs::from("", &self.organization, &self.app_name) {
                        let data_dir = proj_dirs.data_dir();
                        fs::create_dir_all(data_dir).ok();
                        data_dir.join(format!("{}.dat", type_name.to_lowercase()))
                    } else {
                        // Fallback to current directory if platform dirs unavailable
                        PathBuf::from(format!("{}.dat", type_name.to_lowercase()))
                    }
                }
                PersistMode::Embed => {
                    // Embedded resources don't save to disk in prod
                    PathBuf::new()
                }
            }
        }
        #[cfg(not(feature = "prod"))]
        {
            // In dev mode, everything goes to the dev file
            let _ = (type_name, mode); // Suppress warnings
            self.dev_file.clone()
        }
    }

    /// Saves all persistent data to the file.
    pub fn save(&mut self) -> PersistResult<()> {
        #[cfg(not(feature = "prod"))]
        return self.persist_file.save_to_file(&self.dev_file);
        
        #[cfg(feature = "prod")]
        {
            // In production, this is only used as a fallback for dev mode resources
            let fallback_path = PathBuf::from(format!("{}_dev.ron", self.app_name.to_lowercase().replace(" ", "_")));
            self.persist_file.save_to_file(&fallback_path)
        }
    }

    /// Reloads persistent data from the file.
    pub fn load(&mut self) -> PersistResult<()> {
        #[cfg(not(feature = "prod"))]
        {
            self.persist_file = PersistFile::load_from_file(&self.dev_file)?;
            Ok(())
        }
        
        #[cfg(feature = "prod")]
        {
            // In production, this would only be called for fallback scenarios
            let fallback_path = PathBuf::from(format!("{}_dev.ron", self.app_name.to_lowercase().replace(" ", "_")));
            self.persist_file = PersistFile::load_from_file(&fallback_path)?;
            Ok(())
        }
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

    /// Sets the persistence mode for a specific type.
    pub fn set_type_mode(&mut self, type_name: String, mode: PersistMode) {
        self.persist_modes.insert(type_name, mode);
    }

    /// Gets the persistence mode for a specific type.
    pub fn get_type_mode(&self, type_name: &str) -> PersistMode {
        self.persist_modes.get(type_name).copied().unwrap_or(PersistMode::Dev)
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
/// // Recommended: Always provide app info for proper paths
/// app.add_plugins(
///     PersistPlugin::new("MyCompany", "MyGame")
/// );
/// 
/// // Optional: Disable auto-save for manual control
/// app.add_plugins(
///     PersistPlugin::new("MyCompany", "MyGame")
///         .with_auto_save(false)
/// );
/// ```
pub struct PersistPlugin {
    /// Organization name (e.g., "MyCompany")
    pub organization: String,
    /// Application name (e.g., "MyGame")
    pub app_name: String,
    /// Whether to enable auto-save on changes
    pub auto_save: bool,
}

impl Default for PersistPlugin {
    fn default() -> Self {
        Self {
            organization: "DefaultOrg".to_string(),
            app_name: "DefaultApp".to_string(),
            auto_save: true,
        }
    }
}

impl PersistPlugin {
    /// Creates a new PersistPlugin with organization and app name.
    /// 
    /// These are used for:
    /// - Platform-specific paths in production (e.g., ~/Library/Application Support/MyCompany/MyGame/)
    /// - Dev file naming (e.g., mygame_dev.ron)
    pub fn new(organization: impl Into<String>, app_name: impl Into<String>) -> Self {
        Self {
            organization: organization.into(),
            app_name: app_name.into(),
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
        let mut manager = PersistManager::new(self.organization.clone(), self.app_name.clone());
        manager.auto_save = self.auto_save;

        app.insert_resource(manager);

        // Auto-register all Persist types that have been defined
        for registration in inventory::iter::<PersistRegistration> {
            debug!("Auto-registering persist type: {} (mode: {})", registration.type_name, registration.persist_mode);
            
            // Call the registration function first to set up the resource and systems
            (registration.register_fn)(app);
            
            // Then store the mode for this type
            if let Some(mut manager) = app.world_mut().get_resource_mut::<PersistManager>() {
                let mode = match registration.persist_mode {
                    "embed" => PersistMode::Embed,
                    "dynamic" => PersistMode::Dynamic,
                    "secure" => PersistMode::Secure,
                    _ => PersistMode::Dev,
                };
                manager.set_type_mode(registration.type_name.to_string(), mode);
            }
        }
    }
}

/// Register a Persist type with the system.
///
/// This is called automatically by the derive macro and typically
/// doesn't need to be called manually.
pub fn register_persist_type<T: Resource + Persistable + Default>(app: &mut App, auto_save: bool) {
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
    // Load persisted data first in PreStartup
    app.add_systems(PreStartup, load_persisted::<T>);
    // Run persist_system in PostUpdate to ensure it runs after all user systems
    app.add_systems(PostUpdate, persist_system::<T>);
}

/// Generic system to persist a resource when it changes
pub fn persist_system<T: Persistable>(mut manager: ResMut<PersistManager>, resource: Res<T>) {
    let type_name = T::type_name();
    
    // Save on any change, even if just added
    // The load system runs in PreStartup, so if we have user changes in the first frame,
    // we should save them even though the resource is still marked as "added"
    if resource.is_changed() {
        let mode = T::persist_mode();
        #[allow(unused_variables)]  // Used in feature-gated code
        let mode = mode;

        // Don't save embedded resources in production
        #[cfg(feature = "prod")]
        if mode == PersistMode::Embed {
            return;
        }

        if manager.is_auto_save_enabled(type_name) {
            let data = resource.to_persist_data();
            
            // In production, save to mode-specific paths
            #[cfg(feature = "prod")]
            {
                if mode == PersistMode::Dynamic || mode == PersistMode::Secure {
                    let path = manager.get_resource_path(type_name, mode);
                    if !path.as_os_str().is_empty() {
                        let mut file = PersistFile::new();
                        file.set_type_data(type_name.to_string(), data);
                        
                        // For secure mode, we could add encryption here
                        #[cfg(feature = "secure")]
                        if mode == PersistMode::Secure {
                            // TODO: Add encryption/obfuscation
                        }
                        
                        if let Err(e) = file.save_to_file(&path) {
                            error!("Failed to save {} to {:?}: {}", type_name, path, e);
                        } else {
                            debug!("Saved {} to {:?}", type_name, path);
                        }
                        return;
                    }
                }
            }
            
            // Default behavior for dev mode
            debug!("{}: Attempting to save to dev file", type_name);
            manager
                .get_persist_file_mut()
                .set_type_data(type_name.to_string(), data);

            if let Err(e) = manager.save() {
                error!("Failed to auto-save {}: {}", type_name, e);
            } else {
                info!("Auto-saved {} to dev file", type_name);
            }
        }
    }
}

/// Load persisted values on startup
pub fn load_persisted<T: Persistable>(manager: Res<PersistManager>, mut resource: ResMut<T>) {
    let type_name = T::type_name();
    #[allow(unused_variables)]  // Used in feature-gated code
    let mode = T::persist_mode();
    
    // Try to load embedded data first in production
    #[cfg(feature = "prod")]
    if mode == PersistMode::Embed {
        if let Some(embedded_str) = T::embedded_data() {
            // Parse the embedded data
            if embedded_str.ends_with(".ron") || embedded_str.contains("(") {
                // Looks like RON format
                if let Ok(file) = ron::from_str::<PersistFile>(embedded_str) {
                    if let Some(data) = file.get_type_data(type_name) {
                        resource.load_from_persist_data(data);
                        info!("Loaded embedded data for {}", type_name);
                        return;
                    }
                }
            } else {
                // Try JSON format
                if let Ok(file) = serde_json::from_str::<PersistFile>(embedded_str) {
                    if let Some(data) = file.get_type_data(type_name) {
                        resource.load_from_persist_data(data);
                        info!("Loaded embedded data for {}", type_name);
                        return;
                    }
                }
            }
        }
    }
    
    // Load from disk for dynamic/secure modes in production
    #[cfg(feature = "prod")]
    if mode == PersistMode::Dynamic || mode == PersistMode::Secure {
        let path = manager.get_resource_path(type_name, mode);
        if !path.as_os_str().is_empty() && path.exists() {
            if let Ok(file) = PersistFile::load_from_file(&path) {
                if let Some(data) = file.get_type_data(type_name) {
                    resource.load_from_persist_data(data);
                    info!("Loaded {} data for {} from {:?}", 
                        if mode == PersistMode::Secure { "secure" } else { "dynamic" }, 
                        type_name, path);
                    return;
                }
            }
        }
    }
    
    // Default behavior - load from main persist file (dev mode)
    if let Some(data) = manager.get_persist_file().get_type_data(type_name) {
        resource.load_from_persist_data(data);
        info!("Loaded persisted data for {}", type_name);
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
        data.insert("float", std::f64::consts::PI);

        assert_eq!(data.get::<i32>("number"), Some(42));
        assert_eq!(data.get::<String>("text"), Some("hello".to_string()));
        assert_eq!(data.get::<f64>("float"), Some(std::f64::consts::PI));
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
        assert_eq!(
            retrieved.unwrap().get::<String>("test_key"),
            Some("test_value".to_string())
        );

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
        assert_eq!(
            loaded_data.get::<String>("key1"),
            Some("value1".to_string())
        );
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
        assert_eq!(
            loaded_data.get::<String>("name"),
            Some("Ron Test".to_string())
        );
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
        let manager = PersistManager::new("TestOrg", "TestApp");

        assert_eq!(manager.organization, "TestOrg");
        assert_eq!(manager.app_name, "TestApp");
        assert!(manager.auto_save);
        assert!(manager.auto_save_types.is_empty());
        
        #[cfg(not(feature = "prod"))]
        assert_eq!(manager.dev_file, PathBuf::from("testapp_dev.ron"));
    }

    #[test]
    fn test_persist_manager_auto_save_settings() {
        let mut manager = PersistManager::new("TestOrg", "TestApp");

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
        // This test requires being able to control file paths, which is only available in dev mode
        #[cfg(not(feature = "prod"))]
        {
            let temp_dir = TempDir::new().unwrap();
            
            // We need to write to a specific file for this test
            // Create a manager with test org/app
            let mut manager = PersistManager::new("TestOrg", "TestApp");
            
            // For testing, override the dev file path
            manager.dev_file = temp_dir.path().join("test.ron");
            
            let mut data = PersistData::new();
            data.insert("test", "data");
            manager
                .get_persist_file_mut()
                .set_type_data("TestType".to_string(), data);

            // Save
            manager.save().unwrap();

            // Create new manager with same paths and load
            let mut manager2 = PersistManager::new("TestOrg", "TestApp");
            manager2.dev_file = temp_dir.path().join("test.ron");
            manager2.load().unwrap();
            
            let loaded_data = manager2.get_persist_file().get_type_data("TestType");
            assert!(loaded_data.is_some());
            assert_eq!(
                loaded_data.unwrap().get::<String>("test"),
                Some("data".to_string())
            );
        }
        
        // In production mode, just verify basic manager creation
        #[cfg(feature = "prod")]
        {
            let manager = PersistManager::new("TestOrg", "TestApp");
            assert_eq!(manager.organization, "TestOrg");
            assert_eq!(manager.app_name, "TestApp");
            // Platform-specific save/load testing would require actual directories
        }
    }

    #[test]
    fn test_persist_error_display() {
        let io_error = PersistError::IoError("file not found".to_string());
        assert_eq!(format!("{}", io_error), "IO error: file not found");

        let ser_error = PersistError::SerializationError("invalid JSON".to_string());
        assert_eq!(
            format!("{}", ser_error),
            "Serialization error: invalid JSON"
        );

        let res_error = PersistError::ResourceNotFound("MyResource".to_string());
        assert_eq!(format!("{}", res_error), "Resource not found: MyResource");
    }

    #[test]
    fn test_persist_plugin_default() {
        let plugin = PersistPlugin::default();
        assert_eq!(plugin.organization, "DefaultOrg");
        assert_eq!(plugin.app_name, "DefaultApp");
        assert!(plugin.auto_save);
    }

    #[test]
    fn test_persist_plugin_custom() {
        let plugin = PersistPlugin::new("MyOrg", "MyApp").with_auto_save(false);
        assert_eq!(plugin.organization, "MyOrg");
        assert_eq!(plugin.app_name, "MyApp");
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
        assert!(
            content.contains("\"TestType\""),
            "JSON should contain TestType"
        );

        // Test RON format
        let ron_path = temp_dir.path().join("test.ron");
        let mut ron_file = PersistFile::new();
        ron_file.set_type_data("TestType".to_string(), data);
        ron_file.save_to_file(&ron_path).unwrap();

        // RON and JSON will have different formatting
        // Just verify both can be loaded back correctly
        let loaded_json = PersistFile::load_from_file(&json_path).unwrap();
        let loaded_ron = PersistFile::load_from_file(&ron_path).unwrap();

        assert!(loaded_json.get_type_data("TestType").is_some());
        assert!(loaded_ron.get_type_data("TestType").is_some());

        let json_data = loaded_json.get_type_data("TestType").unwrap();
        let ron_data = loaded_ron.get_type_data("TestType").unwrap();

        assert_eq!(
            json_data.get::<String>("test_key"),
            Some("test_value".to_string())
        );
        assert_eq!(
            ron_data.get::<String>("test_key"),
            Some("test_value".to_string())
        );
    }
}
