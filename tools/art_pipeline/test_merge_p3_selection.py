# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_merge_p3_selection.py
import importlib
import unittest
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest.mock import patch


class MergeP3Test(unittest.TestCase):
    def test_review_coverage_rejects_missing_duplicate_and_unknown(self):
        module = importlib.import_module('merge_p3_selection')
        records = {'a:p3': {'id': 'a:p3', 'slot_id': 'A'}}
        verdict = dict(sprite_id='A', candidate_id='a:p3', verdict='ok', reason='',
                       observation='Readable', unfitted_acceptable=True)
        self.assertEqual(module.validate_reviews(records, [verdict]), {'a:p3': verdict})
        for rows in ([], [verdict, verdict], [dict(verdict, candidate_id='unknown')]):
            with self.assertRaises(ValueError):
                module.validate_reviews(records, rows)

    def test_review_semantics_and_unavailable(self):
        module = importlib.import_module('merge_p3_selection')
        records = {'a:p3': {'id': 'a:p3', 'slot_id': 'A', 'conversion_error': 'failed'}}
        row = dict(sprite_id='A', candidate_id='a:p3', verdict='ok', reason='artefact',
                   observation='Explicit observation', unfitted_acceptable=None)
        with self.assertRaises(ValueError):
            module.validate_reviews(records, [row])
        row.update(verdict='reject', reason='detail_destroyed_by_fit')
        with self.assertRaises(ValueError):
            module.validate_reviews(records, [row])
        row.update(verdict='unavailable', reason='')
        self.assertEqual(module.validate_reviews(records, [row]), {'a:p3': row})
        records['a:p3'].update(conversion_error=None, images={'sprite': 'usable.png'})
        with self.assertRaises(ValueError):
            module.validate_reviews(records, [row])

    def test_review_index_rejects_changed_pixels_board_and_ledger(self):
        module = importlib.import_module('merge_p3_selection')
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            ledger = root / 'ledger.json'
            ledger.write_text('{}')
            board = root / 'board.png'
            board.write_bytes(b'original board')
            pixels = root / 'sprite.png'
            pixels.write_bytes(b'original pixels')
            records = {'a:p3': dict(id='a:p3', images={'sprite': str(pixels)})}
            index = root / 'index.json'
            frozen = dict(candidates_sha256=module.sha(ledger),
                          candidate_fingerprints={'a:p3': module.candidate_fingerprint(records['a:p3'])},
                          images_sha256={'board.png': module.sha(board)})
            index.write_text(json.dumps(frozen))
            module.verify_review_index(index, ledger, records)
            for path in (pixels, board, ledger):
                before = path.read_bytes()
                path.write_bytes(b'mutated')
                with self.assertRaises(ValueError):
                    module.verify_review_index(index, ledger, records)
                path.write_bytes(before)

    def test_held_ancestor_and_cycle_are_not_eligible(self):
        module = importlib.import_module('merge_p3_selection')
        records = {'a:p3': {'id': 'a:p3', 'source_candidate': 'a'}, 'a': {'id': 'a'}}
        self.assertTrue(module.held_lineage(records['a:p3'], records, {'a'}))
        self.assertFalse(module.held_lineage(records['a:p3'], records, set()))
        records['a']['source_candidate'] = 'a:p3'
        with self.assertRaises(ValueError):
            module.held_lineage(records['a:p3'], records, set())


    def test_preserves_approvals_and_retries_installer_failures(self):
        module = importlib.import_module('merge_p3_selection')
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            def write(name, value):
                path = root / name
                path.write_text(json.dumps(value))
                return path
            def record(identity, target, attempt, delta):
                return dict(id=identity, slot_id=target, target_slot=target, sheet='N1',
                            attempt=attempt, body='', view='', activity='', frame=0, name='stone',
                            source_kind='generated', images={}, guards=dict(passed=True, errors=[]),
                            metrics=dict(mean_delta_e=delta))
            old = record('old', 'N1_01', 1, 0)
            old_ledger = write('old.json', dict(candidates=[old]))
            entry = dict(candidate_id='old', disposition='derived', ledger=str(old_ledger),
                         special_preserved_field='retain exactly')
            baseline = write('baseline.json', dict(scale=dict(adult_height=28, child_height=21),
                         slots={'N1_01': entry}, accepted={'N1_01': 'old'}, references={'frozen': 'old'}))
            candidates = [record('replace:p3', 'N1_01', 1, 0),
                          record('bad:p3', 'N1_02', 1, 1), record('good:p3', 'N1_02', 2, 2)]
            ledger = write('ledger.json', dict(scale=dict(adult_height=28, child_height=21), candidates=candidates))
            reviews = write('reviews.json', [dict(sprite_id=row['slot_id'], candidate_id=row['id'],
                         verdict='ok', reason='', observation='Readable', unfitted_acceptable=True) for row in candidates])
            board = root / 'board.png'
            board.write_bytes(b'board')
            index = write('index.json', dict(candidates_sha256=module.sha(ledger),
                          candidate_fingerprints={row['id']: module.candidate_fingerprint(row) for row in candidates},
                          images_sha256={'board.png': module.sha(board)}))
            outputs = module.Outputs(root/'selection.json', root/'envelope.json', root/'audit.json')
            def installer(request):
                selection = json.loads(request.selection.read_text())
                if selection['accepted'].get('N1_02') == 'bad:p3':
                    raise module.InstallError('Current combined guard fails')
            with patch.object(module, 'build_plan', side_effect=installer) as validate, \
                 patch.object(module, 'guarded_candidate', return_value=None), \
                 patch.object(module, 'guard_frame', return_value=SimpleNamespace(passed=True)):
                module.merge(root, baseline, ledger, [reviews], outputs, review_index=index)
            result = json.loads(outputs.selection.read_text())
            self.assertEqual(result['slots']['N1_01'], entry)
            self.assertEqual(result['references'], {'frozen': 'old'})
            self.assertEqual(result['accepted']['N1_02'], 'good:p3')
            audit = json.loads(outputs.adjudication.read_text())
            self.assertEqual([row['accepted'] for row in audit['trials']], [False, True])
            self.assertEqual(audit['reviewed_count'], 3)
            self.assertEqual(validate.call_count, 4)
            envelope = json.loads(outputs.envelope.read_text())
            self.assertEqual(len(envelope['candidate_fingerprints']), 1)
            self.assertEqual(envelope['verdicts'][0]['candidate_id'], 'good:p3')

if __name__ == '__main__':
    unittest.main()
