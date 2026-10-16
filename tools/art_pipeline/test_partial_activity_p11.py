"""Offline authorization validation only; never reads or generates actual blind crops."""
import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from activity_rules import EvidenceError
from plan_activity_blind_p11 import terminal_shortage


class TerminalShortageTests(unittest.TestCase):
    def test_exact_record_and_tampering(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            capture = root / 'capture.json'
            capture.write_text('{}')
            image = root / 'capture.png'
            image.write_bytes(b'synthetic fixture')
            counts = [0, 1, 0, 2, 0, 1, 1]
            data = {'schema': 1, 'terminal_shortage': True, 'authorized_by': 'leader',
                    'remaining_capture_work': False, 'reason': 'allowed trials completed',
                    'capture_order': [str(capture)], 'counts': counts, 'required_count': 14,
                    'source_hashes': {str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                                      for p in (capture, image)}}
            record = root / 'terminal.json'
            record.write_text(json.dumps(data))
            self.assertEqual(terminal_shortage(record, [capture], counts)['reason'], data['reason'])
            for key, value in [('terminal_shortage', False), ('authorized_by', 'other'),
                               ('remaining_capture_work', True), ('reason', ''),
                               ('capture_order', []), ('counts', [2] * 7), ('required_count', 5)]:
                invalid = dict(data, **{key: value})
                record.write_text(json.dumps(invalid))
                with self.subTest(key=key), self.assertRaises(EvidenceError):
                    terminal_shortage(record, [capture], counts)
            record.write_text(json.dumps(data))
            image.write_bytes(b'changed after decision')
            with self.assertRaises(EvidenceError):
                terminal_shortage(record, [capture], counts)

    def test_complete_selection_cannot_be_called_partial(self):
        with tempfile.TemporaryDirectory() as temporary:
            record = Path(temporary) / 'terminal.json'
            record.write_text(json.dumps({'schema': 1, 'terminal_shortage': True,
                'authorized_by': 'leader', 'remaining_capture_work': False, 'reason': 'x',
                'capture_order': [], 'counts': [2] * 7, 'required_count': 14}))
            with self.assertRaises(EvidenceError):
                terminal_shortage(record, [], [2] * 7)


if __name__ == '__main__':
    unittest.main()
