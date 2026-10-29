"""Causal validation uses actual identity and samples, never event counts alone."""
import unittest

from m6a_causal import audit
from m6a_evidence import EvidenceError, Node


class CausalTests(unittest.TestCase):
    def chain(self):
        rows = []
        for index, code, value, cause in (
            (0, 6, 40, None), (1, 2, 20, {"Entry": 0}),
            (2, 1, 0, {"Entry": 1}), (3, 0, 1000, {"Sample": {"code": 0, "sample": 0}}),
        ):
            rows.append({"index": index, "kind": "hunger_food_chain", "code": code,
                         "value": value, "cause": cause, "subject": 7, "tick": 100,
                         "sample": {"agent": 7, "tick": 100, "hunger": 1000, "entry": 2} if index == 3 else None})
        rows.append({"index": 4, "kind": "died", "code": 0, "cause": {"Entry": 3}, "subject": 7, "tick": 100})
        return Node({"entries": rows})

    def test_real_chain_retains_unknown_historical_reason(self):
        result = audit(self.chain(), [Node({"tick": 100, "count": 1})], 480, 1440)
        self.assertEqual(result["historical_unharvested_reason_unknown"], 1)

    def test_count_only_cannot_pass_causal_coverage(self):
        with self.assertRaises(EvidenceError):
            audit(Node({"entries": []}), [Node({"tick": 100, "count": 1})], 480, 1440)

    def test_cross_person_sample_fails(self):
        chain = self.chain()
        chain.value["entries"][3]["sample"]["agent"] = 8
        with self.assertRaises(EvidenceError):
            audit(chain, [Node({"tick": 100, "count": 1})], 480, 1440)


if __name__ == "__main__":
    unittest.main()
