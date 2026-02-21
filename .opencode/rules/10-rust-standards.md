# 10 Rust Standards

## Language and Style

- Follow idiomatic Rust with explicit error handling and ownership clarity.
- Use `anyhow::Context` for external I/O and parsing failures.
- Prefer descriptive types and small helper functions over duplicated logic.
- Avoid `unwrap`/`expect` in recoverable paths.

## API and Module Design

- Keep module boundaries clear:
  - config parsing in `config`
  - transport logic in client modules
  - business orchestration in monitor/services
  - data models in `types`
- Public APIs should be minimal and documented.
- Add rustdoc for all new public modules, functions, and structs.

## Async and Concurrency

- Use `tokio` primitives consistently.
- Minimize lock hold durations (`RwLock` write guards should be short-lived).
- Avoid blocking calls in async contexts.

## Performance

- Prefer streaming and incremental parsing over full buffering.
- Avoid excessive cloning of large payloads.
- Keep release profile optimizations intact unless explicitly requested.
