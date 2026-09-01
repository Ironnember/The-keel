import unittest

from keel.core import AttemptState, ExecutionObject, KeelExecutor, PermitSnapshot


class KeelV0Tests(unittest.TestCase):
    def setUp(self):
        self.state = {}
        self.keel = KeelExecutor(self.state)
        self.obj = ExecutionObject(
            actor_id="agent:test-1",
            action="create_record",
            target="sandbox:record-001",
            object_version="v1",
            payload={"message": "boundary-test"},
        )
        self.permit = PermitSnapshot(
            permit_id="permit-001",
            actor_id=self.obj.actor_id,
            action=self.obj.action,
            target=self.obj.target,
            object_version=self.obj.object_version,
            execution_hash=self.obj.canonical_hash(),
            expires_at_ns=200,
        )
        self.effects = []

    def transmit(self, obj):
        self.effects.append(obj.canonical_hash())
        return "provider:effect-001"

    def test_no_permit_denies_without_effect(self):
        with self.assertRaises(PermissionError):
            self.keel.execute_once(None, self.obj, now_ns=100, transmit=self.transmit)
        self.assertEqual(self.effects, [])

    def test_exact_permit_executes_once_but_does_not_reconcile_itself(self):
        receipt = self.keel.execute_once(self.permit, self.obj, now_ns=100, transmit=self.transmit)
        self.assertEqual(receipt.attempt_state, AttemptState.REPORTED_SUCCESS)
        self.assertFalse(receipt.reconciled)
        self.assertEqual(receipt.authority_effect, "none")
        self.assertEqual(len(self.effects), 1)
        with self.assertRaises(PermissionError):
            self.keel.execute_once(self.permit, self.obj, now_ns=101, transmit=self.transmit)
        self.assertEqual(len(self.effects), 1)

    def test_actor_target_version_and_payload_substitution_deny(self):
        mutations = [
            ExecutionObject("agent:evil", self.obj.action, self.obj.target, "v1", self.obj.payload),
            ExecutionObject(self.obj.actor_id, self.obj.action, "sandbox:record-999", "v1", self.obj.payload),
            ExecutionObject(self.obj.actor_id, self.obj.action, self.obj.target, "v2", self.obj.payload),
            ExecutionObject(self.obj.actor_id, self.obj.action, self.obj.target, "v1", {"message": "mutated"}),
        ]
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                with self.assertRaises(PermissionError):
                    self.keel.execute_once(self.permit, mutation, now_ns=100, transmit=self.transmit)
        self.assertEqual(self.effects, [])

    def test_expiry_and_revocation_deny(self):
        with self.assertRaises(PermissionError):
            self.keel.execute_once(self.permit, self.obj, now_ns=201, transmit=self.transmit)
        revoked = PermitSnapshot(**{**self.permit.__dict__, "permit_id": "permit-revoked", "revoked": True})
        with self.assertRaises(PermissionError):
            self.keel.execute_once(revoked, self.obj, now_ns=100, transmit=self.transmit)
        self.assertEqual(self.effects, [])

    def test_restart_after_transmission_started_never_blind_retries(self):
        self.state[self.permit.permit_id] = AttemptState.TRANSMITTING.value
        restarted = KeelExecutor(self.state)
        receipt = restarted.execute_once(self.permit, self.obj, now_ns=100, transmit=self.transmit)
        self.assertEqual(receipt.attempt_state, AttemptState.UNKNOWN)
        self.assertEqual(self.effects, [])

    def test_lost_provider_response_is_unknown_and_survives_restart(self):
        calls = []
        def timeout(_obj):
            calls.append(1)
            raise TimeoutError("response lost")
        first = self.keel.execute_once(self.permit, self.obj, now_ns=100, transmit=timeout)
        self.assertEqual(first.attempt_state, AttemptState.UNKNOWN)
        restarted = KeelExecutor(self.state)
        second = restarted.execute_once(self.permit, self.obj, now_ns=101, transmit=self.transmit)
        self.assertEqual(second.attempt_state, AttemptState.UNKNOWN)
        self.assertEqual(len(calls), 1)
        self.assertEqual(self.effects, [])

    def test_reported_failure_consumes_attempt_without_retry(self):
        def fail(_obj):
            raise RuntimeError("provider rejected")
        receipt = self.keel.execute_once(self.permit, self.obj, now_ns=100, transmit=fail)
        self.assertEqual(receipt.attempt_state, AttemptState.REPORTED_FAILURE)
        with self.assertRaises(PermissionError):
            self.keel.execute_once(self.permit, self.obj, now_ns=101, transmit=self.transmit)
        self.assertEqual(self.effects, [])

    def test_keel_rejects_authority_expansion(self):
        with self.assertRaises(ValueError):
            PermitSnapshot(
                permit_id="bad",
                actor_id=self.obj.actor_id,
                action=self.obj.action,
                target=self.obj.target,
                object_version=self.obj.object_version,
                execution_hash=self.obj.canonical_hash(),
                expires_at_ns=200,
                authority_effect="expand",
            )


if __name__ == "__main__":
    unittest.main()
