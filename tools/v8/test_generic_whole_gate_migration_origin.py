"""Read-only actual retained provenance checks; never decode or run saved worlds."""
import json
import tempfile
import unittest
from pathlib import Path
from generic_gate import GateContractError
from generic_process import file_hash
from generic_whole_gate_migration_origin import authenticate
from generic_whole_gate_migrations import LEGACY_ROWS, LEGACY_PROVENANCE
from mux2_gate_inputs import Json, Node


class RetainedOriginTests(unittest.TestCase):
    def test_actual_seven_origins_and_two_precise_gaps(self) -> None:
        directory = Path('/Users/rexxa/work/mux-data/stage-b')
        paths = [directory / row[0] for row in LEGACY_ROWS]
        paths.extend(directory / relative for relative in LEGACY_PROVENANCE)
        paths.extend((directory / 'base/source-sha256.json', directory / 'b4/BPTH/source-freeze.json'))
        anchors: dict[str, Json] = {str(path): file_hash(path) for path in paths}
        with tempfile.TemporaryDirectory() as temporary:
            selector = Path(temporary).resolve() / 'selector.json'
            _ = selector.write_text(json.dumps({'selectors': {'harness_root': '/Users/rexxa/work/new-world-m6b-dev'}}))
            session = Node({'selector_path': str(selector), 'selector_sha256': file_hash(selector), 'external_inputs': anchors})
            for relative, _, version, _, _ in LEGACY_ROWS:
                with self.subTest(save=relative):
                    result = authenticate(session, directory, directory / relative, version)
                    self.assertEqual(bool(result.missing), version == 8)
                    self.assertEqual(len(result.source_blobs), 0 if version == 8 else 8)
                    print(relative, result.report()['status'])
            incomplete = dict(anchors)
            del incomplete[str(directory / 'base/source-sha256.json')]
            result = authenticate(Node({'selector_path': str(selector), 'selector_sha256': file_hash(selector), 'external_inputs': incomplete}), directory, directory / LEGACY_ROWS[4][0], 7)
            self.assertTrue(result.missing)
            changed = dict(anchors)
            changed[str(directory / 'base/source-sha256.json')] = '0'*64
            with self.assertRaises(GateContractError):
                _ = authenticate(Node({'selector_path': str(selector), 'selector_sha256': file_hash(selector), 'external_inputs': changed}), directory, directory / LEGACY_ROWS[4][0], 7)

    def test_v10_missing_source_never_invents_git_parent(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            save = root / 'input.save'
            _ = save.write_bytes(b'WORLDSIM' + (10).to_bytes(4, 'little'))
            result = authenticate(Node({'external_inputs': {str(save): file_hash(save)}}), root, save, 10)
            self.assertTrue(result.missing)
            self.assertIsNone(result.source_commit)
            self.assertEqual(result.source_blobs, ())


if __name__ == '__main__':
    _ = unittest.main()
