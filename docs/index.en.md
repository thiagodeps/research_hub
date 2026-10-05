# Research Hub 🧬

**High-performance desktop** tool for academic and extensionist data curation, sanitization, and linking.

Research Hub was engineered as a single-process, **offline-first** desktop application: no background servers, no open network ports, no Python runtime dependency, and an embedded single-file SQLite database.

---

## 📸 Screenshots & Showcase

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr)); gap: 12px; margin-bottom: 2rem;">
  <img width="100%" alt="Entity Listing and Management" src="https://github.com/user-attachments/assets/cf25ce56-8ac7-44e6-b2c6-119faac088d9" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="General Curation Dashboard" src="https://github.com/user-attachments/assets/d3e824c3-5f48-4d73-baa4-1430ba497f1a" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="Record Viewing and Editing" src="https://github.com/user-attachments/assets/cbc63fc3-f67c-4925-9f62-d1c3b825134c" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="Relationships and Links between Entities" src="https://github.com/user-attachments/assets/3d0b22a5-f7f3-4a21-9140-8d9e58d856f2" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="Import and Export with Automatic Backup" src="https://github.com/user-attachments/assets/df9658de-af86-43e6-9584-1ea0c01741c9" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
  <img width="100%" alt="Conflict Resolution and Record Merging" src="https://github.com/user-attachments/assets/9a16f816-cf73-4cd1-8c13-8914197b665e" style="border-radius: 8px; border: 1px solid #e2e8f0;" />
</div>

---

## 🛠️ Technologies & Architecture

- **Core Application**: [Rust](https://www.rust-lang.org/) (2021 edition, rustc 1.77.2+)
- **Desktop Shell & IPC**: [Tauri 2.0](https://v2.tauri.app/)
- **Database**: Embedded [SQLite](https://www.sqlite.org/) via crate `rusqlite` (bundled, single local file)
- **Frontend**: [Astro 4](https://astro.build/) + [React 18](https://react.dev/) + [Tailwind CSS 3](https://tailwindcss.com/) (statically compiled and embedded into the binary)
- **Format Handling**: `serde_json`, `zip-rs`, `arrow-rs` / `parquet-rs`
- **Remote Synchronization**: `reqwest` with pure `rustls-tls` and GitHub Git Data API integration
- **Security & Cryptography**: `bcrypt` for local authentication and `0600` POSIX file permissions for local tokens

---

## 📂 Supported Data Domains

Research Hub features a **multi-project selector** (`/projects`) to switch between distinct curation workflows with complete isolation:

### 1. Horizon Project (Academic Curation)
Manages the 15 canonical relational tables from the DataLake:
*Researchers, Students, Research Groups, Initiatives, Awards, Scientific Productions, Knowledge Areas, Advisees, Organizations, Professional Activities, Campuses, Proficiencies, Scholarships, Languages, and Articles*.

Relational columns store serialized JSON arrays, enabling direct read access in big data pipelines without requiring intermediary bridge tables.

### 2. SRC Project (Extension & Teaching)
Manages the consolidated Extension & Teaching file (`src_consolidado.json`):
- **Actions**: Process number, title, nature, type, coordinator, funding, and thematic area.
- **Nested Participations**: Dedicated editor for target audiences and execution teams.
- **Strict Privacy**: Participant personal identifiable information (CPF, email) is kept strictly local, with architectural blocks preventing remote transmission.

### 3. GitHub Synchronization
- **Remote Download**: Import canonical packages directly via GitHub URLs (raw links or release assets), in public or private repositories with a token.
- **Atomic Upload**: Send Horizon canonical packages to GitHub via the Git Data API (creates atomic commits directly without cloning).
- **Secure Local Token**: Personal Access Token stored with strict `0600` POSIX file permissions.

---

## 📋 Code and Runtime Requirements

### Supported Operating Systems
- **Linux**: Ubuntu 22.04+, Debian 12+, Fedora 39+, Arch Linux (x86_64 and aarch64).
- **Windows**: Windows 10 (1809+) or Windows 11 with WebView2 Runtime.
- **macOS**: macOS 10.15+ (Catalina or higher), Universal for Intel and Apple Silicon.

### Linux Native Dependencies
```bash
# Ubuntu / Debian / Mint
sudo apt update && sudo apt install -y build-essential curl wget pkg-config \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev

# Fedora / RHEL
sudo dnf groupinstall "Development Tools" && sudo dnf install -y \
  webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel \
  gcc-c++ pkgconf-pkg-config curl wget

# Arch Linux / Manjaro
sudo pacman -Syu --needed base-devel curl wget webkit2gtk-4.1 libappindicator-gtk3 openssl librsvg
```

### Toolchain Requirements
- **Rust**: 1.77.2+ with `cargo` and `rustc` (`rustup`).
- **Node.js**: 22.x LTS and `npm` 10+.
- **Tauri CLI**: `cargo-tauri` (`cargo install tauri-cli --version "^2.0.0"`).
- **Python (Documentation)**: Python 3.10+ with `mkdocs`, `mkdocs-material`, `mkdocs-static-i18n`.

### Local Storage and Permissions
- SQLite Database: `~/.local/share/br.edu.ifes.researchhub/hub.db` (Linux) or `%APPDATA%\br.edu.ifes.researchhub\hub.db` (Windows).
- GitHub Token: `~/.local/share/br.edu.ifes.researchhub/github_token` (POSIX `0600` permissions).
- No internet access required for standard offline curation.

---

## 🚀 Installation & Getting Started

Download compiled installers from the [official releases page](https://github.com/thiagodeps/research_hub/releases):
- `.deb`, `.rpm`, or `.AppImage` on Linux.
- `.exe` (NSIS) or `.msi` on Windows.
- `.dmg` or `.app` on macOS.

### Default Credentials
- **Username**: `admin@admin.com`
- **Password**: `admin123`
*(New accounts can be registered locally at `/register`)*

---

## 💻 Developer Guide

```bash
make build         # Install frontend dependencies (npm install)
make dev           # Run with hot reload (Tauri + Astro)
make test          # Full test suite: cargo test + vitest
make bundle        # Generate native desktop installers
make docker-bundle # Build Linux installers in an Ubuntu 24.04 container (output in dist/)
```

---

## 📜 Version History & Release Notes

Check out the complete changelog and version notes:
- 👉 [Release Notes (Patch Notes)](patch-notes.md)
