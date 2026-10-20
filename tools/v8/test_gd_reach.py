"""Unit tests for tools/v8/gd_reach.py on five hand-made projects."""
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gd_reach  # noqa: E402


def project(files):
    """Write `files` (relative path -> text) into a fresh directory (no git)."""
    root = Path(tempfile.mkdtemp())
    for rel, text in files.items():
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    return root


PROJECT = '[application]\nrun/main_scene="res://scenes/main.tscn"\n'


class GdReachTest(unittest.TestCase):
    def test_preload_and_load_chain_from_main_scene(self):
        root = project({
            "project.godot": PROJECT,
            "scenes/main.tscn": '[ext_resource type="Script" path="res://a.gd" id="1"]\n',
            "a.gd": 'const B = preload("res://b.gd")\n',
            "b.gd": 'func f():\n\tload("res://c.png")\n',
            "c.png": "x", "unused.gd": "extends Node\n",
        })
        got = gd_reach.reach(root)["reachable"]
        self.assertEqual(set(got), {"project.godot", "scenes/main.tscn", "a.gd", "b.gd", "c.png"})

    def test_class_name_use_reaches_declaring_file_but_comments_do_not(self):
        root = project({
            "project.godot": PROJECT,
            "scenes/main.tscn": '[ext_resource path="res://a.gd" id="1"]\n',
            "a.gd": "var x := Helper.new()\n# OldThing is gone\n",
            "helper.gd": "class_name Helper\nextends RefCounted\n",
            "old.gd": "class_name OldThing\n",
        })
        got = set(gd_reach.reach(root)["reachable"])
        self.assertIn("helper.gd", got)
        self.assertNotIn("old.gd", got)

    def test_autoload_extends_path_and_uid_companion(self):
        root = project({
            "project.godot": PROJECT + '[autoload]\nLocale="*res://core/locale.gd"\n',
            "scenes/main.tscn": "",
            "core/locale.gd": 'extends "res://core/base.gd"\n',
            "core/base.gd": "extends Node\n", "core/base.gd.uid": "uid://x\n",
        })
        got = set(gd_reach.reach(root)["reachable"])
        self.assertTrue({"core/locale.gd", "core/base.gd", "core/base.gd.uid"} <= got)

    def test_json_manifest_literals_and_derived_regions_page(self):
        root = project({
            "project.godot": PROJECT,
            "scenes/main.tscn": '[ext_resource path="res://lib.gd" id="1"]\n',
            "lib.gd": 'const ATLAS := "res://art/atlas.json"\n',
            "art/atlas.json": '{"pages": ["res://art/world_0.png"]}',
            "art/world_0.png": "p", "art/world_0_regions.png": "r", "art/other.png": "o",
        })
        got = set(gd_reach.reach(root)["reachable"])
        self.assertTrue({"art/atlas.json", "art/world_0.png", "art/world_0_regions.png"} <= got)
        self.assertNotIn("art/other.png", got)

    def test_formatted_literal_is_a_dynamic_family(self):
        root = project({
            "project.godot": PROJECT,
            "scenes/main.tscn": '[ext_resource path="res://loc.gd" id="1"]\n',
            "loc.gd": 'var p := "res://localization/compiled/%s.json" % lang\n',
            "localization/compiled/en.json": "{}", "localization/compiled/ko.json": "{}",
            "localization/fluent/en.ftl": "", "localization/en/ui.json": "{}",
            "concat.gd": "",
        })
        result = gd_reach.reach(root)
        got = set(result["reachable"])
        self.assertTrue({"localization/compiled/en.json", "localization/compiled/ko.json"} <= got)
        self.assertNotIn("localization/fluent/en.ftl", got)
        self.assertNotIn("localization/en/ui.json", got)
        self.assertEqual(result["families"], ["localization/compiled/*.json"])
        dir_root = project({
            "project.godot": PROJECT,
            "scenes/main.tscn": '[ext_resource path="res://loc.gd" id="1"]\n',
            "loc.gd": 'var d := DirAccess.open("res://localization/%s/" % lang)\nvar s := load("res://sprites/" + name)\n',
            "localization/en/ui.json": "{}", "localization/fluent/en/messages.ftl": "",
            "localization/key_registry.json": "{}", "sprites/a.png": "", "sprites/deep/b.png": "",
        })
        result = gd_reach.reach(dir_root)
        got = set(result["reachable"])
        self.assertIn("localization/en/ui.json", got)
        self.assertIn("sprites/a.png", got)
        self.assertFalse({"localization/fluent/en/messages.ftl", "localization/key_registry.json", "sprites/deep/b.png"} & got)
        self.assertEqual(result["families"], ["localization/*/*", "sprites/*"])


if __name__ == "__main__":
    unittest.main()
