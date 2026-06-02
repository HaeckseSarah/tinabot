# TinaBot Agent Guide

## Purpose
TinaBot is a Rust workspace for event-driven automation. Native plugins emit events, Lua scripts subscribe to them, and the core dispatcher routes callbacks through queues.

## Workspace Map
- `tina-core`: main binary (`tinabot`). Owns CLI, config, logger, kernel, dispatcher, and the shared Lua runtime.
- `tina-plugin-api`: shared types and traits used by the core and plugins (`Plugin`, `PluginContext`, `Event`, filters, `ScriptRegistry`).
- `plugins/dummy-plugin`: simple local/testing plugin; use it when validating dispatcher or script behavior.
- `plugins/twitch-plugin`: Twitch integration, OAuth/device auth, token refresh, keychain storage, and Lua-exposed Twitch helpers.
- `scripts/`: runtime Lua scripts. Top-level `.lua` files are executed directly. Subdirectories are loaded through `main.lua`, with `require` enabled relative to that directory.
- `tina.toml.dist`: example runtime configuration.

## Build And Validation
- `cargo check --workspace`
- `cargo build --workspace`
- `cargo test --workspace`
- `cargo test` is part of CI, but it is disk-heavy because `mlua` builds vendored Lua. On constrained environments, verify free space before treating test failures as code failures.
- To run locally:

```bash
cp tina.toml.dist tina.toml
cargo run -p tinabot -- --config tina.toml
```

- Useful overrides:

```bash
cargo run -p tinabot -- --config tina.toml --script-path ./scripts
cargo run -p tinabot -- --config tina.toml -D plugin.twitch.broadcaster_user=example
```

- For debugging, set `log.level = "debug"` in `tina.toml`; optionally set `log.file` to capture a persistent trace.

## Runtime Flow
1. `tina-core/src/main.rs` parses CLI args, loads config, builds `Logger`, and constructs `Kernel`.
2. `Kernel::init` creates the dispatcher, boots enabled plugins, registers plugin Lua functions, creates queues, and then loads Lua scripts.
3. Plugins emit events through `PluginContext::send_event`, which prefixes event names with the plugin id.
4. Lua scripts register handlers through `t.on(...)`.
5. `Dispatcher` matches handlers, evaluates filters, and enqueues callbacks in `ActionQueue`.
6. Queues resume Lua coroutines until completion or cancellation.
7. Long-lived plugins keep their own async loop alive until the shared cancellation token fires.

## Editing Rules
- Keep boundaries clean: shared interfaces belong in `tina-plugin-api`, orchestration belongs in `tina-core`, and service-specific code belongs in its plugin crate.
- Prefer small, local changes. The abstractions are still settling, so avoid broad refactors unless the task requires one.
- Preserve event namespacing (`plugin.event`) and config namespacing (`plugin.<id>.*`, `lua.*`, `queue.*`).
- Treat Lua-exposed functions as capabilities. If a function reaches the network, filesystem, process state, secrets, or host services, prefer explicit gating through config, allowlists, or narrowly scoped registration.
- Use the existing logger or `PluginContext` logging for runtime behavior. Avoid adding new `println!` or `eprintln!` paths except for intentional CLI prompts.
- Return `Result` and propagate errors in runtime code. Avoid adding `unwrap`, `expect`, or panic-based control flow in config, network, token, or scripting paths.
- Keep async code non-blocking. Prefer native async APIs over `block_in_place`, `std::thread::sleep`, or nested executors.
- Add tests when changing dispatcher, filter, config, or token logic. Those areas are core infrastructure with weak current coverage.
- Keep comments short and only where the control flow is hard to infer.

## Known Architectural Constraints
- The contents of `scripts/` should be treated as user-authored, potentially untrusted code. Current isolation is weak because the Lua runtime is shared across scripts and plugins, so the sandbox is best-effort rather than a strong security boundary today.
- Plugins are statically wired in `tina-core/src/kernel/plugin_helper.rs`. Do not assume runtime plugin discovery exists.
- Queue execution is only partially bounded right now. Be careful when introducing new event sources or higher-volume callbacks.
- Twitch auth and token handling depend on OS keychain backends and external network availability.
- The runtime currently lacks dedicated caching and rate-limiting layers; future work should introduce host-level caches for external requests and queue-level or plugin-level rate limiting for noisy event sources.

## Architectural Decisions
- The long-term plugin boundary should be ABI-first, with `tina-plugin-api` becoming the central contract for native plugins.
- Near-term priority is a stable in-process C ABI for trusted native plugins. Out-of-process plugin runtimes are not the current target.
- The current Rust trait-based plugin interface is transitional. Rust plugins do not need to keep their current shape if the ABI-first refactor requires changes.
- Rust plugins should eventually consume the same host/plugin contract as other native plugins, ideally through a separate Rust SDK layered on top of the ABI.
- ABI v1 should stay small and avoid exposing Rust-specific runtime details such as `tokio` channels, async traits, `mlua` handles, or closure-based callback registration across the boundary.
- The runtime should be platform-independent and use native OS security services where available: Windows secure storage, macOS Keychain, and Linux-approved alternatives.
- Secure credential storage should support desktop native keychains on platforms that provide them, and it should also allow an encrypted-file fallback for environments without native stores. The fallback should support supplying a decryption key or passphrase at startup.
- The Lua core and loaded scripts should be reloadable at runtime, with a design that separates Lua state loading from event and queue plumbing.
- If useful for enforcing the boundary, existing Rust plugins may be moved into standalone workspace crates or separate projects. Repository layout is secondary to keeping the host/plugin interface clean and stable.

## Patterns To Avoid
- Do not widen Lua stdlib access or `require` behavior without a security review.
- Do not expose new Lua functions with host-side effects unless there is a clear story for limiting who can call them and under what configuration.
- Do not introduce more detached `tokio::spawn` calls without cancellation, backpressure, or error reporting.
- Do not block inside Lua-exposed functions.
- Do not duplicate secrets in logs, config, or alternate stores.
- Do not move plugin-specific helpers into `tina-core` just to avoid extending `tina-plugin-api`.
- Do not rely on undocumented global Lua state; make data flow explicit through events, config, or registered APIs.

## High-Value Follow-Ups
- Add focused tests around token refresh, dispatcher filtering, and queue behavior.
- Reduce panic and unwrap usage in runtime paths.
- Introduce stronger Lua isolation, capability-based sandbox controls for plugin-exposed functions, and queue backpressure before expanding plugin surface area.
- Refactor `tina-plugin-api` toward an ABI-first contract and move host/plugin interactions away from Rust-only types.
- Replace hard-coded plugin construction with a cleaner registration/loading story once the API stabilizes.