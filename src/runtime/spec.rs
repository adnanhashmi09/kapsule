use anyhow::Result;
use std::{fs, path::Path};

use serde::Deserialize;

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct RuntimeSpec {
    oci_version: String,
    root: Root,
    process: Process,
    hostname: Option<String>, // TODO: make required
}

#[serde(rename_all = "camelCase")]
#[derive(Default, Deserialize, Debug)]
pub struct Process {
    #[serde(default)]
    terminal: bool,

    #[serde(default)]
    console_size: ConsoleSize,
    cwd: Option<String>, // TODO: make required

    #[serde(default)]
    env: Vec<String>,

    #[serde(default)]
    args: Vec<String>,
}

#[derive(Default, Deserialize, Debug)]
pub struct ConsoleSize {
    #[serde(default)]
    height: u32,

    #[serde(default)]
    width: u32,
}

#[derive(Deserialize, Debug)]
pub struct Root {
    path: String,
    #[serde(default)]
    readonly: bool,
}

impl RuntimeSpec {
    pub fn load_config(path: &str) -> Result<RuntimeSpec> {
        let path = Path::new(path);
        let path = path.join("config.json");
        println!("{:?}", path);
        let json_str = fs::read_to_string(path)?;
        let spec = serde_json::from_str::<RuntimeSpec>(&json_str)?;
        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use tempdir::TempDir;

    use crate::runtime::spec::RuntimeSpec;

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
          "root": {
            "path": "rootfs"
          },
          "process": {
            "args": ["/bin/sh"]
          }
        }
        "#;

        create_test_bundle(temp_dir.path(), config);

        let spec = RuntimeSpec::load_config(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(spec.oci_version, "1.0.2");
        assert_eq!(spec.process.args, vec!["/bin/sh"]);
        assert_eq!(spec.process.env, Vec::<String>::new());
        assert_eq!(spec.process.terminal, false);
        assert_eq!(spec.process.console_size.height, 0);
        assert_eq!(spec.process.console_size.width, 0);
        assert_eq!(spec.root.readonly, false);
    }
}
