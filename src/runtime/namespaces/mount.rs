#![cfg(target_os = "linux")]

use nix::mount::{mount, umount, MsFlags};
use std::os::unix::fs::chroot;
use std::path::Path;

/// Configure the container's mount namespace.
///
/// This function is called in the child process AFTER clone() creates new
/// namespaces but BEFORE exec() runs the container process. It sets up:
///   1. Private mounts (so changes don't propagate to host)
///   2. chroot into the container's rootfs
///   3. /proc and /sys virtual filesystems
///
/// # Arguments
/// * `rootfs` - Absolute path to the container's root filesystem directory
///
/// # Mount Propagation
/// Linux has mount propagation modes that control whether mount changes
/// propagate to other mount namespaces:
///   - MS_SHARED: changes propagate to/from other namespaces
///   - MS_PRIVATE: changes do NOT propagate (what we want for containers)
///   - MS_SLAVE: changes from master propagate to slave, not vice versa
///   - MS_UNBINDABLE: cannot be bind-mounted (used for locked mounts)
///
/// The MS_REC flag makes this apply recursively to all child mounts,
/// ensuring the entire mount tree is private.
pub fn setup_mounts(rootfs: &Path) -> anyhow::Result<()> {
    // Step 1: Make the current mount tree private
    //
    // Before we can safely pivot to a new rootfs, we must ensure our mounts
    // are private. If the host mount is shared (common in systemd environments),
    // our mount changes would propagate to the host!
    //
    // MS_PRIVATE: mount changes are private to this namespace
    // MS_REC: apply recursively to all existing mounts (the entire tree)
    //
    // Without this, operations like pivot_root or bind mounts could leak
    // container mounts back to the host system.
    mount::<str, str, str, str>(
        Some("none"),                          // device: "none" for remount operations
        "/",                                   // target: the root mount
        None,                                  // source type: derived from filesystem (none here)
        MsFlags::MS_PRIVATE | MsFlags::MS_REC, // make all mounts private
        None,                                  // data: filesystem-specific options
    )?;

    // Step 2: Change the root filesystem to the container's rootfs
    //
    // chroot() changes the process's view of the filesystem root.
    // After this call, "/" refers to `rootfs`, and the original host root
    // is no longer accessible (unless saved via openat2/resolver tricks).
    //
    // IMPORTANT: This is a shallow change. The process's cwd may still be
    // pointing outside the new root. That's why we call set_current_dir("/")
    // immediately after.
    //
    // SECURITY NOTE: chroot does NOT provide security isolation on its own.
    // It only changes path resolution. A process with CAP_SYS_CHROOT can
    // escape via /proc/<pid>/root or by opening fd to a path outside chroot.
    // Proper container isolation comes from namespace + seccomp + capabilities.
    chroot(rootfs)?;

    // Step 3: Change current working directory to the new root
    //
    // After chroot, the process cwd might be "/root" on host but after
    // chroot("/containers/rootfs") that path no longer exists in the
    // new rootfs. We must cd to "/" (which is now rootfs/) to ensure
    // relative path resolution works correctly.
    std::env::set_current_dir("/")?;

    // Step 4: Mount procfs at /proc
    //
    // /proc is a virtual filesystem (procfs) that exposes kernel data structures.
    // It provides:
    //   - /proc/<pid>/* for each process (PID namespace aware inside container)
    //   - /proc/cpuinfo, /proc/meminfo, /proc/uptime, etc.
    //   - /proc/sys/ for tunable kernel parameters
    //
    // Without /proc mounted, process information is unavailable and many
    // tools (ps, top, free, ls /proc) will fail.
    //
    // MS_NOSUID: setuid binaries inside /proc cannot bestow privileges
    // MS_NOEXEC: no executable files allowed in this filesystem
    // MS_NODEV: no device files (prevents /dev/null, /dev/zero, etc. misuse)
    mount::<str, str, str, str>(
        Some("proc"), // device/filesystem type: procfs
        "/proc",      // mount point: where to mount it
        Some("proc"), // source type hint (for /proc, this is "proc")
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
        None, // data: proc-specific options (none needed)
    )?;

    // Step 5: Mount sysfs at /sys
    //
    // sysfs is a virtual filesystem exposing kernel data structures related
    // to devices, drivers, and kernel internal state. It provides:
    //   - /sys/class/ - device classes
    //   - /sys/devices/ - device tree
    //   - /sys/block/ - block devices
    //   - /sys/fs/ - filesystem-specific info
    //
    // Unlike /proc which is for process info, /sys is for system/hardware info.
    // Many system calls and tools depend on /sys for device enumeration.
    //
    // Same security flags as /proc:
    // MS_NOSUID: ignore suid bits
    // MS_NOEXEC: no executables
    // MS_NODEV: no device files
    mount::<str, str, str, str>(
        Some("sys"),   // device/filesystem type: sysfs
        "/sys",        // mount point
        Some("sysfs"), // source type hint
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
        None, // data: none needed
    )?;

    Ok(())
}

pub fn cleanup_mounts() {
    let _ = umount("/proc");
    let _ = umount("/sys");
}
