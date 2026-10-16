"""Recovery ranking replaces existing targets and rolls failed trials back."""
import unittest
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock

from install_v4 import InstallError
from merge_p4_selection import choose_lowest, merge, RecoveryRequest
from test_install_v4 import reviewed_fixture
from salvage_verdicts import candidate_fingerprint, sha


def candidate(identity, error, activity='idle'):
    return dict(id=identity, metrics={'mean_delta_e': error}, body='adult_a',
                view='S', activity=activity, sheet=identity, attempt=1)


class RecoverySelectionTests(unittest.TestCase):
    def test_lowest_error_replaces_existing_target(self) -> None:
        options = {'target': [candidate('expensive', 8),
                              candidate('best', 2)]}
        verify = Mock()
        chosen, trials = choose_lowest(options, verify)
        self.assertEqual(chosen['target']['id'], 'best')
        self.assertTrue(trials[0]['accepted'])

    def test_guard_failure_rolls_back_before_next_target(self) -> None:
        options = {'a': [candidate('fails', 1)],
                   'b': [candidate('passes', 2, 'walk')]}
        def verify(chosen):
            if 'a' in chosen:
                raise InstallError('cycle mismatch')
        chosen, trials = choose_lowest(options, verify)
        self.assertEqual(set(chosen), {'b'})
        self.assertFalse(trials[0]['accepted'])
        self.assertEqual(trials[0]['reason'], 'cycle mismatch')

    def test_next_lowest_passing_candidate_is_selected(self) -> None:
        options = {'target': [candidate('fails', 1),
                              candidate('passes', 2, 'walk')]}
        def verify(chosen):
            if chosen['target']['id'] == 'fails':
                raise InstallError('physical group mismatch')
        chosen, _ = choose_lowest(options, verify)
        self.assertEqual(chosen['target']['id'], 'passes')

    def test_complete_cycle_is_checked_together(self) -> None:
        first = candidate('first', 1)
        second = candidate('second', 2)
        first['sheet'] = second['sheet'] = 'one_sheet'
        def verify(chosen):
            if set(chosen) != {'frame1', 'frame2'}:
                raise InstallError('incomplete cycle')
        chosen, _ = choose_lowest({'frame1': [first], 'frame2': [second]}, verify)
        self.assertEqual(set(chosen), {'frame1', 'frame2'})

    def test_same_source_second_choice_survives_cycle_failure(self) -> None:
        low = candidate('low', 1)
        valid = candidate('valid', 2)
        peer = candidate('peer', 1)
        low['sheet'] = valid['sheet'] = peer['sheet'] = 'same_sheet'
        def verify(chosen):
            if chosen['frame1']['id'] == 'low':
                raise InstallError('cycle silhouette incompatibility')
        chosen, _ = choose_lowest({'frame1': [low, valid], 'frame2': [peer]}, verify)
        self.assertEqual(chosen['frame1']['id'], 'valid')
        self.assertEqual(chosen['frame2']['id'], 'peer')

    def test_three_failures_record_untried_alternatives(self) -> None:
        def verify(chosen):
            raise InstallError('guard failure')
        chosen, trials = choose_lowest({'frame1': [candidate(str(n), n) for n in range(5)]}, verify)
        self.assertFalse(chosen)
        self.assertEqual(len([row for row in trials if row.get('candidate_id')]), 3)
        self.assertEqual(trials[-1]['reason'], 'three_failed_trials')
        self.assertEqual(trials[-1]['untried_candidate_ids'], ['3', '4'])

    def test_real_installer_preserves_audited_fallback_state(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            request = reviewed_fixture(root)
            selection = json.loads(request.selection.read_text())
            for name in ('installation_selection.json', 'reprocess/selection.json'):
                path = root / 'docs/art/production3' / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(json.dumps(selection))
            holds = root / 'docs/art/production2/fill_sheets.json'
            holds.parent.mkdir(parents=True)
            holds.write_text('{}')
            manifest = root / 'assets/art/manifest.json'
            manifest.write_text(json.dumps({'assets': [{
                'sprite_id': 'N3_01', 'path': 'raw.png', 'native_path': 'raw.png',
                'qa_status': 'fallback', 'source': 'generated:old_rejected',
                'fallback_kind': 'retained_installed', 'drawable': True}], 'suppressed': {}}))
            ledger = json.loads((root / 'ledger.json').read_text())
            record = ledger['candidates'][0]
            record.update(id='N3_02@1', slot_id='N3_02', target_slot='N3_02', cell=2, name='stone')
            (root / 'recovery.json').write_text(json.dumps(ledger))
            index = root / 'index.json'
            index.write_text(json.dumps({'candidates_sha256': sha(root / 'recovery.json'),
                'candidate_fingerprints': {record['id']: candidate_fingerprint(record)},
                'images_sha256': {'raw.png': sha(root / 'raw.png')}}))
            review = root / 'review.json'
            review.write_text(json.dumps([{'sprite_id': 'N3_02', 'candidate_id': 'N3_02@1',
                'verdict': 'ok', 'reason': '', 'observation': 'Recognizable stone',
                'unfitted_acceptable': None}]))
            p2_record = dict(record, id='prior_p2', slot_id='N3_03', target_slot='N3_03', cell=3)
            p2_ledger = root / 'p2_ledger.json'
            p2_ledger.write_text(json.dumps(dict(ledger, candidates=[p2_record])))
            p2_review = root / 'p2_review.json'
            p2_review.write_text(json.dumps({'candidate_fingerprints': {'N3_03': candidate_fingerprint(p2_record)},
                'verdicts': [{'sprite_id': 'N3_03', 'candidate_id': 'prior_p2', 'verdict': 'ok'}],
                'prereview_bindings_sha256': {str(manifest): 'historical_pre_install_manifest'}}))
            p2_selection = root / 'docs/art/production4/recovery/selection_run/selection.json'
            p2_selection.parent.mkdir(parents=True)
            p2_decision = dict(candidate_id='prior_p2', disposition='accepted', ledger=str(p2_ledger),
                ledger_sha256=sha(p2_ledger), verdict=str(p2_review), verdict_sha256=sha(p2_review))
            p2_selection.write_text(json.dumps(dict(selection, slots={'N3_03': p2_decision})))
            installed = json.loads(manifest.read_text())
            installed['assets'].append(dict(installed['assets'][0], sprite_id='N3_03',
                qa_status='accepted', source='generated:prior_p2', fallback_kind='', candidate_id='prior_p2'))
            manifest.write_text(json.dumps(installed))
            before = manifest.read_bytes()
            plan, report = merge(RecoveryRequest(root, Path('recovery.json'), Path('index.json'),
                (Path('review.json'),), Path('output')))
            by_id = {sprite.slot_id: sprite for sprite in plan.sprites}
            self.assertEqual(by_id['N3_01'].qa_status, 'fallback')
            self.assertEqual(by_id['N3_01'].source, 'generated:old_rejected')
            self.assertEqual(by_id['N3_02'].qa_status, 'accepted')
            self.assertEqual(by_id['N3_03'].candidate_id, 'prior_p2')
            self.assertEqual(report['selected'], {'N3_02': 'N3_02@1'})
            self.assertEqual(manifest.read_bytes(), before)
            self.assertFalse((root / 'assets/art/production').exists())
            explicit = RecoveryRequest(root, Path('recovery.json'), Path('index.json'),
                (Path('review.json'),), Path('explicit_output'),
                (p2_selection, root / 'docs/art/production3/installation_selection.json'))
            explicit_plan, _ = merge(explicit)
            self.assertEqual({sprite.slot_id for sprite in explicit_plan.sprites}, set(by_id))


if __name__ == '__main__':
    unittest.main()
