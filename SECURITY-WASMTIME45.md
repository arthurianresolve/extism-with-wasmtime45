# Wasmtime 45 Security Fork

This repository is a minimal fork of `extism/extism` v1.21.0 for consumers that
need the Extism Rust SDK API with Wasmtime 45.x before upstream Extism publishes
a compatible release.

## Dependency Changes

- `wasmtime`: `41` -> `45`
- `wasi-common`: `41` -> `45`
- `wiggle`: `41` -> `45`

The three crates are upgraded together because Extism's runtime integrates
Wasmtime and WASIp1 through `wasi-common`/`wiggle`; mixing release trains is not
expected to be safe.

## Security Context

Wasmtime 45 includes fixes from the April 2026 Wasmtime advisory set and the
May 2026 WASI permission advisory:

- Critical aarch64 Cranelift sandbox escape: GHSA-jhxm-h53p-jm7w
- Critical Winch sandbox escape: GHSA-xx5w-cvp6-jv83
- Component-model string transcoding issues: GHSA-hx6p-xpx3-jvvv,
  GHSA-394w-hwhg-8vgm, GHSA-jxhv-7h78-9775
- Winch table and data leakage issues: GHSA-f984-pcp8-v2p7,
  GHSA-q49f-xg75-m9xw, GHSA-m9w2-8782-2946
- Pooling allocator data leakage: GHSA-6wgr-89rj-399p
- WASI `path_open(TRUNCATE)` host write-permission bypass: GHSA-2r75-cxrj-cmph

Reference: https://bytecodealliance.org/articles/wasmtime-security-advisories

## Compatibility Note

Wasmtime 45 raises the Rust minimum supported version to 1.93.0. Consumers that
depend on this fork must build plugin-enabled targets with Rust 1.93.0 or newer.

This fork should be treated as temporary. Prefer returning to upstream Extism as
soon as an official release supports the required Wasmtime security baseline.
