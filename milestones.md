# Kapsule OCI Runtime Compliance Plan

**Goal**: Make kapsule an OCI-compliant container runtime that Docker can use as a drop-in replacement via `--runtime=kapsule`.

**Target**: Docker 24+, OCI Runtime Spec v1.0.x - v1.1.x
**Integration**: Docker as root (dockerd → kapsule directly)
**Scope**: Namespaces + cgroups, external rootfs

---

## Milestone 1: OCI Runtime CLI Interface

**Goal**: Implement the OCI runtime binary interface that Docker expects.

### TODOs

- [ ] 1.1 **Create `src/runtime/` module structure**

  Create proper module hierarchy under `src/runtime/`:
  ```
  src/runtime/
  ├── mod.rs           # Module exports + re-exports
  ├── cli.rs           # Command parsing (create, start, state, kill, delete, pause, resume)
  ├── commands/        # Command handlers
  │   ├── mod.rs
  │   ├── create.rs    # create <id> <bundle>
  │   ├── start.rs     # start <id>
  │   ├── state.rs     # state <id> → JSON
  │   ├── kill.rs      # kill <id> <signal>
  │   ├── delete.rs    # delete <id>
  │   ├── pause.rs     # pause <id>
  │   └── resume.rs    # resume <id>
  └── spec.rs          # Existing RuntimeSpec (expand it)
  ```

  **QA Scenarios**:
  - Build succeeds: `make build`
  - `kapsule --help` → shows create/start/state/kill/delete/pause/resume commands
  - `kapsule spec` → outputs default OCI spec JSON to stdout

- [ ] 1.2 **Implement `--version` and `--help`**

  Binary should respond to standard CLI flags.

  **QA Scenarios**:
  - `./build/kapsule --version` → outputs version string
  - `./build/kapsule --help` → outputs usage info

- [ ] 1.3 **Wire CLI to main.rs**

  Replace current Clap-based `Create <container_id> <bundle>` subcommand with OCI command dispatch:
  - `kapsule create <id> <bundle>` → reads `<bundle>/config.json` → creates container
  - `kapsule start <id>` → starts the created container
  - `kapsule state <id>` → prints JSON state to stdout
  - `kapsule kill <id> <signal>` → sends signal to container
  - `kapsule delete <id>` → removes container state
  - `kapsule pause <id>` → pauses container (cgroups freezer)
  - `kapsule resume <id>` → resumes paused container

  **QA Scenarios**:
  - `kapsule create testcontainer /root/bundle` → creates container, returns
  - `kapsule start testcontainer` → starts it
  - `kapsule state testcontainer` → outputs `{"status": "running", "pid": ..., ...}`
  - `kapsule kill testcontainer 15` → sends SIGTERM
  - `kapsule delete testcontainer` → removes state

---

## Milestone 2: OCI Spec Compliance

**Goal**: Expand RuntimeSpec to cover all fields Docker containers need.

### TODOs

- [ ] 2.1 **Expand RuntimeSpec struct**

  Add missing OCI spec fields:
  - `hostname`: make required (not Option)
  - `cwd`: make required (not Option)
  - Linux-specific: `linux` block with namespaces, cgroups, sysctl, resources
  - `hooks`: prestart, poststart, poststop hooks

  **References**:
  - OCI config spec: `https://github.com/opencontainers/runtime-spec/blob/main/config.md`

- [ ] 2.2 **Validate spec in create command**

  Validate required fields exist and are valid before container creation.

  **QA Scenarios**:
  - `kapsule create <id> <bundle-without-hostname>` → should warn or use default
  - `kapsule create <id> <bundle-with-hostname>` → should succeed

- [ ] 2.3 **Remove debug println! artifacts**

  Remove `println!("{:?}", path)` from `load_config()` (spec.rs:52).

  **QA Scenarios**:
  - `kapsule create test /root/bundle 2>&1 | grep -v "^Ok" | grep -c "debug"` → 0

---

## Milestone 3: Container State Management

**Goal**: Track container state persistently so `state`, `kill`, `delete`, `pause`, `resume` commands work.

### TODOs

- [ ] 3.1 **Define container state directory**

  Container state stored at: `/run/kapsule/<container-id>/`
  - `state.json` — current state (pid, status, start-time)
  - `pidfile` — pid of container process

- [ ] 3.2 **Implement state command**

  Returns JSON:
  ```json
  {
    "version": "1.0.2",
    "id": "<container-id>",
    "status": "running|created|paused|stopped",
    "pid": 1234,
    "bundle": "/path/to/bundle",
    "annotations": {}
  }
  ```

  **QA Scenarios**:
  - After `kapsule create test /root/bundle`: state shows `{"status": "created", "pid": 0}`
  - After `kapsule start test`: state shows `{"status": "running", "pid": 1234}`

- [ ] 3.3 **Implement delete command**

  Removes container state directory and any resources.

  **QA Scenarios**:
  - After `kapsule delete test`: `/run/kapsule/test/` is gone

---

## Milestone 4: Namespace Isolation

**Goal**: Full Linux namespace support (UTS, PID, mount, IPC, network, user).

### TODOs

- [ ] 4.1 **Refactor container.rs into namespace module**

  Move namespace setup into `src/runtime/namespaces/`:
  ```
  namespaces/
  ├── mod.rs
  ├── uts.rs      # UTS (hostname, domainname)
  ├── pid.rs      # PID namespace
  ├── mount.rs    # Mount namespace + pivot_root
  ├── ipc.rs      # IPC namespace
  ├── network.rs  # Network namespace
  └── user.rs     # User namespace
  ```

- [ ] 4.2 **Implement pivot_root / chroot**

  Use `root.path` from spec to pivot into container rootfs.
  Replace hardcoded `/ubuntu-fs` with spec-based path.

  **QA Scenarios**:
  - `root.path = "rootfs"` → pivot to `<bundle>/rootfs`
  - Container hostname set from spec

- [ ] 4.3 **Implement container init process**

  The cloned child process should:
  1. Setup namespaces
  2. Pivot root
  3. Mount /proc
  4. Exec the process from `spec.process.args`

  **QA Scenarios**:
  - `kapsule start <id>` → runs container process, not just exits

---

## Milestone 5: Cgroups Resource Limits

**Goal**: Apply cgroups v2 resource limits from spec.

### TODOs

- [ ] 5.1 **Add cgroups dependency**

  Add `rustix` or use `cgroups-rs` crate for cgroups management.

- [ ] 5.2 **Create cgroup for container**

  On `create`: create cgroup at `/sys/fs/cgroup/kapsule/<container-id>/`

- [ ] 5.3 **Apply resource limits**

  From `spec.linux.resources`:
  - CPU: cpu.shares, cpu.cfs_quota, cpu.cfs_period
  - Memory: memory.limit_in_bytes, memory.soft_limit_in_bytes
  - PIDs: pids.max

- [ ] 5.4 **Implement pause/resume via cgroups freezer**

  **QA Scenarios**:
  - `kapsule pause <id>` → container processes frozen (SIGSTOP via cgroup)
  - `kapsule resume <id>` → container processes unfrozen

---

## Milestone 6: Docker Integration

**Goal**: Docker can use kapsule as its runtime.

### TODOs

- [ ] 6.1 **Install kapsule on system path**

  Either:
  - `sudo cp build/kapsule /usr/local/bin/kapsule`
  - Or: `make install` target

- [ ] 6.2 **Configure Docker to use kapsule**

  Create `/etc/docker/daemon.json`:
  ```json
  {
    "runtimes": {
      "kapsule": {
        "path": "/usr/local/bin/kapsule"
      }
    }
  }
  ```

  Or use `--default-runtime=kapsule` or `--runtime=kapsule` per-container.

- [ ] 6.3 **Test with Docker**

  ```bash
  docker run --runtime=kapsule hello-world
  docker run --runtime=kapsule -it ubuntu bash
  ```

  **QA Scenarios**:
  - `docker run --runtime=kapsule hello-world` → pulls image, runs via kapsule
  - Container is isolated (separate hostname, pid namespace)
  - `docker stop` works (sends SIGTERM via kill)
  - `docker rm` works (deletes container)

---

## Milestone 7: Integration Tests

**Goal**: Automated tests proving Docker + kapsule works.

### TODOs

- [ ] 7.1 **Test namespace isolation**

  ```bash
  # Container should have isolated hostname
  docker run --runtime=kapsule --hostname=myhost hostname  # → myhost
  
  # Container should have isolated PID
  docker run --runtime=kapsule ps aux  # → PID 1 is init
  ```

- [ ] 7.2 **Test resource limits**

  ```bash
  # Memory limit
  docker run --runtime=kapsule --memory=128m stress --vm 1 --vm-bytes 100M
  
  # CPU limit  
  docker run --runtime=kapsule --cpus=0.5 some-workload
  ```

- [ ] 7.3 **Test signals and lifecycle**

  ```bash
  docker run --runtime=kapsule -d sleep 100
  docker stop <container>
  docker logs <container>  # should show SIGTERM was received
  docker rm <container>
  ```

---

## Milestone F: Final Verification

**Goal**: OCI runtime compliance verified.

### TODOs

- [ ] F1. **OCI runtime-spec validation**

  Use `oci-runtime-tool` to validate spec:
  ```bash
  oci-runtime-tool validate --spec /root/bundle/config.json
  ```

- [ ] F2. **Docker Integration Test Suite**

  Run Docker's own runtime tests against kapsule.

- [ ] F3. **Performance baseline**

  Measure container launch time with kapsule vs runc.

---

## Commit Strategy

- **m1-cli**: Initial CLI structure + command dispatch
- **m2-spec**: OCI spec field expansions
- **m3-state**: Container state management
- **m4-namespaces**: Linux namespace isolation
- **m5-cgroups**: Cgroups resource limits
- **m6-docker**: Docker integration
- **m7-tests**: Integration tests

---

## Success Criteria

- [ ] `kapsule create <id> <bundle>` reads config.json and sets up namespaces
- [ ] `kapsule start <id>` runs the container process
- [ ] `kapsule state <id>` returns valid JSON with correct status
- [ ] `kapsule kill <id> <sig>` sends signals to container
- [ ] `kapsule delete <id>` cleans up all resources
- [ ] `kapsule pause/resume <id>` works via cgroups freezer
- [ ] `docker run --runtime=kapsule hello-world` works end-to-end
- [ ] `docker stop`, `docker rm` work with kapsule-managed containers

---

## Scope Boundaries

**IN**:
- OCI runtime binary interface (create/start/state/kill/delete/pause/resume)
- Linux namespace isolation (UTS, PID, mount, IPC, network, user)
- Cgroups v2 resource limits (CPU, memory, pids)
- Docker as root integration
- External rootfs (pre-populated)

**OUT**:
- Container image pulling/extraction (layers.rs - existing code, not wired for runtime use)
- Kubernetes CRI integration (containerd shim v2)
- Cgroups v1 (cgroups v2 only)
- checkpoint/restore functionality
- Seccomp, AppArmor, SELinux (future milestone)
