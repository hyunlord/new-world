# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -B -m unittest test_overlay_parent
"""Actual Git application only; no compiler or native product execution."""
import json
import subprocess
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from typing import final

from generic_build import Artifact, BuildSpec, Overlay, overlay_files
from generic_gate import GateContractError
from generic_process import file_hash
from generic_sources import safe_path


@final
class OverlayParentTests(unittest.TestCase):
    def __init__(self, methodName: str = 'runTest') -> None:
        super().__init__(methodName)
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.home = Path(temp.name).resolve()
        self.root = self.home/'source'
        self.root.mkdir()
        self.name = 'rust/crates/sim-bridge/src/ffi/gate_observer/new.rs'
        (self.root/'rust/crates/sim-bridge/src/ffi').mkdir(parents=True)
        _ = (self.root/'rust/crates/sim-bridge/src/ffi/base.rs').write_text('base\n')
        _ = self.git('init', '-q')
        _ = self.git('add', '.')
        _ = self.git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-qm', 'fixture')
        self.sha = self.git('rev-parse', 'HEAD').strip()
        tool = Path(sys.executable).resolve()
        self.spec = BuildSpec(self.root, self.sha, tool, file_hash(tool), tool, file_hash(tool),
                              self.home, self.home/'output', ('sim-test',), (Artifact('worldsim_probe', 'release/worldsim_probe'),))

    def git(self, *args: str) -> str:
        return subprocess.check_output(['git', '-C', str(self.root), *args], text=True)

    def overlay(self, name: str, declared: str | None = None) -> BuildSpec:
        patch = self.home/'overlay.patch'
        _ = patch.write_text(f'diff --git a/{name} b/{name}\nnew file mode 100644\n--- /dev/null\n+++ b/{name}\n@@ -0,0 +1 @@\n+observer\n')
        data = self.home/'postimage'
        _ = data.write_text('observer\n')
        selected = declared if declared is not None else name
        manifest = self.home/'overlay.json'
        _ = manifest.write_text(json.dumps({'schema_version': 1, 'product_base_sha': self.sha,
            'observer_patch_sha256': file_hash(patch), 'allowed_paths': [selected],
            'files': [{'path': selected, 'base0_sha256': None, 'observer_sha256': file_hash(data), 'operation': 'add'}]}))
        return replace(self.spec, overlay=Overlay(patch, file_hash(patch), manifest, file_hash(manifest)))

    def test_declared_add_creates_missing_parent_through_git_apply(self) -> None:
        self.assertFalse((self.root/self.name).parent.exists())
        changed = overlay_files(self.overlay(self.name))
        self.assertEqual(set(changed), {self.name})
        self.assertEqual((self.root/self.name).read_text(), 'observer\n')

    def test_existing_symlink_parent_rejected(self) -> None:
        outside = self.home/'outside'
        outside.mkdir()
        (self.root/self.name).parent.symlink_to(outside, target_is_directory=True)
        with self.assertRaises(GateContractError):
            _ = overlay_files(self.overlay(self.name))
        self.assertEqual(list(outside.iterdir()), [])

    def test_existing_file_parent_rejected(self) -> None:
        _ = (self.root/self.name).parent.write_text('not a directory')
        with self.assertRaises(GateContractError):
            _ = overlay_files(self.overlay(self.name))

    def test_undeclared_patch_path_rejected_without_creating_parent(self) -> None:
        with self.assertRaises(GateContractError):
            _ = overlay_files(self.overlay(self.name, self.name.replace('new.rs', 'other.rs')))
        self.assertFalse((self.root/self.name).parent.exists())

    def test_strict_default_still_rejects_missing_parent(self) -> None:
        with self.assertRaises(FileNotFoundError):
            _ = safe_path(self.root, self.name)

    def test_traversal_rejected_before_application(self) -> None:
        with self.assertRaises(GateContractError):
            _ = overlay_files(self.overlay('tools/v8/../../escape.rs'))
