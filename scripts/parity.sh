#!/usr/bin/env bash
# Round-trip verification (SEP-022, updated by SEP-024).
#
# The Python oracle was removed with the backend. What replaced it is the
# round-trip test itself, which asserts against the reference archive:
# 626 entries out, 596 preserved untouched, the nested 22.6 MB archive
# byte-identical, and every one of the 15 tables restored to its original
# column types and order.
#
# Byte comparison of the whole package was never a valid criterion —
# compression settings and writer metadata differ legitimately between
# implementations. Schema plus values is the real contract.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/src-tauri"
exec cargo test --lib -- --nocapture \
  reference_archive_matches_python_row_counts \
  real_archive_round_trip_preserves_schema_and_entries
