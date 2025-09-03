use bevy::prelude::*;
use bevy_persist::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default, Serialize, Deserialize, Persist)]
struct GameSettings {
    pub volume: f32,
    pub difficulty: u32,
    pub player_name: String,
}

#[derive(Resource, Default, Serialize, Deserialize, Persist)]
#[persist(auto_save = false)] // This resource won't auto-save
struct GraphicsSettings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub vsync: bool,
    pub anti_aliasing: u8,
}

#[derive(Resource, Default, Serialize, Deserialize, Persist)]
struct PlayerProgress {
    pub level: u32,
    pub experience: u32,
    pub gold: u32,
    pub unlocked_items: Vec<String>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PersistPlugin::new("game_data.ron"))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_game_settings,
                update_graphics_settings,
                update_player_progress,
                manual_save_system,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    game_settings: Res<GameSettings>,
    graphics_settings: Res<GraphicsSettings>,
    player_progress: Res<PlayerProgress>,
) {
    commands.spawn(Camera2d);

    println!("=== Current Settings ===");
    println!("\nGame Settings:");
    println!("  Volume: {}", game_settings.volume);
    println!("  Difficulty: {}", game_settings.difficulty);
    println!("  Player Name: {}", game_settings.player_name);

    println!("\nGraphics Settings (manual save only):");
    println!("  Resolution: {:?}", graphics_settings.resolution);
    println!("  Fullscreen: {}", graphics_settings.fullscreen);
    println!("  VSync: {}", graphics_settings.vsync);
    println!("  Anti-aliasing: {}x", graphics_settings.anti_aliasing);

    println!("\nPlayer Progress:");
    println!("  Level: {}", player_progress.level);
    println!("  Experience: {}", player_progress.experience);
    println!("  Gold: {}", player_progress.gold);
    println!("  Unlocked Items: {:?}", player_progress.unlocked_items);

    println!("\n=== Controls ===");
    println!("SPACE - Modify game settings (auto-saves)");
    println!("G - Modify graphics settings (no auto-save)");
    println!("P - Modify player progress (auto-saves)");
    println!("S - Manual save all settings");
}

fn update_game_settings(mut settings: ResMut<GameSettings>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        settings.volume = (settings.volume + 0.1).min(1.0);
        settings.difficulty = (settings.difficulty + 1) % 5;

        println!("\n[Auto-saved] Updated game settings:");
        println!("  Volume: {}", settings.volume);
        println!("  Difficulty: {}", settings.difficulty);
    }
}

fn update_graphics_settings(
    mut settings: ResMut<GraphicsSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        settings.anti_aliasing = match settings.anti_aliasing {
            0 => 2,
            2 => 4,
            4 => 8,
            _ => 0,
        };
        settings.vsync = !settings.vsync;

        println!("\n[Not auto-saved] Updated graphics settings:");
        println!("  Anti-aliasing: {}x", settings.anti_aliasing);
        println!("  VSync: {}", settings.vsync);
        println!("  (Press S to save manually)");
    }
}

fn update_player_progress(
    mut progress: ResMut<PlayerProgress>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        progress.experience += 100;
        if progress.experience >= 1000 {
            progress.level += 1;
            progress.experience = 0;
            progress.gold += 50;
            let new_item = format!("Item_{}", progress.level);
            progress.unlocked_items.push(new_item);
        }

        println!("\n[Auto-saved] Updated player progress:");
        println!("  Level: {}", progress.level);
        println!("  Experience: {}", progress.experience);
        println!("  Gold: {}", progress.gold);
        if let Some(last_item) = progress.unlocked_items.last() {
            println!("  New item unlocked: {}", last_item);
        }
    }
}

fn manual_save_system(
    mut manager: ResMut<PersistManager>,
    keyboard: Res<ButtonInput<KeyCode>>,
    graphics_settings: Res<GraphicsSettings>,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        // Manually save graphics settings
        let data = graphics_settings.to_persist_data();
        manager
            .get_persist_file_mut()
            .set_type_data("GraphicsSettings".to_string(), data);

        if let Err(e) = manager.save() {
            eprintln!("Failed to save: {}", e);
        } else {
            println!("\n[Manual Save] All settings saved to disk!");
        }
    }
}
