"""Real tiny Git source projection tests; no build or runtime evidence is minted."""
import json
import unittest
from pathlib import Path

import test_generic_identity_materialization as identity_fixtures
from generic_original_harness_projection import (
    OriginalHarnessContext,
    projected_manifest,
)
from generic_process import file_hash
from generic_sources import freeze_source, git, manifest_hash
from generic_whole_gate_source import materialize
from mux2_gate_inputs import Json, Node


class ProjectionTests(unittest.TestCase):
    def fixture(self, apply_tools: bool = True) -> tuple[OriginalHarnessContext, Node, Node, Path]:
        owner = identity_fixtures.MaterializationIdentityTests()
        self.addCleanup(owner.doCleanups)
        prepared, roles, _ = owner.prepared()
        h = prepared.selectors.harness_root
        clone = h.parent / 'original-h-only'
        _ = git(h, 'clone', '-q', str(h), str(clone))
        _ = git(clone, 'checkout', '--detach', prepared.selectors.published_sha)
        original = Node(freeze_source(clone, prepared.selectors.published_sha))
        receipt = materialize(prepared, roles)
        path = prepared.selectors.output / 'materialization.json'
        session = Node({'schema': 'whole_gate_session.v1',
                        'materialization': {'path': str(path), 'sha256': file_hash(path)},
                        'harness': {'root': str(h), 'source_sha': prepared.selectors.harness_sha},
                        'arms': receipt.field('arms').value, 'binding': receipt.field('binding').value,
                        'declarations': receipt.field('declarations').value})
        selected = clone.parent / 'projection-session.json'
        _ = selected.write_text(json.dumps(session.value))
        context = OriginalHarnessContext(selected, file_hash(selected), 'base')
        expected = projected_manifest(receipt.field('harness'), original)
        for name, raw in expected.items():
            if apply_tools and name.startswith('tools/v8/') and name.endswith(('.py', '.sh')):
                target = clone / name
                target.parent.mkdir(parents=True, exist_ok=True)
                _ = target.write_bytes((h / name).read_bytes())
                target.chmod(0o755 if Node(raw).array()[0].text() == '100755' else 0o644)
        return context, session, original, clone

    def test_complete_product_plus_h_only_source(self) -> None:
        context, session, original, root = self.fixture()
        expected = context.expected(session, original)
        for name, raw in original.field('tracked_manifest').mapping().items():
            if not (name.startswith('tools/v8/') and name.endswith(('.py', '.sh'))):
                fields = Node(raw).array()
                self.assertEqual(expected[name], [fields[0].value, fields[2].value])
        self.assertNotIn('tools/v8/observer.gd', expected)
        self.assertNotIn('tools/v8/gate_f10_no_kill.json', expected)
        effective: dict[str, Json] = {'manifest': expected, 'sha256': manifest_hash(expected)}
        # This is only a source-shaped row, never passed off as an actual build receipt.
        source_row = Node({'root': str(root), 'source_sha': original.field('source_sha').value,
                           'source_before': original.value, 'source_kind': 'DERIVED_OBSERVER',
                           'effective_source_before': effective, 'effective_source_after': effective})
        self.assertEqual(context.validate_build(session, source_row), expected)

    def test_actual_product_omission_observer_data_and_extra_h_reject(self) -> None:
        context, session, original, root = self.fixture()
        candidate_names = ['rust/crates/sim-bridge/src/lib.rs', 'tools/v8/observer.gd',
                           'tools/v8/gate_f10_no_kill.json', 'tools/v8/foreign.py']
        for name in candidate_names:
            path = root / name
            before = path.read_bytes() if path.exists() else None
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_text('unadmitted bytes\n')
            with self.subTest(name=name), self.assertRaises(ValueError):
                _ = context.expected(session, original)
            if before is None:
                path.unlink()
            else:
                _ = path.write_bytes(before)
        name = next(iter(original.field('tracked_manifest').mapping()))
        path = root / name
        before = path.read_bytes()
        path.unlink()
        with self.assertRaises((ValueError, OSError)):
            _ = context.expected(session, original)
        _ = path.write_bytes(before)

    def test_changed_session_and_missing_h_or_mode_reject(self) -> None:
        context, session, original, root = self.fixture()
        path = root / 'tools/v8/gate.sh'
        original_mode = path.stat().st_mode
        path.chmod(original_mode ^ 0o100)
        with self.assertRaises(ValueError):
            _ = context.expected(session, original)
        path.chmod(original_mode)
        before = path.read_bytes()
        path.unlink()
        with self.assertRaises((ValueError, OSError)):
            _ = context.expected(session, original)
        _ = path.write_bytes(before)
        _ = context.session_path.write_text('{}')
        with self.assertRaises(ValueError):
            _ = context.expected(session, original)

    def test_synthetic_pair_cannot_be_promoted(self) -> None:
        from generic_build import WorkspaceBuildPair, recheck_workspace_pair
        with self.assertRaises(ValueError):
            _ = recheck_workspace_pair(WorkspaceBuildPair(Path('/not-an-actual-pair'),
                                                         'BUILD_ANTECEDENTS_ONLY', None, None))

    def test_prepared_overlay_applies_only_h_using_existing_generic_source_path(self) -> None:
        from generic_build import BuildSpec, overlay_files
        from generic_original_harness_overlay import prepare_overlay
        context, session, original, root = self.fixture(False)
        overlay = prepare_overlay(context, session, root, root.parent / 'h-only-overlay')
        self.assertEqual(freeze_source(root, original.field('source_sha').text()), original.mapping())
        unused = root.parent / 'never-executed'
        spec = BuildSpec(root, original.field('source_sha').text(), unused, '0' * 64,
                         unused, '0' * 64, unused, unused, (), (), overlay=overlay)
        changed = overlay_files(spec)
        self.assertTrue(changed)
        self.assertTrue(all(name.startswith('tools/v8/') and name.endswith(('.py', '.sh')) for name in changed))
        _ = context.expected(session, original)
        self.assertFalse((root / 'tools/v8/observer.gd').exists())
        self.assertFalse((root / 'tools/v8/gate_f10_no_kill.json').exists())
