# Keel 1.0 Boundary Recovery

Status: proposed recovery doctrine. Historical Keel material is reference evidence, not canonical implementation.

## Recovered purpose

Keel is the deterministic execution substrate beneath Pulpo governance.

```text
Intelligence
    |
    v
Pulpo governance and authority
    |
    | exact bounded permit
    v
Keel deterministic execution
    |
    v
External systems
    |
    v
Independent evidence -> Pulpo reconciliation
```

## Constitutional separation

Pulpo owns:

- identity/authority evaluation;
- policy and budget decision;
- approval binding;
- exact consequence authorization;
- one-use permit semantics;
- canonical governance evidence and reconciliation;
- authority revocation and expiry.

Keel owns:

- deterministic execution of an already-authorized exact operation;
- execution adapters and provider connectivity;
- process/runtime supervision;
- health/readiness gates;
- transactional state transition where the target permits it;
- bounded retries only when Pulpo semantics permit them;
- rollback/recovery mechanics where meaningful;
- secret/provider credential custody needed only for execution;
- resource limits, isolation, and execution telemetry;
- execution receipts returned to Pulpo for reconciliation.

Keel does **not** own:

- human or machine authority;
- policy expansion;
- approval authority;
- a competing evidence ledger;
- a competing learning/memory authority;
- permission to broaden a Pulpo permit;
- permission to reinterpret an exact target into a different consequence;
- permission to retry a consequential operation after uncertain outcome unless the governing contract explicitly allows it.

## Recovered historical material

Historical Keel notebooks/documents describe useful infrastructure patterns including:

- provider-agnostic gateway/proxy behavior;
- payload normalization and provider adapters;
- circuit breakers and graceful degradation;
- async execution and connection management;
- multi-tenant credential isolation;
- health-gated container startup;
- telemetry and token/cost accounting;
- resource supervision;
- local/hybrid/sovereign deployment modes;
- pre-flight resource/budget reservation concepts;
- serialized long-horizon task suspension.

These are candidate capabilities, not verified Keel 1.0 features.

Historical material also assigns governance, routing, RAG, moral-policy, budget authority, and audit truth to Keel/Pulpo inconsistently. Those responsibilities are **not** imported wholesale. Current Pulpo constitutional boundaries supersede them.

## Clean Keel 1.0 contract

Keel should initially accept only a normalized execution request derived from a Pulpo-issued permit:

```json
{
  "permit_id": "opaque-id",
  "intent_hash": "sha256:...",
  "actor_id": "exact-actor",
  "action": "exact-action",
  "resource": "exact-resource",
  "object_version": "exact-version",
  "expires_at": "...",
  "execution_profile": "sandbox-v0"
}
```

Keel must validate the permit through the Pulpo-defined verification boundary before gaining a transmission right. Keel may not manufacture or expand any field.

Keel returns an execution receipt, not an authorization verdict:

```json
{
  "permit_id": "opaque-id",
  "attempt_id": "keel-attempt-id",
  "attempted": true,
  "provider_claim": "accepted|rejected|unknown",
  "provider_reference": "sanitized-reference",
  "started_at": "...",
  "finished_at": "..."
}
```

A Keel receipt is evidence input. It is not proof that the consequence actually occurred. Pulpo reconciliation must compare it with independent observation where consequence semantics require that.

## V0 proof

Do not rebuild the historical gateway.

Build one minimal Keel executor for one harmless sandbox effect and prove:

1. no Pulpo permit -> no execution;
2. valid exact permit -> one bounded attempt;
3. target substitution -> deny before provider transmission;
4. actor substitution -> deny;
5. expired/revoked permit -> deny;
6. replayed permit -> no second effect;
7. malformed execution object -> deny;
8. Keel crash before transmission -> restart without inventing success;
9. lost provider response -> unknown/reconciliation path, no blind consequential retry;
10. provider success claim without independent observation -> not reconciled success;
11. Keel cannot widen policy, budget, scope, credentials, or authority;
12. execution receipt is returned to Pulpo's canonical evidence/reconciliation path rather than a second ledger.

## Donovan / external OS comparison

A deterministic Agent Platform OS should be tested as a peer execution substrate, not as Pulpo authority.

Frozen comparison:

- valid Pulpo authority + valid execution invariants -> exactly one effect;
- invalid Pulpo authority + valid execution invariants -> no effect;
- valid Pulpo authority + invalid execution invariants -> no effect or deterministic rollback;
- invalid authority + invalid execution invariants -> no effect.

Then test replay, actor substitution, target substitution, stale authority, partial execution, false `done`, and uncertain provider response.

This tests whether Keel and an external deterministic OS solve the same execution-plane problem while remaining subordinate to independent consequence authority.

## Nonclaims

This document does not claim that historical Keel code exists, that historical architecture was secure, that the old notebooks are canonical, or that Keel 1.0 is implemented.

It freezes the recovered boundary so implementation can proceed without importing old architectural drift.
