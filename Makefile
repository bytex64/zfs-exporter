BINARY = zfs-exporter
BUILD := release

.PHONY: install
install: target/$(BUILD)/$(BINARY)
	install -m755 target/$(BUILD)/$(BINARY) /usr/local/bin/$(BINARY)
	install -m755 rc.d/zfs-exporter /usr/local/etc/rc.d/zfs-exporter

target/release/$(BINARY):
	cargo build --release

target/debug/$(BINARY):
	cargo build
