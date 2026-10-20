"""Unit tests for tools/v8/compare_chronicles.py: equal, a grown vocabulary, and the failures."""
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from compare_chronicles import compare  # noqa: E402


def write(directory, doc):
    directory.mkdir(parents=True, exist_ok=True)
    (directory / 'chronicle_0.json').write_text(json.dumps(doc))


class CompareChronicles(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.base, self.cand = root / 'base', root / 'cand'
        self.doc = {'git_sha': 'a', 'kinds': ['born', 'died'], 'entries': [{'tick': 3, 'kind': 0}]}
        write(self.base, self.doc)

    def tearDown(self):
        self.tmp.cleanup()

    def test_equal_but_git_sha(self):
        write(self.cand, dict(self.doc, git_sha='b'))
        self.assertIn('1 files equal', compare(self.base, self.cand))

    def test_vocabulary_may_grow_at_its_end(self):
        write(self.cand, dict(self.doc, kinds=['born', 'died', 'hut_built']))
        self.assertIn('equal', compare(self.base, self.cand))

    def test_changed_entry_fails(self):
        write(self.cand, dict(self.doc, entries=[{'tick': 4, 'kind': 0}]))
        with self.assertRaises(ValueError):
            compare(self.base, self.cand)

    def test_reordered_or_shrunk_vocabulary_fails(self):
        for kinds in (['died', 'born'], ['born'], ['born', 'hut_built', 'died']):
            write(self.cand, dict(self.doc, kinds=kinds))
            with self.assertRaises(ValueError, msg=str(kinds)):
                compare(self.base, self.cand)


if __name__ == '__main__':
    unittest.main()
