# Agent Instructions for kapsule

## Build/Test Commands
- Build: `make build` (cross-compiles to aarch64-unknown-linux-musl using Docker)
- Test all: `make test` (runs `cargo test` in Docker container)
- Test single: `docker exec -t rust-build-env cargo test <test_name>` (run specific test)
- Check: `make check` (runs `cargo check --target aarch64-unknown-linux-musl`)
- Format: `cargo fmt`
- Lint: `cargo clippy`

## Code Style Guidelines
- **Imports**: Group std, external crates, then local modules (blank line between groups)
- **Naming**: snake_case for functions/variables, PascalCase for types/structs/enums
- **Error Handling**: Use `anyhow::Result<T>`, `?` operator, `Context` for descriptive error messages
- **Async**: Use tokio runtime with `#[tokio::main]`, `spawn_blocking` for CPU-intensive tasks
- **Types**: Use serde with `#[derive(Deserialize, Serialize)]` for JSON, derive common traits
- **Structure**: Organize code in impl blocks, use modules for separation, pub(crate) for internal APIs
- **Progress**: Use `indicatif` for progress bars on long-running operations
- **Formatting**: Follow `cargo fmt` output exactly
- **Linting**: Address all `cargo clippy` warnings and suggestions