pub mod manifests;

use std::{fs::create_dir_all, path::PathBuf, time::Duration};

use crate::{
    layers::manifests::{
        config::ImageConfigManifest,
        platform_specific_image::{ImageManifest, ImageMedia},
    },
    sys::{arch, sysinfo},
};

use anyhow::{anyhow, bail, Context, Error, Result};
use flate2::read::GzDecoder;
use futures_util::{future::join_all, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;
use tar::Archive;
use tokio::task::spawn_blocking;
use tokio_util::io::{StreamReader, SyncIoBridge};

pub struct ContainerImageFetcher {
    client: Client,
    image_repo: String,
    image_name: String,
    image_tag: String,
    token: String,
}

impl ContainerImageFetcher {
    pub async fn new(image: &str) -> Result<Self> {
        let (namespace, image_name) = image.split_once('/').unwrap_or(("library", image));
        let (image_name, image_tag) = image.split_once(':').unwrap_or((image_name, "latest"));
        let http_client = Client::builder()
            .timeout(Duration::from_secs(600))
            .connect_timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()?;

        let token = Self::fetch_and_set_anonymous_token(&http_client, &namespace, &image_name)
            .await
            .context(format!(
                "Failed to authenticate with Docker registry for {}/{}",
                namespace, image_name
            ))?;

        Ok(Self {
            client: http_client,
            image_repo: namespace.to_string(),
            image_name: image_name.to_string(),
            image_tag: image_tag.to_string(),
            token: token,
        })
    }

    async fn fetch_and_set_anonymous_token(
        http_client: &Client,
        image_repo: &str,
        image_name: &str,
    ) -> Result<String> {
        let response = http_client
            .get("https://auth.docker.io/token")
            .query(&[
                ("service", "registry.docker.io"),
                (
                    "scope",
                    format!("repository:{}/{}:pull", image_repo, image_name).as_str(),
                ),
            ])
            .send()
            .await
            .context("Failed to send authentication request to Docker registry")?
            .error_for_status()
            .context("Docker registry returned error status for authentication")?;

        let json_value: Value = response
            .json()
            .await
            .context("Failed to parse authentication response as JSON")?;

        let token = json_value
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Token field missing or invalid in authentication response"))?;

        Ok(token.to_string())
    }

    async fn fetch_multi_platform_image_index(&self) -> Result<Value> {
        let response = self
            .client
            .get(format!(
                "https://registry-1.docker.io/v2/{}/{}/manifests/{}",
                self.image_repo, self.image_name, self.image_tag
            ))
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .context(format!(
                "Failed to fetch manifest for {}:{}",
                self.image_name, self.image_tag
            ))?
            .error_for_status()
            .context(format!(
                "Registry returned error for {}:{}",
                self.image_name, self.image_tag
            ))?;

        let json_response: Value = response
            .json()
            .await
            .context("Failed to parse manifest index as JSON")?;

        Ok(json_response)
    }

    async fn fetch_platform_specific_image_manifest(&self, digest: &str) -> Result<Value> {
        let response = self
            .client
            .get(format!(
                "https://registry-1.docker.io/v2/{}/{}/manifests/{}",
                self.image_repo, self.image_name, digest
            ))
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .context(format!(
                "Failed to fetch platform manifest for digest {}",
                digest
            ))?
            .error_for_status()
            .context(format!(
                "Registry returned error for platform manifest {}",
                digest
            ))?;

        let json_response: Value = response
            .json()
            .await
            .context("Failed to parse platform manifest as JSON")?;

        Ok(json_response)
    }

    async fn fetch_manifest_blob(&self, digest: &str, media_type: &str) -> Result<Response> {
        let response = self
            .client
            .get(format!(
                "https://registry-1.docker.io/v2/{}/{}/blobs/{}",
                self.image_repo, self.image_name, digest
            ))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", media_type)
            .send()
            .await
            .context(format!("Failed to fetch blob {}", digest))?
            .error_for_status()
            .context(format!("Registry returned error for blob {}", digest))?;

        Ok(response)
    }

    pub async fn fetch_image(&self) -> Result<()> {
        let sysinfo = sysinfo::get_platform_information()?;

        let arch = sysinfo.machine.to_string();
        let sys_variant = sysinfo.machine.get_variant();

        let arch = arch
            .ok_or_else(|| anyhow::anyhow!("Cannot determine the platform of the host system."))?;

        let json_image_index: Value = self.fetch_multi_platform_image_index().await?;

        let manifests = json_image_index
            .get("manifests")
            .and_then(|m| m.as_array())
            .ok_or_else(|| anyhow::anyhow!("No 'manifests' array found"))?;

        let host_arch_manifest: Vec<&Value> = manifests
            .iter()
            .filter(|manifest| {
                manifest
                    .get("annotations")
                    .and_then(|ann| ann.get("vnd.docker.reference.type"))
                    .and_then(Value::as_str)
                    != Some("attestation-manifest")
            })
            .filter(|manifest| {
                manifest
                    .get("platform")
                    .and_then(|platform| platform.get("architecture"))
                    .and_then(Value::as_str)
                    == Some(arch)
            })
            .collect();

        let platform_specific_manifest_digest = match host_arch_manifest.len() {
            0 => bail!("No manifest found for the host arch."),
            1 => self.extract_digest_from_manifest(host_arch_manifest[0])?,
            _ => {
                let host_arch_variant_manifest = host_arch_manifest.iter().find(|manifest| {
                    let manifest_variant = manifest
                        .get("platform")
                        .and_then(|platform| platform.get("variant"))
                        .and_then(Value::as_str)
                        .and_then(arch::Variant::from_str);

                    manifest_variant == sys_variant
                });

                let manifest_for_variant = host_arch_variant_manifest.ok_or_else(|| {
                    anyhow::anyhow!("No manifest found for host arch and variant.")
                })?;

                self.extract_digest_from_manifest(manifest_for_variant)?
            }
        };

        let image_manifest = self
            .fetch_platform_specific_image_manifest(&platform_specific_manifest_digest)
            .await?;

        let image_manifest_parsed: ImageManifest = serde_json::from_value(image_manifest)?;

        let config_response: Value = self
            .fetch_manifest_blob(
                &image_manifest_parsed.config.digest,
                &image_manifest_parsed.config.media_type,
            )
            .await?
            .json()
            .await?;

        let config: ImageConfigManifest = serde_json::from_value(config_response)?;

        let mp = MultiProgress::new();
        let spinner_style = ProgressStyle::with_template("{spinner} {msg}")?.tick_chars("/|\\- ");
        let fetchers = image_manifest_parsed.layers.iter().map(|layer| {
            let pb = mp.add(ProgressBar::new_spinner());
            pb.set_style(spinner_style.clone());
            pb.enable_steady_tick(std::time::Duration::from_millis(120));

            self.fetch_layer(layer, pb)
        });

        let errors: Vec<_> = join_all(fetchers)
            .await
            .into_iter()
            .filter_map(|r| r.err())
            .collect();

        if !errors.is_empty() {
            bail!("Failed to fetch {} layer(s): {:?}", errors.len(), errors);
        }

        Ok(())
    }

    async fn fetch_layer(&self, layer: &ImageMedia, pb: ProgressBar) -> Result<()> {
        const MAX_RETRIES: u32 = 3;

        for attempt in 1..=MAX_RETRIES {
            match self._fetch_layer(layer, pb.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) if attempt < MAX_RETRIES => {
                    pb.set_message(format!(
                        "Retry {}/{} for {}",
                        attempt, MAX_RETRIES, &layer.digest
                    ));
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
                Err(e) => {
                    pb.finish_with_message(format!(
                        "✗ Failed {} after {} retries",
                        &layer.digest, MAX_RETRIES
                    ));
                    return Err(e);
                }
            }
        }
        unreachable!()
    }

    async fn _fetch_layer(&self, layer: &ImageMedia, pb: ProgressBar) -> Result<()> {
        // # TODO: Fetch layers that have already been fetched
        let digest = layer.digest.clone();
        pb.set_message(format!("Downloading {}", layer.digest));

        let resp = self
            .fetch_manifest_blob(&layer.digest, &layer.media_type)
            .await?;

        let mut stream = resp.bytes_stream();

        let async_stream = StreamReader::new(stream.map(|byte_stream| {
            byte_stream.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }));

        let digest_for_closure = layer.digest.clone();
        let output_dir: PathBuf = ["./extracted_layers", &digest].iter().collect();

        spawn_blocking(move || -> Result<()> {
            pb.set_message(format!("Extracting {}", &digest_for_closure));

            let sync_reader = SyncIoBridge::new(async_stream);
            let gz_decoder = GzDecoder::new(sync_reader);
            let mut archive = Archive::new(gz_decoder);
            create_dir_all(&output_dir).context("Failed to create output directory")?;

            match archive.unpack(&output_dir) {
                Ok(_) => {
                    pb.finish_with_message(format!("Done {}", &digest_for_closure));
                    Ok(())
                }
                Err(e) => Err(e).context(format!(
                    "Failed to extract layer {} to {:?}",
                    &digest_for_closure, output_dir
                )),
            }
        })
        .await
        .context("Layer extraction task panicked")?
        .context(format!("Failed to extract layer {}", &digest))?;

        Ok(())
    }

    fn extract_digest_from_manifest(&self, manifest: &Value) -> Result<String> {
        manifest
            .get("digest")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| anyhow!("Cannot extract manifest from digest"))
    }
}
