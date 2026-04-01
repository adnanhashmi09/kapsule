# Agent Instructions for kapsule

**Generated:** 2026-04-01
**Commit:** 77c62c6
**Branch:** main

## OVERVIEW
Minimal container runtime written in Rust. Binary crate targeting aarch64-unknown-linux-musl with Docker-based cross-compilation and Lima VM for testing.

## STRUCTURE
```
kapsule/
├── src/
│   ├── main.rs          # CLI entry (krt binary, #[tokio::main])
│   ├── run.rs            # Unused namespace runner (pub fn run)
│   ├── container.rs      # Container state
│   ├── errors.rs         # Error codes
│   ├── layers.rs         # Image fetching + extraction
│   ├── layers/manifests/ # OCI manifest parsing (config, multi-arch, platform)
│   ├── runtime/          # OCI RuntimeSpec
│   └── sys/              # Linux syscall wrappers (uname, arch detection)
├── Cargo.toml            # Binary crate, edition 2021
├── Makefile              # Dockerized build + Lima VM targets
└── kapsule.yml           # Lima VM config (aarch64 Ubuntu 24.04)
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| CLI parsing | `src/main.rs` | Clap-based, subcommand: `Create <container_id> <bundle>` |
| Container creation | `src/run.rs` | `pub fn run()` — Linux namespaces (clone), NOT wired to CLI |
| Image fetch + extract | `src/layers.rs` | `ContainerImageFetcher`, Docker registry auth, layer extraction |
| OCI spec parsing | `src/runtime/spec.rs` | `RuntimeSpec::load_config`, serde JSON |
| Syscall wrappers | `src/sys/sysinfo.rs` | `get_platform_information()`, libc::uname |
| Arch detection | `src/sys/arch.rs` | `Arch` enum, `Variant` detection |

## BUILD COMMANDS
```bash
make build      # Docker cross-compile to aarch64-unknown-linux-musl → build/kapsule
make check      # cargo check for target
make test       # cargo test inside Docker
make vm-start   # Start Lima VM (kapsule.yml)
make run        # Run binary inside VM
make shell      # Shell into VM
```

## CODE STYLE
- **Imports**: Group std → external crates → local modules (blank line between)
- **Naming**: snake_case functions/vars, PascalCase types/structs/enums
- **Error Handling**: `anyhow::Result<T>`, `?` operator, `.context()` for messages
- **Async**: `#[tokio::main]`, `spawn_blocking` for CPU-heavy work (tar extraction)
- **Types**: `#[derive(Deserialize, Serialize, Debug)]` + serde JSON
- **Progress**: `indicatif` `ProgressBar` for layer downloads

## ANTI-PATTERNS (THIS PROJECT)
- `src/run.rs` `pub fn run()` is dead code — not invoked by any CLI path
- `src/run.rs` called `run::run` in main.rs but never dispatched (commented)
- Layer extraction extracts to `./extracted_layers/<digest>/` — TODO: needs proper OCI layout

## UNIQUE STYLES
- No `src/lib.rs` — pure binary crate
- Multi-arch image resolution via manifest annotation filtering
- HTTP retry loop with `indicatif` spinner updates on layer fetch failures

## TEST CONVENTIONS
- Inline `#[cfg(test)] mod tests` inside each module
- Fixtures use `tempdir::TempDir` + manual `create_test_bundle()` helpers
- No `tests/` directory (integration tests not yet added)
- No `#[tokio::test]` — all current tests are sync

## GOTCHAS
- **Linux-only**: `#![cfg(target_os = "linux")]` commented out but required for full build
- **Cross-compile**: Requires `musl-tools` + `libssl-dev` inside Docker container
- **Lima VM**: Requires `virtiofs` mount; kapsule.yml provisions Ubuntu 24.04 cloud image
- **TODO in layers.rs:300**: extracted layers need proper OCI directory layout (`etc/` or `opt/`)
- **TODO in spec.rs**: `hostname` and `cwd` are Option<String> but should be required fields
