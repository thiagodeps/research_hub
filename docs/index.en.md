# Research Hub 🧬

**Desktop** tool for academic data curation. Receives the `exports_canonical.zip` package from the DataLake, allows correcting, merging, and linking records, and exports the package back — keeping all files it doesn't manage completely intact.

Single-process application: no server, no browser, no Python, no network.

## Technologies

- **Rust + Tauri 2.0** — core, window, and IPC communication
- **SQLite** (`rusqlite`, embedded) — single-file database
- **arrow-rs** — reading and writing canonical `.parquet` files
- **Astro + React + Tailwind** — UI, statically compiled and embedded in the binary

## Install

Download the installer from the [releases page](https://github.com/RafaelDeps/research_hub/releases):
`.deb`, `.rpm`, or `.AppImage` on Linux, `.exe` on Windows.

> **Windows:** On the first run, SmartScreen might display "Windows protected your PC". Click on *More info* → *Run anyway*. This warning appears because the binary is unsigned — a conscious decision for a personal use tool.

## Usage

1. **Sign in** — `admin@admin.com` / `admin123`.
2. **Import** — In the dashboard, choose the `exports_canonical.zip` or drag it into the window. This **replaces the current database**; an automatic backup is created before replacing, and its path is shown on the screen.
3. **Curate** — Navigate through the 15 entities in the sidebar. Search, sort, edit, merge duplicates, and create links.
4. **Export** — Generates the canonical package with the original data types restored and the other ZIP files preserved byte by byte.

Data is stored in `~/.local/share/br.edu.ifes.researchhub/hub.db` on Linux and in `%APPDATA%\br.edu.ifes.researchhub\hub.db` on Windows. To backup, just copy this file.

## Development

Prerequisites: [Rust](https://rustup.rs), Node 22+ and, on Linux:

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev pkg-config
```

> If the terminal was opened from a snap (VS Code, some emulators), it exports an `LD_LIBRARY_PATH` to `/snap/core20` which collides with the system glibc and crashes the binary with `undefined symbol: __libc_pthread_init`. The Makefile already removes the variable; to run the executable directly, use `env -u LD_LIBRARY_PATH ./src-tauri/target/debug/research-hub`.

```bash
make build    # frontend dependencies
make dev      # run with hot reload
make test     # cargo test + vitest
make bundle   # native installers
```

### Container Build

To generate Linux installers without installing Rust, Node, and Webkit on the host machine:

```bash
make docker-bundle    # output in dist/
```

The image intentionally uses Ubuntu 24.04: compiling against an older glibc ensures the binaries run on more systems, never fewer.

#### Running generated artifacts

The generated files will be in the `dist/` folder and will belong to the `root` user.
You have 3 methods to test or install the application on Linux:

**1. AppImage (Runs without installing - Recommended for quick testing)**
```bash
cd dist/
# Change owner to your user (avoids running as root)
sudo chown $USER:$USER ResearchHub_*.AppImage
# Give execute permission
chmod +x ResearchHub_*.AppImage
# Run
./ResearchHub_*.AppImage
```

**2. Debian/Ubuntu Installer (.deb)**
```bash
cd dist/
sudo apt install ./ResearchHub_*.deb
```

**3. Fedora/RHEL Installer (.rpm)**
```bash
cd dist/
sudo dnf install ./ResearchHub-*.rpm
```

> **Docker here is a build tool, not a runtime tool.** This is a desktop application: running it inside a container would require exposing the host's X11 socket, which is worse than simply installing the `.deb` produced by the image. If you still need to (a machine without system libraries, for example), the way is `-e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix --device /dev/dri`, with the caveats of hardware acceleration and database persistence that this implies.

## Data Domains

Researchers, Students, Research Groups, Initiatives, Awards, Scientific Productions, Knowledge Areas, Advisees/Mentorships, Organizations, Professional Activities, Campus, Proficiencies, Scholarships, Languages, and Articles.

Relationship columns store JSON arrays, avoiding associative tables and keeping reads direct in Big Data pipelines.

## Migrating from the web version

There is no automatic migration from the old `backend/test.db`. The supported path is to re-import the `exports_canonical.zip` into the app — the result is equivalent, since the database was always derived from it.

## Documentation

- `.specify/memory/constitution.md` — architecture rules
- `docs/estudo-migracao-rust-tauri.md` — the study that originated the rewrite
- `specs/` — feature specifications
