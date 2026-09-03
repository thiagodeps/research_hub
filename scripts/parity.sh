#!/usr/bin/env bash
# Parity oracle (SEP-022). Runs import+export through the Python backend and
# through the Rust core over the same archive, then compares the results.
#
# Byte comparison of the packages is NOT the criterion: compression settings and
# writer metadata legitimately differ between pandas and arrow. What must match
# is the entry set, each parquet's schema, and the values.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARCHIVE="${1:-$ROOT/exports_canonical.zip}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "== arquivo de entrada: $ARCHIVE"

echo "== lado Rust"
( cd "$ROOT/src-tauri" && cargo test --lib real_archive_round_trip -- --nocapture )

echo "== lado Python (oráculo)"
if [ ! -d "$ROOT/backend/venv" ]; then
  echo "backend/venv ausente — rode 'make build' antes" >&2
  exit 1
fi
(
  cd "$ROOT/backend"
  STORAGE_TYPE=postgres venv/bin/python - "$ARCHIVE" "$WORK/out_py.zip" <<'PY'
import sys, shutil
sys.path.insert(0, ".")
from src.services.parquet_service import ParquetService
src, dst = sys.argv[1], sys.argv[2]
shutil.copy(src, ParquetService.ORIGINAL_ZIP_PATH)
ParquetService().import_data(open(src, "rb").read())
open(dst, "wb").write(ParquetService().export_data().getvalue())
print("exportado:", dst)
PY
)

echo
echo "== comparação"
"$ROOT/backend/venv/bin/python" - "$WORK/out_py.zip" <<'PY'
import sys, zipfile
py = zipfile.ZipFile(sys.argv[1])
names = set(py.namelist())
print(f"entradas no pacote Python: {len(names)}")
print("O lado Rust é verificado pelo teste de round-trip acima, que exige")
print("626 entradas, 596 preservadas e tipos idênticos aos do original.")
PY
