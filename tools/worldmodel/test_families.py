"""Pass 9 section 10: branch families on a tiny dataset (2 seeds x 2 variants x 2
replicates x 3 days). Needs the release recorder; skips when it is not built.

Run: python3 -m unittest discover -s tools/worldmodel -p test_families.py
"""
import json
import os
import tempfile
import unittest
from pathlib import Path

import numpy as np

import make_dataset
from wm_data import load_family

ROOT = Path(__file__).resolve().parents[2]
BIN = Path(os.environ.get("WM_BIN", ROOT / "rust/target/release/worldsim_wm"))
FORK = 1440


@unittest.skipUnless(BIN.exists(), f"recorder not built: {BIN}")
class FamilyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.tmp.name)
        sched = cls.root / "schedules"
        sched.mkdir()
        for seed in (5, 6):
            (sched / f"s{seed}_gift.json").write_text(json.dumps([{"tick": 2000, "kind": "gift_food", "settlement": "home", "amount": 10}]))
            (sched / f"s{seed}_rain.json").write_text(json.dumps([{"tick": 2500, "kind": "rain", "settlement": "home", "amount": 5}]))
        env = {"WM_BIN": str(BIN), "WM_DATA_ROOT": str(cls.root), "WORLDSIM_GIT_SHA": "test"}
        old = {k: os.environ.get(k) for k in env}
        os.environ.update(env)
        try:
            with open(os.devnull, "w") as log:
                args = make_dataset.argparse.Namespace(
                    root=cls.root, name="tiny", scene="game_v15", seeds="5..7", days=3, every=60, fork_tick=FORK,
                    replicates=2, schedule_dir=sched, family_splits="train=5..6,test=6..7", agents=True, workers=2, no_run=False)
                make_dataset.run_families(args, log)
        finally:
            for k, v in old.items():
                if v is None:
                    os.environ.pop(k, None)
                else:
                    os.environ[k] = v
        cls.manifest = json.loads((cls.root / "tiny" / "families_manifest.json").read_text())

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_manifest_lists_families_with_splits_per_root_seed(self) -> None:
        m = self.manifest
        self.assertEqual((m["family_count"], m["member_count"]), (2, 12))
        self.assertEqual({f["root_seed"]: f["split"] for f in m["families"]}, {5: "train", 6: "test"})
        for fam in m["families"]:
            self.assertEqual(fam["fork_tick"], FORK)
            self.assertEqual(sorted({(x["schedule_id"], x["replicate"]) for x in fam["members"]}),
                             [(i, k) for i in ("gift", "none", "rain") for k in (0, 1)])
            self.assertTrue(all(x["path"].startswith(fam["path"] + "/") for x in fam["members"]))

    def test_load_family_by_replicate_and_shared_prefix(self) -> None:
        fam = load_family(self.root / "tiny" / "game_v15_s5")
        self.assertEqual((fam.seed, fam.scene, fam.fork_tick, fam.replicates), (5, "game_v15", FORK, 2))
        self.assertEqual(sorted(fam.variants), ["gift", "rain"])
        row = FORK // 60
        digests = [ep.arrays["row_digest.npy"] for ep in [*fam.factual.values(), *fam.variants["gift"].values(),
                                                         *fam.variants["rain"].values()]]
        for d in digests[1:]:
            np.testing.assert_array_equal(d[:row + 1], digests[0][:row + 1])
        # Replicates differ after the fork; the factual and a variant of one replicate agree until the intervention.
        self.assertFalse(np.array_equal(fam.factual[0].arrays["row_digest.npy"], fam.factual[1].arrays["row_digest.npy"]))
        gift_row = 2000 // 60
        np.testing.assert_array_equal(fam.factual[1].arrays["row_digest.npy"][:gift_row + 1],
                                      fam.variants["gift"][1].arrays["row_digest.npy"][:gift_row + 1])
        self.assertEqual(fam.factual[0].meta["noise_fork"], {"tick": FORK, "replicate": 0})
        # Agents carry lineage keys; the founders' keys match across members.
        keys0 = fam.factual[0].lineage_keys()
        self.assertTrue((keys0[:24] == fam.variants["rain"][1].lineage_keys()[:24]).all())

    def test_splits_must_cover_every_seed_once(self) -> None:
        with self.assertRaises(ValueError):
            make_dataset.family_splits("train=5..6", range(5, 7))
        with self.assertRaises(ValueError):
            make_dataset.family_splits("train=5..7,test=6..7", range(5, 7))
        self.assertEqual(make_dataset.family_splits("a=5..6,b=6..7", range(5, 7)), {5: "a", 6: "b"})


if __name__ == "__main__":
    unittest.main()
