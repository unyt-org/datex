# DATEX

[![Twitter badge][]][Twitter link] [![Discord badge][]][Discord link]

<img align="right" src="assets/datex-logo-light.svg" width="150px" alt="The DATEX logo">

This repository is the central monorepo for [DATEX](https://datex.unyt.org), containing everything needed to build, run, and evolve the DATEX ecosystem — from core libraries to specific implementations for native targets, crypto backends, macros, documentation, and architectural planning.

All projects in this monorepo are open source and licensed under the
[MIT License](./LICENSE). We might move some parts of the repository to separate
repositories in the future, but for now, everything is contained here for easier
collaboration and development.


## Structure
- [crates/](./crates) - Contains all the Rust crates for the DATEX ecosystem
  - [datex/](./crates/datex) - The full DATEX library including networking,
  compiler and decompiler, written in Rust
  - [datex-crypto-facade/](./crates/datex-crypto-facade) - A facade crate
  providing a unified interface for different DATEX Crypto implementations
  - [datex-crypto-native/](./crates/datex-crypto-native) - A native
  implementation of the DATEX Crypto trait
  - [datex-crypto-web/](./crates/datex-crypto-web) - A web implementation of
  the DATEX Crypto trait using WebCrypto API
  - [datex-macros/](./crates/datex-macros) - Procedural macros for the DATEX crates
  - [datex-native-macros/](./crates/datex-native-macros) - Procedural macros
  for native targets
- [docs/](./docs) - Documentation for the DATEX ecosystem
  - [guide/](./docs/guide) - Collection of guides for contributing to the DATEX ecosystem
- [datex-language/](./datex-language) - The DATEX language definition used for syntax highlighting, documentation, and other tooling
- [assets/](./assets) - Assets for the DATEX ecosystem such as logos and
  images

## Environment

- [DATEX Specification](https://github.com/unyt-org/datex-specification) - The
  specification of DATEX, including protocols, syntax, and semantics. The
  specification is work in progress and is not yet complete. It is being
  developed in parallel with the implementation of the DATEX Core. The
  repository is currently private, but will be made public in the future and is
  available to contributors on [request](https://unyt.org/contact).
- [DATEX Core JS](https://github.com/unyt-org/datex-js) - A JavaScript
  interface to the DATEX Core, built on top of this crate. Includes a
  WebAssembly build for running DATEX in the browser or server-side with
  [Deno](https://deno.land/), [Node.js](https://nodejs.org/), and
  [Bun](https://bun.sh/) and trait implementations using standard web APIs such
  as
  [WebCrypto](https://developer.mozilla.org/en-US/docs/Web/API/Web_Crypto_API)
  and [WebSocket](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket).
- [DATEX CLI](https://github.com/unyt-org/datex-cli) - A command line interface
  for the DATEX Core, built on top of this crate. Provides a simple way to run
  DATEX scripts and interact with the DATEX Runtime in a REPL-like environment.
- [DATEX Core ESP32](https://github.com/unyt-org/datex-esp32) - A port of
  the DATEX Core to the
  [ESP32](https://www.espressif.com/en/products/socs/esp32) platform, allowing
  you to run DATEX on microcontrollers of the Espressif family.
- [DATEX Core CPP](https://github.com/unyt-org/datex-cpp) - A C++ port of
  the DATEX Core, allowing you to run DATEX on platforms that support C++. _This
  port is still in development and not functional._
- [DATEX Core JS (legacy)](https://github.com/unyt-org/datex-core-js-legacy) - A
  legacy version of the DATEX Core JS, implemented in TypeScript. This version
  will be replaced by the new DATEX Core JS implementation.

## Contributing

**We welcome every contribution!**<br> Please take a look at the
[DATEX Core contribution guidelines](https://github.com/unyt-org/datex/blob/main/CONTRIBUTING.md) and the unyt.org
[contribution guidlines](https://github.com/unyt-org/.github/blob/main/CONTRIBUTING.md).

[Twitter badge]: https://img.shields.io/twitter/follow/unytorg.svg?style=social&label=Follow
[Twitter link]: https://twitter.com/intent/follow?screen_name=unytorg
[Discord badge]: https://img.shields.io/discord/928247036770390016?logo=discord&style=social
[Discord link]: https://unyt.org/discord

---

<sub>&copy; unyt 2026 • [unyt.org](https://unyt.org)</sub>
