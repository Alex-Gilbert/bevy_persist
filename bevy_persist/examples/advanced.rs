use bevy::prelude::*;
use bevy_persist::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

// Game balance settings - These are tweaked during development
// In production, they're embedded in the binary as constants
#[derive(Resource, Default, Serialize, Deserialize, Persist)]
#[cfg_attr(not(feature = "prod"), persist(auto_save = true))]  // Auto-save in dev
#[cfg_attr(feature = "prod", persist(embed = "game_balance.ron"))]  // Embed in prod
struct GameBalance {
    pub enemy_health_base: f32,
    pub player_damage_base: f32,
    pub xp_multiplier: f32,
    pub drop_rate_common: f32,
    pub drop_rate_rare: f32,
}

// User preferences - Always saved to platform-specific directories
#[derive(Resource, Default, Serialize, Deserialize, Persist)]
#[persist(dynamic)]
struct UserPreferences {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub graphics_quality: u32,
    pub vsync: bool,
    pub fullscreen: bool,
}

// Player save data - Should be protected from tampering
#[derive(Resource, Default, Serialize, Deserialize, Persist)]
#[persist(secure, auto_save = false)]  // Manual save only, protected location
struct PlayerProgress {
    pub level: u32,
    pub experience: u32,
    pub gold: u32,
    pub unlocked_items: Vec<String>,
    pub achievements: Vec<String>,
    pub playtime_seconds: u32,
}

fn main() {
    env_logger::init();
    
    let mut app = App::new();
    
    // Production-ready configuration
    #[cfg(feature = "secure")]
    let persist_plugin = PersistPlugin::new("ExampleStudio", "AdvancedGame")
        .with_secret("demo_secret_2024"); // In production, derive from hardware ID or user account
    
    #[cfg(not(feature = "secure"))]
    let persist_plugin = PersistPlugin::new("ExampleStudio", "AdvancedGame");
    
    app.add_plugins(MinimalPlugins)
        .add_plugins(persist_plugin)
        // Resources are auto-registered by the Persist derive macro
        .add_systems(Startup, setup)
        .add_systems(Update, game_loop);
    
    println!("\n=== Bevy Persist Advanced Example ===");
    println!("Demonstrates multiple persistence strategies for a real game.\n");
    
    #[cfg(feature = "dev")]
    {
        println!("🔧 DEVELOPMENT MODE:");
        println!("  • Game Balance: Saved to advancedgame_dev.ron (tweakable)");
        println!("  • User Preferences: Saved to advancedgame_dev.ron");
        println!("  • Player Progress: Saved to advancedgame_dev.ron");
        println!("\nAll data in one file for easy testing and tweaking!");
    }
    
    #[cfg(feature = "prod")]
    {
        println!("📦 PRODUCTION MODE:");
        println!("  • Game Balance: Embedded from game_balance.ron (read-only)");
        println!("  • User Preferences: Saved to user config directory");
        #[cfg(feature = "secure")]
        println!("  • Player Progress: Encrypted with AES-256-GCM (.dat file)");
        #[cfg(not(feature = "secure"))]
        println!("  • Player Progress: Saved to user data directory (protected)");
        println!("\nProper separation of concerns for shipping!");
    }
    
    println!();
    app.run();
}

fn setup(
    balance: Res<GameBalance>,
    prefs: Res<UserPreferences>,
    progress: Res<PlayerProgress>,
) {
    println!("=== Current Game State ===\n");
    
    println!("Game Balance ({}editable):", 
        if cfg!(feature = "prod") { "not " } else { "" });
    println!("  Enemy Base Health: {:.1}", balance.enemy_health_base);
    println!("  Player Base Damage: {:.1}", balance.player_damage_base);
    println!("  XP Multiplier: {:.2}x", balance.xp_multiplier);
    println!("  Drop Rates: Common {:.1}%, Rare {:.1}%", 
        balance.drop_rate_common * 100.0, balance.drop_rate_rare * 100.0);
    
    println!("\nUser Preferences:");
    println!("  Volume: Master {:.0}%, Music {:.0}%, SFX {:.0}%", 
        prefs.master_volume * 100.0, 
        prefs.music_volume * 100.0, 
        prefs.sfx_volume * 100.0);
    println!("  Graphics: {}", match prefs.graphics_quality {
        0 => "Low", 1 => "Medium", 2 => "High", 3 => "Ultra", _ => "Custom"
    });
    println!("  Display: {} {}", 
        if prefs.fullscreen { "Fullscreen" } else { "Windowed" },
        if prefs.vsync { "VSync ON" } else { "VSync OFF" });
    
    println!("\nPlayer Progress:");
    println!("  Level {} ({} XP)", progress.level, progress.experience);
    println!("  Gold: {}", progress.gold);
    println!("  Unlocked Items: {}", progress.unlocked_items.len());
    println!("  Achievements: {}/{}", progress.achievements.len(), 10);
    println!("  Playtime: {} hours", progress.playtime_seconds / 3600);
    
    println!("\n=== Commands ===");
    println!("1 - Adjust game balance (dev only)");
    println!("2 - Change user preferences");
    println!("3 - Simulate gameplay progress");
    println!("4 - Manual save progress");
    println!("5 - Display all current values");
    println!("q - Quit");
    println!();
}

fn game_loop(
    mut balance: ResMut<GameBalance>,
    mut prefs: ResMut<UserPreferences>,
    mut progress: ResMut<PlayerProgress>,
    mut manager: ResMut<PersistManager>,
    mut exit: EventWriter<AppExit>,
) {
    print!("> ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        match input.trim() {
            "1" => {
                #[cfg(feature = "dev")]
                {
                    println!("\n[Dev Mode] Tweaking game balance...");
                    balance.enemy_health_base *= 1.1;
                    balance.player_damage_base *= 1.05;
                    balance.xp_multiplier *= 0.95;
                    balance.drop_rate_rare *= 1.2;
                    
                    println!("  Enemy Health: {:.1} (+10%)", balance.enemy_health_base);
                    println!("  Player Damage: {:.1} (+5%)", balance.player_damage_base);
                    println!("  XP Rate: {:.2}x (-5%)", balance.xp_multiplier);
                    println!("  Rare Drops: {:.1}% (+20%)", balance.drop_rate_rare * 100.0);
                    println!("\nChanges auto-saved to advancedgame_dev.ron!");
                    println!("💡 TIP: These values will be embedded when building with --features prod");
                }
                
                #[cfg(feature = "prod")]
                {
                    println!("\n❌ Game balance is read-only in production!");
                    println!("These values were embedded at compile time from game_balance.ron");
                    println!("To change them, rebuild in dev mode, tweak, then rebuild for production.");
                }
            }
            "2" => {
                println!("\nAdjusting user preferences...");
                prefs.master_volume = ((prefs.master_volume + 0.1).min(1.0) * 10.0).round() / 10.0;
                prefs.graphics_quality = (prefs.graphics_quality + 1) % 4;
                prefs.vsync = !prefs.vsync;
                
                println!("  Master Volume: {:.0}%", prefs.master_volume * 100.0);
                println!("  Graphics: {}", match prefs.graphics_quality {
                    0 => "Low", 1 => "Medium", 2 => "High", 3 => "Ultra", _ => "Custom"
                });
                println!("  VSync: {}", if prefs.vsync { "ON" } else { "OFF" });
                
                #[cfg(feature = "prod")]
                println!("\n✅ Saved to user preferences directory");
                #[cfg(not(feature = "prod"))]
                println!("\n✅ Auto-saved to advancedgame_dev.ron");
            }
            "3" => {
                println!("\nSimulating gameplay...");
                
                // Simulate gaining XP and leveling up
                let xp_gained = 250;
                progress.experience += xp_gained;
                println!("  Gained {} XP!", xp_gained);
                
                if progress.experience >= (progress.level + 1) * 1000 {
                    progress.level += 1;
                    progress.experience = 0;
                    println!("  LEVEL UP! Now level {}", progress.level);
                    
                    // Unlock item every 2 levels
                    if progress.level % 2 == 0 {
                        let item = format!("Legendary_Sword_{}", progress.level / 2);
                        progress.unlocked_items.push(item.clone());
                        println!("  🎉 Unlocked: {}", item);
                    }
                }
                
                // Simulate finding gold
                let gold_found = 50 + (progress.level * 10);
                progress.gold += gold_found;
                println!("  Found {} gold (total: {})", gold_found, progress.gold);
                
                // Update playtime
                progress.playtime_seconds += 60;
                
                println!("\n⚠️  Progress NOT auto-saved (manual save required)");
                println!("Use command '4' to save your progress");
            }
            "4" => {
                println!("\nManually saving player progress...");
                
                // Force save the PlayerProgress resource
                let data = progress.to_persist_data();
                
                #[cfg(feature = "prod")]
                {
                    let path = manager.get_resource_path("PlayerProgress", PersistMode::Secure);
                    let mut file = PersistFile::new();
                    file.set_type_data("PlayerProgress".to_string(), data);
                    
                    if let Err(e) = file.save_to_file(&path) {
                        println!("❌ Failed to save progress: {}", e);
                    } else {
                        println!("✅ Progress saved to secure location: {:?}", path);
                    }
                }
                
                #[cfg(not(feature = "prod"))]
                {
                    manager.get_persist_file_mut()
                        .set_type_data("PlayerProgress".to_string(), data);
                    if let Err(e) = manager.save() {
                        println!("❌ Failed to save progress: {}", e);
                    } else {
                        println!("✅ Progress saved to advancedgame_dev.ron");
                    }
                }
            }
            "5" => {
                println!("\n=== All Current Values ===");
                
                println!("\nGame Balance:");
                println!("  Enemy Health: {:.1}", balance.enemy_health_base);
                println!("  Player Damage: {:.1}", balance.player_damage_base);
                println!("  XP Multiplier: {:.2}x", balance.xp_multiplier);
                println!("  Drop Rates: Common {:.1}%, Rare {:.1}%", 
                    balance.drop_rate_common * 100.0, balance.drop_rate_rare * 100.0);
                
                println!("\nUser Preferences:");
                println!("  Master Volume: {:.0}%", prefs.master_volume * 100.0);
                println!("  Music Volume: {:.0}%", prefs.music_volume * 100.0);
                println!("  SFX Volume: {:.0}%", prefs.sfx_volume * 100.0);
                println!("  Graphics: {}", match prefs.graphics_quality {
                    0 => "Low", 1 => "Medium", 2 => "High", 3 => "Ultra", _ => "Custom"
                });
                println!("  Fullscreen: {}", prefs.fullscreen);
                println!("  VSync: {}", prefs.vsync);
                
                println!("\nPlayer Progress:");
                println!("  Level: {}", progress.level);
                println!("  Experience: {}/{}", progress.experience, (progress.level + 1) * 1000);
                println!("  Gold: {}", progress.gold);
                println!("  Items: {:?}", progress.unlocked_items);
                println!("  Achievements: {:?}", progress.achievements);
                println!("  Playtime: {} hours {} minutes", 
                    progress.playtime_seconds / 3600,
                    (progress.playtime_seconds % 3600) / 60);
            }
            "q" | "quit" => {
                println!("\nShutting down...");
                
                // Save progress before exiting
                let data = progress.to_persist_data();
                manager.get_persist_file_mut()
                    .set_type_data("PlayerProgress".to_string(), data);
                let _ = manager.save();
                
                println!("✓ User preferences saved (auto)");
                println!("✓ Player progress saved (manual)");
                
                #[cfg(feature = "dev")]
                println!("✓ Game balance saved (for embedding later)");
                
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