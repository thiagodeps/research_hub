CARGO := $(HOME)/.cargo/bin/cargo

# Terminals launched from a snap (VS Code, some emulators) export an
# LD_LIBRARY_PATH pointing at /snap/core20 libraries, which collide with the
# system glibc and make the binary die with:
#   symbol lookup error: ... undefined symbol: __libc_pthread_init
# Dropping the variable is enough; it is never needed to run this app.
RUN := env -u LD_LIBRARY_PATH -u SNAP -u SNAP_NAME

.PHONY: build dev run test bundle docker-bundle clean

## Install frontend dependencies (Rust deps are fetched by cargo on demand).
build:
	cd frontend && npm install

## Run the desktop app with hot reload.
dev run:
	cd src-tauri && $(RUN) $(CARGO) tauri dev

## Full suite: Rust core + frontend bridge.
test:
	cd src-tauri && $(RUN) $(CARGO) test
	cd frontend && npm test

## Native installers for the current platform.
bundle:
	cd src-tauri && $(RUN) $(CARGO) tauri build

## Reproducible Linux installers, built in a container (no host toolchain).
docker-bundle:
	docker build -t research-hub-builder .
	@mkdir -p dist
	@id=$$(docker create research-hub-builder) \
	  && docker cp $$id:/out/. dist/ \
	  && docker rm -v $$id >/dev/null
	@ls -lh dist/

clean:
	rm -rf frontend/node_modules frontend/dist src-tauri/target dist
