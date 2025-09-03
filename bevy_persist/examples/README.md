# Bevy Persist Examples

These examples demonstrate best practices for using `bevy_persist` in real games.

## Basic Example - User Settings
Shows how to persist user preferences (volume, graphics, player name) that should always be saved to platform-specific directories.

**Key Pattern:** Uses `#[persist(dynamic)]` for user-modifiable settings.

```bash
# Run in development mode (saves to local dev_settings.ron)
cargo run --example basic

# Run in production mode (saves to platform config directory)
cargo run --example basic --no-default-features --features prod
```

## Advanced Example - Complete Game Persistence
Demonstrates all three persistence strategies in a real game context:

1. **Game Balance** (`#[persist(embed)]` in prod) - Constants tweaked during development
2. **User Preferences** (`#[persist(dynamic)]`) - Player's personal settings  
3. **Save Game Data** (`#[persist(secure)]`) - Protected game progress

```bash
# Development: All data in one file for easy testing
cargo run --example advanced

# Production: Proper separation of concerns
cargo run --example advanced --no-default-features --features prod
```

This example clearly shows how the same code behaves differently in dev vs prod modes,
with appropriate messages and behavior changes based on the active features.

## When to Use Each Mode

### `#[persist(embed)]` - Compiled Constants
- Game balance values
- Level data
- Enemy stats
- Item properties
- Any data you tweak during development but ship as read-only

### `#[persist(dynamic)]` - User Settings
- Audio/video preferences
- Control bindings
- UI preferences
- Accessibility options
- Any settings the player should be able to modify

### `#[persist(secure)]` - Protected Data
- Save games
- Player progress
- Achievements
- Unlocked content
- Any data that shouldn't be easily tampered with

## Development Workflow

1. **During Development**: Use default features, everything saves to local RON files
2. **Tweak Values**: Modify game balance through your game's debug UI
3. **Test Production**: Build with `--features prod` to test production behavior
4. **Ship**: Embedded values are compiled in, user data goes to proper directories

## Platform Directories

In production mode with `.with_app_info("YourCompany", "YourGame")`:

### Windows
- User Settings: `%APPDATA%\YourCompany\YourGame\*.ron`
- Save Games: `%LOCALAPPDATA%\YourCompany\YourGame\*.dat`

### macOS
- User Settings: `~/Library/Application Support/YourCompany/YourGame/*.ron`
- Save Games: `~/Library/Application Support/YourCompany/YourGame/*.dat`

### Linux
- User Settings: `~/.config/YourCompany/YourGame/*.ron`
- Save Games: `~/.local/share/YourCompany/YourGame/*.dat`
