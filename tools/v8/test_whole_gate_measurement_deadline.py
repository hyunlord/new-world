"""Private files, injected clocks/samples and intercepted launch only; no native child."""
import json
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_whole_gate as gate
import generic_whole_gate_perf_admission as admission
from core_p7_admission import AdmissionError
from generic_gate import GateContractError
from generic_process import ProcessSpec, file_hash
from generic_whole_gate_environment import process_spec
from generic_whole_gate_source import prepare_session
from mux2_gate_inputs import Node


class LaunchIntercept(RuntimeError):
    """Intentional stop at the actual launch boundary; never a successful receipt."""


class DeadlineTests(unittest.TestCase):
    def session(self, root: Path) -> Node:
        selector = root/'selector.json'
        _ = selector.write_text('{"measurement_deadline_epoch":4000.0}')
        bash = Path('/bin/bash').resolve()
        return Node({'measurement_deadline_epoch': 4000., 'selector_path': str(selector),
            'selector_sha256': file_hash(selector), 'materialization': {}, 'output': str(root/'out'),
            'environment': {}, 'external_inputs': {}, 'bash': {'path': str(bash), 'sha256': file_hash(bash)},
            'arms': {'base': {'root': str(root), 'source_inputs': {}, 'environment': {}}}})

    def test_selector_and_reflected_deadline_cannot_drift(self) -> None:
        with TemporaryDirectory() as directory:
            session = self.session(Path(directory).resolve())
            self.assertEqual(admission.session_deadline(session), 4000.)
            session.mapping()['measurement_deadline_epoch'] = 5000.
            with self.assertRaises(ValueError):
                _ = admission.session_deadline(session)
            session.mapping()['measurement_deadline_epoch'] = 4000.
            _ = Path(session.field('selector_path').text()).write_text('{"measurement_deadline_epoch":4000.0} ')
            with self.assertRaises(GateContractError):
                _ = admission.session_deadline(session)

    def test_only_verified_session_supplies_environment_deadline(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            session = self.session(root)
            with patch.dict('os.environ', {admission.DEADLINE_ENV: '999999'}):
                spec = process_spec(session, 'base', ('original', '--unchanged'), root/'process', {}, None)
            self.assertEqual(dict(spec.environment)[admission.DEADLINE_ENV], '4000.0')
            self.assertEqual(spec.argv[-2:], ('original', '--unchanged'))
            for node in (session.field('environment'), session.field('arms').field('base').field('environment')):
                node.mapping()[admission.DEADLINE_ENV] = '999999'
                with self.assertRaises(GateContractError):
                    _ = process_spec(session, 'base', ('original',), root/'process', {}, None)
                del node.mapping()[admission.DEADLINE_ENV]
            with self.assertRaises(GateContractError):
                _ = process_spec(session, 'base', ('original',), root/'process', {admission.DEADLINE_ENV: '999999'}, None)

    def test_prepare_rejects_deadline_before_materialization(self) -> None:
        with TemporaryDirectory() as directory:
            path = Path(directory)/'selector.json'
            for value in (True, 0, -1, float('nan'), float('inf'), 99.):
                selector = {name: None for name in ('selectors', 'output', 'layers', 'python', 'bash',
                    'environment', 'external_inputs', 'acceptance', 'observations')}
                payload = json.dumps(dict(selector, schema='whole_gate.v1', measurement_deadline_epoch=value))
                _ = path.write_text(payload)
                with self.subTest(value=value), patch('generic_whole_gate_source.time.time', return_value=100.), \
                        patch('generic_cli_prepare.prepare_cli', side_effect=AssertionError('must not materialize')) as prepare:
                    with self.assertRaises(ValueError):
                        _ = prepare_session(path)
                    prepare.assert_not_called()

    def test_perf_shell_admission_only_preserves_child_and_failure(self) -> None:
        # Exercise the exact checked-in shell function with shell builtins replacing
        # Python admission and the measuring executable. No native process is launched.
        source = Path(__file__).with_name('perf.sh').read_text()
        function = source[source.index('admitted_run() {'):source.index('\n{ echo "role')]
        for failure in (0, 75):
            shell = '\n'.join(('set -eu', 'SCRIPT_DIR=/exact/harness', 'OUT=/exact/output',
                'WORLDSIM_MEASUREMENT_DEADLINE_EPOCH=4000',
                f'python3() {{ printf "admission:%s\\n" "$*"; return {failure}; }}',
                'original_child() { printf "child:%s\\n" "$*"; }', function,
                'admitted_run original-label original_child "argument with spaces" --unchanged'))
            result = subprocess.run(['/bin/bash', '-c', shell], capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, failure)
            lines = result.stdout.splitlines()
            self.assertEqual(lines[0], 'admission:/exact/harness/generic_whole_gate_perf_admission.py '
                + '/exact/output/admission.jsonl original-label 4000')
            self.assertEqual(lines[1:], [] if failure else ['child:argument with spaces --unchanged'])

    def test_nav_wait_precedes_unchanged_launch_or_prevents_it(self) -> None:
        for allowed in (False, True):
            with self.subTest(allowed=allowed):
                self.nav_case(allowed)

    def nav_case(self, allowed: bool) -> None:
        original_admit = admission.admit_measurement
        with TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            session = self.session(root)
            elapsed: list[float] = []
            values = iter(((9.42, 2., 3.), (7.9 if allowed else 9.42, 2., 3.)))
            def admit(log: Path, label: str, deadline: float) -> tuple[float, float, float]:
                return original_admit(log, label, deadline if allowed else 1000.,
                    sample=lambda: next(values), clock=lambda: sum(elapsed), sleep=elapsed.append)
            seen: list[ProcessSpec] = []
            def intercept(spec: ProcessSpec) -> None:
                seen.append(spec)
                raise LaunchIntercept('private launch interception')
            with patch.object(gate, 'load_session', return_value=session), \
                    patch.object(gate, 'begin', return_value=(0, None)), \
                    patch.object(gate, 'stage_root', return_value=root/'stage'), \
                    patch('generic_whole_gate_observations.route', return_value={}), \
                    patch.object(admission, 'admit_measurement', side_effect=admit), \
                    patch.object(gate, 'execute', side_effect=intercept), \
                    self.assertRaises(LaunchIntercept if allowed else AdmissionError):
                _ = gate.run_step(root/'session', 'a'*64, 'base', 27, 'nav_bench',
                                      ('original-perf', 'nav-bench', '--out', 'original-output'))
            self.assertEqual(len(seen), int(allowed))
            self.assertEqual(elapsed, [900])
            if allowed:
                self.assertEqual(seen[0].argv[-4:], ('original-perf', 'nav-bench', '--out', 'original-output'))
            rows = [json.loads(line) for line in (root/'stage/measurement-admission.jsonl').read_text().splitlines()]
            self.assertEqual(len(rows), 2)
            self.assertFalse((root/'stage/stage.json').exists())
            self.assertFalse((root/'stage/process/process.json').exists())


if __name__ == '__main__':
    _ = unittest.main()
