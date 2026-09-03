CARGO := $(HOME)/.cargo/bin/cargo

.PHONY: build dev run test bundle clean

## Install frontend dependencies (Rust deps are fetched by cargo on demand).
build:
	cd frontend && npm install

## Run the desktop app with hot reload.
dev run:
	cd src-tauri && $(CARGO) tauri dev

## Full suite: Rust core + frontend bridge.
test:
	cd src-tauri && $(CARGO) test
	cd frontend && npm test

## Native installers for the current platform.
bundle:
	cd src-tauri && $(CARGO) tauri build

clean:
	rm -rf frontend/node_modules frontend/dist src-tauri/target
