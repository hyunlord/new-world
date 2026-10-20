"""Tests for tools/v8/bridge_api.py and the bridge API invariant (goal 4.2, 12).

The live check: the exports of rust/crates/sim-bridge equal the V7 base list
(docs/v8/bridge_api_v7_final.txt) minus the names V8-0 removed on purpose
(docs/v8/bridge_api_removed.txt).
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

    def test_live_bridge_equals_v7_final_minus_the_removed_names(self):
        expected = set(bridge_api.read_list(ROOT / "docs/v8/bridge_api_v7_final.txt"))
        removed = set(bridge_api.read_list(ROOT / "docs/v8/bridge_api_removed.txt"))
        self.assertTrue(removed <= expected, "a removed name is not in the base list")
        self.assertEqual(set(bridge_api.scan_repo(ROOT)), expected - removed)


if __name__ == "__main__":
    unittest.main()
