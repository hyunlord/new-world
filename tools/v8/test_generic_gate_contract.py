# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_gate_contract.py
"""Pure Stage0 contract boundaries; synthetic evidence is not runtime admission.

Synthetic inputs exercise rejection boundaries, not source authentication or worlds.
Raw JSON fixture dictionaries are deliberately mutable only inside individual tests.
"""
from __future__ import annotations

import json
import unittest
from collections.abc import Callable, Sequence
from copy import deepcopy
from typing import Final, TypeAlias

from generic_gate import (
    GateContractError,
    finalize_publication,
    validate_binding,
    validate_coverage,
    validate_declared_fixture,
    validate_ordinary_safety,
)
from mux2_gate_inputs import Node

Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]
RawDocument: TypeAlias = dict[str, Json]
B: Final = "b" * 40
H: Final = "1" * 40
D: Final = "d" * 40
A: Final = "a" * 40
U: Final = "e" * 40
INPUT: Final = "2" * 64
BINARY: Final = "3" * 64


def json_cases(rows: Sequence[tuple[str, Json]]) -> Sequence[tuple[str, Json]]:
    return rows


def json_values(rows: Sequence[Json]) -> Sequence[Json]:
    return rows


def mapping(value: Json) -> RawDocument:
    return Node(value, "/fixture").mapping()


def array(value: Json) -> list[Json]:
    match value:
        case list() as rows:
            return rows
        case _:
            raise AssertionError("fixture must be an array")


def safety() -> RawDocument:
    return {"classification": "ordinary", "complete": True, "initial_tick": 0,
            "end_tick": 14400, "planned_end_tick": 14400, "founders": 10,
            "founder_census_tick": 0, "base_alive": 10, "candidate_alive": 8,
            "lost": [], "base_lost": [], "prior_loss_history_complete": True,
            "base_prior_loss_history_complete": True}


def binding() -> RawDocument:
    arms: dict[str, Json] = {name: {"source": B, "build_source": B, "binary_digest": BINARY,
                   "input_digest": INPUT, "output_root": f"/synthetic/{name}"}
            for name in ("base", "candidate")}
    return {"purpose": "self_proof", "harness": H,
            "products": {"base": B, "candidate": B}, "frozen_main": B,
            "merge_bases": [B], "clean": True, "frozen_source_matches": True,
            "declarations": [], "receipts": arms,
            "expected_binary_digests": {"base": BINARY, "candidate": BINARY}}


def declared_fixture() -> RawDocument:
    return {"purpose": "publication", "base": B, "candidate": A,
            "file_committed": True, "file_commit": A,
            "candidate_ancestors_after_base": [H, D, A],
            "known_fixtures": ["colony_u", "core_steward_off"],
            "declarations": [{"fixture_id": "colony_u", "base": B,
                              "behavior_commit": D, "reason": "Intentional synthetic change"}],
            "fixture_id": "colony_u", "semantic_equal": False, "safety": safety()}


def occurrence(arm: str, seed: int) -> RawDocument:
    return {"fixture_id": "colony_u", "arm": arm, "seed": seed, "repeat": 0,
            "role": "primary", "instance": "initial", "input_digest": INPUT,
            "initial_tick": 0, "end_tick": 14400,
            "terminal_status": "COMPLETE", "exit_code": 0}


def final_input() -> RawDocument:
    return {"purpose": "publication", "harness": A, "candidate": A,
            "frozen_source_matches": True, "required_steps": [10, 25, 41, 50],
            "steps": [{"step": n, "source": A, "status": "PASS"}
                      for n in (10, 25, 50)],
            "stall": {"step": 41, "source": A, "status": "WAIVER_ELIGIBLE",
                      "authenticated_complete_capture": True,
                      "ordinary_safety_complete": True}}


class GenericGateContracts(unittest.TestCase):
    def test_raw_json_boundary_rejects_duplicate_nonfinite_and_nonobject_inputs(self) -> None:
        for raw in ('{"purpose":"self_proof","purpose":"publication"}',
                    '{"unused":1e10000}', '{"unused":NaN}', '[]', 'null', '{'):
            with self.subTest(raw=raw), self.assertRaises(GateContractError):
                validate_binding(raw)

    def test_declaration_schema_cannot_carry_a_safety_override(self) -> None:
        raw = declared_fixture()
        raw["declarations"] = [{"fixture_id": "colony_u", "base": B,
            "behavior_commit": D, "reason": "Intentional synthetic change", "skip_safety": True}]
        self.rejected(validate_declared_fixture, raw)

    def test_missing_unknown_purpose_or_empty_final_registry_rejected(self) -> None:
        for value in (None, "unknown"):
            raw = binding()
            raw["purpose"] = value
            self.rejected(validate_binding, raw)
        raw = final_input()
        raw["required_steps"] = []
        self.rejected(finalize_publication, raw)

    def rejected(self, check: Callable[[str], None | str], raw: RawDocument) -> None:
        # When: parse and validate a deliberately invalid boundary document.
        with self.assertRaises(GateContractError) as caught:
            _ = check(json.dumps(raw))
        # Then: rejection includes actionable machine and human context.
        self.assertTrue(caught.exception.code)
        self.assertTrue(caught.exception.path)
        self.assertTrue(str(caught.exception).strip())

    def test_self_binding_accepts_independent_outputs_with_identical_binary_bytes(self) -> None:
        # Given: authenticated-input assumptions only; these are synthetic receipts.
        raw = binding()
        # When/Then: a valid binding subcheck is not publication authorization.
        self.assertIsNone(validate_binding(json.dumps(raw)))

    def test_binding_rejects_drift_dirty_or_ambiguous_merge_base(self) -> None:
        for key, value in json_cases((("clean", False), ("clean", 1),
                           ("frozen_source_matches", False), ("merge_bases", []),
                           ("merge_bases", [B, H]), ("frozen_main", U))):
            with self.subTest(key=key, value=value):
                raw = binding()  # Given
                raw[key] = value
                self.rejected(validate_binding, raw)  # When/Then

    def test_binding_rejects_harness_product_wrong_build_hash_input_or_shared_output(self) -> None:
        for key, value in json_cases((("source", H), ("build_source", H),
                           ("binary_digest", "4" * 64), ("input_digest", "5" * 64),
                           ("output_root", "/synthetic/base"))):
            with self.subTest(key=key):
                raw = binding()
                receipts: dict[str, Json] = {name: {"source": B, "build_source": B, "binary_digest": BINARY,
                                   "input_digest": INPUT, "output_root": f"/synthetic/{name}"}
                            for name in ("base", "candidate")}
                mapping(receipts["candidate"])[key] = value
                raw["receipts"] = receipts
                self.rejected(validate_binding, raw)

    def test_relative_survival_boundaries(self) -> None:
        for founders, base, candidate, accepted in ((10, 10, 8, True), (10, 10, 7, False),
                (10, 6, 6, True), (10, 6, 5, False), (1, 0, 0, True)):
            with self.subTest(founders=founders, base=base, candidate=candidate):
                raw = safety()
                raw.update(founders=founders, base_alive=base, candidate_alive=candidate)
                if accepted:
                    self.assertIsNone(validate_ordinary_safety(json.dumps(raw)))
                else:
                    self.rejected(validate_ordinary_safety, raw)

    def test_safety_rejects_missing_wrong_type_or_negative_integer(self) -> None:
        fields = ("founders", "base_alive", "candidate_alive", "initial_tick",
                  "end_tick", "planned_end_tick", "founder_census_tick")
        for field in fields:
            for bad in json_values((None, True, False, -1, 1.5, float("nan"), float("inf"))):
                with self.subTest(field=field, bad=bad):
                    raw = safety()
                    raw[field] = bad
                    self.rejected(validate_ordinary_safety, raw)
            raw = safety()
            del raw[field]
            self.rejected(validate_ordinary_safety, raw)

    def test_safety_rejects_missing_or_malformed_loss_observation_in_either_arm(self) -> None:
        for field in ("lost", "base_lost"):
            for bad in json_values((None, False, {}, "[]", 1, [[900]], [[True, "unit_knowledge", 1]])):
                with self.subTest(field=field, bad=bad):
                    raw = safety()
                    raw[field] = bad
                    self.rejected(validate_ordinary_safety, raw)
            raw = safety()
            del raw[field]
            self.rejected(validate_ordinary_safety, raw)

    def test_repaired_candidate_accepts_well_formed_nonempty_base_loss(self) -> None:
        raw = safety()  # Given: candidate has complete empty loss history and sufficient survivors.
        raw.update(base_alive=0, end_tick=28800, planned_end_tick=28800)
        raw["base_lost"] = [[28560, "clay_digging", 9]]
        self.assertIsNone(validate_ordinary_safety(json.dumps(raw)))  # When/Then: reference loss is not candidate loss.

    def test_native_optional_holder_is_not_missing_loss_history(self) -> None:
        raw = safety()
        raw["base_lost"] = [[20, "unit_knowledge", None]]
        self.assertIsNone(validate_ordinary_safety(json.dumps(raw)))
        raw["lost"] = [[20, "unit_knowledge", None]]
        self.rejected(validate_ordinary_safety, raw)

    def test_equal_base_core_off_loss_cannot_be_waived(self) -> None:
        # Given: synthetic reuse of vector07's documented loss, not a new world run.
        raw = safety()
        raw.update(base_alive=0, candidate_alive=0, end_tick=28800, planned_end_tick=28800,
                   lost=[[28560, "clay_digging", 9]], base_lost=[[28560, "clay_digging", 9]])
        self.rejected(validate_ordinary_safety, raw)  # Equality never waives ordinary loss.

    def test_safety_rejects_erased_history_wrong_horizon_or_control_reclassification(self) -> None:
        for key, value in json_cases((("prior_loss_history_complete", False), ("base_prior_loss_history_complete", False), ("complete", False),
                ("complete", 1), ("founder_census_tick", 1), ("end_tick", 14399),
                ("classification", "expected_injected_control"), ("skip_safety", True))):
            with self.subTest(key=key):
                raw = safety()
                raw[key] = value
                self.rejected(validate_ordinary_safety, raw)
        for field in ("prior_loss_history_complete", "base_prior_loss_history_complete"):
            raw = safety()
            del raw[field]
            self.rejected(validate_ordinary_safety, raw)

    def test_valid_declaration_admits_only_its_semantic_difference(self) -> None:
        self.assertIsNone(validate_declared_fixture(json.dumps(declared_fixture())))

    def test_equal_safe_fixture_needs_no_declaration(self) -> None:
        raw = declared_fixture()
        raw.update(semantic_equal=True, declarations=[])
        self.assertIsNone(validate_declared_fixture(json.dumps(raw)))

    def test_declaration_cannot_waive_survival_or_knowledge_loss(self) -> None:
        for key, value in json_cases((("candidate_alive", 7), ("lost", [[20, "unit_knowledge", 1]]))):
            with self.subTest(key=key):
                raw, observed = declared_fixture(), safety()
                observed[key] = value
                raw["safety"] = observed
                self.rejected(validate_declared_fixture, raw)

    def test_declaration_rejects_invalid_history_scope_and_reason(self) -> None:
        for key, value in json_cases((("fixture_id", "unknown"), ("reason", "  "),
                           ("behavior_commit", U), ("behavior_commit", "abc1234"), ("base", U))):
            with self.subTest(key=key):
                raw = declared_fixture()
                entry: RawDocument = {"fixture_id": "colony_u", "base": B, "behavior_commit": D,
                         "reason": "Intentional synthetic change"}
                entry[key] = value
                raw["declarations"] = [entry]
                self.rejected(validate_declared_fixture, raw)

    def test_declaration_rejects_duplicates_uncommitted_file_and_proof_purposes(self) -> None:
        for key, value in json_cases((("file_committed", False), ("file_commit", B),
                           ("purpose", "self_proof"), ("purpose", "defect_proof"))):
            raw = declared_fixture()
            raw[key] = value
            self.rejected(validate_declared_fixture, raw)
        raw = declared_fixture()
        entry: RawDocument = {"fixture_id": "colony_u", "base": B, "behavior_commit": D,
                 "reason": "Intentional synthetic change"}
        raw["declarations"] = [entry, deepcopy(entry)]
        self.rejected(validate_declared_fixture, raw)

    def test_registered_baseline_or_f10_cannot_be_declaration_exempt(self) -> None:
        for fixture in ("baseline_identity", "f10_expected_control", "identity.baseline_samples",
                        "identity.baseline_twins", "determinism.baseline", "core.baseline10",
                        "knowledge.f10_injected"):
            raw = declared_fixture()
            raw.update(known_fixtures=[fixture], fixture_id=fixture)
            raw["declarations"] = [{"fixture_id": fixture, "base": B,
                                    "behavior_commit": D, "reason": "Forbidden exemption"}]
            self.rejected(validate_declared_fixture, raw)

    def test_undeclared_semantic_difference_is_rejected(self) -> None:
        raw = declared_fixture()
        raw["declarations"] = []
        self.rejected(validate_declared_fixture, raw)

    def test_coverage_accepts_exact_arm_seed_set(self) -> None:
        rows: list[Json] = [occurrence(arm, seed) for arm in ("base", "candidate") for seed in range(4)]
        raw = {"planned": rows, "observed": deepcopy(rows), "constructor_audit_complete": True}
        self.assertIsNone(validate_coverage(json.dumps(raw)))

    def test_coverage_rejects_duplicate_replacing_missing_and_changed_identity(self) -> None:
        rows: list[Json] = [occurrence(arm, seed) for arm in ("base", "candidate") for seed in range(4)]
        for key, value in json_cases((("seed", 2), ("seed", True), ("repeat", False), ("repeat", 1),
                ("role", "replay"), ("instance", "loaded"), ("end_tick", 14399),
                ("input_digest", "6" * 64), ("terminal_status", "NOT_RUN"), ("exit_code", 1))):
            observed = deepcopy(rows)
            mapping(observed[-1])[key] = value
            self.rejected(validate_coverage, {"planned": rows, "observed": observed,
                                             "constructor_audit_complete": True})
        for observed in (rows[:-1], rows + [occurrence("base", 4)]):
            self.rejected(validate_coverage, {"planned": rows, "observed": observed,
                                             "constructor_audit_complete": True})
        self.rejected(validate_coverage, {"planned": rows, "observed": rows,
                                         "constructor_audit_complete": False})

    def test_complete_conjunction_accepts_only_provisional_waiver_plus_other_passes(self) -> None:
        self.assertEqual(finalize_publication(json.dumps(final_input())), "PASS_WITH_WAIVER")

    def test_all_strict_steps_pass_without_waiver(self) -> None:
        raw = final_input()
        raw["steps"] = [{"step": n, "source": A, "status": "PASS"} for n in (10, 25, 41, 50)]
        raw["stall"] = None
        self.assertEqual(finalize_publication(json.dumps(raw)), "PASS")

    def test_finalizer_rejects_self_proof_even_with_green_steps(self) -> None:
        raw = final_input()
        raw.update(purpose="self_proof", harness=H, candidate=B)
        raw["steps"] = [{"step": n, "source": B, "status": "PASS"} for n in (10, 25, 41, 50)]
        raw["stall"] = None
        self.rejected(finalize_publication, raw)

    def test_waiver_rejects_failed_missing_duplicate_stale_or_not_run_step(self) -> None:
        valid: list[Json] = [{"step": n, "source": A, "status": "PASS"} for n in (10, 25, 50)]
        for key, value in json_cases((("status", "FAIL"), ("status", "NOT_RUN"), ("source", B))):
            rows = deepcopy(valid)
            mapping(rows[1])[key] = value
            raw = final_input()
            raw["steps"] = rows
            self.rejected(finalize_publication, raw)
        for rows in json_values((valid[:-1], valid + [deepcopy(valid[1])])):
            raw = final_input()
            raw["steps"] = rows
            self.rejected(finalize_publication, raw)
        for key, value in json_cases((("frozen_source_matches", False), ("harness", H))):
            raw = final_input()
            raw[key] = value
            self.rejected(finalize_publication, raw)

    def test_waiver_requires_authenticated_complete_capture_and_safety(self) -> None:
        for key in ("authenticated_complete_capture", "ordinary_safety_complete"):
            raw = final_input()
            stall: RawDocument = {"step": 41, "source": A, "status": "WAIVER_ELIGIBLE",
                     "authenticated_complete_capture": True, "ordinary_safety_complete": True}
            stall[key] = False
            raw["stall"] = stall
            self.rejected(finalize_publication, raw)


if __name__ == "__main__":
    _ = unittest.main()
