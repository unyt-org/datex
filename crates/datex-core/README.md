# DATEX
> The full DATEX library written in Rust.
[![](https://img.shields.io/crates/v/datex-core.svg)](https://crates.io/crates/datex-core) [![Twitter badge][]][Twitter link] [![Discord badge][]][Discord link]

<img align="right" src="../../assets/datex-logo-light.svg" width="150px" alt="The DATEX logo">

This repository contains the full DATEX library including networking, compiler
and decompiler, written in Rust. The DATEX crate is used in
[DATEX Web](https://github.com/unyt-org/datex-web), which provides a
JavaScript interface to the DATEX Runtime. The
[DATEX CLI](https://github.com/unyt-org/datex-cli) is also built on top of this
crate and provides a command line interface for the DATEX Runtime.

## Project Structure
- [src/](./src) - Contains the source code of the crate
  - [ast/](./src/ast) - Abstract syntax tree (AST) modules
  - [compiler/](./src/compiler) - Compiler for the DATEX language
  - [crypto/](./src/crypto) - Cryptographic trait and a native implementation
  - [decompiler/](./src/decompiler) - Decompiler for the DATEX language
  - [dif/](./src/dif) - Abstract data interface for data exchange with external
	systems
  - [global/](./src/global) - Global constants and structures
  - [libs/](./src/libs) - Library modules such as core library and standard
	library
  - [network/](./src/network) - Network protocol implementation and
	communication interfaces
  - [parser/](./src/parser) - DXB parser and instruction handler
  - [references/](./src/references) - Reference implementation, observers and
	mutators
  - [runtime/](./src/runtime) - Runtime for executing DXB
  - [serde/](./src/serde) - Serialization and deserialization of DATEX values
  - [traits/](./src/traits) - Shared traits for values, types and references
  - [types/](./src/types) - Type system implementation
  - [utils/](./src/utils) - Utility functions and traits
  - [values/](./src/values) - Value implementation, core values and value
	containers
- [benches/](./benches) - Benchmarks for the crate for performance testing
- [tests/](./tests) - Integration tests for the crate


## Development

### Building the Project

The project is built with Rust Nightly
([`rustc 1.95.0-nightly`](https://releases.rs/docs/1.95.0/)). To build the
project, run:

```bash
cargo build
```

### Running Tests

Tests can be run with the following command:

```bash
cargo test
```

### Clippy

To apply clippy fixes, run the following command:

```bash
cargo clippy --fix
```

### Running Benchmarks

The benchmarks in the `benches` directory can be run with the following command:

```bash
cargo bench
```

Benchmarks are also run automatically in the GitHub CI on every push to the main
branch or a pull request branch.


## Contributing

**We welcome every contribution!**<br> Please take a look at the
[DATEX Core contribution guidelines](./CONTRIBUTING.md) and the unyt.org
[contribution guidlines](https://github.com/unyt-org/.github/blob/main/CONTRIBUTING.md).

[Twitter badge]: https://img.shields.io/twitter/follow/unytorg.svg?style=social&label=Follow
[Twitter link]: https://twitter.com/intent/follow?screen_name=unytorg
[Discord badge]: https://img.shields.io/discord/928247036770390016?logo=discord&style=social
[Discord link]: https://unyt.org/discord

---

<sub>&copy; unyt 2026 • [unyt.org](https://unyt.org)</sub>
