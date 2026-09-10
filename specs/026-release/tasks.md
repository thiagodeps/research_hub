# Tasks: Release Actions

**Feature Branch**: `026-release`  
**Created**: 2026-09-09  

## Tasks

- [x] **Task 1**: Update `.github/workflows/release.yml` with the new matrix build strategy for macOS, Windows, and Linux.
  - [x] Implement OS matrix (`macos-latest`, `windows-latest`, `ubuntu-latest`).
  - [x] Add specific dependencies for Ubuntu (`libwebkit2gtk-4.1-dev`, etc.).
  - [x] Add Apple Silicon targets and universal targets for macOS.
  - [x] Configure `tauri-action` to build and upload release artifacts based on OS.
  
- [x] **Task 2**: Fix `apt-get update` Hash Sum mismatch on Ubuntu GitHub Action runners.
  - [x] Remove `google-chrome.list` before running `sudo apt-get update` in `.github/workflows/release.yml`.
