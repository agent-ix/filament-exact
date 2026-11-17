// SPDX-License-Identifier: AGPL-3.0-or-later
//! Executable binding for the Verus proofs of this crate (docs/verus-proofs.md).
//!
//! The tests run real Verus through `scripts/verus.sh`; they are ignored by
//! default and need `VERUS` to name a Verus binary. `make verus` and
//! `make verus-mutants` run the same scripts.

use ix_trace_rs::trace;
use std::process::Command;

fn script(path: &str) -> Command {
    assert!(
        std::env::var_os("VERUS").is_some(),
        "set VERUS to a Verus binary to run this gate"
    );
    let mut command = Command::new(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")));
    command.current_dir(env!("CARGO_MANIFEST_DIR"));
    command
}

/// Every proof verifies: Verus reports no error on the production source.
#[trace("TC-956", "FR-408-AC-13")]
#[trace("TC-957", "FR-409-AC-11")]
#[trace("TC-953", "FR-405-AC-17")]
#[test]
#[ignore = "runs Verus (a few seconds); set VERUS and run with `make verus`"]
fn verus_proofs_verify() {
    let output = script("scripts/verus.sh").output().expect("start verus.sh");
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && log.contains(" verified, 0 errors"),
        "the Verus proofs do not verify:\n{log}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Every recorded mutant fails a proof with a failed proof obligation; a run
/// that only exhausts the solver's resource limit fails the script.
#[trace("TC-956", "FR-408-AC-14")]
#[trace("TC-957", "FR-409-AC-12")]
#[trace("TC-953", "FR-405-AC-18")]
#[test]
#[ignore = "runs Verus once per mutant (a few minutes); set VERUS and run with `make verus-mutants`"]
fn verus_mutants_are_killed() {
    for mutants in [
        "scripts/ir1544_verus_mutants.py",
        "scripts/ir1556_verus_mutants.py",
        "scripts/ir1551_verus_mutants.py",
    ] {
        let output = script(mutants).output().expect("start the mutant script");
        let log = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() && log.contains("all mutants killed"),
            "{mutants} left a mutant alive or unreliable:\n{log}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
