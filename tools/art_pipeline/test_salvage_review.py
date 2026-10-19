"""Review evidence must preserve pixels and not imply technical acceptance."""
import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

import numpy as np
from PIL import Image

from salvage_review import ReviewCandidate, ReviewInput, choose_candidate, enlarge, load_review, render_sheet


class SalvageReviewTests(unittest.TestCase):
    def test_exact_nearest_enlargement(self) -> None:
        source = Image.new('RGBA', (2, 1))
        source.putdata([(10, 20, 30, 255), (80, 90, 100, 0)])
        result = np.asarray(enlarge(source, 8))
        self.assertEqual(result.shape, (8, 16, 4))
        np.testing.assert_array_equal(result[0, :8], np.tile((10, 20, 30, 255), (8, 1)))
        np.testing.assert_array_equal(result[0, 8:], np.tile((80, 90, 100, 0), (8, 1)))

    def test_selection_and_ranked_preview_are_distinct(self) -> None:
        a = ReviewCandidate('a', 'U1_01', '', None, None, None, Path('a.png'), (2, 0, 1))
        b = ReviewCandidate('b', 'U1_01', '', None, None, None, Path('b.png'), (1, 0, 8))
        self.assertEqual(choose_candidate(ReviewInput((a,b), (), ()), 'U1_01'), b)
        self.assertEqual(choose_candidate(ReviewInput((a,b), (('U1_01','a'),), ()), 'U1_01'), a)
        self.assertIsNone(choose_candidate(ReviewInput((a,b), (), ()), 'U1_02'))

    def test_schema_projection_and_full_sheet_render(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'cell.png'
            Image.new('RGBA', (16,16), (20,40,60,255)).save(source)
            path = root / 'candidates.json'
            path.write_text(json.dumps({'schema_version':1, 'candidates':[{
                'id':'U1_01@1','slot_id':'U1_01','body':'','images':{
                    key:str(source) for key in ('raw','unfitted','native','sprite')},
                'guards':{'errors':['not accepted'], 'warnings':[]},
                'metrics':{'mean_delta_e':4}}], 'selected':{},'references':{}}))
            review = load_review(path)
            self.assertEqual(review.candidates[0].priority, (1,0,4.0))
            self.assertEqual(review.source_digest, hashlib.sha256(path.read_bytes()).hexdigest())
            output = render_sheet(review, 'U1', root / 'review.png', (50,70,30))
            with Image.open(output) as result:
                self.assertGreater(result.height, 16 * 100)
                self.assertEqual(result.mode, 'RGB')
                pixels = np.asarray(result)
                self.assertTrue(np.any(np.all(pixels == (50,70,30), axis=-1)))


if __name__ == '__main__':
    unittest.main()
