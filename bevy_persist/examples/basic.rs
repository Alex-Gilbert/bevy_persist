use bevy::prelude::*;
use bevy_persist::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

// User settings that should persist across game sessions
// These are things the player can change in the options menu
#[derive(Resource, Default, Serialize, Deserialize, Persist)]
#[persist(dynamic)]  // Save to platform-specific user config directory
struct UserSettings {
    pub volume: f32,
    pub graphics_quality: u32,
    pub player_name: String,
}

fn main() {
    // Initialize logging
    env_logger::init();
    
    // Create a simple Bevy app without any rendering
    let mut app = App::new();
    
    // Configure plugin with app info (required)
    let persist_plugin = PersistPlugin::new("ExampleCompany", "BasicExample");
    
    app.add_plugins(MinimalPlugins)
        .add_plugins(persist_plugin)
        // UserSettings is auto-registered by the Persist derive macro
        .add_systems(Startup, display_settings)
        .add_systems(Update, cli_interface);
    
    println!("\n=== Bevy Persist Basic Example ===");
    println!("This example demonstrates user settings persistence.\n");
    
    #[cfg(feature = "dev")]
    println!("DEV MODE: Settings saved to local 'basicexample_dev.ron' for testing");
    
    #[cfg(feature = "prod")]
    {
        println!("PRODUCTION MODE: Settings saved to:");
        #[cfg(target_os = "windows")]
        println!("  %APPDATA%\\ExampleCompany\\BasicExample\\usersettings.ron");
        #[cfg(target_os = "macos")]
        println!("  ~/Library/Application Support/ExampleCompany/BasicExample/usersettings.ron");
        #[cfg(target_os = "linux")]
        println!("  ~/.config/ExampleCompany/BasicExample/usersettings.ron");
    }
    
    println!();
    
    // Run the app
    app.run();
}

fn display_settings(settings: Res<UserSettings>) {
    println!("Current settings (loaded from disk if exists):");
    println!("  Volume: {:.1}%", settings.volume * 100.0);
    println!("  Graphics Quality: {}", match settings.graphics_quality {
        0 => "Low",
        1 => "Medium",
        2 => "High",
        3 => "Ultra",
        _ => "Custom"
    });
    println!("  Player Name: {}", if settings.player_name.is_empty() { 
        "<not set>" 
    } else { 
        &settings.player_name 
    });
    
    println!("\n=== Commands ===");
    println!("1 - Adjust volume");
    println!("2 - Change graphics quality");
    println!("3 - Set player name");
    println!("4 - Display current settings");
    println!("q - Quit");
    println!("\nSettings auto-save on change!\n");
}

fn cli_interface(
    mut settings: ResMut<UserSettings>,
    mut exit: EventWriter<AppExit>,
) {
    print!("> ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        match input.trim() {
            "1" => {
                settings.volume = ((settings.volume + 0.1).min(1.0) * 10.0).round() / 10.0;
                println!("Volume set to: {:.0}%", settings.volume * 100.0);
            }
            "2" => {
                settings.graphics_quality = (settings.graphics_quality + 1) % 4;
                println!("Graphics quality set to: {}", match settings.graphics_quality {
                    0 => "Low",
                    1 => "Medium",
                    2 => "High",
                    3 => "Ultra",
                    _ => "Custom"
                });
            }
            "3" => {
                print!("Enter player name: ");
                io::stdout().flush().unwrap();
                let mut name = String::new();
                if io::stdin().read_line(&mut name).is_ok() {
                    settings.player_name = name.trim().to_string();
                    println!("Player name set to: {}", settings.player_name);
                }
            }
            "4" => {
                println!("\nCurrent settings:");
                println!("  Volume: {:.0}%", settings.volume * 100.0);
                println!("  Graphics Quality: {}", match settings.graphics_quality {
                    0 => "Low",
                    1 => "Medium", 
                    2 => "High",
                    3 => "Ultra",
                    _ => "Custom"
                });
                println!("  Player Name: {}", if settings.player_name.is_empty() { 
                    "<not set>" 
                } else { 
                    &settings.player_name 
                });
            }
            "q" | "quit" | "exit" => {
                println!("Exiting... (settings have been auto-saved)");
                exit.send(AppExit::Success);
            }
            _ => {
                if !input.trim().is_empty() {
                    println!("Unknown command: {}", input.trim());
                }
            }
        }
    }
}