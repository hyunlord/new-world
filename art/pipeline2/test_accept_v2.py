# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run --with scikit-image python -m unittest art.pipeline2.test_accept_v2
"""Acceptance must fail closed on provenance, style or semantic gaps."""
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

from art.pipeline2 import accept_v2
from art.pipeline2.accept_provenance_v2 import build_sidecar
from art.pipeline2.judge import ModelScores


class AcceptanceTests(unittest.TestCase):
    def test_wrong_raw_or_cross_family_references_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = [root / name for name in ('candidate', 'a', 'b', 'card', 'working')]
            for path in paths:
                _ = path.write_bytes(path.name.encode())
            raw, a, b, card, working = paths
            refs = ((a, accept_v2.sha(a)), (b, accept_v2.sha(b)))
            review = accept_v2.Review(raw, accept_v2.sha(raw), refs, card,
                                      accept_v2.sha(card), ModelScores(4, 4, 4, 4, 'Test only.'))
            candidate = accept_v2.Candidate('axe', 'items', 'items', 1, raw,
                                           accept_v2.sha(raw), working, accept_v2.sha(working))
            allowed = {accept_v2.sha(a), accept_v2.sha(b)}
            self.assertEqual(accept_v2.review_problem(candidate, review, allowed), None)
            bad = replace(review, raw_sha256='wrong')
            self.assertEqual(accept_v2.review_problem(candidate, bad, allowed), 'RAW_IDENTITY_MISMATCH')
            self.assertEqual(accept_v2.review_problem(candidate, review, {accept_v2.sha(a)}),
                             'SAME_FAMILY_REFERENCES_MISMATCH')
            _ = card.write_bytes(b'changed')
            self.assertEqual(accept_v2.review_problem(candidate, review, allowed), 'SCORECARD_HASH_MISMATCH')

    def test_promotion_preserves_original_pixels_and_sidecar(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            working, evidence, raw = (root / name for name in ('working', 'evidence', 'raw'))
            for path in (working, evidence, raw):
                _ = path.write_bytes(path.name.encode())
            candidate = accept_v2.Candidate('meadow', 'ground', 'ground', 1, raw,
                                           accept_v2.sha(raw), working, accept_v2.sha(working))
            destination = root / 'art/style05/env/ground/meadow.png'
            destination.parent.mkdir(parents=True)
            _ = destination.write_bytes(b'historical original')
            _ = destination.with_suffix('.json').write_text('{"old": true}')
            old_hash = accept_v2.sha(destination)
            with patch.object(accept_v2, 'ROOT', root):
                accept_v2.promote(candidate, '{"status": "accepted under judge v2, TRIAL2b"}\n')
            archive = root / 'art/style05/env/history/trial2/ground' / ('meadow-' + old_hash[:12])
            self.assertEqual(archive.with_suffix('.png').read_bytes(), b'historical original')
            self.assertEqual(archive.with_suffix('.json').read_text(), '{"old": true}')
            self.assertEqual(destination.read_bytes(), working.read_bytes())
            self.assertIn('accepted under judge v2, TRIAL2b', destination.with_suffix('.json').read_text())

    def test_missing_or_changed_processing_provenance_cannot_promote(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            raw, working, evidence = (root / name for name in ('raw', 'working.png', 'evidence'))
            for path in (raw, working, evidence):
                _ = path.write_bytes(path.name.encode())
            candidate = accept_v2.Candidate('axe', 'items', 'items', 1, raw,
                                           accept_v2.sha(raw), working, accept_v2.sha(working))
            review = accept_v2.Review(raw, accept_v2.sha(raw), (), evidence,
                                      accept_v2.sha(evidence), ModelScores(4, 4, 4, 4, 'Test only.'))
            with self.assertRaisesRegex(ValueError, 'Missing required processing sidecar'):
                build_sidecar(candidate, review, evidence)
            provenance = root / 'provenance.json'
            _ = provenance.write_text('{"prompt": "original"}')
            pinned = replace(candidate, provenance=provenance, provenance_sha256=accept_v2.sha(provenance))
            _ = provenance.write_text('{"prompt": "changed"}')
            with self.assertRaisesRegex(ValueError, 'Processing provenance changed'):
                build_sidecar(pinned, review, evidence)

    def test_c2_style_pass_is_independent_of_semantic_promotion(self) -> None:
        self.assertTrue(accept_v2.accepted(True, True))
        self.assertFalse(accept_v2.accepted(False, True))
        self.assertFalse(accept_v2.accepted(True, False))
        style_pass = accept_v2.accepted(True, True)
        self.assertFalse(accept_v2.promotion_eligible(style_pass, None))
        self.assertFalse(accept_v2.promotion_eligible(style_pass, False))
        self.assertTrue(accept_v2.promotion_eligible(style_pass, True))
        self.assertFalse(accept_v2.promotion_eligible(False, True))



if __name__ == '__main__':
    unittest.main()
