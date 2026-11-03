"""Private source fixtures only; no Godot, Cargo or native owner launch."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError
from generic_identity_overlay import EXPLICIT, compatible
from generic_process import file_hash
from generic_whole_gate_play_plan import build_plan
from mux2_gate_inputs import Node


class PlayHarnessTests(unittest.TestCase):
    def session(self, root: Path) -> Node:
        paths = [root/'harness/tools/play.sh', root/'product/tools/play.sh']
        for path in paths:
            path.parent.mkdir(parents=True)
            _ = path.write_text('#!/bin/bash\n# admitted H protocol launcher fixture\n')
            path.chmod(0o755)
        digest = file_hash(paths[0])
        return Node({'harness': {'root': str(root/'harness'), 'inputs': {'tools/play.sh': digest}},
            'arms': {'base': {'root': str(root/'product'), 'source_inputs': {'tools/play.sh': digest}}},
            'output': str(root/'out')})

    def test_exact_launcher_in_identity_selection_and_closed_namespace(self) -> None:
        self.assertIn('tools/play.sh', EXPLICIT)
        self.assertTrue(compatible('tools/play.sh'))
        for name in ('tools/other.sh', 'tools/play.py', 'scripts/play.sh', 'tools/play.sh/evil'):
            self.assertFalse(compatible(name))

    def test_old_product_launcher_rejects_before_output_or_environment(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            session = self.session(root)
            launcher = root/'product/tools/play.sh'
            _ = launcher.write_text('#!/bin/bash\n# old direct Godot version launcher\n')
            session.field('arms').field('base').field('source_inputs').mapping()['tools/play.sh'] = file_hash(launcher)
            output = root/'out/steps/base/21-play_check/play-owned'
            with patch('generic_whole_gate_play_plan.load_session', return_value=session), \
                 patch('generic_whole_gate_play_plan.process_spec', side_effect=AssertionError('environment reached')), \
                 self.assertRaisesRegex(GateContractError, 'launcher'):
                _ = build_plan(root/'session.json', 'a'*64, 'base', output)
            self.assertFalse((root/'out').exists())

    def test_exact_h_launcher_source_and_mode_required(self) -> None:
        from generic_play_harness import check_launcher
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            session = self.session(root)
            check_launcher(session, 'base')
            launcher = root/'product/tools/play.sh'
            launcher.chmod(0o644)
            with self.assertRaises(GateContractError): check_launcher(session, 'base')
            launcher.chmod(0o755)
            del session.field('arms').field('base').field('source_inputs').mapping()['tools/play.sh']
            with self.assertRaises(ValueError): check_launcher(session, 'base')

    def test_changed_h_or_symlink_cannot_replace_launcher(self) -> None:
        from generic_play_harness import check_launcher
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            session = self.session(root)
            launcher = root/'product/tools/play.sh'
            launcher.unlink();launcher.symlink_to(root/'harness/tools/play.sh')
            with self.assertRaises(GateContractError): check_launcher(session, 'base')
            launcher.unlink();_ = launcher.write_bytes((root/'harness/tools/play.sh').read_bytes());launcher.chmod(0o755)
            _ = (root/'harness/tools/play.sh').write_text('changed H')
            with self.assertRaises(GateContractError): check_launcher(session, 'base')


if __name__ == '__main__':
    _ = unittest.main()
