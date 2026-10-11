# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_harvest_p5.py
"""Physical-set search and inherited approval regression boundaries."""
from copy import deepcopy
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from PIL import Image
from harvest_p5 import (HarvestError, binding_matches, completion_guard, completions,
                        copy_phase, envelope_row, normalize)
from salvage_verdicts import candidate_fingerprint
from v4_contracts import WorldScale


class HarvestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.cache = Path(self.directory.name)
        self.palette = [(3, 0, 0), (120, 80, 40), (220, 180, 90)]

    def records(self, activity: str = 'walk', heights: tuple[int, ...] = (28, 28, 28, 28)) -> list[dict]:
        rows = []
        for index, height in enumerate(heights):
            image = Image.new('RGBA', (19, height), (120, 80, 40, 255))
            if index % 2:
                for x in range(5):
                    image.putpixel((x, height - 2), (220, 180, 90, 255))
            path = self.cache / f'{activity}{index}.png'
            image.save(path)
            rows.append(dict(id=f'V_a1_{index+1:02d}@1', slot_id=f'V_a1_{index+1:02d}', sheet='V_a1',
                cell=index+1, attempt=1, body='adult_a', view='S', activity=activity, frame=index,
                name=f'{activity}_{index}', source_kind='generated', raw_path=str(path), raw_box=[0,0,19,height],
                images=dict(native=str(path), sprite=str(path)), factor=1.,
                metrics=dict(mean_delta_e=1., palette_match_percent=100., binary_alpha=True, berry_texels=0, fire_texels=0),
                guards=dict(passed=True, errors=[], warnings=[]), review_status='ok', conversion_error=None))
        return rows

    def test_complete_lower_ranked_set_is_not_exhausted_by_mixed_trials(self) -> None:
        ledger = json.loads(Path('docs/art/production4/recovery/candidates.json').read_text())
        rows = [normalize(r, WorldScale()) for r in ledger['candidates']
                if r['sheet'] == 'V_c2' and r['activity'] == 'run' and r['attempt'] == 3]
        found = completions(rows, {f:f'V_c2_{f+5:02d}' for f in range(4)}, self.cache, self.palette)
        self.assertEqual(found[0].kind, 'C0')
        self.assertEqual(completion_guard(found[0], WorldScale()), ())
        self.assertEqual({r['attempt'] for r in found[0].rows}, {3})

    def test_mixed_physical_sets_never_form_a_completion(self) -> None:
        rows = self.records()
        rows[1]['attempt'] = 2
        with self.assertRaises(HarvestError):
            completions(rows, dict(enumerate('abcd')), self.cache, self.palette)

    def test_c1_replaces_usable_frame_breaking_head_top(self) -> None:
        rows = self.records(heights=(28, 30, 28, 28))
        results = completions(rows, dict(enumerate('abcd')), self.cache, self.palette)
        self.assertIn('head_top', completion_guard(results[0], WorldScale()))
        passing = [c for c in results if not completion_guard(c, WorldScale())]
        self.assertEqual(passing[0].kind, 'C1')
        self.assertEqual(passing[0].rows[1]['source_candidate'], rows[3]['id'])
        self.assertEqual(passing[0].rows[1]['derivation_rule'], 'same_cycle_phase_copy')

    def test_two_pose_only_for_carry(self) -> None:
        for activity in ('walk', 'run', 'carry', 'carry_wood'):
            rows = self.records(activity)[:2]
            results = completions(rows, dict(enumerate('abcd')), self.cache, self.palette)
            self.assertEqual(bool(results), activity in ('carry', 'carry_wood'))
            if results:
                self.assertEqual(results[0].kind, 'C2')
                self.assertEqual(results[0].copies, 2)
                self.assertTrue(all(r.get('derivation_rule', 'same_cycle_phase_copy') == 'same_cycle_phase_copy' for r in results[0].rows))

    def test_no_procedural_or_cross_set_derivation(self) -> None:
        rows = self.records()
        with self.assertRaises(HarvestError):
            copy_phase(rows[0], rows[1], self.cache, self.palette, WorldScale())
        rows[2]['attempt'] = 2
        with self.assertRaises(HarvestError):
            copy_phase(rows[0], rows[2], self.cache, self.palette, WorldScale())

    def test_changed_record_loses_verdict(self) -> None:
        row = self.records()[0]
        fingerprints = {row['id']: candidate_fingerprint(row)}
        self.assertTrue(binding_matches(row, fingerprints))
        row['frame'] = 2
        self.assertFalse(binding_matches(row, fingerprints))

    def test_noncycle_requires_every_expected_frame(self) -> None:
        rows = self.records('drink')[:1]
        self.assertEqual(completions(rows, {0:'first', 1:'second'}, self.cache, self.palette), ())

    def test_completed_installation_evidence_cannot_be_overwritten(self) -> None:
        from merge_p5_selection import prepare
        from install_v4 import InstallError
        (self.cache / 'adjudication.json').write_text('{}')
        with self.assertRaises(InstallError):
            prepare(output=self.cache)

    def test_copy_envelope_names_approved_source(self) -> None:
        rows = self.records()
        derived = copy_phase(rows[0], rows[2], self.cache, self.palette, WorldScale())
        approval = dict(sprite_id=rows[2]['slot_id'], candidate_id=rows[2]['id'], verdict='ok', reason='')
        result = envelope_row(derived, {rows[2]['id']: approval})
        self.assertEqual(result['inherited_from'], rows[2]['id'])
        self.assertEqual(result['candidate_id'], derived['id'])
        self.assertEqual(result['verdict'], 'ok')
        self.assertEqual(approval['candidate_id'], rows[2]['id'])


if __name__ == '__main__':
    unittest.main()
