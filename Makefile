VM_NAME=kapsule

BINARY_NAME=kapsule
BUILD_DIR=build
TARGET=aarch64-unknown-linux-musl

.PHONY: all clean build

all: clean build

build: $(BUILD_DIR)/$(BINARY_NAME)

$(BUILD_DIR)/$(BINARY_NAME):
	mkdir -p $(BUILD_DIR)
	docker run --rm -v $(PWD):/app -w /app rust:alpine \
		sh -c "rustup target add $(TARGET) && \
		cargo build --target $(TARGET) --release"
	cp target/$(TARGET)/release/$(BINARY_NAME) $(BUILD_DIR)/

clean:
	rm -rf $(BUILD_DIR)/$(BINARY_NAME)
	cargo clean

vm-start:
	limactl start --name=$(VM_NAME) ./kapsule.yml

vm-stop:
	limactl stop $(VM_NAME)

vm-delete:
	limactl delete $(VM_NAME)

vm-clean: vm-stop vm-clean

run:
	@limactl shell $(VM_NAME) sudo -i $(PWD)/build/$(BINARY_NAME) run /bin/bash

shell:
	@limactl shell $(VM_NAME) sudo su -
