use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ============================================================================
// Main RuntimeSpec
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeSpec {
    #[serde(rename = "ociVersion")]
    pub oci_version: String,

    #[serde(default)]
    pub hostname: String, // Now required, default "kapsule"

    #[serde(default)]
    pub domainname: String, // New field

    #[serde(default)]
    pub mounts: Vec<Mount>, // New field

    pub root: Root,
    pub process: Process,

    #[serde(default, rename = "linux")]
    pub linux: Option<Linux>, // New field

    #[serde(default, rename = "hooks")]
    pub hooks: Option<Hooks>, // New field
}

// ============================================================================
// Root
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Root {
    pub path: String,
    #[serde(default)]
    pub readonly: bool,
}

// ============================================================================
// Process
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Process {
    #[serde(default)]
    pub terminal: bool,

    #[serde(default)]
    pub console_size: ConsoleSize,

    pub cwd: String, // Now required, default "/"

    #[serde(default)]
    pub env: Vec<String>,

    #[serde(default)]
    pub args: Vec<String>,

    // Linux-specific process fields
    #[serde(default, rename = "capabilities")]
    pub capabilities: Option<Capabilities>,

    #[serde(default, rename = "rlimits")]
    pub rlimits: Vec<Rlimit>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConsoleSize {
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub width: u32,
}

// ============================================================================
// Mounts
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mount {
    pub destination: String,

    #[serde(default)]
    #[serde(rename = "type")]
    pub mount_type: Option<String>,

    #[serde(default)]
    pub source: Option<String>,

    #[serde(default)]
    pub options: Vec<String>,
}

// ============================================================================
// Linux Configuration
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Linux {
    #[serde(default)]
    pub namespaces: Vec<LinuxNamespace>,

    #[serde(default, rename = "cgroupsPath")]
    pub cgroups_path: Option<String>,

    #[serde(default)]
    pub resources: Option<LinuxResources>,

    #[serde(default)]
    pub sysctl: HashMap<String, String>,

    #[serde(default)]
    pub devices: Vec<LinuxDevice>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinuxNamespace {
    #[serde(rename = "type")]
    pub namespace_type: String,

    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LinuxResources {
    #[serde(default)]
    pub memory: Option<LinuxMemory>,

    #[serde(default)]
    pub cpu: Option<LinuxCpu>,

    #[serde(default)]
    pub pids: Option<LinuxPids>,

    #[serde(default)]
    pub devices: Vec<LinuxDeviceCgroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinuxMemory {
    #[serde(default)]
    pub limit: Option<i64>,

    #[serde(default)]
    pub reservation: Option<i64>,

    #[serde(default)]
    pub swap: Option<i64>,

    #[serde(default)]
    pub kernel: Option<i64>,

    #[serde(default, rename = "kernelTCP")]
    pub kernel_tcp: Option<i64>,

    #[serde(default)]
    pub swappiness: Option<u64>,

    #[serde(default, rename = "disableOOMKiller")]
    pub disable_oom_killer: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinuxCpu {
    #[serde(default)]
    pub shares: Option<u64>,

    #[serde(default)]
    pub quota: Option<i64>,

    #[serde(default)]
    pub burst: Option<u64>,

    #[serde(default)]
    pub period: Option<u64>,

    #[serde(default, rename = "realtimeRuntime")]
    pub realtime_runtime: Option<i64>,

    #[serde(default, rename = "realtimePeriod")]
    pub realtime_period: Option<u64>,

    #[serde(default)]
    pub cpus: Option<String>,

    #[serde(default)]
    pub mems: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinuxPids {
    pub limit: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinuxDevice {
    #[serde(rename = "type")]
    pub device_type: String,

    pub path: String,

    #[serde(default)]
    pub major: Option<i64>,

    #[serde(default)]
    pub minor: Option<i64>,

    #[serde(default, rename = "fileMode")]
    pub file_mode: Option<u32>,

    #[serde(default)]
    pub uid: Option<u32>,

    #[serde(default)]
    pub gid: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinuxDeviceCgroup {
    #[serde(default)]
    pub allow: bool,

    #[serde(default)]
    #[serde(rename = "type")]
    pub device_type: Option<String>,

    #[serde(default)]
    pub major: Option<i64>,

    #[serde(default)]
    pub minor: Option<i64>,

    #[serde(default)]
    pub access: Option<String>,
}

// ============================================================================
// Capabilities (Linux process)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Capabilities {
    #[serde(default)]
    pub effective: Vec<String>,

    #[serde(default)]
    pub bounding: Vec<String>,

    #[serde(default)]
    pub inheritable: Vec<String>,

    #[serde(default)]
    pub permitted: Vec<String>,

    #[serde(default)]
    pub ambient: Vec<String>,
}

// ============================================================================
// Rlimits
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rlimit {
    #[serde(rename = "type")]
    pub rlimit_type: String,

    pub soft: u64,
    pub hard: u64,
}

// ============================================================================
// Hooks
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Hooks {
    #[serde(default, rename = "prestart")]
    pub prestart: Vec<Hook>,

    #[serde(default, rename = "createRuntime")]
    pub create_runtime: Vec<Hook>,

    #[serde(default, rename = "createContainer")]
    pub create_container: Vec<Hook>,

    #[serde(default, rename = "startContainer")]
    pub start_container: Vec<Hook>,

    #[serde(default, rename = "poststart")]
    pub poststart: Vec<Hook>,

    #[serde(default, rename = "poststop")]
    pub poststop: Vec<Hook>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Hook {
    pub path: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub env: Vec<String>,

    #[serde(default)]
    pub timeout: Option<i32>,
}

// ============================================================================
// RuntimeSpec impl
// ============================================================================

impl RuntimeSpec {
    /// Load and parse config.json from bundle directory
    pub fn load_config(bundle_path: &str) -> anyhow::Result<RuntimeSpec> {
        let config_path = Path::new(bundle_path).join("config.json");
        let json_str = fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("failed to read config.json: {}", e))?;
        let spec: RuntimeSpec = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("failed to parse config.json: {}", e))?;
        Ok(spec)
    }

    /// Validate the spec has required fields
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.oci_version.is_empty() {
            anyhow::bail!("ociVersion is required");
        }
        if self.root.path.is_empty() {
            anyhow::bail!("root.path is required");
        }
        if self.process.cwd.is_empty() {
            anyhow::bail!("process.cwd is required");
        }
        if self.process.args.is_empty() {
            anyhow::bail!("process.args is required");
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};
    use tempdir::TempDir;

    use crate::runtime::spec::{
        Hook, Hooks, Linux, LinuxCpu, LinuxMemory, LinuxNamespace, LinuxPids, LinuxResources,
        Mount, RuntimeSpec,
    };

    fn create_test_bundle(dir: &Path, config_json: &str) {
        fs::create_dir_all(dir.join("rootfs")).unwrap();
        fs::write(dir.join("config.json"), config_json).unwrap();
    }

    #[test]
    fn test_load_valid_minimal_config() {
        let temp_dir = TempDir::new("runtime_spec_test").unwrap();
        let config = r#"
        {
          "ociVersion": "1.0.2",
          "hostname": "mycontainer",
          "root": {
            "path": "rootfs"
          },
          "process": {
            "cwd": "/",
            "args": ["/bin/sh"]
          }
        }
        "#;

        create_test_bundle(temp_dir.path(), config);

        let spec = RuntimeSpec::load_config(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(spec.oci_version, "1.0.2");
        assert_eq!(spec.hostname, "mycontainer");
        assert_eq!(spec.process.args, vec!["/bin/sh"]);
        assert_eq!(spec.process.cwd, "/");
        assert_eq!(spec.process.env, Vec::<String>::new());
        assert_eq!(spec.process.terminal, false);
        assert_eq!(spec.process.console_size.height, 0);
        assert_eq!(spec.process.console_size.width, 0);
        assert_eq!(spec.root.readonly, false);
    }

    #[test]
    fn test_load_config_with_linux_namespaces() {
        let temp_dir = TempDir::new("runtime_spec_test").unwrap();
        let config = r#"
        {
          "ociVersion": "1.0.2",
          "hostname": "testcontainer",
          "root": {
            "path": "rootfs"
          },
          "process": {
            "cwd": "/",
            "args": ["/bin/sh"]
          },
          "linux": {
            "namespaces": [
              {"type": "pid"},
              {"type": "network"},
              {"type": "mount", "path": ""}
            ],
            "cgroupsPath": "/mycontainer",
            "resources": {
              "memory": {
                "limit": 1073741824,
                "swappiness": 60
              },
              "cpu": {
                "shares": 1024,
                "quota": 100000
              },
              "pids": {
                "limit": 100
              }
            },
            "sysctl": {
              "net.core.somaxconn": "1024"
            }
          }
        }
        "#;

        create_test_bundle(temp_dir.path(), config);

        let spec = RuntimeSpec::load_config(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(spec.oci_version, "1.0.2");

        let linux = spec.linux.unwrap();
        assert_eq!(linux.namespaces.len(), 3);
        assert_eq!(linux.namespaces[0].namespace_type, "pid");
        assert_eq!(linux.namespaces[1].namespace_type, "network");
        assert_eq!(linux.namespaces[2].namespace_type, "mount");
        assert_eq!(linux.cgroups_path.as_deref(), Some("/mycontainer"));

        let resources = linux.resources.unwrap();
        let memory = resources.memory.unwrap();
        assert_eq!(memory.limit, Some(1073741824));
        assert_eq!(memory.swappiness, Some(60));

        let cpu = resources.cpu.unwrap();
        assert_eq!(cpu.shares, Some(1024));
        assert_eq!(cpu.quota, Some(100000));

        let pids = resources.pids.unwrap();
        assert_eq!(pids.limit, 100);

        assert_eq!(
            linux.sysctl.get("net.core.somaxconn"),
            Some(&"1024".to_string())
        );
    }

    #[test]
    fn test_load_config_with_hooks() {
        let temp_dir = TempDir::new("runtime_spec_test").unwrap();
        let config = r#"
        {
          "ociVersion": "1.0.2",
          "hostname": "testcontainer",
          "root": {
            "path": "rootfs"
          },
          "process": {
            "cwd": "/",
            "args": ["/bin/sh"]
          },
          "hooks": {
            "prestart": [
              {
                "path": "/usr/bin/oci-hook",
                "args": ["oci-hook", "prestart"],
                "env": ["FOO=bar"],
                "timeout": 30
              }
            ],
            "poststop": [
              {
                "path": "/usr/bin/cleanup",
                "args": ["cleanup"]
              }
            ]
          }
        }
        "#;

        create_test_bundle(temp_dir.path(), config);

        let spec = RuntimeSpec::load_config(temp_dir.path().to_str().unwrap()).unwrap();

        let hooks = spec.hooks.unwrap();
        assert_eq!(hooks.prestart.len(), 1);
        assert_eq!(hooks.prestart[0].path, "/usr/bin/oci-hook");
        assert_eq!(hooks.prestart[0].args, vec!["oci-hook", "prestart"]);
        assert_eq!(hooks.prestart[0].env, vec!["FOO=bar"]);
        assert_eq!(hooks.prestart[0].timeout, Some(30));

        assert_eq!(hooks.poststop.len(), 1);
        assert_eq!(hooks.poststop[0].path, "/usr/bin/cleanup");
    }

    #[test]
    fn test_load_config_with_mounts() {
        let temp_dir = TempDir::new("runtime_spec_test").unwrap();
        let config = r#"
        {
          "ociVersion": "1.0.2",
          "hostname": "testcontainer",
          "root": {
            "path": "rootfs"
          },
          "process": {
            "cwd": "/",
            "args": ["/bin/sh"]
          },
          "mounts": [
            {
              "destination": "/proc",
              "type": "proc",
              "source": "proc"
            },
            {
              "destination": "/dev",
              "type": "tmpfs",
              "source": "tmpfs",
              "options": ["nosuid", "strictatime", "mode=755"]
            }
          ]
        }
        "#;

        create_test_bundle(temp_dir.path(), config);

        let spec = RuntimeSpec::load_config(temp_dir.path().to_str().unwrap()).unwrap();

        assert_eq!(spec.mounts.len(), 2);
        assert_eq!(spec.mounts[0].destination, "/proc");
        assert_eq!(spec.mounts[0].mount_type.as_deref(), Some("proc"));
        assert_eq!(spec.mounts[1].destination, "/dev");
        assert_eq!(
            spec.mounts[1].options,
            vec!["nosuid", "strictatime", "mode=755"]
        );
    }

    #[test]
    fn test_validate_spec_success() {
        let spec = RuntimeSpec {
            oci_version: "1.0.2".to_string(),
            hostname: "test".to_string(),
            domainname: "".to_string(),
            mounts: vec![],
            root: crate::runtime::spec::Root {
                path: "rootfs".to_string(),
                readonly: false,
            },
            process: crate::runtime::spec::Process {
                terminal: false,
                console_size: crate::runtime::spec::ConsoleSize::default(),
                cwd: "/".to_string(),
                env: vec![],
                args: vec!["/bin/sh".to_string()],
                capabilities: None,
                rlimits: vec![],
            },
            linux: None,
            hooks: None,
        };

        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_validate_spec_missing_oci_version() {
        let spec = RuntimeSpec {
            oci_version: "".to_string(),
            hostname: "test".to_string(),
            domainname: "".to_string(),
            mounts: vec![],
            root: crate::runtime::spec::Root {
                path: "rootfs".to_string(),
                readonly: false,
            },
            process: crate::runtime::spec::Process {
                terminal: false,
                console_size: crate::runtime::spec::ConsoleSize::default(),
                cwd: "/".to_string(),
                env: vec![],
                args: vec!["/bin/sh".to_string()],
                capabilities: None,
                rlimits: vec![],
            },
            linux: None,
            hooks: None,
        };

        assert!(spec.validate().is_err());
        assert_eq!(
            spec.validate().unwrap_err().to_string(),
            "ociVersion is required"
        );
    }

    #[test]
    fn test_validate_spec_missing_root_path() {
        let spec = RuntimeSpec {
            oci_version: "1.0.2".to_string(),
            hostname: "test".to_string(),
            domainname: "".to_string(),
            mounts: vec![],
            root: crate::runtime::spec::Root {
                path: "".to_string(),
                readonly: false,
            },
            process: crate::runtime::spec::Process {
                terminal: false,
                console_size: crate::runtime::spec::ConsoleSize::default(),
                cwd: "/".to_string(),
                env: vec![],
                args: vec!["/bin/sh".to_string()],
                capabilities: None,
                rlimits: vec![],
            },
            linux: None,
            hooks: None,
        };

        assert!(spec.validate().is_err());
        assert_eq!(
            spec.validate().unwrap_err().to_string(),
            "root.path is required"
        );
    }

    #[test]
    fn test_validate_spec_missing_cwd() {
        let spec = RuntimeSpec {
            oci_version: "1.0.2".to_string(),
            hostname: "test".to_string(),
            domainname: "".to_string(),
            mounts: vec![],
            root: crate::runtime::spec::Root {
                path: "rootfs".to_string(),
                readonly: false,
            },
            process: crate::runtime::spec::Process {
                terminal: false,
                console_size: crate::runtime::spec::ConsoleSize::default(),
                cwd: "".to_string(),
                env: vec![],
                args: vec!["/bin/sh".to_string()],
                capabilities: None,
                rlimits: vec![],
            },
            linux: None,
            hooks: None,
        };

        assert!(spec.validate().is_err());
        assert_eq!(
            spec.validate().unwrap_err().to_string(),
            "process.cwd is required"
        );
    }

    #[test]
    fn test_validate_spec_missing_args() {
        let spec = RuntimeSpec {
            oci_version: "1.0.2".to_string(),
            hostname: "test".to_string(),
            domainname: "".to_string(),
            mounts: vec![],
            root: crate::runtime::spec::Root {
                path: "rootfs".to_string(),
                readonly: false,
            },
            process: crate::runtime::spec::Process {
                terminal: false,
                console_size: crate::runtime::spec::ConsoleSize::default(),
                cwd: "/".to_string(),
                env: vec![],
                args: vec![],
                capabilities: None,
                rlimits: vec![],
            },
            linux: None,
            hooks: None,
        };

        assert!(spec.validate().is_err());
        assert_eq!(
            spec.validate().unwrap_err().to_string(),
            "process.args is required"
        );
    }
}
