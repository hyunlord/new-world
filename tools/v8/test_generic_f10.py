"""F10's intentional loss never exempts unrelated loss or unaccepted commands."""
import copy
import json
import unittest
from pathlib import Path

from generic_f10 import KILL, companion_delta, injection
from generic_gate import GateContractError
from generic_observations import Observation
from mux2_gate_inputs import Json
from test_generic_gate_contract import (
    RawDocument,
    json_cases,
    mapping,
)


class F10Tests(unittest.TestCase):
    def inputs(self) -> tuple[Observation, list[Json], RawDocument]:
        common: RawDocument = {"tick": 1800, "alive": 10, "script_index": 0, "script_spec": KILL,
                  "resolved_command_index": None, "debug_kill_agent": None,
                  "history_complete": True, "knowledge_lost": []}
        routed = dict(common, phase="before_tick_commands")
        resolved = dict(common, phase="before_debug_kill", resolved_command_index=0, debug_kill_agent=9)
        observed = Observation(0, 5760, 10, 9, True, ((1800, "doorway", 9),), json.dumps([routed, resolved]))
        commands: list[Json] = [{"tick": 1800, "cmd": "debug_kill", "agent": 9, "result": {"accepted": True}}]
        summary: RawDocument = {"m4": {"lost": [[1800, "doorway", 9]], "node_names": ["doorway"],
                          "status_end": [4], "waits_end": [[1, 0, 0, "doorway"]]}}
        return observed, commands, summary

    def test_committed_companion_exact_delta(self) -> None:
        root = Path(__file__).parent
        companion_delta((root / "m4_f10_script.json").read_text(), (root / "gate_f10_no_kill.json").read_text())
        with self.assertRaises(GateContractError):
            companion_delta(json.dumps([KILL, {"tick": 1, "cmd": "x"}]), "[]")

    def test_native_injection_binds_actual_boundary(self) -> None:
        observed, commands, summary = self.inputs()
        pre = injection(observed, json.dumps([KILL]), json.dumps(commands), json.dumps(summary))
        self.assertEqual((pre.terminal_tick, pre.alive, pre.losses), (1800, 10, ()))

    def test_missing_rejected_wrong_target_receipts_fail(self) -> None:
        observed, commands, summary = self.inputs()
        variants = [[], [dict(mapping(commands[0]), agent=8)], [dict(mapping(commands[0]), result={"accepted": False})],
                    [dict(mapping(commands[0]), tick=1799)]]
        for variant in variants:
            with self.subTest(variant=variant), self.assertRaises(GateContractError):
                _ = injection(observed, json.dumps([KILL]), json.dumps(variant), json.dumps(summary))

    def test_additional_loss_and_failed_oracle_rejected(self) -> None:
        observed, commands, summary = self.inputs()
        wrong = Observation(0, 5760, 10, 9, True, ((1800, "clay_digging", 9),), observed.checkpoints_json)
        with self.assertRaises(GateContractError):
            _ = injection(wrong, json.dumps([KILL]), json.dumps(commands), json.dumps(summary))
        for key, value in json_cases((("lost", []), ("status_end", [3]), ("waits_end", []),
                           ("waits_end", [[True, None, None, "doorway"]]),
                           ("waits_end", [[1, 0, 0, "doorway", 5]]))):
            changed = copy.deepcopy(summary)
            mapping(changed["m4"])[key] = value
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = injection(observed, json.dumps([KILL]), json.dumps(commands), json.dumps(changed))


if __name__ == "__main__":
    _ = unittest.main()
