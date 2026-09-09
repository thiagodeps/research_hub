# Implementation Plan: Release Actions

**Feature Branch**: `026-release`  
**Created**: 2026-09-09  
**Status**: Draft  

## Phase 1: GitHub Actions Workflow Update

1. **Modify `.github/workflows/release.yml`**:
   - Rename to standard `Release` if not already named correctly.
   - Combine the current logic into a single matrix strategy for macOS, Windows, and Linux.
   - Extract Linux system dependency installation logic to a conditional step (`if: matrix.os == 'ubuntu-latest'`).
   - Use `tauri-apps/tauri-action` for all three OSes in the build matrix.
   - Configure artifact uploads for all OS types on `workflow_dispatch`.

2. **Validation**:
   - Ensure YAML syntax is correct.
   - Ensure the correct Tauri dependencies and targets are specified for Linux and macOS.

## Phase 2: Documentation

1. **Update README.md** (if necessary):
   - Note the availability of the application for Linux, macOS, and Windows.
   - Note that binaries are unsigned and require bypass for Gatekeeper (macOS) and SmartScreen (Windows).
