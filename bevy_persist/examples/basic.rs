use bevy::prelude::*;
use bevy_persist::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default, Serialize, Deserialize, Persist)]
struct GameSettings {
    pub volume: f32,
    pub difficulty: u32,
    pub player_name: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PersistPlugin::new("game_settings.ron"))
        .add_systems(Startup, setup)
        .add_systems(Update, update_settings)
        .run();
}

fn setup(mut commands: Commands, settings: Res<GameSettings>) {
    commands.spawn(Camera2d);

    println!("Current settings:");
    println!("  Volume: {}", settings.volume);
    println!("  Difficulty: {}", settings.difficulty);
    println!("  Player Name: {}", settings.player_name);
    println!("\nPress SPACE to modify settings (they will auto-save)");
}

fn update_settings(mut settings: ResMut<GameSettings>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        settings.volume = (settings.volume + 0.1).min(1.0);
        settings.difficulty = (settings.difficulty + 1) % 5;

        println!("Updated settings:");
        println!("  Volume: {}", settings.volume);
        println!("  Difficulty: {}", settings.difficulty);
    }
}
