# src/layers/

**Commit:** 77c62c6

## OVERVIEW
Docker registry image fetching, multi-arch manifest resolution, and layer extraction with progress bars

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Registry auth | `layers.rs:46-93` | `fetch_and_set_anonymous_token`, Docker hub |
| Multi-arch resolution | `layers.rs:186-231` | Filter by arch + variant annotation |
| Layer fetch + extract | `layers.rs:177-358` | `fetch_image()` → `fetch_layer()` → `spawn_blocking` |
| Progress bars | `layers.rs:250-258` | `MultiProgress` + `ProgressBar` with steady tick |
| Manifest parsing | `layers/manifests/` | config, multi_arch, platform_specific |
| Layer retry loop | `layers.rs:276-296` | 3 attempts, 2s sleep, `MAX_RETRIES` constant |

## STRUCTURE
```
layers/
├── manifests/
│   ├── config.rs                  # ImageConfigManifest (OCI config blob)
│   ├── multi_arch_image_index.rs # MultiArchImageIndex
│   └── platform_specific_image.rs # ImageManifest, ImageMedia (digest, media_type, layers)
└── layers.rs                     # ContainerImageFetcher
```

## CONVENTIONS
- `reqwest::Client` with 600s timeout, 30s connect, 90s idle pool
- Token auth: `https://auth.docker.io/token?service=registry.docker.io&scope=repository:{repo}:pull`
- `#[tokio::main]` async entry, CPU-heavy work in `spawn_blocking` (tar+gzip extraction)
- `indicatif::ProgressBar` with `{spinner} {msg}` template and steady tick
- Layer caching: `.{digest}.mrk` marker files, skip re-extract if exists
- Extracts to `./extracted_layers/<digest>/` (needs OCI layout fix — TODO at line 300)
- Manifest annotation filter: `vnd.docker.reference.type != attestation-manifest`

## ANTI-PATTERNS (THIS MODULE)
- Extracts to `./extracted_layers/<digest>/` instead of proper OCI dirs (etc/, opt/) — TODO at layers.rs:300
