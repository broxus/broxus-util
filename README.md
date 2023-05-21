# broxus-util &emsp; [![crates-io-batch]][crates-io-link] [![docs-badge]][docs-url] [![rust-version-badge]][rust-version-link] [![workflow-badge]][workflow-link]

[crates-io-batch]: https://img.shields.io/crates/v/broxus-util.svg

[crates-io-link]: https://crates.io/crates/broxus-util

[docs-badge]: https://docs.rs/broxus-util/badge.svg

[docs-url]: https://docs.rs/broxus-util

[rust-version-badge]: https://img.shields.io/badge/rustc-1.65+-lightgray.svg

[rust-version-link]: https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html

[workflow-badge]: https://img.shields.io/github/actions/workflow/status/broxus/broxus-util/master.yml?branch=master

[workflow-link]: https://github.com/broxus/broxus-util/actions?query=workflow%3Amaster

A collection of utils used at Broxus.

## Features

- `argh` - [`argh`](https://crates.io/crates/argh) helpers
- `serde` - various [`serde`](https://crates.io/crates/serde) helpers
- `config` - config parser with environment variables injection
- `log4rs` - custom logger initialization
- `web` - error converters and object builder
- `alloc` - jemalloc allocator
- `alloc-profiling` - profiling tools for jemalloc

> Default: `serde`, `config`, `log4rs`

## Contributing

We welcome contributions to the project! If you notice any issues or errors, feel free to open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](https://opensource.org/license/mit/).
