"""Freeze selected review pixels exactly, rejecting conflicts before any writes."""
from dataclasses import replace
import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from PIL import Image

from hair_freeze_p4 import freeze_selected_hair
from install_v4 import InstallError, InstallPlan, InstallRequest, PlannedSprite, digest
from merge_p4_selection import RecoveryRequest
from salvage_verdicts import candidate_fingerprint


class HairFreezeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.output = self.root / 'output'
        self.output.mkdir()
        self.registry_path = self.root / 'tools/art_pipeline/reviewed_hair_maps_p4.json'
        self.registry_path.parent.mkdir(parents=True)
        self.registry_path.write_text(json.dumps({'version': 1, 'frames': {}}))
        self.sprite = Image.new('RGBA', (2, 4), (100, 80, 60, 255))
        self.sprite.save(self.root / 'sprite.png')
        mask = Image.new('RGBA', (2, 4), (0, 0, 0, 255))
        mask.putpixel((0, 0), (3, 1, 0, 255))
        mask.save(self.root / 'mask.png')
        self.record = {'id': 'candidate', 'slot_id': 'physical',
                       'images': {'sprite': str(self.root / 'sprite.png'),
                                  'sprite_hair_mask': str(self.root / 'mask.png')}}
        ledger = self.output / 'ledger.json'
        ledger.write_text(json.dumps({'candidates': [self.record]}))
        review = self.output / 'review.json'
        review.write_text(json.dumps({'candidate_fingerprints': {'physical': candidate_fingerprint(self.record)},
            'verdicts': [{'candidate_id': 'candidate', 'verdict': 'ok'}]}))
        self.selection = self.output / 'selection.json'
        self.selection.write_text(json.dumps({'slots': {'V_a1_01': {'candidate_id': 'candidate',
            'disposition': 'accepted', 'ledger': str(ledger), 'ledger_sha256': digest(ledger),
            'verdict': str(review), 'verdict_sha256': digest(review)}}}))
        (self.output / 'adjudication.json').write_text(json.dumps({'selected': {'V_a1_01': 'candidate'},
            'selection_sha256': digest(self.selection)}))
        planned = PlannedSprite('V_a1_01', 'adult_a', 'S', 'idle', 0, 'accepted', 'generated:test',
            '', False, '', '', self.sprite, self.sprite, 'candidate')
        self.plan = InstallPlan(InstallRequest(self.root, self.selection), digest(self.selection),
            (planned,), {}, ((self.selection, digest(self.selection)),), ())
        self.request = RecoveryRequest(self.root, ledger, review, (review,), self.output)

    def test_copies_reviewed_bytes_and_preserves_unrelated_records(self) -> None:
        unrelated = {'mask_path': 'mask.png', 'mask_sha256': digest(self.root / 'mask.png'),
            'body': 'adult_b', 'activity': 'idle', 'size': [2, 4], 'source': 'installed_map',
            'rgba_sha256': hashlib.sha256(self.sprite.tobytes()).hexdigest(), 'verdict': 'ok'}
        self.registry_path.write_text(json.dumps({'version': 1, 'frames': {'unrelated': unrelated}}))
        self.assertEqual(freeze_selected_hair(self.request, self.plan), 1)
        registry = json.loads(self.registry_path.read_text())
        self.assertEqual(registry['frames']['unrelated'], unrelated)
        row = registry['frames']['V_a1_01']
        self.assertIn('/recovery/', row['mask_path'])
        self.assertEqual(row['verdict'], 'ok')
        self.assertEqual(row['candidate_id'], 'candidate')
        self.assertEqual((self.root / row['mask_path']).read_bytes(), (self.root / 'mask.png').read_bytes())
        self.assertEqual(row['rgba_sha256'], hashlib.sha256(self.sprite.tobytes()).hexdigest())
        self.assertEqual(row['mask_sha256'], digest(self.root / 'mask.png'))

    def test_conflicting_existing_mask_fails_before_write(self) -> None:
        old = Image.new('RGBA', (2, 4), (0, 0, 0, 255))
        old.putpixel((1, 0), (3, 0, 0, 255))
        old.save(self.root / 'old.png')
        self.registry_path.write_text(json.dumps({'version': 1, 'frames': {'other': {
            'mask_path': 'old.png', 'mask_sha256': digest(self.root / 'old.png'),
            'body': 'adult_a', 'activity': 'idle', 'size': [2, 4], 'source': 'installed_map',
            'rgba_sha256': hashlib.sha256(self.sprite.tobytes()).hexdigest(), 'verdict': 'ok'}}}))
        before = self.registry_path.read_bytes()
        with self.assertRaisesRegex(InstallError, 'Conflicting existing'):
            freeze_selected_hair(self.request, self.plan)
        self.assertEqual(self.registry_path.read_bytes(), before)
        self.assertFalse((self.registry_path.parent / 'hair_masks_p4').exists())

    def test_stale_registry_pin_is_rejected_before_write(self) -> None:
        plan = replace(self.plan, evidence=(*self.plan.evidence, (self.registry_path, digest(self.registry_path))))
        with self.assertRaisesRegex(InstallError, 'pre-freeze'):
            freeze_selected_hair(self.request, plan)
        self.assertFalse((self.registry_path.parent / 'hair_masks_p4').exists())

    def test_rejected_review_is_not_frozen(self) -> None:
        review = self.output / 'review.json'
        payload = json.loads(review.read_text())
        payload['verdicts'][0]['verdict'] = 'reject'
        review.write_text(json.dumps(payload))
        selection = json.loads(self.selection.read_text())
        selection['slots']['V_a1_01']['verdict_sha256'] = digest(review)
        self.selection.write_text(json.dumps(selection))
        (self.output / 'adjudication.json').write_text(json.dumps({'selected': {'V_a1_01': 'candidate'},
            'selection_sha256': digest(self.selection)}))
        plan = replace(self.plan, selection_sha256=digest(self.selection), evidence=())
        with self.assertRaisesRegex(InstallError, 'fresh approval'):
            freeze_selected_hair(self.request, plan)
        self.assertFalse((self.registry_path.parent / 'hair_masks_p4').exists())


if __name__ == '__main__':
    unittest.main()
