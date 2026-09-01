# Keel 1.0 Boundary Recovery

Status: **Proposed recovery doctrine. Not an implementation claim.**

## Evidence basis

This recovery is distilled from historical Keel material held outside this repository, including the Repository Constitution / Architecture documents and Keel Gateway OS + Pulpo planning material. Historical documents contain useful mechanics but also collapse routing, retrieval, authorization, orchestration, governance, execution, and evidence into one system. Those collapsed responsibilities are **not** carried forward as canonical design.

## Recovered purpose

Keel is the deterministic execution substrate beneath Pulpo governance.

```text
Intelligence / agents
        |
        v
Pulpo governance and evidence plane
  exact intent -> authority -> policy -> decision -> one-use permit
        |
        v
Keel deterministic execution substrate
  validate permit contract -> stage capability -> execute bounded effect
  -> supervise state -> recover/rollback where possible -> report observation
        |
        v
External systems / host / provider
        |
        v
Independent evidence -> Pulpo reconciliation
```

## Constitutional separation

`INTELLIGENCE != AUTHORITY`

`PULPO != EXECUTOR`

`KEEL != AUTHORITY`

`EXECUTION SAFETY != AUTHORIZATION`

`VALID PAYLOAD != AUTHORIZED CONSEQUENCE`

`KEEL REPORT != ACCEPTED CONSEQUENCE`

`ROLLBACK != ERASE HISTORY`

Keel may refuse execution for execution-safety reasons. It may never create, widen, repair, infer, or substitute Pulpo authority.

## Pulpo owns

- exact intent identity and object binding;
- authenticated authority and human approval semantics;
- policy and budget decision;
- permit issuance, expiry, revocation and one-use semantics;
- canonical governance/audit evidence obligations;
- reconciliation of executor reports against independent observation;
- outcome learning and recommendations, without self-authority.

## Keel owns

- deterministic execution adapters;
- host/runtime capability staging after a valid Pulpo permit is presented;
- process supervision and bounded retries where the permit explicitly allows them;
- readiness and health gates;
- configuration/schema validation;
- resource limits and execution containment;
- provider/API translation required to perform the exact authorized object;
- transactional state transition where the target supports it;
- rollback/recovery where technically possible;
- drift detection for execution-owned state;
- execution receipts and sanitized observations returned to Pulpo;
- secrets custody required solely for execution, inaccessible to proposing intelligence.

## Keel explicitly does not own

- an independent authority service;
- a second policy engine that can authorize consequence;
- a second canonical evidence ledger;
- human approval semantics;
- authority expansion based on local success, model confidence, identity, RBAC, retrieval, or memory;
- autonomous substitution of target, action, budget, actor, provider consequence, or permit scope;
- a general agent orchestrator that bypasses Pulpo consequence admission.

## Historical material: retain, adapt, reject

### Retain / adapt

Historical Keel documents contain useful execution mechanics:

- provider-agnostic adapters and payload normalization;
- async I/O and connection pooling;
- circuit breakers and graceful degradation;
- streaming/backpressure handling;
- deterministic configuration and schema migrations;
- health/readiness gates;
- non-root container execution;
- secrets isolation;
- telemetry and cost measurement;
- bounded resource controls;
- state serialization for long waits;
- tool/payload validation.

These are execution capabilities only. Each must remain subordinate to an exact Pulpo permit when the operation is consequential.

### Reject / quarantine

Do not revive historical concepts that would duplicate Pulpo or weaken current constitutional boundaries:

- Keel as a moral/governance authority;
- Keel-side semantic intent authorization;
- Keel-issued authority based on RBAC, virtual API keys, quotas, or model routing;
- automatic provider substitution when substitution changes the authorized consequence;
- a separate authoritative audit/evidence truth;
- agent/tool orchestration that can cause consequence without Pulpo admission;
- claims of guaranteed cost recovery, perfect safety, or production readiness without executable evidence.

## Minimal Pulpo <-> Keel contract

Keel accepts a normalized execution request containing at minimum:

```text
permit_id
intent_hash
actor_id
exact_action
exact_resource
exact_object_hash
issued_at
expires_at
attempt_budget
pulpo_authority_profile
```

Keel must verify the contract through a pinned Pulpo verifier before capability staging. A valid signature alone is insufficient if the exact object, actor, expiry, revocation state, attempt budget, or execution profile does not match.

Keel returns a non-authoritative execution receipt containing at minimum:

```text
permit_id
attempt_id
exact_object_hash
started_at
finished_at
executor_identity
transport_result
provider_reference_if_any
sanitized_observation
```

Pulpo reconciles that receipt against independent evidence. Keel cannot mark its own consequence accepted or valuable.

## Keel 1.0 smallest executable proof

Do not rebuild the historical gateway first.

Build one disposable sandbox executor with one action: create an inert local record.

Frozen matrix:

1. valid exact Pulpo permit + valid execution object -> one effect;
2. no permit -> deny before capability staging;
3. malformed/invalid permit -> deny;
4. expired permit -> deny;
5. revoked permit -> deny;
6. actor substitution -> deny;
7. resource/target substitution -> deny;
8. payload mutation after authorization -> deny;
9. replay spent permit -> deny;
10. executor reports success but independent observer sees no effect -> Pulpo reconciliation mismatch;
11. execution fails mid-transition -> deterministic failure/rollback evidence;
12. restart -> spent/failed state cannot become executable again.

Then ablate Pulpo and Keel independently to prove the separation:

- Pulpo without Keel: authorized but unsafe/unavailable execution must not become accepted consequence.
- Keel without Pulpo: technically executable operation must not gain authority.

## Relationship to external Agent Platform OS experiments

A third-party deterministic Agent Platform OS should be evaluated as a **Keel-class execution substrate**, not as Pulpo authority.

The comparison question is:

> Given the same frozen Pulpo permit contract, can Keel and an external deterministic OS independently enforce execution invariants without either system manufacturing authority?

That creates a clean cross-validation surface rather than a product-architecture collision.

## Admission rule

This document recovers and narrows doctrine only. It does not authorize implementation, merge, deployment, credentials, external effects, or migration of historical Keel code.

Before Keel becomes canonical again:

1. inventory remaining historical Gemini / Notebook / Drive artifacts;
2. classify each claim as Recorded, Verified, Inferred, Proposed, or Rejected;
3. map useful mechanics against current Pulpo proofs so no responsibility is duplicated;
4. implement only the minimal sandbox executor contract;
5. prove success, denial, replay, restart, substitution, failure and reconciliation behavior;
6. obtain independent exact-head review.
