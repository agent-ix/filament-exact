#!/usr/bin/env bash
# Run the Verus proofs of this crate (`make verus`).
#
#   VERUS=/path/to/verus scripts/verus.sh [SRC_DIR] [-- extra verus arguments]
#
# The proofs are the `cfg(verus_keep_ghost)` modules and attributes of the
# crate; no other build sees them. The script builds the crate's dependencies
# with the toolchain the Verus binary names, then runs Verus on `src/lib.rs`
# (or on `SRC_DIR/lib.rs`, which the mutant script uses). It succeeds only if
# Verus reports no error.
set -euo pipefail

PINNED="0.2026.11.02.8bf68ee"
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
  echo "verus.sh: no Verus binary at '$VERUS'; set VERUS to a $PINNED release" >&2
  exit 2
fi
info="$("$VERUS" --version)"
version="$(printf '%s\n' "$info" | sed -n 's/^ *Version: *//p')"
toolchain="$(printf '%s\n' "$info" | sed -n 's/^ *Toolchain: *\([^ ]*\).*/\1/p')"
if [ "$version" != "$PINNED" ]; then
  echo "verus.sh: Verus $version found, the proofs are checked with $PINNED" >&2
  exit 2
fi

# The dependencies, compiled by the compiler Verus itself drives.
TARGET="${VERUS_TARGET_DIR:-$ROOT/target-verus}"
(cd "$ROOT" && RUSTUP_TOOLCHAIN="$toolchain" cargo build --locked --lib --quiet --target-dir "$TARGET")
DEPS="$TARGET/debug/deps"
externs=()
for crate in allocator_api2 arrayvec num_bigint num_integer num_traits once_cell thiserror \
  triomphe spin unicode_normalization; do
  rlib="$(find "$DEPS" -name "lib${crate}-*.rlib" | head -n 1)"
  if [ -z "$rlib" ]; then
    echo "verus.sh: dependency $crate was not built in $DEPS" >&2
    exit 2
  fi
  externs+=(--extern "$crate=$rlib")
done

out="$("$VERUS" --crate-type lib --crate-name filament_exact --edition 2021 "$SRC/lib.rs" \
  -L "dependency=$DEPS" "${externs[@]}" --triggers-mode silent "$@" 2>&1)" && status=0 || status=$?
printf '%s\n' "$out" | grep -v '^warning: unexpected `cfg` condition' || true
if [ "$status" -ne 0 ] || ! printf '%s\n' "$out" | grep -q 'verification results:: [1-9][0-9]* verified, 0 errors'; then
  echo "verus.sh: FAILED" >&2
  exit 1
fi
echo "verus.sh: Verus $version, toolchain $toolchain: verified"
