"""Native sidecars must not turn partial worlds or matching losses into PASS."""
import copy
import json
import unittest

from generic_gate import GateContractError
from generic_observations import Observation, ProbeWorld, ordinary_pair, parse_probe
from mux2_gate_inputs import Json
from test_generic_gate_contract import (
    RawDocument,
    array,
    json_cases,
    json_values,
    mapping,
)


def receipt() -> RawDocument:
    return {"schema_version": 1, "scene": "colony", "seed": 0, "occurrence": "primary",
            "requested_ticks": 28800, "requested_terminal_tick": 28800, "initial_tick": 0,
            "initial_population": 10, "terminal_tick": 28800, "terminal_alive": 8,
            "terminal_complete": True, "history_complete": True,
            "history_source": "fresh_world_append_only_chronicle",
            "knowledge_capability": {"enabled": True, "source": "resources.rules.colony.knowledge"},
            "knowledge_lost": [], "deaths": {name: 0 for name in
                ("starvation", "dehydration", "exposure", "combat", "old_age")}, "checkpoints": []}


WORLD = ProbeWorld("colony", 0, "primary", 28800)


def parse(value: Json) -> Observation:
    return parse_probe(json.dumps(value), WORLD)


class NativeObservationTests(unittest.TestCase):
    def test_complete_independent_pair(self) -> None:
        self.assertEqual(ordinary_pair(parse(receipt()), parse(receipt()))["candidate_alive"], 8)

    def test_incomplete_wrong_or_forged_native_fields_fail(self) -> None:
        for key, value in json_cases((("seed", 1), ("occurrence", "determinism_twin"), ("scene", "baseline"),
                           ("requested_ticks", 12), ("terminal_tick", 28799), ("terminal_complete", False),
                           ("history_complete", False), ("initial_tick", 1), ("terminal_alive", True),
                           ("initial_population", -1), ("terminal_tick", None),
                           ("history_source", "prior_history_unverified"), ("deaths", {}))):
            with self.subTest(key=key, value=value):
                data = receipt()
                data[key] = value
                with self.assertRaises(GateContractError):
                    _ = parse(data)

    def test_schema_missing_extra_and_duplicate_fail(self) -> None:
        for key in receipt():
            data = receipt()
            del data[key]
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = parse(data)
        data = receipt()
        data["trusted"] = True
        with self.assertRaises(GateContractError):
            _ = parse(data)
        with self.assertRaises(GateContractError):
            _ = parse_probe(json.dumps(receipt())[:-1] + ',"seed":0}', WORLD)

    def test_matching_base_loss_is_not_a_safety_exception(self) -> None:
        data = receipt()
        data["terminal_alive"] = 0
        data["knowledge_lost"] = [{"tick": 28560, "node": "clay_digging", "holder": 9}]
        with self.assertRaisesRegex(GateContractError, "KnowledgeLost"):
            _ = ordinary_pair(parse(data), parse(data))
        self.assertEqual(ordinary_pair(parse(data), parse(receipt()))["base_alive"], 0)

    def test_survival_uses_base_relative_floor(self) -> None:
        base, candidate = receipt(), receipt()
        base["terminal_alive"] = 5
        candidate["terminal_alive"] = 5
        _ = ordinary_pair(parse(base), parse(candidate))
        candidate["terminal_alive"] = 4
        with self.assertRaisesRegex(GateContractError, "survival"):
            _ = ordinary_pair(parse(base), parse(candidate))

    def test_disabled_capability_requires_native_null_and_pair_match(self) -> None:
        data = receipt()
        mapping(data["knowledge_capability"])["enabled"] = False
        with self.assertRaises(GateContractError):
            _ = parse(data)
        data["knowledge_lost"] = None
        _ = ordinary_pair(parse(data), parse(data))
        with self.assertRaises(GateContractError):
            _ = ordinary_pair(parse(data), parse(receipt()))

    def test_loss_schema_order_and_bounds(self) -> None:
        for rows in json_values(([{"tick": 1, "node": "x", "holder": None}],
                     [{"tick": 1, "node": "x", "holder": 2**64 - 1}])):
            data = receipt()
            data["knowledge_lost"] = rows
            self.assertEqual(len(parse(data).losses), 1)
        for rows in json_values((None, [{"tick": 28801, "node": "x", "holder": None}],
                     [{"tick": 1, "node": "x", "holder": -1}],
                     [{"tick": 1, "node": "x"}],
                     [{"tick": 2, "node": "x", "holder": None},
                      {"tick": 1, "node": "x", "holder": None}])):
            data = receipt()
            data["knowledge_lost"] = rows
            with self.subTest(rows=rows), self.assertRaises(GateContractError):
                _ = parse(data)

    def test_f10_checkpoint_is_actual_boundary_not_safety_exemption(self) -> None:
        data = receipt()
        checkpoint: RawDocument = {"phase": "before_debug_kill", "tick": 1800, "alive": 10,
                      "script_index": 0, "script_spec": {"tick": 1800, "cmd": "debug_kill"},
                      "resolved_command_index": 0, "debug_kill_agent": 9,
                      "history_complete": True, "knowledge_lost": []}
        data["checkpoints"] = [checkpoint]
        _ = parse(data)
        omitted = copy.deepcopy(data)
        omitted["knowledge_lost"] = [{"tick": 1, "node": "x", "holder": None}]
        with self.assertRaisesRegex(GateContractError, "omits earlier"):
            _ = parse(omitted)
        shrinking = copy.deepcopy(data)
        loss: RawDocument = {"tick": 1800, "node": "x", "holder": None}
        shrinking["knowledge_lost"] = [loss]
        shrinking["checkpoints"] = [copy.deepcopy(checkpoint), copy.deepcopy(checkpoint)]
        mapping(array(shrinking["checkpoints"])[0])["knowledge_lost"] = [loss]
        with self.assertRaisesRegex(GateContractError, "history shrank"):
            _ = parse(shrinking)
        for key, value in json_cases((("history_complete", False), ("debug_kill_agent", None),
                           ("tick", 28801), ("phase", "accepted"),
                           ("knowledge_lost", [{"tick": 1, "node": "x", "holder": None}]))):
            mutated = copy.deepcopy(data)
            mapping(array(mutated["checkpoints"])[0])[key] = value
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = parse(mutated)


if __name__ == "__main__":
    _ = unittest.main()
