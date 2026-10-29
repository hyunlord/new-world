"""Tests for tools/v8/bridge_api.py and the bridge API invariant (goal 4.2, 12).

The live check: the exports of rust/crates/sim-bridge equal the current pass's
list (M6a: docs/v8/bridge_api_m6a.txt, gate step 13). Prior pass lists stay
as historical records. V8-0's record stays: its
removed names are not exported again.
"""
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bridge_api  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]

SOURCE = '''
#[derive(GodotClass)]
#[class(base=Node)]
pub struct WorldSimNode {
    engine: SimEngine,
}

#[godot_api]
impl INode for WorldSimNode {
    fn init(base: Base<Node>) -> Self { todo() }
    fn process(&mut self, delta: f64) {}
}

#[godot_api]
impl WorldSimNode {
    #[signal]
    fn world_reset();

    /// A read.
    #[func]
    #[allow(clippy::too_many_arguments)]
    fn get_thing(&self) -> VarDictionary { VarDictionary::new() }

    #[func(rename = renamed_thing)]
    pub fn internal_name(&self) -> i64 { 0 }

    fn not_exported(&self) {}
}
'''


class BridgeApiTest(unittest.TestCase):
    def test_scan_source_finds_class_funcs_signals_and_renames(self):
        self.assertEqual(
            bridge_api.scan_source(SOURCE),
            {"class WorldSimNode", "signal WorldSimNode.world_reset", "func WorldSimNode.get_thing",
             "func WorldSimNode.renamed_thing"},
        )

    def test_trait_impls_and_unmarked_methods_are_not_exports(self):
        found = bridge_api.scan_source(SOURCE)
        self.assertNotIn("func WorldSimNode.process", found)
        self.assertNotIn("func WorldSimNode.not_exported", found)

    def test_scan_repo_reads_every_bridge_file(self):
        root = Path(tempfile.mkdtemp())
        src = root / "rust/crates/sim-bridge/src/ffi"
        src.mkdir(parents=True)
        (src / "a.rs").write_text(SOURCE)
        (src / "b.rs").write_text("#[godot_api]\nimpl WorldSimNode {\n    #[func]\n    fn get_other(&self) {}\n}\n")
        self.assertIn("func WorldSimNode.get_other", bridge_api.scan_repo(root))
        self.assertIn("func WorldSimNode.get_thing", bridge_api.scan_repo(root))

    def test_live_bridge_equals_the_current_list(self):
        # The current list checked by gate step 13; historical lists remain unchanged.
        expected = set(bridge_api.read_list(ROOT / "docs/v8/bridge_api_m6a.txt"))
        self.assertEqual(set(bridge_api.scan_repo(ROOT)), expected)

    def test_v8_0_removed_names_stay_removed(self):
        base = set(bridge_api.read_list(ROOT / "docs/v8/bridge_api_v7_final.txt"))
        removed = set(bridge_api.read_list(ROOT / "docs/v8/bridge_api_removed.txt"))
        self.assertTrue(removed <= base, "a removed name is not in the base list")
        self.assertFalse(removed & set(bridge_api.scan_repo(ROOT)))


if __name__ == "__main__":
    unittest.main()
