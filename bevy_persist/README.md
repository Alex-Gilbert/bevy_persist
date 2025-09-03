# bevy_persist

Automatic persistence for Bevy resources with change detection.

## Features

- **Automatic Save/Load**: Resources are automatically saved when modified and loaded on startup
- **Multiple Formats**: Support for JSON and RON serialization formats  
- **Change Detection**: Only saves when resources actually change, minimizing disk I/O
- **Derive Macro**: Simple `#[derive(Persist)]` to make any resource persistent
- **Flexible Configuration**: Customize save paths, formats, and save strategies per resource

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_persist = "0.1.0"
```

## Quick Start

```rust
use bevy::prelude::*;
use bevy_persist::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default, Serialize, Deserialize, Persist)]
struct Settings {
    volume: f32,
    difficulty: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PersistPlugin)
        .init_resource::<Settings>()
        .run();
}
```

## Advanced Usage

### Manual Save Control

```rust
#[derive(Resource, Serialize, Deserialize, Persist)]
#[persist(auto_save = false)]
struct GraphicsSettings {
    resolution: (u32, u32),
    fullscreen: bool,
}

// Manual save in a system
fn save_graphics(
    mut manager: ResMut<PersistManager>,
    settings: Res<GraphicsSettings>,
) {
    if settings.is_changed() {
        let data = settings.to_persist_data();
        manager.get_persist_file_mut()
            .set_type_data("GraphicsSettings".to_string(), data);
        manager.save().expect("Failed to save");
    }
}
```

### RON Format

Use RON format for more readable configuration files:

```rust
App::new()
    .add_plugins(PersistPlugin::new("settings.ron"))
    // ...
```

## Examples

Check out the `examples/` directory for more usage examples:
- `basic.rs` - Simple settings persistence
- `advanced.rs` - Complex game state with multiple persistent resources

Run examples with:
```bash
cargo run --example basic
cargo run --example advanced
```

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.