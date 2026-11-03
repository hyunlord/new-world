"""Exercise the actual shell's acceptance arguments without launching producers."""
import subprocess
import tempfile
import unittest
from pathlib import Path

from generic_whole_gate_acceptance import STEPS
from generic_whole_gate_contract import stage_root
from mux2_gate_inputs import Node


class ShellPathTests(unittest.TestCase):
    def test_original_acceptance_slots_pass_canonical_stage_paths(self) -> None:
        source = (Path(__file__).parent / 'gate.sh').read_text()
        lines = [line for line in source.splitlines()
                 if line.startswith('step ') and 'generic_whole_gate_acceptance.py run ' in line]
        self.assertEqual(len(lines), len(STEPS))
        with tempfile.TemporaryDirectory(prefix='whole gate paths ') as temporary:
            output = Path(temporary).resolve()
            session = Node({'output': str(output)})
            body = 'LOG="$1/candidate"; SESSION="$1/session.json"; SESSION_SHA=fixture\n'
            body += 'step() { printf "%s\\n" "${@: -1}"; }\n'
            body += '\n'.join(lines)
            actual = subprocess.run(['/bin/bash', '-c', body, 'path-test', str(output)],
                                    capture_output=True, text=True, check=True)
            expected = [str(stage_root(session, 'candidate', number, name) / 'acceptance')
                        for number, name in STEPS.items()]
            self.assertEqual(actual.stdout.splitlines(), expected)


if __name__ == '__main__':
    _ = unittest.main()
