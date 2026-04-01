# src/runtime/

**Commit:** 77c62c6

## OVERVIEW
OCI RuntimeSpec parsing and loading from bundle config.json

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| RuntimeSpec struct | `spec.rs:6-13` | `#[serde(rename_all = "camelCase")]` |
| Config loading | `spec.rs:49-56` | `load_config()` joins `config.json` to path |
| Test fixture | `spec.rs:69-99` | `create_test_bundle()` + `TempDir` |
| Process struct | `spec.rs:15-30` | Terminal, cwd, env, args |
| Root struct | `spec.rs:41-46` | path + readonly |

## CONVENTIONS
- `#[serde(rename_all = "camelCase")]` for all OCI structs
- `#[serde(default)]` on optional fields (terminal, env, args, console_size, readonly)
- `load_config()` returns `Result<RuntimeSpec>` with `?` and `fs::read_to_string`
- Test pattern: `tempdir::TempDir` + manual `create_test_bundle()` helper
- All test assertions check exact deserialized values

## ANTI-PATTERNS (THIS MODULE)
- `hostname` and `cwd` are `Option<String>` but TODO: should be required (spec.rs:12,23)
- `load_config()` contains `println!("{:?}", path)` debug artifact (spec.rs:52)
