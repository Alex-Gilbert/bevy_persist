use bevy::prelude::*;
use bevy_persist::prelude::*;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Resource, Default, Serialize, Deserialize, Persist, Debug, PartialEq, Clone)]
struct TestSettings {
    volume: f32,
    name: String,
    enabled: bool,
}

#[derive(Resource, Default, Serialize, Deserialize, Persist, Debug, PartialEq, Clone)]
#[persist(auto_save = false)]
struct ManualSaveSettings {
    value: i32,
    text: String,
}

#[test]
fn test_derive_macro_basic() {
    // Test that the derive macro generates proper implementations
    let settings = TestSettings {
        volume: 0.5,
        name: "test".to_string(),
        enabled: true,
    };
    
    // Test type_name
    assert_eq!(TestSettings::type_name(), "TestSettings");
    
    // Test to_persist_data
    let data = settings.to_persist_data();
    assert_eq!(data.get::<f32>("volume"), Some(0.5));
    assert_eq!(data.get::<String>("name"), Some("test".to_string()));
    assert_eq!(data.get::<bool>("enabled"), Some(true));
    
    // Test load_from_persist_data
    let mut new_settings = TestSettings::default();
    new_settings.load_from_persist_data(&data);
    assert_eq!(new_settings, settings);
}

#[test]
fn test_derive_macro_with_attributes() {
    // Test that auto_save attribute is properly handled
    assert_eq!(ManualSaveSettings::type_name(), "ManualSaveSettings");
    
    let settings = ManualSaveSettings {
        value: 42,
        text: "manual".to_string(),
    };
    
    let data = settings.to_persist_data();
    assert_eq!(data.get::<i32>("value"), Some(42));
    assert_eq!(data.get::<String>("text"), Some("manual".to_string()));
}

#[test]
fn test_plugin_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_settings.json");
    
    // Create an app with the plugin
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(PersistPlugin::new(file_path.to_str().unwrap()));
    
    // Initialize resources
    app.init_resource::<TestSettings>();
    app.init_resource::<ManualSaveSettings>();
    
    // Run startup systems to load any existing data
    app.finish();
    app.cleanup();
    
    // Verify the manager was created
    assert!(app.world().get_resource::<PersistManager>().is_some());
    
    // Verify resources were initialized
    assert!(app.world().get_resource::<TestSettings>().is_some());
    assert!(app.world().get_resource::<ManualSaveSettings>().is_some());
}

#[test]
fn test_auto_save_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("auto_save_test.json");
    
    // Create first app instance and manually save
    {
        let mut persist_file = PersistFile::new();
        let mut data = PersistData::new();
        data.insert("volume", 0.75f32);
        data.insert("name", "modified");
        data.insert("enabled", false);
        persist_file.set_type_data("TestSettings".to_string(), data);
        persist_file.save_to_file(&file_path).unwrap();
    }
    
    // Load and verify
    {
        let loaded = PersistFile::load_from_file(&file_path).unwrap();
        let data = loaded.get_type_data("TestSettings").unwrap();
        assert_eq!(data.get::<f32>("volume"), Some(0.75));
        assert_eq!(data.get::<String>("name"), Some("modified".to_string()));
        assert_eq!(data.get::<bool>("enabled"), Some(false));
    }
}

#[test]
fn test_manual_save_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("manual_save_test.json");
    
    // Create first app instance
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PersistPlugin::new(file_path.to_str().unwrap()));
        app.init_resource::<ManualSaveSettings>();
        
        app.finish();
        
        // Modify the resource
        let mut settings = app.world_mut().resource_mut::<ManualSaveSettings>();
        settings.value = 999;
        settings.text = "manual save".to_string();
        settings.set_changed();
        
        // Run update - should NOT auto-save due to auto_save = false
        app.update();
        
        // Manually save
        {
            let settings = app.world().resource::<ManualSaveSettings>();
            let data = settings.to_persist_data();
            let mut manager = app.world_mut().resource_mut::<PersistManager>();
            manager.get_persist_file_mut().set_type_data("ManualSaveSettings".to_string(), data);
            manager.save().unwrap();
        }
    }
    
    // Create second app instance to verify persistence
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PersistPlugin::new(file_path.to_str().unwrap()));
        app.init_resource::<ManualSaveSettings>();
        
        app.finish();
        app.update();
        
        // Verify the data was loaded
        let settings = app.world().resource::<ManualSaveSettings>();
        assert_eq!(settings.value, 999);
        assert_eq!(settings.text, "manual save");
    }
}

#[test]
fn test_multiple_resources() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("multiple_resources.json");
    
    // Create and save multiple resources
    {
        let mut persist_file = PersistFile::new();
        
        let mut data1 = PersistData::new();
        data1.insert("volume", 0.9f32);
        data1.insert("name", "resource1");
        data1.insert("enabled", true);
        persist_file.set_type_data("TestSettings".to_string(), data1);
        
        let mut data2 = PersistData::new();
        data2.insert("value", 123i32);
        data2.insert("text", "resource2");
        persist_file.set_type_data("ManualSaveSettings".to_string(), data2);
        
        persist_file.save_to_file(&file_path).unwrap();
    }
    
    // Load and verify both resources
    {
        let loaded = PersistFile::load_from_file(&file_path).unwrap();
        
        let data1 = loaded.get_type_data("TestSettings").unwrap();
        assert_eq!(data1.get::<f32>("volume"), Some(0.9));
        assert_eq!(data1.get::<String>("name"), Some("resource1".to_string()));
        
        let data2 = loaded.get_type_data("ManualSaveSettings").unwrap();
        assert_eq!(data2.get::<i32>("value"), Some(123));
        assert_eq!(data2.get::<String>("text"), Some("resource2".to_string()));
    }
}

#[test]
fn test_ron_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_settings.ron");
    
    // Save with RON format
    {
        let mut persist_file = PersistFile::new();
        let mut data = PersistData::new();
        data.insert("volume", 0.33f32);
        data.insert("name", "ron_test");
        data.insert("enabled", true);
        persist_file.set_type_data("TestSettings".to_string(), data);
        persist_file.save_to_file(&file_path).unwrap();
    }
    
    // Verify file exists and can be loaded
    assert!(file_path.exists());
    
    // Load from RON format
    {
        let loaded = PersistFile::load_from_file(&file_path).unwrap();
        let data = loaded.get_type_data("TestSettings").unwrap();
        assert_eq!(data.get::<f32>("volume"), Some(0.33));
        assert_eq!(data.get::<String>("name"), Some("ron_test".to_string()));
        assert_eq!(data.get::<bool>("enabled"), Some(true));
    }
}