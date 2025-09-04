# bevy_persist

Automatic persistence for Bevy resources with change detection.

## Features

- **Automatic Save/Load**: Resources are automatically saved when modified and loaded on startup
- **Multiple Formats**: Support for JSON and RON serialization formats  
- **Change Detection**: Only saves when resources actually change, minimizing disk I/O
- **Derive Macro**: Simple `#[derive(Persist)]` to make any resource persistent
- **Flexible Configuration**: Customize save paths, formats, and save strategies per resource
- **Production Ready**: Different persistence modes for development vs production
- **Platform Support**: Automatic platform-specific paths for user data
- **Embedded Resources**: Compile tweaked values directly into your binary
- **Encryption Support**: Optional AES-256-GCM encryption for secure save data

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
        .add_plugins(
            PersistPlugin::new("MyCompany", "MyGame")
        )
        .run();
}
```

### Secure Encryption

For sensitive save data, use the `secure` feature to enable AES-256-GCM encryption:

```rust
#[derive(Resource, Default, Serialize, Deserialize, Persist)]
#[cfg_attr(feature = "secure", persist(secure))]
struct SaveGame {
    level: u32,
    score: u32,
    unlocked_items: Vec<String>,
}

// In your app setup:
.add_plugins(
    PersistPlugin::new("MyCompany", "MyGame")
        .with_secret("your_secret_key") // Required for encryption
)
```

## Production Usage

bevy_persist is designed primarily as a development tool for tweaking game parameters, but includes production features for shipping games:

- **Development Mode** (default): All resources save to local RON files for easy tweaking
- **Production Mode**: Different persistence strategies for different types of data:
  - `#[persist(embed)]` - Embed tweaked values into the binary (game balance, level data)
  - `#[persist(dynamic)]` - Save to platform-specific user directories (settings, preferences) 
  - `#[persist(secure)]` - Encrypted save data with AES-256-GCM (game progress, achievements)

See [PRODUCTION.md](PRODUCTION.md) for detailed production usage guide.

## Documentation

Full documentation is available at [docs.rs/bevy_persist](https://docs.rs/bevy_persist).

## Examples

Check out the `examples/` directory for more usage examples:
- `basic.rs` - Simple settings persistence
- `advanced.rs` - All persistence modes including encryption

Run examples with:
```bash
cargo run --example basic
cargo run --example advanced

# Test production mode
cargo run --example advanced --no-default-features --features prod

# Test production with encryption
cargo run --example advanced --no-default-features --features prod,secure
```

## CI/CD

[![CI](https://github.com/Alex-Gilbert/bevy_persist/actions/workflows/ci.yml/badge.svg)](https://github.com/Alex-Gilbert/bevy_persist/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Alex-Gilbert/bevy_persist/branch/main/graph/badge.svg)](https://codecov.io/gh/Alex-Gilbert/bevy_persist)
[![Crates.io](https://img.shields.io/crates/v/bevy_persist.svg)](https://crates.io/crates/bevy_persist)
[![Documentation](https://docs.rs/bevy_persist/badge.svg)](https://docs.rs/bevy_persist)
[![License](https://img.shields.io/crates/l/bevy_persist.svg)](./LICENSE-MIT)

This project uses GitHub Actions for continuous integration:

- **Pull Requests**: Automatically runs formatting, linting, tests, and security checks
- **Releases**: Automated publishing to crates.io when tags are pushed
- **Dependencies**: Automated dependency updates via Dependabot

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.