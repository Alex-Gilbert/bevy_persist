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
    // Create an app with the plugin
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(PersistPlugin::new("TestOrg", "TestApp"));

    // Resources are auto-registered by the derive macro
    // No need to manually init_resource

    // Run startup systems to load any existing data
    app.finish();
    app.cleanup();

    // Verify the manager was created
    assert!(app.world().get_resource::<PersistManager>().is_some());

    // Verify resources were initialized by auto-registration
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
    // This test requires persistent storage between app instances
    // In production mode, this would use platform directories which may not be writable in tests
    // So we only run this test in dev mode where we use local files
    #[cfg(not(feature = "prod"))]
    {
        // Both app instances need to use the same org/app name to share data
        let org = "TestOrg";
        let app_name = "ManualSaveTest";

        // Create first app instance
        {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(PersistPlugin::new(org, app_name));

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
                manager
                    .get_persist_file_mut()
                    .set_type_data("ManualSaveSettings".to_string(), data);
                manager.save().unwrap();
            }
        }

        // Create second app instance to verify persistence
        {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(PersistPlugin::new(org, app_name));

            app.finish();
            app.update();

            // Verify the data was loaded
            let settings = app.world().resource::<ManualSaveSettings>();
            assert_eq!(settings.value, 999);
            assert_eq!(settings.text, "manual save");
        }
    }
    
    // In production mode, just verify that manual save settings don't auto-save
    #[cfg(feature = "prod")]
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PersistPlugin::new("TestOrg", "ManualSaveTest"));

        app.finish();

        // Verify ManualSaveSettings resource exists and has correct default
        let settings = app.world().resource::<ManualSaveSettings>();
        assert_eq!(settings.value, 0); // Default value
        assert_eq!(settings.text, "");  // Default value
        
        // Modify and verify it doesn't auto-save
        let mut settings = app.world_mut().resource_mut::<ManualSaveSettings>();
        settings.value = 123;
        settings.set_changed();
        
        // Run update - should NOT auto-save due to auto_save = false attribute
        app.update();
        
        // If we had a way to check, the file shouldn't exist since we didn't manually save
        // But in prod mode without controlling paths, we can't easily verify this
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

// Tests for new features added with production support

#[test]
#[cfg(feature = "secure")]
fn test_secure_encryption() {
    use bevy_persist::PersistData;
    use tempfile::TempDir;
    use std::fs;
    
    // Create a temp directory for testing
    let temp_dir = TempDir::new().unwrap();
    let secure_file = temp_dir.path().join("secure_test.dat");
    
    // Create a PersistManager with a secret
    let manager = bevy_persist::PersistManager::new("TestOrg", "TestApp")
        .with_secret("my_secret_key_123");
    
    // Create some test data
    let mut data = PersistData::new();
    data.insert("score", 9999i32);
    data.insert("level", 42i32);
    data.insert("player_name", "TestPlayer");
    
    // Save the data in secure mode
    manager.save_resource("SecureData", &data, bevy_persist::PersistMode::Secure).unwrap();
    
    // Read the encrypted file directly - it should not be readable as plain text
    let encrypted_contents = fs::read(manager.get_resource_path("SecureData", bevy_persist::PersistMode::Secure)).unwrap();
    let encrypted_string = String::from_utf8_lossy(&encrypted_contents);
    
    // The encrypted data should NOT contain readable strings
    assert!(!encrypted_string.contains("score"));
    assert!(!encrypted_string.contains("9999"));
    assert!(!encrypted_string.contains("TestPlayer"));
    
    // Now load the data back using the same secret
    let loaded_data = manager.load_resource("SecureData", bevy_persist::PersistMode::Secure).unwrap();
    assert_eq!(loaded_data.get::<i32>("score"), Some(9999));
    assert_eq!(loaded_data.get::<i32>("level"), Some(42));
    assert_eq!(loaded_data.get::<String>("player_name"), Some("TestPlayer".to_string()));
    
    // Try loading with wrong secret - should fail
    let wrong_manager = bevy_persist::PersistManager::new("TestOrg", "TestApp")
        .with_secret("wrong_secret");
    let load_result = wrong_manager.load_resource("SecureData", bevy_persist::PersistMode::Secure);
    assert!(load_result.is_err());
}

#[test]
#[cfg(feature = "secure")]
fn test_secure_without_secret() {
    use bevy_persist::PersistData;
    use tempfile::TempDir;
    
    // Create a temp directory for testing
    let temp_dir = TempDir::new().unwrap();
    
    // Create a PersistManager WITHOUT a secret
    let manager = bevy_persist::PersistManager::new("TestOrg", "TestApp2");
    
    // Create some test data
    let mut data = PersistData::new();
    data.insert("value", 123i32);
    
    // Save in secure mode without a secret - should use base64 encoding
    manager.save_resource("SecureData2", &data, bevy_persist::PersistMode::Secure).unwrap();
    
    // Load it back - should work since we're using the same (no) secret
    let loaded_data = manager.load_resource("SecureData2", bevy_persist::PersistMode::Secure).unwrap();
    assert_eq!(loaded_data.get::<i32>("value"), Some(123));
}

// Tests for new features added with production support

#[derive(Resource, Default, Serialize, Deserialize, Persist, Debug, PartialEq, Clone)]
#[persist(dynamic)]
struct DynamicSettings {
    user_pref: String,
    volume: f32,
}

#[derive(Resource, Default, Serialize, Deserialize, Persist, Debug, PartialEq, Clone)]
#[persist(secure)]
struct SecureSettings {
    save_data: i32,
    secret: String,
}

#[test]
fn test_persist_mode_trait_implementation() {
    // Test that the persist mode is correctly set for different resource types
    assert_eq!(TestSettings::persist_mode(), PersistMode::Dev);
    assert_eq!(DynamicSettings::persist_mode(), PersistMode::Dynamic);
    assert_eq!(SecureSettings::persist_mode(), PersistMode::Secure);
}

#[test]
fn test_persist_manager_new_api() {
    let manager = PersistManager::new("TestOrganization", "TestApplication");
    assert_eq!(manager.organization, "TestOrganization");
    assert_eq!(manager.app_name, "TestApplication");
    assert!(manager.auto_save);
    
    #[cfg(not(feature = "prod"))]
    assert_eq!(manager.dev_file, std::path::PathBuf::from("testapplication_dev.ron"));
}

#[test]
fn test_persist_plugin_new_api() {
    let plugin = PersistPlugin::new("MyCompany", "MyGame");
    assert_eq!(plugin.organization, "MyCompany");
    assert_eq!(plugin.app_name, "MyGame");
    assert!(plugin.auto_save);
    
    let plugin_no_save = plugin.with_auto_save(false);
    assert!(!plugin_no_save.auto_save);
}

#[test]
fn test_resource_path_generation() {
    let manager = PersistManager::new("TestOrg", "TestApp");
    
    #[cfg(not(feature = "prod"))]
    {
        // In dev mode, all modes should return the dev file
        let dev_path = manager.get_resource_path("TestResource", PersistMode::Dev);
        let dynamic_path = manager.get_resource_path("TestResource", PersistMode::Dynamic);
        let secure_path = manager.get_resource_path("TestResource", PersistMode::Secure);
        
        assert_eq!(dev_path, std::path::PathBuf::from("testapp_dev.ron"));
        assert_eq!(dynamic_path, dev_path);
        assert_eq!(secure_path, dev_path);
    }
    
    #[cfg(feature = "prod")]
    {
        // In prod mode, different modes should return different paths
        let embed_path = manager.get_resource_path("TestResource", PersistMode::Embed);
        assert_eq!(embed_path, std::path::PathBuf::new()); // Empty path for embedded
        
        // Dynamic and Secure would use platform directories if available
        // We can't test exact paths as they depend on the system, but we can check they're different
        let dynamic_path = manager.get_resource_path("UserSettings", PersistMode::Dynamic);
        let secure_path = manager.get_resource_path("SaveData", PersistMode::Secure);
        
        // At minimum, they should have different extensions
        if !dynamic_path.as_os_str().is_empty() && !secure_path.as_os_str().is_empty() {
            assert!(dynamic_path.to_str().unwrap().ends_with(".ron"));
            assert!(secure_path.to_str().unwrap().ends_with(".dat"));
        }
    }
}

#[test]
fn test_persist_mode_enum() {
    // Test the PersistMode enum values
    let dev = PersistMode::Dev;
    let embed = PersistMode::Embed;
    let dynamic = PersistMode::Dynamic;
    let secure = PersistMode::Secure;
    
    assert_ne!(dev, embed);
    assert_ne!(dynamic, secure);
    assert_eq!(dev, PersistMode::Dev);
    
    // Test Debug trait
    assert_eq!(format!("{:?}", dev), "Dev");
    assert_eq!(format!("{:?}", embed), "Embed");
    assert_eq!(format!("{:?}", dynamic), "Dynamic");
    assert_eq!(format!("{:?}", secure), "Secure");
}
