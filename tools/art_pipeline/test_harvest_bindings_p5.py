"""Mutation regressions for the final approval boundary."""
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from harvest_bindings_p5 import freeze, verify_prepared, bind_review
from harvest_integrity_p5 import content_changed
from install_v4 import InstallError
from salvage_verdicts import atomic_json


class BindingTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.output = Path(self.temp.name)
        atomic_json(self.output/'installer_candidates.json', {'candidates': []})
        self.board = self.output/'board.png'
        self.gif = self.output/'animation.gif'
        self.board.write_bytes(b'board')
        self.gif.write_bytes(b'animation')
        self.proposal = dict(id='group',candidate_ids=[],board=str(self.board),gif=str(self.gif))
        freeze(self.output,[self.proposal],{})

    def test_mutated_ledger_cannot_be_rebound(self):
        atomic_json(self.output/'installer_candidates.json', {'candidates': [], 'tampered':True})
        with self.assertRaises(InstallError):
            verify_prepared(self.output,[self.proposal],{}, {})

    def test_mutated_candidate_order_rejected(self):
        proposal = dict(self.proposal,candidate_ids=['new'])
        with self.assertRaisesRegex(InstallError,'order changed'):
            verify_prepared(self.output,[proposal],{}, {})

    def test_changed_board_or_animation_rejected(self):
        for path in (self.board,self.gif):
            original = path.read_bytes()
            path.write_bytes(original+b'changed')
            with self.assertRaises(InstallError):
                verify_prepared(self.output,[self.proposal],{}, {})
            path.write_bytes(original)

    def test_review_bytes_and_binding_bytes_are_immutable(self):
        review = self.output/'reviews.json'
        atomic_json(review, {'verdicts':[]})
        bind_review(self.output,review)
        original = review.read_bytes()
        review.write_bytes(original+b' ')
        with self.assertRaises(InstallError):
            bind_review(self.output,review)
        review.write_bytes(original)
        binding = self.output/'animation/candidate_bindings.json'
        binding.write_bytes(binding.read_bytes()+b' ')
        with self.assertRaises(InstallError):
            bind_review(self.output,review)

    def test_covered_harvest_frame_rejects_changed_rendered_mask(self):
        from copy import deepcopy
        from unittest.mock import patch
        import harvest_integrity_p5 as audit
        root = Path.cwd()
        manifest = {r['sprite_id']:r for r in json.loads((root/'assets/art/manifest.json').read_text())['assets']
                    if r.get('sprite_id') and r.get('drawable',True)}
        content = deepcopy(audit.current_content(root,manifest))
        content['V_a2_05']['mask']['rgba_sha256'] = '0'*64
        with patch.object(audit,'current_content',return_value=content), patch.object(audit,'atomic_json'):
            result = audit.audit_all(root)
        self.assertFalse(result['passed'])
        self.assertIn('V_a2_05:installed_mask_changed',result['errors'])

    def test_same_identity_changed_native_or_mask_is_changed(self):
        row = dict(candidate_id='same',source='same',qa_status='accepted')
        baseline = {'frame': {'native_path':'original','mask':'original'}}
        for field in ('native_path','mask'):
            current = {'frame':dict(baseline['frame'],**{field:'changed'})}
            self.assertTrue(content_changed('frame',row,row,current,baseline))
        self.assertFalse(content_changed('frame',row,row,baseline,baseline))


if __name__ == '__main__':
    unittest.main()
