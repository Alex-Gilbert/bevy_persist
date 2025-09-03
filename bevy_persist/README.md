# bevy_persist

Automatic persistence for Bevy resources with change detection.

## Features

- **Automatic Save/Load**: Resources are automatically saved when modified and loaded on startup
- **Multiple Formats**: Support for JSON and RON serialization formats
- **Change Detection**: Only saves when resources actually change, minimizing disk I/O
- **Derive Macro**: Simple `#[derive(Persist)]` to make any resource persistent
- **Flexible Configuration**: Customize save paths, formats, and save strategies per resource

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_persist = "0.1"
```

Make a resource persistent:

```rust
use bevy::prelude::*;
use bevy_persist::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Reflect, Serialize, Deserialize, Persist)]
#[persist(path = "settings.json")]
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

### Custom Save Strategies

```rust
#[derive(Resource, Reflect, Serialize, Deserialize, Persist)]
#[persist(path = "game_state.ron", format = "ron", strategy = "immediate")]
struct GameState {
    level: u32,
    score: u32,
}
```

### Multiple Persistent Resources

```rust
App::new()
    .add_plugins(PersistPlugin)
    .init_resource::<Settings>()
    .init_resource::<GameState>()
    .init_resource::<PlayerProfile>()
    .run();
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