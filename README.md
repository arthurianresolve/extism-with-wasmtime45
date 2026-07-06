## Fork Notice: Wasmtime 46 Runtime Baseline

This repository is a temporary fork of `extism/extism` for Rust hosts that need
`Extism` `v1.30.0` API compatibility with a `wasmtime` `46` security baseline before an
official upstream Extism release supports it.

Base upstream: `extism/extism` tag `v1.30.0`, commit `7038ad1`.

Fork repository: `arthurianresolve/extism-with-wasmtime46`.

### What This Fork Covers

- Rust SDK/runtime source in `runtime/`.
- The workspace crates needed by that Rust runtime: `runtime`, `manifest`,
  `convert`, and `convert-macros`.
- WASIp1 execution through Extism's existing `wasi-common` / `wiggle` path.
- The Extism Rust APIs used by Caliburn's `plugins-wasm` integration: loading a
  `.wasm` file, registering host functions, calling plugin exports, memory
  exchange helpers, and per-call plugin construction.
- A Rust runtime pool fix for concurrent checkout under slow plugin creation.

### Changes From Upstream v1.30.0

- Upgraded runtime dependency train:
  - `wasmtime`: `43` -> `46.0.1`
  - `wasi-common`: `43` -> `46.0.1`
  - `wiggle`: `43` -> `46.0.1`
- Raised the pinned Rust toolchain from `1.91.0` to `1.96.1` because Wasmtime 46
  requires Rust `1.94.0` or newer.
- Rechecked the Rust 1.96.1 syntax and lint surface; no source changes were
  required for either the workspace crates or the standalone kernel crate.
- Updated workspace metadata to identify this fork as
  `1.30.0+wasmtime46` and point repository metadata at this GitHub repository.
- Adapted the runtime to Wasmtime 45 and 46 API changes:
  - `Linker::get` now returns `Result<Extern, wasmtime::Error>` instead of
    `Option<Extern>`.
  - Host functions now bridge `anyhow::Error` into `wasmtime::Error` with
    `ToWasmtimeResult` where Wasmtime requires its own error type.
  - Runtime error contexts now use `wasmtime::error::Context` on Wasmtime call
    paths.
  - Resource limiter and fuel errors now use `wasmtime::Error` internally and
    convert back to Extism's public `anyhow::Error` surface at API boundaries.
  - Removed deprecated `Config::async_support(false)` usage.
- Fixed pool checkout behavior by reserving capacity under the mutex and
  creating plugins outside the mutex. This prevents unrelated waiters from
  spending their timeout behind slow Wasmtime plugin compilation.
- Aligned test code with the Rust 1.95 / Wasmtime 45 lint surface by marking a
  derive-only conversion fixture with `#[expect(dead_code)]` and removing an
  unnecessary clone from a `Copy` `wasmtime::Val` in the pool test.
- Integrated the cargo dependency refresh that was pending on fork maintenance
  branches:
  - `toml`: `0.9` -> `1.1`
  - `sha2`: `0.10` -> `0.11`
  - `criterion`: `0.7.0` -> `0.8.2`
  - `rand`: `0.9.0` -> `0.10.1`
  - `schemars`: `0.8` -> `1.2`
  - `prost`: `0.14.1` -> `0.14.4`
  - `protobuf`: `3.2.0` -> `4.35.0-release`
- Adapted the optional `extism-convert` `protobuf` wrapper to the `protobuf`
  4.x API by replacing the removed 3.x `Message::write_to_bytes` and
  `Message::parse_from_bytes` calls with `Serialize::serialize` and
  `Parse::parse`.
- Rechecked dependency freshness on the `protobuf-4.x` branch on 2026-06-08:
  the direct Wasmtime, WASI, Wiggle, Prost, and Protobuf floors were current;
  `libc` remains on the stable `0.2` line rather than the `1.0.0-alpha` line.
- Rechecked dependency freshness on the `protobuf-4.x` branch on 2026-07-05
  and upgraded the Wasmtime dependency train from `45.0.3` to `46.0.1`.
- Adapted fuel-limit handling for Wasmtime 46 so wrapper setup work does not
  consume the caller's configured guest-execution fuel budget, and failed guest
  execution resets the store before the next call.
- Checked the public wrapper API surface against the Wasmtime 46 migration:
  existing Rust host APIs remain source-compatible, and the generated C header
  keeps the existing `EXTISM_PTR` spelling without exporting cbindgen's generic
  Rust `PTR` helper.
- Disabled the implicit Wasmtime default cache configuration lookup on Android
  when no cache config is supplied, avoiding `extism/extism#851` plugin
  construction failures while preserving explicit `with_cache_config`,
  `EXTISM_CACHE_CONFIG`, and `with_cache_disabled` behavior.
- Updated the reusable GitHub composite action pins to current workflow
  equivalents: `actions/checkout@v6`,
  `actions-rust-lang/setup-rust-toolchain@v1`, `Swatinem/rust-cache@v2`, and
  `actions/cache@v5`.
- Adapted optional manifest JSON Schema generation to the Schemars 1.2 API and
  updated the Rand property-test import for Rand 0.10.
- Added `SECURITY-WASMTIME46.md` documenting the Wasmtime advisory baseline and
  why the fork exists.

### Security Coverage

The Wasmtime 46 baseline includes the April 2026 Wasmtime advisory set, the
May 2026 WASI permission advisory, and the June 2026 WASI FilePerms advisory.
This fork is intended to clear the Wasmtime
41.x advisory lane that blocked Caliburn while Extism still depended on
Wasmtime `^41`.

Covered issue classes include:

- Critical aarch64 Cranelift sandbox escape.
- Critical Winch sandbox escape.
- Component-model string transcoding issues.
- Winch table and data-leakage issues.
- Pooling allocator data leakage.
- WASI `path_open(TRUNCATE)` host write-permission bypass.
- WASI hard-link and rename destination permission checks.

See `SECURITY-WASMTIME46.md` for advisory identifiers and links.

### Explicit Non-Coverage

This fork is intentionally narrow. It does **not** cover:

- Published crates.io packages.
- `libextism` C ABI release artifacts or downstream C-ABI package publishing.
- Python SDK wheels, Node/npm packages, Java artifacts, .NET/NuGet packages,
  RubyGems, Packagist packages, CPAN packages, opam packages, or Hackage
  packages.
- The separate Go SDK or JavaScript SDK repositories.
- Python/C-ABI lifecycle reports such as `extism_plugin_free` memory leak
  reports unless they are proven to affect the Rust SDK path directly.
- WASI Preview 2, the Component Model, or WIT-based plugin interfaces.
- WASI threads support.
- Async cancellation support for guest code blocked in uninterrupted sleeps or
  long-running host functions.
- `allowed_paths` redesign, single-file mounts, or breaking manifest path
  semantics.
- New observability/Observe SDK integration.
- A new runtime API for listing all exported plugin functions.

### Validation Evidence

The fork was validated with:

- `cargo check -p extism`
- `CARGO_INCREMENTAL=0 cargo test -p extism`
- `cargo fmt --all -- --check`
- `CARGO_INCREMENTAL=0 cargo clippy -p extism -- -D warnings`
- `cargo audit`

Additional Rust 1.95 syntax and lint alignment was validated with:

- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p extism-convert test -- --nocapture`
- `cargo test -p extism test_pool_with_captured_builder -- --nocapture`

The full parallel Rust runtime test suite passed after the pool checkout fix.

The cargo dependency refresh was validated with:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p extism-manifest --features json_schema --examples`
- `cargo check -p extism-convert --features protobuf`
- `cargo test -p extism-convert --features protobuf`
- `cargo update`
- `cargo test -p extism test_toml_manifest -- --nocapture`
- `cargo test -p extism check_alloc_with_load_and_store -- --nocapture`
- `cargo test -p extism --benches --no-run`
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo bench -p extism --no-run`

The Wasmtime 46 upgrade and API-surface compatibility pass was validated with:

- `cargo test -p extism`
- `cargo check -p extism --no-default-features`
- `cargo check --workspace`
- `cargo check -p extism-convert --features protobuf`
- `cargo test -p extism-convert --features protobuf`
- `cargo check --manifest-path kernel/Cargo.toml --target wasm32-unknown-unknown`
- `cargo fmt --all -- --check`

The Rust 1.96.1 toolchain bump was syntax-checked with:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo check -p extism-convert --features protobuf`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check --manifest-path kernel/Cargo.toml --target wasm32-unknown-unknown`

The Android default cache config fallback fix for `extism/extism#851` was
validated with:

- `cargo test -p extism android_skips_implicit_default_cache_config`
- `cargo check -p extism --no-default-features`
- `cargo clippy -p extism --all-targets -- -D warnings`
- `cargo fmt --all -- --check`

Prefer returning to upstream Extism as soon as an official release supports the
required Wasmtime security baseline.

<div align="center">
    <a href="https://extism.org">
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset=".github/assets/logo-horizontal-darkmode.png">
        <img alt="Extism - the WebAssembly framework" width="75%" style="max-width: 600px" src=".github/assets/logo-horizontal.png">
    </picture>
    </a>

[![Discord](https://img.shields.io/discord/1011124058408112148?color=%23404eed&label=Community%20Chat&logo=Discord&logoColor=%23404eed)](https://extism.org/discord)
![GitHub Org's stars](https://img.shields.io/github/stars/extism)
![Downloads](https://img.shields.io/crates/d/extism-manifest)
![GitHub License](https://img.shields.io/github/license/extism/extism)
![GitHub release (with filter)](https://img.shields.io/github/v/release/extism/extism)

</div>

# Overview

Extism is a lightweight framework for building with WebAssembly (Wasm). It
supports running Wasm code on servers, the edge, CLIs, IoT, browsers and
everything in between. Extism is designed to be "universal" in that it supports
a common interface, no matter where it runs.

> **Note:** One of the primary use cases for Extism is **building extensible
> software & plugins**. You want to be able to execute arbitrary, untrusted code
> from your users? Extism makes this safe and practical to do.

Additionally, Extism adds some extra utilities on top of standard Wasm runtimes.
For example, we support persistent memory/module-scope variables, secure &
host-controlled HTTP without WASI, runtime limiters & timers, simpler host
function linking, and more. Extism users build:

- plug-in systems
- FaaS platforms
- code generators
- web applications
- & much more...

# Supported Targets

We currently provide releases for the following targets:

- aarch64-apple-darwin
- aarch64-unknown-linux-gnu
- aarch64-unknown-linux-musl
- x86_64-apple-darwin
- x86_64-pc-windows-gnu
- x86_64-pc-windows-msvc
- x86_64-unknown-linux-gnu
- x86_64-unknown-linux-musl

For Android we suggest taking a look at the [Chicory SDK](https://github.com/extism/chicory-sdk) for a pure Java
Extism runtime.

# Run WebAssembly In Your App

Pick a SDK to import into your program, and refer to the documentation to get
started:

| Type        | Language                                                                                       | Source Code                                                             | Package                                                                 |
| ----------- | ---------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| Rust SDK    | <img alt="Rust SDK" src="https://extism.org/img/sdk-languages/rust.svg" width="50px"/>         | https://github.com/extism/extism/tree/main/runtime                      | [Crates.io](https://crates.io/crates/extism)                            |
| JS SDK      | <img alt="JS SDK" src="https://extism.org/img/sdk-languages/js.svg" width="50px"/>             | https://github.com/extism/js-sdk <br/>(supports Web, Node, Deno & Bun!) | [NPM](https://www.npmjs.com/package/@extism/extism)                     |
| Elixir SDK  | <img alt="Elixir SDK" src="https://extism.org/img/sdk-languages/elixir.svg" width="50px"/>     | https://github.com/extism/elixir-sdk                                    | [Hex](https://hex.pm/packages/extism)                                   |
| Go SDK      | <img alt="Go SDK" src="https://extism.org/img/sdk-languages/go.svg" width="50px"/>             | https://github.com/extism/go-sdk                                        | [Go mod](https://pkg.go.dev/github.com/extism/go-sdk)                   |
| Haskell SDK | <img alt="Haskell SDK" src="https://extism.org/img/sdk-languages/haskell.svg" width="50px"/>   | https://github.com/extism/haskell-sdk                                   | [Hackage](https://hackage.haskell.org/package/extism)                   |
| Java SDK    | <img alt="Java SDK" src="https://extism.org/img/sdk-languages/java-android.svg" width="50px"/> | https://github.com/extism/java-sdk                                      | [Sonatype](https://central.sonatype.com/artifact/org.extism.sdk/extism) |
| .NET SDK    | <img alt=".NET SDK" src="https://extism.org/img/sdk-languages/dotnet.svg" width="50px"/>       | https://github.com/extism/dotnet-sdk <br/>(supports C# & F#!)           | [Nuget](https://www.nuget.org/packages/Extism.Sdk)                      |
| OCaml SDK   | <img alt="OCaml SDK" src="https://extism.org/img/sdk-languages/ocaml.svg" width="50px"/>       | https://github.com/extism/ocaml-sdk                                     | [opam](https://opam.ocaml.org/packages/extism/)                         |
| Perl SDK    | <img alt="Perl SDK" src="https://extism.org/img/sdk-languages/perl.svg" width="50px"/>         | https://github.com/extism/perl-sdk                                      | [CPAN](https://metacpan.org/pod/Extism)                                 |
| PHP SDK     | <img alt="PHP SDK" src="https://extism.org/img/sdk-languages/php.svg" width="50px"/>           | https://github.com/extism/php-sdk                                       | [Packagist](https://packagist.org/packages/extism/extism)               |
| Python SDK  | <img alt="Python SDK" src="https://extism.org/img/sdk-languages/python.svg" width="50px"/>     | https://github.com/extism/python-sdk                                    | [PyPi](https://pypi.org/project/extism/)                                |
| Ruby SDK    | <img alt="Ruby SDK" src="https://extism.org/img/sdk-languages/ruby.svg" width="50px"/>         | https://github.com/extism/ruby-sdk                                      | [RubyGems](https://rubygems.org/gems/extism)                            |
| Zig SDK     | <img alt="Zig SDK" src="https://extism.org/img/sdk-languages/zig.svg" width="50px"/>           | https://github.com/extism/zig-sdk                                       | N/A                                                                     |
| C SDK       | <img alt="C SDK" src="https://extism.org/img/sdk-languages/c.svg" width="50px"/>               | https://github.com/extism/extism/tree/main/libextism                    | N/A                                                                     |
| C++ SDK     | <img alt="C++ SDK" src="https://extism.org/img/sdk-languages/cpp.svg" width="50px"/>           | https://github.com/extism/cpp-sdk                                       | N/A                                                                     |

# Compile WebAssembly to run in Extism Hosts

Extism Hosts (running the SDK) must execute WebAssembly code that has a
[PDK, or Plug-in Development Kit](https://extism.org/docs/concepts/pdk), library
compiled in to the `.wasm` binary. PDKs make it easy for plug-in / extension
code authors to read input from the host and return data back, read provided
configuration, set/get variables, make outbound HTTP calls if allowed, and more.

Pick a PDK to import into your Wasm program, and refer to the documentation to
get started:

| Type               | Language                                                                                                   | Source Code                                                   | Package                                                   |
| ------------------ | ---------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- | --------------------------------------------------------- |
| Rust PDK           | <img alt="Rust PDK" src="https://extism.org/img/sdk-languages/rust.svg" width="50px"/>                     | https://github.com/extism/rust-pdk                            | [Crates.io](https://crates.io/crates/extism-pdk)          |
| JS PDK             | <img alt="JS PDK" src="https://extism.org/img/sdk-languages/js.svg" width="50px"/>                         | https://github.com/extism/js-pdk                              | N/A                                                       |
| Python PDK         | <img alt="Python PDK" src="https://extism.org/img/sdk-languages/python.svg" width="50px"/>                 | https://github.com/extism/python-pdk                          | N/A                                                       |
| Go PDK             | <img alt="Go PDK" src="https://extism.org/img/sdk-languages/go.svg" width="50px"/>                         | https://github.com/extism/go-pdk                              | [Go mod](https://pkg.go.dev/github.com/extism/go-pdk)     |
| Haskell PDK        | <img alt="Haskell PDK" src="https://extism.org/img/sdk-languages/haskell.svg" width="50px"/>               | https://github.com/extism/haskell-pdk                         | [Hackage](https://hackage.haskell.org/package/extism-pdk) |
| AssemblyScript PDK | <img alt="AssemblyScript PDK" src="https://extism.org/img/sdk-languages/assemblyscript.svg" width="50px"/> | https://github.com/extism/assemblyscript-pdk                  | [NPM](https://www.npmjs.com/package/@extism/as-pdk)       |
| .NET PDK           | <img alt=".NET PDK" src="https://extism.org/img/sdk-languages/dotnet.svg" width="50px"/>                   | https://github.com/extism/dotnet-pdk <br/>(supports C# & F#!) | [Nuget](https://www.nuget.org/packages/Extism.Pdk)        |
| C PDK              | <img alt="C PDK" src="https://extism.org/img/sdk-languages/c.svg" width="50px"/>                           | https://github.com/extism/c-pdk                               | N/A                                                       |
| C++ PDK            | <img alt="C++ PDK" src="https://extism.org/img/sdk-languages/cpp.svg" width="50px"/>                       | https://github.com/extism/cpp-pdk                             | N/A                                                       |
| Zig PDK            | <img alt="Zig PDK" src="https://extism.org/img/sdk-languages/zig.svg" width="50px"/>                       | https://github.com/extism/zig-pdk                             | N/A                                                       |

# Generating Bindings

It's often very useful to define a schema to describe the function signatures
and types you want to use between Extism SDK and PDK languages.

[XTP Bindgen](https://github.com/dylibso/xtp-bindgen) is an open source
framework to generate PDK bindings for Extism plug-ins. It's used by the
[XTP Platform](https://www.getxtp.com/), but can be used outside of the platform
to define any Extism compatible plug-in system.

## 1. Install the `xtp` CLI.

See installation instructions
[here](https://docs.xtp.dylibso.com/docs/cli#installation).

## 2. Create a schema using our OpenAPI-inspired IDL:

```yaml
version: v1-draft
exports: 
  CountVowels:
      input: 
          type: string
          contentType: text/plain; charset=utf-8
      output:
          $ref: "#/components/schemas/VowelReport"
          contentType: application/json
# components.schemas defined in example-schema.yaml...
```

> See an example in [example-schema.yaml](./example-schema.yaml), or a full
> "kitchen sink" example on
> [the docs page](https://docs.xtp.dylibso.com/docs/concepts/xtp-schema/).

## 3. Generate bindings to use from your plugins:

```
xtp plugin init --schema-file ./example-schema.yaml
  > 1. TypeScript                      
    2. Go                              
    3. Rust                            
    4. Python                          
    5. C#                              
    6. Zig                             
    7. C++                             
    8. GitHub Template                 
    9. Local Template
```

This will create an entire boilerplate plugin project for you to get started
with. Implement the empty function(s), and run `xtp plugin build` to compile
your plugin.

> For more information about XTP Bindgen, see the
> [dylibso/xtp-bindgen](https://github.com/dylibso/xtp-bindgen) repository and
> the official
> [XTP Schema documentation](https://docs.xtp.dylibso.com/docs/concepts/xtp-schema).

# Support

## Discord

If you experience any problems or have any questions, please join our
[Discord](https://extism.org/discord) and let us know. Our community is very
responsive and happy to help get you started.

## Usage

Head to the [project website](https://extism.org) for more information and docs.
Also, consider reading an [overview](https://extism.org/docs/overview) of Extism
and its goals & approach.

## Contribution

Thank you for considering a contribution to Extism, we are happy to help you
make a PR or find something to work on!

The easiest way to start would be to join the
[Discord](https://extism.org/discord) or open an issue on the
[`extism/proposals`](https://github.com/extism/proposals) issue tracker, which
can eventually become an Extism Improvement Proposal (EIP).

For more information, please read the
[Contributing](https://extism.org/docs/concepts/contributing) guide.

---

## Who's behind this?

Extism is an open-source product from the team at:

<p align="left">
  <a href="https://dylibso.com" _target="blanks"><img width="200px" src="https://user-images.githubusercontent.com/7517515/198204119-5afdebb9-a5d8-4322-bd2a-46179c8d7b24.svg"/></a>
</p>

_Reach out and tell us what you're building! We'd love to help:_
<a href="mailto:hello@dylibso.com">hello@dylibso.com</a>
