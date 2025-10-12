pub mod manifests;

use std::{fs::create_dir_all, path::PathBuf, time::Duration};

use crate::{
    layers::manifests::{
        config::ImageConfigManifest,
        platform_specific_image::{ImageManifest, ImageMedia},
    },
    sys::{arch, sysinfo},
};

use anyhow::{Context, Error, Ok, Result};
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
        let http_client = Client::new();
        let token =
            Self::fetch_and_set_anonymous_token(&http_client, &namespace, &image_name).await?;

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
            .await?
            .error_for_status()?;

        let json_value: Value = response.json().await?;

        let token = json_value
            .get("token")
            .and_then(|v| v.as_str())
            .expect("Token not found in the response returned by docker auth api.");

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
            .await?
            .error_for_status()?;

        let json_response: Value = response.json().await?;

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
            .await?
            .error_for_status()?;

        let json_response: Value = response.json().await?;

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
            .await?
            .error_for_status()?;

        Ok(response)
    }

    pub async fn fetch_image(&self) -> Result<()> {
        // TODO: Better error handling here
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
            0 => return Err(anyhow::anyhow!("No manifest found for the host arch.")),
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

        join_all(fetchers)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }

    async fn fetch_layer(&self, layer: &ImageMedia, pb: ProgressBar) -> Result<()> {
        let digest = layer.digest.clone();
        pb.set_message(format!("Downloading {}", layer.digest));

        let resp = self
            .fetch_manifest_blob(&layer.digest, &layer.media_type)
            .await?;

        let mut stream = resp.bytes_stream();

        let async_stream = StreamReader::new(stream.map(|byte_stream| {
            byte_stream.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }));

        let digest = layer.digest.clone();
        let output_dir: PathBuf = ["./extracted_layers", &digest].iter().collect();

        spawn_blocking(move || {
            pb.set_message(format!("Extracting {}", digest));

            let sync_reader = SyncIoBridge::new(async_stream);
            let gz_decoder = GzDecoder::new(sync_reader);
            let mut archive = Archive::new(gz_decoder);
            create_dir_all(&output_dir).context("Failed to create output directory")?;
            archive
                .unpack(&output_dir)
                .context("Failed to extract tar archive")?;

            // println!("Layer extracted to: {}", &output_dir.display());
            pb.finish_with_message(format!("Done {}", digest));
            Ok(())
        })
        .await?;

        Ok(())
    }

    fn extract_digest_from_manifest(&self, manifest: &Value) -> Result<String> {
        let digest = manifest
            .get("digest")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Cannot extract manifest from digest"))?;

        return Ok(digest.to_owned());
    }
}
