"""Tests for `check_docs_helpers.py schema` (M1.5 goal 8.1): docs/v8/snapshot_schema.md must name
every `get_*` #[func] of the bridge and exactly the dictionary keys each one sets.

A fixture bridge (a node with direct keys, a builder, a computed key, a builder whose keys depend on a
literal `false` argument, a scalar getter, a test-only decoy) checks each failure; the live check runs on
this repository.
"""
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_docs_helpers as helpers  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]

NODE = '''
use super::things::{thing_to_dict, rows_to_dict};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct WorldSimNode {
    engine: SimEngine,
}

#[godot_api]
impl INode for WorldSimNode {
    fn process(&mut self, delta: f64) {}
}

#[godot_api]
impl WorldSimNode {
    /// Built by a builder.
    #[func]
    fn get_thing(&self) -> VarDictionary {
        thing_to_dict(collect_thing(&self.engine.world))
    }

    /// Direct keys plus a computed key.
    #[func]
    fn get_direct(&self, x: i32) -> VarDictionary {
        let mut dict = VarDictionary::new();
        dict.set("tile_x", x);
        for ch in Channel::all() {
            dict.set(channel_key(*ch), 0u8);
        }
        dict
    }

    #[func]
    fn get_paged(&self) -> VarDictionary {
        rows_to_dict(collect_rows(), true)
    }

    #[func]
    fn get_unpaged(&self) -> VarDictionary {
        rows_to_dict(
            collect_rows(),
            false,
        )
    }

    /// A scalar.
    #[func]
    fn get_count(&self) -> i64 {
        7
    }

    #[func]
    fn cmd_order(&mut self) -> VarDictionary {
        result_to_dict(submit())
    }
}
'''

THINGS = '''
pub(super) fn channel_key(ch: Channel) -> &'static str {
    match ch {
        Channel::Warmth => "warmth",
        Channel::Light => "light",
    }
}

pub(super) fn thing_to_dict(t: Thing) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("ids", t.ids);
    dict.set(
        "xs",
        PackedInt32Array::from(t.xs.as_slice()),
    );
    for (key, values) in [("ages", &t.ages), ("kinds", &t.kinds)] {
        dict.set(key, PackedInt32Array::from(values.as_slice()));
    }
    dict
}

pub(super) fn rows_to_dict(rows: Rows, with_page: bool) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("tick", rows.tick);
    if with_page {
        dict.set("next", rows.next);
    }
    dict
}

pub(super) fn result_to_dict(r: CommandResult) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("accepted", r.accepted);
    dict
}

#[cfg(test)]
mod tests {
    fn thing_to_dict() -> VarDictionary {
        let mut dict = VarDictionary::new();
        dict.set("decoy", 1);
        dict
    }
}
'''

DOC = '''# Snapshot schema

Schema version: 1

### `get_thing`
| key | type |
|---|---|
| `ids` | PackedInt64Array |
| `xs` | PackedInt32Array |
| `ages` | PackedInt32Array |
| `kinds` | PackedInt32Array |

### `get_direct`
| key | type |
|---|---|
| `tile_x` | int |
| `warmth` | int |
| `light` | int |

### `get_paged`
| key | type |
|---|---|
| `tick` | int |
| `next` | int |

### `get_unpaged`
| key | type |
|---|---|
| `tick` | int |

### `get_count`
A scalar; keys: none.

## Text renderer glyphs
| layer | glyph |
|---|---|
| `people` | `@` |
'''


class SchemaDocsTest(unittest.TestCase):
    def fixture(self, doc=DOC):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        root = Path(tmp.name)
        ffi = root / "rust/crates/sim-bridge/src/ffi"
        ffi.mkdir(parents=True)
        (ffi / "node.rs").write_text(NODE)
        (ffi / "things.rs").write_text(THINGS)
        (root / "docs/v8").mkdir(parents=True)
        (root / helpers.SCHEMA_DOC).write_text(doc)
        return root

    def test_extraction_follows_builders_computed_keys_and_false_branches(self):
        extracted, problems = helpers.schema_extract(self.fixture())
        self.assertEqual(problems, [])
        self.assertEqual(sorted(extracted), ["get_count", "get_direct", "get_paged", "get_thing", "get_unpaged"])
        self.assertEqual(extracted["get_thing"], ("VarDictionary", ["ages", "ids", "kinds", "xs"]))
        self.assertEqual(extracted["get_direct"][1], ["light", "tile_x", "warmth"])
        self.assertEqual(extracted["get_paged"][1], ["next", "tick"])
        self.assertEqual(extracted["get_unpaged"][1], ["tick"])
        self.assertEqual(extracted["get_count"], ("i64", []))

    def test_complete_doc_passes(self):
        _, problems = helpers.schema_problems(self.fixture())
        self.assertEqual(problems, [])

    def test_a_missing_key_fails(self):
        _, problems = helpers.schema_problems(self.fixture(DOC.replace("| `kinds` | PackedInt32Array |\n", "")))
        self.assertEqual(problems, ["absent key: get_thing.kinds"])

    def test_a_missing_computed_key_fails(self):
        _, problems = helpers.schema_problems(self.fixture(DOC.replace("| `light` | int |\n", "")))
        self.assertEqual(problems, ["absent key: get_direct.light"])

    def test_an_extra_key_fails(self):
        doc = DOC.replace("| `tick` | int |\n\n### `get_count`", "| `tick` | int |\n| `next` | int |\n\n### `get_count`")
        _, problems = helpers.schema_problems(self.fixture(doc))
        self.assertEqual(problems, ["extra key: get_unpaged.next (not set by the bridge)"])

    def test_a_missing_or_extra_getter_fails(self):
        _, problems = helpers.schema_problems(self.fixture(DOC.replace("### `get_count`", "### `get_counted`")))
        self.assertEqual(problems, ["absent getter: get_count", "extra getter: get_counted (no such get_* #[func])"])

    def test_a_missing_doc_fails(self):
        root = self.fixture()
        (root / helpers.SCHEMA_DOC).unlink()
        _, problems = helpers.schema_problems(root)
        self.assertEqual(problems, [f"{helpers.SCHEMA_DOC} does not exist"])

    def nested_fixture(self) -> Path:
        """Mirror MUX's qualified serializers and nested ui_facts module layout."""
        root = self.fixture()
        ffi = root / "rust/crates/sim-bridge/src/ffi"
        node = ffi / "node.rs"
        node.write_text(node.read_text() + '''
#[godot_api]
impl WorldSimNode {
    #[func]
    fn get_ui_facts(&self) -> VarDictionary {
        super::ui_facts::ui_facts_to_dict(&facts())
    }
    #[func]
    fn get_steward(&self) -> VarDictionary {
        super::steward::steward_to_dict(&state())
    }
}
''')
        (ffi / "ui_facts").mkdir()
        (ffi / "ui_facts/mod.rs").write_text("pub(super) use marshal::ui_facts_to_dict;")
        (ffi / "ui_facts/marshal.rs").write_text('''
pub(in crate::ffi) fn ui_facts_to_dict(v: &Facts) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("people", people_to_dict(&v.people));
    d.set("tech", super::super::things::thing_to_dict(&v.tech));
    d
}
fn people_to_dict(rows: &[Person]) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("hps", rows.hps);
    d.set("entity_ids", rows.entity_ids);
    d
}
''')
        (ffi / "steward.rs").write_text('''
pub(super) fn steward_to_dict(v: &State) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("enabled", v.enabled);
    d.set("rules", v.rules);
    d
}
''')
        doc = root / helpers.SCHEMA_DOC
        doc.write_text(doc.read_text() + '''
### `get_ui_facts`
| `people` | Dictionary |
| `tech` | Dictionary |
| `hps` | PackedFloat64Array |
| `entity_ids` | PackedInt64Array |
| `ids` | PackedInt64Array |
| `xs` | PackedInt32Array |
| `ages` | PackedInt32Array |
| `kinds` | PackedInt32Array |
### `get_steward`
| `enabled` | bool |
| `rules` | Dictionary |
''')
        return root

    def test_qualified_builders_follow_nested_modules(self) -> None:
        root = self.nested_fixture()
        extracted, problems = helpers.schema_problems(root)
        self.assertEqual(problems, [])
        self.assertEqual(extracted["get_ui_facts"][1],
                         ["ages", "entity_ids", "hps", "ids", "kinds", "people", "tech", "xs"])
        self.assertEqual(extracted["get_steward"][1], ["enabled", "rules"])

    def test_nested_missing_key_fails(self) -> None:
        root = self.nested_fixture()
        doc = root / helpers.SCHEMA_DOC
        doc.write_text(doc.read_text().replace("| `hps` | PackedFloat64Array |\n", ""))
        _, problems = helpers.schema_problems(root)
        self.assertEqual(problems, ["absent key: get_ui_facts.hps"])

    def test_live_schema_doc_matches_the_bridge(self):
        extracted, problems = helpers.schema_problems(ROOT)
        self.assertEqual(problems, [])
        # The M1.5 additions are documented (goal 8.1).
        self.assertTrue({"year_index", "season_index", "day_of_year", "days_per_year"} <= set(extracted["get_time_snapshot"][1]))
        self.assertIn("ticks_per_year", extracted["get_world_info"][1])
        self.assertTrue({"ages_years", "infants", "pregnant_days"} <= set(extracted["get_agent_snapshot"][1]))


if __name__ == "__main__":
    unittest.main()
