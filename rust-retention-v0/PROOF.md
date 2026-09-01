# Rust Retention Executor V0 — Frozen Proof Contract

Status: **Experimental / noncanonical / do not deploy as production authority.**

Base lineage: `recovery/keel1.0-boundary` at `07b975c6e7e6a7b65b1b1d2f36673ec7d6636bc5`.

## Purpose

Test a narrow Rust/Axum Keel-class execution seam for an already-authorized evidence-deletion consequence without moving Pulpo's authority boundary.

This experiment does **not** decide retention policy, issue permits, approve deletion, create canonical Pulpo evidence, or claim production key custody.

## Frozen invariants

1. No deletion occurs without an explicit deletion authorization object.
2. The authorization must bind the exact `permit_id`, actor, action, evidence identifier, evidence hash, object hash, expiry, and policy hash.
3. Path identifier substitution is denied.
4. Evidence-hash substitution is denied.
5. Object-hash substitution is denied.
6. Expired authorization is denied.
7. A spent permit cannot execute a second deletion within the same executor state.
8. A failed mismatch must leave evidence bytes present.
9. A successful deletion must remove the exact in-memory evidence object before returning `DeletionExecuted`.
10. The returned manifest binds the deletion request/execution event hashes into a Merkle root.
11. The manifest is execution evidence only; it cannot self-certify Pulpo reconciliation.
12. SHA-256/Merkle integrity does not prove physical media erasure, secret zeroization, production containment, or third-party reproduction.

## Negative proof target

A malicious request that presents a valid-shaped authorization but substitutes any of the exact evidence identifier, evidence hash, or authorization object hash must fail closed and leave the evidence object untouched.

## Explicit nonclaims

- Rust does not by itself prove secret zeroization.
- This V0 contains no master-key custody and therefore cannot prove the claimed Python `C-Layer Replica Trap` eliminated in production.
- This V0 does not establish the claimed `15 million monthly tokens` performance threshold.
- The event/manifest state is not a second canonical ledger.
- A successful local/CI test is software-boundary evidence, not external containment or cold reproduction.
- Repository admission into Keel `main` remains separately governed and currently unresolved.
