import requests
import json


def get_docker_token(repository):
    """Fetch an anonymous Docker token."""
    auth_url = "https://auth.docker.io/token"
    params = {"service": "registry.docker.io", "scope": f"repository:{repository}:pull"}
    response = requests.get(auth_url, params=params)
    response.raise_for_status()
    return response.json().get("token")


def get_manifest(repository, reference, token):
    """
    Fetch a manifest from Docker registry.
    reference can be a tag (e.g., 'latest') or digest (e.g., 'sha256:...')
    """
    url = f"https://registry-1.docker.io/v2/{repository}/manifests/{reference}"
    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json",
    }
    response = requests.get(url, headers=headers)
    response.raise_for_status()
    return response.json()


def get_blob(repository, digest, token):
    """Fetch a blob (config or layer) from Docker registry."""
    url = f"https://registry-1.docker.io/v2/{repository}/blobs/{digest}"
    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.docker.container.image.v1+json",
    }
    response = requests.get(url, headers=headers)
    response.raise_for_status()
    return response.json()


def save_json(data, filename):
    """Save data to JSON file."""
    with open(filename, "w") as f:
        json.dump(data, f, indent=2)
    print(f"✓ Saved to: {filename}")


def main():
    repository = "library/redis"
    tag = "latest"

    # Get token
    print("Getting Docker token...")
    token = get_docker_token(repository)

    # Get manifest list
    print(f"\nFetching manifest list for {repository}:{tag}...")
    manifest_list = get_manifest(repository, tag, token)
    save_json(manifest_list, "redis_manifest_list.json")

    # Get platform-specific manifest (amd64)
    digest = "sha256:4d344fd752d4ec85e71bb40b5dedc95d1c7bd2b947f002498dfbf68d78a43c30"
    print(f"\nFetching platform-specific manifest...")
    platform_manifest = get_manifest(repository, digest, token)
    save_json(platform_manifest, "redis_amd64_manifest.json")

    # Get config blob
    config_digest = (
        "sha256:1c390e3bb5cb9c724e3947b45e57c8d73f0afe2b308141261c5669d7d97a452e"
    )
    print(f"\nFetching config blob...")
    config = get_blob(repository, config_digest, token)
    save_json(config, "redis_config.json")

    print("\n" + "=" * 50)
    print(
        f"Platform manifest config digest: {platform_manifest.get('config', {}).get('digest')}"
    )
    print(f"Number of layers: {len(platform_manifest.get('layers', []))}")
    print(f"Config architecture: {config.get('architecture')}")
    print(f"Config OS: {config.get('os')}")
    print("=" * 50)


if __name__ == "__main__":
    main()
