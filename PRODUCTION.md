# Production Usage Guide

This guide explains how to use `bevy_persist` in production environments, including the different persistence modes and feature flags.

## Feature Flags

### Development Mode (default)
```toml
[dependencies]
bevy_persist = "0.1.0"  # Uses default "dev" feature
```

In development mode:
- All resources are saved to a single local file (e.g., `settings.ron`)
- Perfect for tweaking game parameters during development
- Changes are instantly visible after restart
- No platform-specific paths

### Production Mode
```toml
[dependencies]
bevy_persist = { version = "0.1.0", default-features = false, features = ["prod"] }
```

In production mode:
- Different persistence modes available (embed, dynamic, secure)
- Platform-specific directories for user data
- Embedded resources compiled into the binary
- Optional encryption for save data

## Persistence Modes

### 1. Embed Mode - Compile-time Constants
Use for game balance, level data, and other values you tweak during development but ship as constants.

```rust
// During development, this saves to your local RON file
// In production, the RON file is embedded in the binary
#[derive(Resource, Serialize, Deserialize, Persist)]
#[cfg_attr(feature = "prod", persist(embed = "game_balance.ron"))]
struct GameBalance {
    enemy_health: f32,
    player_damage: f32,
    spawn_rate: f32,
}
```

**Development workflow:**
1. Run your game in dev mode
2. Tweak values through your game's debug UI
3. Values auto-save to `game_balance.ron`
4. When ready to ship, the RON file is embedded in the binary

**Production behavior:**
- Values are read from the embedded file
- No disk writes occur
- Players cannot modify these values

### 2. Dynamic Mode - User Settings
Use for graphics settings, audio preferences, keybindings, etc.

```rust
#[derive(Resource, Serialize, Deserialize, Persist)]
#[persist(dynamic)]
struct UserSettings {
    volume: f32,
    graphics_quality: u32,
    fullscreen: bool,
}
```

**Production paths:**
- **Windows**: `%APPDATA%\YourCompany\YourGame\usersettings.ron`
- **macOS**: `~/Library/Application Support/YourCompany/YourGame/usersettings.ron`
- **Linux**: `~/.config/YourCompany/YourGame/usersettings.ron`

### 3. Secure Mode - Protected Save Data
Use for save games, player progress, achievements, etc.

```rust
#[derive(Resource, Serialize, Deserialize, Persist)]
#[persist(secure)]
struct SaveGame {
    level: u32,
    inventory: Vec<Item>,
    achievements: HashSet<String>,
}
```

**Production paths:**
- **Windows**: `%LOCALAPPDATA%\YourCompany\YourGame\savegame.dat`
- **macOS**: `~/Library/Application Support/YourCompany/YourGame/savegame.dat`
- **Linux**: `~/.local/share/YourCompany/YourGame/savegame.dat`

**Security features** (when `secure` feature enabled):
- TODO: Basic obfuscation to discourage casual editing
- TODO: Optional encryption for sensitive data
- Different file extension (`.dat` instead of `.ron`)

## Setting Up Your App

### Basic Setup
```rust
use bevy::prelude::*;
use bevy_persist::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            PersistPlugin::new("dev_settings.ron")  // Dev file
                .with_app_info("YourCompany", "YourGame")  // For production paths
        )
        .init_resource::<GameBalance>()
        .init_resource::<UserSettings>()
        .init_resource::<SaveGame>()
        .run();
}
```

### Conditional Compilation
```rust
// Different behavior for dev vs prod
#[cfg(feature = "dev")]
fn tweaking_ui(mut balance: ResMut<GameBalance>) {
    // Show UI for tweaking values
}

#[cfg(feature = "prod")]
fn tweaking_ui() {
    // No-op in production
}
```

## Building for Production

### Development Build
```bash
# Default dev mode - all persistence goes to local files
cargo run --example production

# Explicitly with dev feature
cargo run --example production --features dev
```

### Production Build
```bash
# Production mode with platform-specific paths
cargo run --example production --no-default-features --features prod

# Production with secure/encryption features
cargo run --example production --no-default-features --features secure

# Release build for distribution
cargo build --release --no-default-features --features prod
```

## Migration from Development to Production

### Step 1: Identify Resource Types
Categorize your resources:
- **Game Constants** → Use `#[persist(embed)]`
- **User Preferences** → Use `#[persist(dynamic)]`
- **Save Data** → Use `#[persist(secure)]`

### Step 2: Prepare Embedded Data
1. Run your game in dev mode
2. Tweak all values to desired ship values
3. Copy the generated RON files to your project
4. Reference them in the embed attribute

### Step 3: Test Production Build
```bash
# Test with production features locally
cargo run --no-default-features --features prod

# Verify file locations
# - Check platform-specific directories are created
# - Ensure embedded resources load correctly
# - Test save/load of dynamic and secure resources
```

### Step 4: Package for Distribution
1. Build with `--release` and production features
2. Embedded resources are compiled into the binary
3. No need to ship RON files
4. First run will create necessary directories

## Best Practices

### DO:
- Use `embed` mode for game balance and tuning parameters
- Use `dynamic` mode for user-facing settings
- Use `secure` mode for progress and achievements
- Provide app info for proper platform paths
- Test production builds before shipping

### DON'T:
- Don't use `embed` for data that changes per-user
- Don't use `dynamic` for game balance (too easy to modify)
- Don't ship development RON files with your game
- Don't hardcode paths - use the platform-specific system

## Example: Complete Game Setup

```rust
use bevy::prelude::*;
use bevy_persist::prelude::*;
use serde::{Deserialize, Serialize};

// Game constants - embedded in production
#[derive(Resource, Serialize, Deserialize, Persist)]
#[cfg_attr(feature = "prod", persist(embed = "balance/game.ron"))]
struct GameBalance {
    enemy_stats: EnemyStats,
    weapon_damage: WeaponTable,
    level_progression: Vec<LevelData>,
}

// User settings - saved to config directory
#[derive(Resource, Serialize, Deserialize, Persist)]
#[persist(dynamic)]
struct Settings {
    audio: AudioSettings,
    video: VideoSettings,
    controls: KeyBindings,
}

// Save game - protected from tampering
#[derive(Resource, Serialize, Deserialize, Persist)]
#[persist(secure)]
struct PlayerSave {
    current_level: usize,
    unlocked_abilities: Vec<String>,
    statistics: PlayerStats,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            PersistPlugin::default()
                .with_app_info("AwesomeGames", "SuperAdventure")
        )
        .init_resource::<GameBalance>()
        .init_resource::<Settings>()
        .init_resource::<PlayerSave>()
        .run();
}
```

## Troubleshooting

### Files not saving in production
- Ensure you've set app info with `.with_app_info()`
- Check directory permissions
- Verify the `prod` feature is enabled

### Embedded data not loading
- Ensure the RON/JSON file path is relative to your Cargo.toml
- Verify the file exists at compile time
- Check that the file contains the correct resource type

### Platform-specific issues
- Windows: Check `%APPDATA%` and `%LOCALAPPDATA%` environment variables
- macOS: Ensure app has file system permissions
- Linux: Check XDG environment variables

## Future Enhancements

The following features are planned but not yet implemented:
- Actual encryption for secure mode
- Compression for large save files
- Cloud save synchronization support
- Migration tools for save format changes
- Checksum validation for tamper detection