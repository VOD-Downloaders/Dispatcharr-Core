# Dispatcharr-Core

The core library powering all Dispatcharr VOD downloading functionality. This crate is **not user-facing** — it exposes no binaries or UI. If you're looking to download VODs, head to the consumer projects built on top of this library:

- [Dispatcharr-cli](https://github.com/VOD-Downloaders/Dispatcharr-cli) - Command-line interface
- [Dispatcharr-plugin](https://github.com/VOD-Downloaders/Dispatcharr-plugin) - Dispatcharr plugin

> [!WARNING]
> This software is currently in alpha stages, there may be bugs and breaking changes to the API.

## Features

- Authenticate and communicate with a [Dispatcharr](https://github.com/Dispatcharr/Dispatcharr) instance via its REST API
- Fetch, parse, and resolve VOD stream metadata
- Download VOD streams with support for MKV and MP4/ISO container formats
- Structured error types for clean integration into CLI tools and plugins
- Blocking HTTP client suitable for embedding in both synchronous CLI and plugin contexts
- JSON serialization/deserialization of all Dispatcharr API payloads

## Usage

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
dispatcharr_download_core = { git = "https://github.com/VOD-Downloaders/Dispatcharr-Core" }
```

This crate is not (yet) published to crates.io. Pin to a specific commit or tag for stability in production use.

## Contributing

Contributions are welcome. Since this is a core library, changes here affect both the CLI and plugin.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-change`)
3. Make your changes and ensure everything compiles (`cargo build`)
4. Run tests (`cargo test`)
5. Run the linter (`cargo clippy`)
6. Format your code (`cargo fmt` from [`rustfmt`](https://github.com/rust-lang/rustfmt))
7. Open a pull request with a clear description of what you changed and why

### Guidelines

- Keep the public API minimal and well-documented
- Prefer returning `Result` types over panicking
- New dependencies should be justified — this is a library crate

## Third-Party Libraries

| Crate | Version | License | Purpose |
|---|---|---|---|
| [chrono](https://crates.io/crates/chrono) | 0.4 | MIT / Apache-2.0 | Timestamp handling |
| [reqwest](https://crates.io/crates/reqwest) | 0.12 | MIT / Apache-2.0 | Blocking HTTP client for Dispatcharr API communication |
| [serde](https://crates.io/crates/serde) | 1 | MIT / Apache-2.0 | Serialization/deserialization framework |
| [serde_json](https://crates.io/crates/serde_json) | 1 | MIT / Apache-2.0 | JSON parsing for API payloads |
| [symphonia](https://crates.io/crates/symphonia) | 0.5 | MPL-2.0 | Audio/video container handling (MKV, MP4/ISO) |

## License

This project is licensed under the **GNU General Public License v2.0**. See [LICENSE](LICENSE.txt) for the full license text.
