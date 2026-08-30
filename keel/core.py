"""Keel V0: deterministic execution of an already-authorized exact operation.

Keel does not decide authority. It consumes a Pulpo-issued permit snapshot,
validates exact execution bindings, records attempt state needed for restart
safety, and returns an execution receipt as evidence input for reconciliation.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import hashlib
import json
from typing import Callable, MutableMapping


class AttemptState(str, Enum):
    PREPARED = "PREPARED"
    TRANSMITTING = "TRANSMITTING"
    REPORTED_SUCCESS = "REPORTED_SUCCESS"
    REPORTED_FAILURE = "REPORTED_FAILURE"
    UNKNOWN = "UNKNOWN"


@dataclass(frozen=True)
class ExecutionObject:
    actor_id: str
    action: str
    target: str
    object_version: str
    payload: dict

    def canonical_hash(self) -> str:
        value = {
            "actor_id": self.actor_id,
            "action": self.action,
            "target": self.target,
            "object_version": self.object_version,
            "payload": self.payload,
        }
        raw = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
        return hashlib.sha256(raw).hexdigest()


@dataclass(frozen=True)
class PermitSnapshot:
    permit_id: str
    actor_id: str
    action: str
    target: str
    object_version: str
    execution_hash: str
    expires_at_ns: int
    revoked: bool = False
    authority_effect: str = "none"

    def __post_init__(self) -> None:
        if not self.permit_id or not self.execution_hash:
            raise ValueError("permit identity and exact execution hash are required")
        if self.authority_effect != "none":
            raise ValueError("Keel cannot consume authority-expanding permits")


@dataclass(frozen=True)
class ExecutionReceipt:
    permit_id: str
    execution_hash: str
    attempt_state: AttemptState
    provider_reference: str | None = None
    detail: str = ""
    reconciled: bool = False
    authority_effect: str = "none"

    def __post_init__(self) -> None:
        if self.reconciled:
            raise ValueError("Keel receipts cannot self-certify reconciliation")
        if self.authority_effect != "none":
            raise ValueError("execution receipts cannot carry authority")


class KeelExecutor:
    """One-shot executor with caller-supplied durable attempt storage.

    `state` must survive process restart in real deployments. V0 accepts a
    MutableMapping so restart semantics can be proven without creating a new
    canonical ledger.
    """

    def __init__(self, state: MutableMapping[str, str]):
        self._state = state

    @staticmethod
    def _validate(permit: PermitSnapshot, obj: ExecutionObject, now_ns: int) -> None:
        if permit.revoked:
            raise PermissionError("permit revoked")
        if now_ns > permit.expires_at_ns:
            raise PermissionError("permit expired")
        if not obj.actor_id or not obj.action or not obj.target or not obj.object_version:
            raise ValueError("execution object is incomplete")
        if permit.actor_id != obj.actor_id:
            raise PermissionError("actor substitution denied")
        if permit.action != obj.action:
            raise PermissionError("action substitution denied")
        if permit.target != obj.target:
            raise PermissionError("target substitution denied")
        if permit.object_version != obj.object_version:
            raise PermissionError("object version substitution denied")
        if permit.execution_hash != obj.canonical_hash():
            raise PermissionError("payload or execution object mutation denied")

    def execute_once(
        self,
        permit: PermitSnapshot | None,
        obj: ExecutionObject,
        *,
        now_ns: int,
        transmit: Callable[[ExecutionObject], str],
    ) -> ExecutionReceipt:
        if permit is None:
            raise PermissionError("Pulpo permit required")
        self._validate(permit, obj, now_ns)

        prior = self._state.get(permit.permit_id)
        if prior is not None:
            state = AttemptState(prior)
            if state in {AttemptState.TRANSMITTING, AttemptState.UNKNOWN}:
                return ExecutionReceipt(
                    permit.permit_id,
                    permit.execution_hash,
                    AttemptState.UNKNOWN,
                    detail="prior transmission outcome unknown; blind retry denied",
                )
            raise PermissionError("permit already consumed")

        # Persist before crossing the provider boundary. A crash after this
        # write cannot silently become a fresh attempt after restart.
        self._state[permit.permit_id] = AttemptState.TRANSMITTING.value
        try:
            provider_reference = transmit(obj)
        except TimeoutError:
            self._state[permit.permit_id] = AttemptState.UNKNOWN.value
            return ExecutionReceipt(
                permit.permit_id,
                permit.execution_hash,
                AttemptState.UNKNOWN,
                detail="provider response lost; independent observation required",
            )
        except Exception as exc:
            self._state[permit.permit_id] = AttemptState.REPORTED_FAILURE.value
            return ExecutionReceipt(
                permit.permit_id,
                permit.execution_hash,
                AttemptState.REPORTED_FAILURE,
                detail=f"provider reported failure: {type(exc).__name__}",
            )

        self._state[permit.permit_id] = AttemptState.REPORTED_SUCCESS.value
        return ExecutionReceipt(
            permit.permit_id,
            permit.execution_hash,
            AttemptState.REPORTED_SUCCESS,
            provider_reference=provider_reference,
            detail="provider reported success; Pulpo reconciliation still required",
        )
