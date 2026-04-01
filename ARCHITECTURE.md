# Kapsule OCI Runtime — Architecture Document

**Version:** 0.1.0  
**Last Updated:** 2026-04-01  
**Commit:** 77c62c6

---

## Overview

Kapsule is a minimal OCI (Open Container Initiative) runtime written in Rust. It implements the OCI runtime specification to create and run containers using Linux namespaces, chroot, and other isolation primitives.

### Key Design Decisions

- **Binary-only crate**: No `lib.rs` — pure binary for minimal attack surface
- **Cross-compilation**: Builds on macOS (x86_64) targeting `aarch64-unknown-linux-musl` via Docker
- **Lima VM for testing**: Ubuntu 24.04 ARM64 cloud image with virtiofs mount
- **Async image fetching**: `tokio` + `reqwest` for pulling images from registries
- **State persistence**: JSON files in `/run/kapsule/<id>/state.json`
- **Namespace isolation**: Uses `nix::sched::clone` with `Box<dyn FnOnce>` pattern

---

## Project Structure

```
kapsule/
├── src/
│   ├── main.rs              # CLI entry point, argument parsing
│   ├── runtime/
│   │   ├── cli.rs           # Command enum, argument parser
│   │   ├── commands/
│   │   │   ├── mod.rs       # execute() dispatcher
│   │   │   ├── create.rs    # create <id> <bundle>
│   │   │   ├── start.rs     # start <id>
│   │   │   ├── state_cmd.rs # state <id>
│   │   │   ├── kill.rs      # kill <id> <signal>
│   │   │   ├── delete.rs    # delete <id>
│   │   │   ├── pause.rs     # pause <id>
│   │   │   └── resume.rs    # resume <id>
│   │   ├── namespaces/      # Linux namespace primitives
│   │   │   ├── mod.rs       # exports CloneFlags, clone_with_stack, setup_mounts
│   │   │   ├── clone_runner.rs  # nix::sched::clone wrapper
│   │   │   ├── uts.rs       # set_hostname()
│   │   │   ├── mount.rs      # setup_mounts(), cleanup_mounts()
│   │   │   ├── pid.rs       # wait_for_child()
│   │   │   └── ipc.rs       # passive (CLONE_NEWIPC set automatically)
│   │   ├── spec.rs          # OCI RuntimeSpec parsing
│   │   └── state.rs         # ContainerState, StateManager
│   ├── layers/              # Image fetching + OCI manifest parsing
│   │   ├── manifests/       # config.rs, platform_specific_image.rs
│   │   └── mod.rs
│   ├── errors.rs            # Exit codes (currently unused)
│   └── sys/                # Platform detection (Linux-only stubs on macOS)
├── Cargo.toml
├── Makefile
└── kapsule.yml             # Lima VM config
```

---

## CLI Command Architecture

### Command Flow

```
User: kapsule <command> [args]
        │
        ▼
main.rs: execute_cli()
        │ 1. Parse --version, --help
        │ 2. Collect env args
        ▼
cli.rs: parse_args() ──► Command enum
        │
        ▼
commands/mod.rs: execute(cmd) ──► match command
        │
        ├──► create::create(id, bundle)
        ├──► start::start(id)       [Linux only]
        ├──► state_cmd::state(id)
        ├──► kill::kill(id, signal)
        ├──► delete::delete(id)
        ├──► pause::pause(id)
        └──► resume::resume(id)
```

### Command Summary

| Command | Description | State Transition |
|---------|-------------|------------------|
| `create <id> <bundle>` | Validate bundle, save container state | → Created |
| `start <id>` | Fork child in new namespaces, exec container process | Created → Running → Stopped |
| `state <id>` | Print JSON container state | — |
| `kill <id> <signal>` | Send signal to container (stubbed) | — |
| `delete <id>` | Remove container state directory | — |
| `pause <id>` | Pause container (stubbed — just updates status) | Running → Paused |
| `resume <id>` | Resume paused container (stubbed) | Paused → Running |

---

## Detailed Command Flows

### 1. `create` Command

```
kapsule create <id> <bundle>
        │
        ▼
┌──────────────────────────────────────┐
│ create(id, bundle)                  │
│  1. Check bundle path exists         │
│  2. Check bundle/config.json exists   │
│  3. Load & validate RuntimeSpec      │
│  4. Build ContainerState              │
│     - version: "1.0.2"                │
│     - id: <id>                        │
│     - status: Created                 │
│     - pid: 0                          │
│     - bundle: canonicalized path      │
│  5. StateManager::save()              │
│     → writes /run/kapsule/<id>/       │
│       └── state.json                  │
└──────────────────────────────────────┘
        │
        ▼
"created" printed to stdout
```

**Files involved:**
- `src/runtime/commands/create.rs`
- `src/runtime/spec.rs` — RuntimeSpec parsing
- `src/runtime/state.rs` — StateManager

---

### 2. `start` Command (Most Complex)

```
kapsule start <id>
        │
        ▼
┌─────────────────────────────────────────────────────────┐
│ start(id)                                              │
│  1. Verify container exists and status = Created        │
│  2. Load RuntimeSpec from bundle/config.json            │
│  3. Build CloneFlags:                                   │
│     CLONE_NEWUTS  │ CLONE_NEWPID │ CLONE_NEWNS │ CLONE_NEWIPC │
│  4. Extract from spec:                                  │
│     - hostname (for UTS namespace)                      │
│     - rootfs_path (for chroot)                          │
│     - program, args, cwd, env (for exec)                 │
│  5. Build child_fn closure:                            │
│     ┌───────────────────────────────────────────────┐  │
│     │ Box<dyn FnOnce() -> i32> = move || {          │  │
│     │   set_hostname(hostname)                       │  │
│     │   setup_mounts(rootfs_path)  [chroot + mounts] │  │
│     │   set_current_dir(cwd)                         │  │
│     │   set env vars                                  │  │
│     │   Command::new(program).args(args).exec()      │  │
│     │ }                                               │  │
│     └───────────────────────────────────────────────┘  │
│  6. clone_with_stack(CloneConfig { flags, child_fn })  │
│     └──► creates child process in new namespaces       │
│  7. Parent records child_pid in state                  │
│  8. StateManager::save() → Running                     │
│  9. StateManager::write_pidfile()                     │
│ 10. wait_for_child(child_pid)  [blocks until child]   │
│ 11. Update state → Stopped                             │
│ 12. StateManager::save()                              │
└─────────────────────────────────────────────────────────┘
        │
        ▼
"started" printed to stdout
```

**Namespace Isolation in `start`:**

```
Parent Process (kapsule)          Child Process (container)
─────────────────────────        ─────────────────────────
│                                │
│  clone(CLONE_NEWUTS) ──────────►│  [NEW UTS namespace]
│  clone(CLONE_NEWPID) ──────────►│  [NEW PID namespace]
│  clone(CLONE_NEWNS)  ──────────►│  [NEW mount namespace]
│  clone(CLONE_NEWIPC) ──────────►│  [NEW IPC namespace]
│                                │
│                                │  set_hostname()
│                                │  setup_mounts()
│                                │    - MS_PRIVATE on /
│                                │    - chroot(rootfs)
│                                │    - mount proc, sys
│                                │  set env, cwd
│                                │  exec(program)
│                                │
│  waitpid(child)                │  [process runs]
│  ◄─────────────────────────────│  [process exits]
```

**Files involved:**
- `src/runtime/commands/start.rs`
- `src/runtime/namespaces/mod.rs`
- `src/runtime/namespaces/clone_runner.rs` — the `nix::sched::clone` wrapper
- `src/runtime/namespaces/uts.rs` — `set_hostname()`
- `src/runtime/namespaces/mount.rs` — `setup_mounts()`, `chroot`
- `src/runtime/namespaces/pid.rs` — `wait_for_child()`

**Critical Implementation Detail — `FnOnce` to `FnMut` Conversion:**

`nix::sched::clone` requires `Box<dyn FnMut() -> isize>`. The child function is `Box<dyn FnOnce() -> i32>`. This is solved using `Option` + `.take()`:

```rust
// clone_runner.rs
pub fn clone_with_stack(config: CloneConfig) -> anyhow::Result<Pid> {
    let mut child_fn_opt = config.child_fn;  // Option<Box<dyn FnOnce>>

    let boxed_fn: Box<dyn FnMut() -> isize + Send + 'static> = Box::new(move || {
        if let Some(child_fn) = child_fn_opt.take() {  // .take() → Option<Box<dyn FnOnce>>
            let code = child_fn();                       // consumes the FnOnce
            code as isize
        } else {
            -1isize
        }
    });

    unsafe {
        nix::sched::clone(
            boxed_fn,
            stack_slice,
            config.flags,
            Some(libc::SIGCHLD),
        )
    }
}
```

---

### 3. `state` Command

```
kapsule state <id>
        │
        ▼
┌──────────────────────────────────────┐
│ state(id)                            │
│  1. Check container exists           │
│  2. StateManager::load(id)           │
│     → reads /run/kapsule/<id>/       │
│       state.json                     │
│  3. serde_json::to_string(&state)   │
└──────────────────────────────────────┘
        │
        ▼
{
  "version": "1.0.2",
  "id": "mycontainer",
  "status": "running",
  "pid": 12345,
  "bundle": "/root/bundle",
  "annotations": {}
}
```

---

### 4. `delete` Command

```
kapsule delete <id>
        │
        ▼
┌──────────────────────────────────────┐
│ delete(id)                           │
│  1. Check container exists           │
│  2. StateManager::remove(id)         │
│     → rm -rf /run/kapsule/<id>/     │
└──────────────────────────────────────┘
        │
        ▼
"deleted"
```

---

### 5. `kill` Command (Stubbed)

```
kapsule kill <id> <signal>
        │
        ▼
┌──────────────────────────────────────┐
│ kill(id, signal)                     │
│  1. Check container exists           │
│  2. Verify pid != 0                  │
│  3. [STUB] - no actual signal sent   │
│     Note: Milestone 4 item           │
└──────────────────────────────────────┘
        │
        ▼
"killed"
```

---

### 6. `pause` / `resume` Commands (Stubbed)

These only update the state status — no actual process suspension via cgroups or signals.

```
kapsule pause <id>
        │
        ▼
┌──────────────────────────────────────┐
│ pause(id)                            │
│  1. Check exists, status = Running    │
│  2. state.status = Paused            │
│  3. StateManager::save()             │
└──────────────────────────────────────┘
        │
        ▼
"paused"

kapsule resume <id>
        │
        ▼
┌──────────────────────────────────────┐
│ resume(id)                           │
│  1. Check exists, status = Paused    │
│  2. state.status = Running           │
│  3. StateManager::save()             │
└──────────────────────────────────────┘
        │
        ▼
"resumed"
```

---

## OCI Bundle Format

Kapsule expects an OCI bundle directory with this structure:

```
bundle/
├── config.json    # OCI runtime config (required)
└── rootfs/        # Root filesystem (directory or chroot target)
```

**Minimal `config.json`:**

```json
{
  "ociVersion": "1.0.2",
  "hostname": "mycontainer",
  "root": {
    "path": "rootfs",
    "readonly": false
  },
  "process": {
    "cwd": "/",
    "args": ["/bin/sh", "-c", "echo hello"]
  }
}
```

**Full OCI spec fields supported:**
- `hostname`, `domainname`
- `root.path`, `root.readonly`
- `process.terminal`, `process.console_size`, `process.cwd`, `process.env`, `process.args`
- `process.capabilities` (effective, bounding, inheritable, permitted, ambient)
- `process.rlimits`
- `mounts[]` (destination, type, source, options)
- `linux.namespaces[]` (type, path)
- `linux.cgroups_path`, `linux.sysctl`, `linux.resources`
- `linux.devices[]`
- `hooks` (prestart, createRuntime, createContainer, startContainer, poststart, poststop)

---

## State Management

Container state is persisted in `/run/kapsule/<id>/`:

```
/run/kapsule/
└── <container-id>/
    ├── state.json    # ContainerState JSON
    └── pidfile       # PID of container process (unused but written)
```

**ContainerState struct:**

```rust
pub struct ContainerState {
    pub version: String,          // OCI version (e.g., "1.0.2")
    pub id: String,               // Container ID
    pub status: Status,            // Created | Running | Paused | Stopped
    pub pid: u32,                  // PID of container's child process
    pub bundle: PathBuf,           // Canonical path to bundle directory
    pub annotations: HashMap,      // OCI annotations
}
```

---

## Namespace Isolation Details

### clone_with_stack()

Uses `nix::sched::clone` to create a child process with new namespaces:

```rust
pub fn clone_with_stack(config: CloneConfig) -> anyhow::Result<Pid>
```

**How it works:**
1. Allocates 8MB stack for child process
2. Converts `Box<dyn FnOnce>` → `Box<dyn FnMut>` via `Option::take()`
3. Calls `nix::sched::clone(boxed_fn, stack, flags, SIGCHLD)`
4. Returns child's PID to parent

**Key flags used:**
- `CLONE_NEWUTS` — Isolates hostname, domainname
- `CLONE_NEWPID` — New PID namespace (PID 1 inside container)
- `CLONE_NEWNS` — New mount namespace
- `CLONE_NEWIPC` — New IPC namespace

### set_hostname()

```rust
pub fn set_hostname(hostname: &str) -> anyhow::Result<()>
```

Uses `nix::unistd::sethostname()` inside the child process after clone.

### setup_mounts()

```rust
pub fn setup_mounts(rootfs: &Path) -> anyhow::Result<()>
```

**Steps:**
1. `mount("none", "/", None, MS_PRIVATE | MS_REC, None)` — Make mounts private
2. `chroot(rootfs)` — Change root filesystem
3. `set_current_dir("/")` — Ensure we're in the new root
4. `mount("proc", "/proc", "proc", MS_NOSUID | MS_NOEXEC | MS_NODEV, None)` — Mount proc
5. `mount("sys", "/sys", "sysfs", ...)` — Mount sys

### wait_for_child()

```rust
pub fn wait_for_child(pid: Pid) -> anyhow::Result<WaitStatus>
```

Uses `nix::sys::wait::waitpid()` to block parent until child exits, returning exit status.

---

## Build & Deployment

### Cross-Compilation

```
macOS (x86_64)  ───►  Docker (rust:bookworm)  ───►  aarch64-unknown-linux-musl
                            │
                            ├── musl-tools
                            ├── libssl-dev
                            └── rustup target add
```

### Makefile Targets

| Target | Description |
|--------|-------------|
| `make build` | Docker cross-compile → `build/kapsule` |
| `make check` | `cargo check --target aarch64-unknown-linux-musl` |
| `make test` | `cargo test` inside Docker |
| `make vm-start` | Start Lima VM |
| `make vm-rootfs` | Bootstrap Ubuntu rootfs in `/root/bundle/rootfs` via debootstrap |
| `make run` | Run kapsule in VM |
| `make shell` | Shell into Lima VM as root |
| `make install` | Copy binary to `/usr/local/bin/kapsule` |

### vm-rootfs Target

```makefile
vm-rootfs:
    apt-get install -y debootstrap
    debootstrap --arch arm64 noble /root/bundle/rootfs http://ports.ubuntu.com/ubuntu-ports/
    mkdir -p /root/bundle/rootfs/proc /root/bundle/rootfs/sys /root/bundle/rootfs/tmp
    # creates /root/bundle/config.json
```

---

## Error Handling

- Uses `anyhow::Result<T>` throughout
- Context propagation with `.context()` and `.with_context()`
- Error messages printed to stderr, exit code 1
- Exit codes defined in `src/errors.rs` (currently unused — all errors go through anyhow)

---

## Known Limitations & TODOs

### Namespace Implementations (Incomplete)

The following Linux namespaces are **not yet implemented**:

| Namespace | Flag | Status | Notes |
|-----------|------|--------|-------|
| UTS | `CLONE_NEWUTS` | ✅ Implemented | `set_hostname()` — hostname/domainname isolation |
| PID | `CLONE_NEWPID` | ✅ Implemented | `wait_for_child()` — PID namespace (PID 1 inside container) |
| Mount | `CLONE_NEWNS` | ✅ Implemented | `setup_mounts()` — chroot, /proc, /sys mounts |
| IPC | `CLONE_NEWIPC` | ⚠️ Partial | Automatic via clone flag, but no message queue or semaphore cleanup |
| **Network** | `CLONE_NEWNET` | ❌ Not implemented | No network namespace setup, no veth pairs, no eth0 inside container |
| **User** | `CLONE_NEWUSER` | ❌ Not implemented | No user namespace mapping (UID/GID translation) |
| **Cgroup** | `CLONE_NEWCGROUP` | ❌ Not implemented | No cgroup namespace isolation |

### Command Implementations (Stubbed)

1. **`kill` is stubbed** — No actual signal is sent to the container process
2. **`pause`/`resume` are stubbed** — Only update state, no cgroup/signal suspension

### Other TODOs

3. **No image pull integration** — `ContainerImageFetcher` exists but is unused; `create` only validates bundle
4. **No proper OCI image layout** — Extracted layers go to `./extracted_layers/<digest>/` instead of OCI layout
5. **`hostname`/`cwd` should be required fields** — Currently `Option<String>` but treated as required
6. **`src/run.rs` is dead code** — Not invoked from any CLI path
7. **Lima VM restart wipes rootfs** — `vm-rootfs` must be re-run after `make vm-start`

---

## Rootful vs Rootless Architecture

Kapsule uses **runtime detection** (like youki/crun) to support both rootful and rootless containers:

### Detection Logic

```rust
// Runtime detection: am I running as root?
fn is_rootless() -> bool {
    unsafe { libc::geteuid() != 0 }
}
```

### Rootful Mode (Docker compatible, runs as root)

```
docker run → containerd-shim → kapsule (root) → container
                                    ↑
                              OCI runtime Shim
                              CLONE_NEWUTS|PID|NS|IPC|NET
                              veth pair for networking
```

**Requirements:**
- Network namespace (veth pairs)
- `kill` implementation (sends signals to container process)
- `pause`/`resume` (cgroup freezing or signals)

### Rootless Mode (Podman compatible, runs as unprivileged user)

```
podman run → conmon → kapsule --rootless (uid 100000) → container
                              │
                         OCI runtime Shim
                         CLONE_NEWUTS|PID|NS|IPC|NET|USER
                         slirp4netns for networking
                         fuse-overlayfs for filesystem
```

**Requirements:**
- User namespace (UID/GID mapping) — mandatory
- slirp4netns for networking (userspace TAP)
- fuse-overlayfs for filesystem operations
- No privileged operations

### Key Differences

| Feature | Rootful | Rootless |
|---------|---------|----------|
| Network | veth pairs | slirp4netns |
| Filesystem | mount --bind | fuse-overlayfs |
| User NS | Optional | Mandatory |
| Daemon | Optional | None (conmon monitors) |
| Capabilities | Keep required ones | All via UserNS |
| Cgroups | Full hierarchy | Delegated subtree |

### Shim Interface (Docker/Podman Compatible)

Both Docker and Podman invoke OCI runtimes through a shim interface. Implementing this allows kapsule to be swapped in for runc:

```
Docker/Podman → runtime shim socket → kapsule create/start/delete
```

---

## Milestones

| Milestone | Status | Description |
|-----------|--------|-------------|
| M1 | ✅ Done | OCI Runtime CLI (create/start/state/kill/delete/pause/resume) |
| M2 | ✅ Done | RuntimeSpec expansion (OCI fields: linux, hooks, mounts, etc.) |
| M3 | ✅ Done | Container state management + pidfile |
| M4 | ✅ Done | Namespace isolation (UTS, PID, Mount, IPC — basic) |
| **M5** | 📋 TODO | **Minimum Docker swap-out (rootful)** |
| | | Network namespace with veth pairs |
| | | Working `kill` command (send signals to container) |
| | | `pause`/`resume` via cgroup freeze or SIGSTOP/SIGCONT |
| | | Shim interface for Docker/Podman invocation |
| **M6** | 📋 TODO | **Rootless support (like youki)** |
| | | User namespace (UID/GID mapping, mandatory) |
| | | slirp4netns integration for rootless networking |
| | | fuse-overlayfs for rootless filesystem |
| | | Rootless detection at runtime |
| **M7** | 📋 TODO | **Production hardening** |
| | | Capabilities dropping (CAP_NET_ADMIN, etc.) |
| | | Seccomp filters |
| | | `/dev` setup (devpts, devshm, mqueue) |
| | | `pivot_root` instead of `chroot` |
| | | Cgroup v2 resource limits |
| | | Prestart/poststop hooks |
| **Future** | 📋 TODO | Image pull integration |
| | | OCI image layout |
| | | Cgroup namespace |
