# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_journal
"""Journal ordering and immutable namespace boundary regressions, no native claims."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from mux2_gate_inputs import Json, Node

from generic_process import atomic_json
from generic_whole_gate_journal import (
    anchored,
    begin,
    entries,
    publish,
    publish_terminal,
    recheck,
)


class JournalTests(unittest.TestCase):
    def test_terminal_publication_rejects_changed_intended_mode(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            original = Node(anchored(root))
            terminal = root/'stage.json'

            def tampered(path: Path, value: dict[str, Json], *, mode: int | None = None) -> None:
                atomic_json(path, value, mode=mode)
                path.chmod(0o777)

            with (patch('generic_whole_gate_journal.atomic_json', side_effect=tampered),
                  self.assertRaises(ValueError)):
                publish_terminal(root, terminal, {'status': 'PASS'}, original)

    def test_swapped_first_step_rejected_before_launch(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            session = Node({'output': str(Path(temporary).resolve())})
            with self.assertRaises(ValueError):
                _ = begin(session, 'candidate', 2)

    def test_returned_terminal_cannot_be_rewritten(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            session = Node({'output': str(root)})
            step = root/'step'
            step.mkdir()
            ordinal, previous = begin(session, 'candidate', 1)
            terminal = step/'stage.json'
            atomic_json(terminal, {'step': 1, 'state': 'RETURNED', 'producer_anchors': {}})
            publish(session, 'candidate', 1, terminal, ordinal, previous)
            _ = terminal.write_text('{"step":1,"state":"forged"}')
            with self.assertRaises(ValueError):
                _ = entries(session, 'candidate')

    def test_callback_cannot_adopt_new_file_directory_or_mode(self) -> None:
        for mutation in ('file', 'directory', 'mode'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                file = root/'original'
                _ = file.write_text('original')
                file.chmod(0o600)
                original = Node(anchored(root))
                match mutation:
                    case 'file': _ = (root/'late').write_text('injected')
                    case 'directory': (root/'late').mkdir()
                    case 'mode': file.chmod(0o777)
                with self.assertRaises(ValueError):
                    recheck(root, original)


if __name__ == '__main__':
    _ = unittest.main()
