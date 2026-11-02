"""UI native identity remains exact across reset/load and JSON unsigned values."""
import copy
import json
import unittest

from generic_gate import GateContractError
from generic_ui_observations import (
    UIInstance,
    parse_native,
    require_fresh_history,
    validate_progress,
)
from mux2_gate_inputs import Json
from test_generic_ui_receipts import arr, obj


def native() -> dict[str, Json]:
    return {"schema_version": 1, "status": "OBSERVED", "node_instance_id": "42", "instances": [{
        "instance_ordinal": 0, "reason": "bootstrap", "seed": str(2**64 - 1),
        "initial_tick": "0", "initial_population": 2, "initial_agent_ids": ["0", str(2**64 - 1)],
        "terminal_tick": "30", "terminal_alive": 2, "terminal_agent_ids": ["0", str(2**64 - 1)], "closed": False,
        "history_complete": True, "history_gap": None, "chronicle_cursor": 2,
        "knowledge_enabled": True, "knowledge_capability_source": "resources.rules.colony.knowledge",
        "knowledge_lost": []}]}


def parse(value: Json) -> tuple[UIInstance, ...]:
    return parse_native(json.dumps(value), 42)


class UINativeTests(unittest.TestCase):
    def test_unsigned_and_null_holder_exact(self) -> None:
        data = native()
        obj(data, "instances", 0)["knowledge_lost"] = [
            {"tick": "20", "node": "x", "node_index": 2, "holder": None},
            {"tick": "20", "node": "x", "node_index": 2, "holder": str(2**64 - 1)}]
        instance = parse(data)[0]
        self.assertEqual(instance.seed, 2**64 - 1)
        self.assertEqual(instance.state.losses[1][2], 2**64 - 1)
        self.assertIsNone(instance.state.losses[0][2])

    def test_terminal_census_exact_and_empty(self) -> None:
        data = native()
        self.assertEqual(parse(data)[0].terminal_agent_ids, (0, 2**64 - 1))
        obj(data, "instances", 0).update(terminal_alive=0, terminal_agent_ids=[])
        self.assertEqual(parse(data)[0].terminal_agent_ids, ())

    def test_invalid_terminal_census_rejected(self) -> None:
        cases: tuple[Json, ...] = (["0"], ["0", "0"], ["1", "0"], ["0", 1], ["0", "01"],
                    ["0", "-1"], ["0", str(2**64)], None)
        for ids in cases:
            data = native()
            obj(data, "instances", 0)["terminal_agent_ids"] = ids
            with self.subTest(ids=ids), self.assertRaises(GateContractError):
                _ = parse(data)
        data = native()
        del obj(data, "instances", 0)["terminal_agent_ids"]
        with self.assertRaises(GateContractError):
            _ = parse(data)

    def test_loaded_history_never_becomes_fresh_from_empty_loss(self) -> None:
        data = native()
        row = obj(data, "instances", 0)
        row["reason"] = "load"
        with self.assertRaisesRegex(GateContractError, "self-authenticate"):
            _ = parse(data)
        row.update(reason="load", history_complete=False, history_gap="initial_history_unauthenticated")
        with self.assertRaisesRegex(GateContractError, "save-prefix"):
            _ = require_fresh_history(parse(data)[0])

    def test_wrong_types_and_census_rejected(self) -> None:
        cases: tuple[tuple[str, Json], ...] = (("initial_tick", 0), ("seed", "01"), ("seed", str(2**64)),
                           ("terminal_tick", "-1"), ("initial_population", 3),
                           ("initial_agent_ids", ["0", "0"]), ("closed", 0),
                           ("history_gap", "gap"), ("knowledge_lost", None))
        for key, value in cases:
            data = native()
            obj(data, "instances", 0)[key] = value
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = parse(data)

    def test_retained_closed_reset_and_monotonic_checkpoints(self) -> None:
        first = native()
        second = copy.deepcopy(first)
        obj(second, "instances", 0)["closed"] = True
        new = copy.deepcopy(obj(second, "instances", 0))
        new.update(instance_ordinal=1, reason="reset", closed=False, terminal_tick="0", chronicle_cursor=0)
        arr(second, "instances").append(new)
        validate_progress(parse(first), parse(second))
        third = copy.deepcopy(second)
        obj(third, "instances", 0)["terminal_alive"] = 1
        obj(third, "instances", 0)["terminal_agent_ids"] = ["0"]
        with self.assertRaisesRegex(GateContractError, "closed native"):
            validate_progress(parse(second), parse(third))
        with self.assertRaisesRegex(GateContractError, "shrank"):
            validate_progress(parse(second), parse(first))

    def test_same_tick_loss_prefix_cannot_shrink(self) -> None:
        first = native()
        obj(first, "instances", 0)["knowledge_lost"] = [{"tick": "30", "node": "x", "node_index": 2, "holder": None}]
        with self.assertRaisesRegex(GateContractError, "regressed"):
            validate_progress(parse(first), parse(native()))


if __name__ == "__main__":
    _ = unittest.main()
