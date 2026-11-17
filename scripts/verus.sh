#!/usr/bin/env bash
# Run the Verus proofs of this crate (`make verus`).
#
#   VERUS=/path/to/verus scripts/verus.sh [SRC_DIR] [-- extra verus arguments]
#
# The proofs are the `cfg(verus_keep_ghost)` modules and attributes of the
# crate; no other build sees them. The script builds the crate's dependencies
# with the toolchain the Verus binary names, then runs Verus on `src/lib.rs`
# (or on `SRC_DIR/lib.rs`, which the mutant scripts use). It succeeds only if
# Verus reports no error.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERUS="${VERUS:-verus}"
SRC="$ROOT/src"
if [ $# -gt 0 ] && [ "$1" != "--" ]; then
  SRC="$1"
  shift
fi
if [ $# -gt 0 ] && [ "$1" = "--" ]; then
  shift
fi

if ! command -v "$VERUS" >/dev/null 2>&1; then
  echo "verus.sh: no Verus binary at '$VERUS'; set VERUS to one" >&2
  exit 2
fi
info="$("$VERUS" --version)"
version="$(printf '%s\n' "$info" | sed -n 's/^ *Version: *//p')"
toolchain="$(printf '%s\n' "$info" | sed -n 's/^ *Toolchain: *\([^ ]*\).*/\1/p')"

# The dependencies, compiled by the compiler Verus itself drives. The extern
# list is what cargo reports: the library's direct dependencies, each with the
# artifact this build produced for it.
TARGET="${VERUS_TARGET_DIR:-$ROOT/target-verus}"
externs=()
while IFS= read -r pair; do
  externs+=(--extern "$pair")
done < <(
  cd "$ROOT"
  export RUSTUP_TOOLCHAIN="$toolchain"
  metadata="$(mktemp)"
  cargo metadata --locked --format-version 1 >"$metadata"
  cargo build --locked --lib --quiet --target-dir "$TARGET" --message-format=json |
    python3 -c '
import json, sys

metadata = json.load(open(sys.argv[1]))
root = metadata["resolve"]["root"]
node = next(n for n in metadata["resolve"]["nodes"] if n["id"] == root)
# Normal dependencies only: the library does not link dev or build ones.
wanted = {
    dep["pkg"]: dep["name"]
    for dep in node["deps"]
    if any(kind["kind"] is None for kind in dep["dep_kinds"])
}
artifacts = {}
for line in sys.stdin:
    try:
        message = json.loads(line)
    except ValueError:
        continue
    if message.get("reason") != "compiler-artifact" or message["package_id"] not in wanted:
        continue
    kinds = message["target"]["kind"]
    if "lib" not in kinds and "rlib" not in kinds and "proc-macro" not in kinds:
        continue
    files = [f for f in message["filenames"] if f.endswith((".rlib", ".so", ".dylib", ".dll"))]
    if files:
        artifacts[message["package_id"]] = files[0]
missing = [name for pkg, name in wanted.items() if pkg not in artifacts]
if missing:
    sys.exit("verus.sh: cargo built no artifact for " + ", ".join(sorted(missing)))
for pkg, name in sorted(wanted.items(), key=lambda item: item[1]):
    print(name + "=" + artifacts[pkg])
' "$metadata"
  rm -f "$metadata"
)
if [ "${#externs[@]}" -eq 0 ]; then
  echo "verus.sh: could not list the crate's dependencies" >&2
  exit 2
fi

out="$("$VERUS" --crate-type lib --crate-name filament_exact --edition 2021 "$SRC/lib.rs" \
  -L "dependency=$TARGET/debug/deps" "${externs[@]}" --triggers-mode silent "$@" 2>&1)" && status=0 || status=$?
printf '%s\n' "$out" | grep -v '^warning: unexpected `cfg` condition' || true
if [ "$status" -ne 0 ] || ! printf '%s\n' "$out" | grep -q 'verification results:: [1-9][0-9]* verified, 0 errors'; then
  echo "verus.sh: FAILED" >&2
  exit 1
fi
echo "verus.sh: Verus $version, toolchain $toolchain: verified"
