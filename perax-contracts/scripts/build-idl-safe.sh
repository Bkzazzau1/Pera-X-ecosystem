#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

PROGRAM_DIR="programs/perax-core"
PROGRAM_NAME="perax_core"
OUT_JSON="target/idl/${PROGRAM_NAME}.json"
OUT_TS="target/types/${PROGRAM_NAME}.ts"
RAW_OUT="$(mktemp)"

cleanup() {
  rm -f "$RAW_OUT"
}
trap cleanup EXIT

mkdir -p target/idl target/types

(
  cd "$PROGRAM_DIR"
  ANCHOR_IDL_BUILD_NO_DOCS=FALSE \
  ANCHOR_IDL_BUILD_RESOLUTION=TRUE \
  ANCHOR_IDL_BUILD_SKIP_LINT=FALSE \
  ANCHOR_IDL_BUILD_PROGRAM_PATH="$(pwd)" \
  RUSTFLAGS="-A warnings" \
  cargo +1.85.0 test __anchor_private_print_idl \
    --features idl-build \
    -- \
    --show-output \
    --quiet
) >"$RAW_OUT" 2>&1 || {
  cat "$RAW_OUT"
  exit 1
}

python3 - "$RAW_OUT" "$OUT_JSON" <<'PY'
import json
import re
import sys

raw_path, out_path = sys.argv[1], sys.argv[2]
text = open(raw_path, encoding="utf-8").read()

address_match = re.search(
    r"--- IDL begin address ---\n(.*?)\n--- IDL end address ---",
    text,
    re.S,
)
program_match = re.search(
    r"--- IDL begin program ---\n(.*?)\n--- IDL end program ---",
    text,
    re.S,
)

if not address_match or not program_match:
    raise SystemExit(f"Could not find Anchor IDL markers in build output:\n{text}")

address = json.loads(json.loads(address_match.group(1).strip()))
idl = json.loads(program_match.group(1))
idl["address"] = address

with open(out_path, "w", encoding="utf-8") as output:
    json.dump(idl, output, indent=2)
    output.write("\n")
PY

anchor idl type "$OUT_JSON" -o "$OUT_TS"
printf 'Generated %s and %s\n' "$OUT_JSON" "$OUT_TS"
