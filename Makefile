VM_NAME=kapsule

BINARY_NAME=kapsule
BUILD_DIR=build
TARGET=aarch64-unknown-linux-musl
DOCKER_CONTAINER=rust-build-env

.PHONY: all clean build docker-start docker-stop docker-shell bundle

all: clean build

docker-start:
	@if ! docker ps --format '{{.Names}}' | grep -q '^$(DOCKER_CONTAINER)$$'; then \
		docker run -dit --name $(DOCKER_CONTAINER) \
			-v $(PWD):/app -w /app \
			-v $(PWD)/target:/app/target \
			rust:bookworm bash; \
		docker exec -it $(DOCKER_CONTAINER) bash -c "\
			apt-get update && \
			apt-get install -y pkg-config libssl-dev musl-tools && \
			rustup target add $(TARGET)"; \
	fi

docker-stop:
	@docker stop $(DOCKER_CONTAINER) || true
	@docker rm $(DOCKER_CONTAINER) || true

build: docker-start
	@docker exec -t $(DOCKER_CONTAINER) cargo build --target $(TARGET) --release
	@mkdir -p $(BUILD_DIR)
	@cp target/$(TARGET)/release/$(BINARY_NAME) $(BUILD_DIR)/

check: 
	@docker exec -t $(DOCKER_CONTAINER) cargo check --target $(TARGET)

test: 
	@docker exec -t $(DOCKER_CONTAINER) cargo test

clean:
	@rm -rf $(BUILD_DIR)/$(BINARY_NAME)
	@docker exec $(DOCKER_CONTAINER) cargo clean || true

docker-shell:
	@docker exec -it $(DOCKER_CONTAINER) bash

vm-start:
	limactl start --name=$(VM_NAME) ./kapsule.yml

vm-stop:
	limactl stop $(VM_NAME)

vm-delete:
	limactl delete $(VM_NAME)

vm-clean: vm-stop

nsenter-host:
	docker run -it --privileged --pid=host debian nsenter -t 1 -m -u -n -i sh


run: bundle
	@limactl shell $(VM_NAME) sudo -i $(PWD)/build/$(BINARY_NAME) create container_id_abcded /root/bundle/

bundle:
	@limactl shell $(VM_NAME) sudo mkdir -p /root/bundle/rootfs
	@limactl shell $(VM_NAME) sudo tee /root/bundle/config.json > /dev/null << 'EOFBUNDLE'
{
  "ociVersion": "1.0.2",
  "root": {
    "path": "rootfs",
    "readonly": false
  },
  "process": {
    "terminal": false,
    "cwd": "/",
    "env": ["PATH=/usr/local/sbin:/usr/local/bin:/bin:/usr/bin:/sbin:/usr/sbin"],
    "args": ["/bin/sh", "-c", "echo hello"]
  },
  "hostname": "container"
}
EOFBUNDLE

shell:
	@limactl shell $(VM_NAME) sudo su -
