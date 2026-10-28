"""Keep legacy performance mapping standalone while dispatching UI in its caller."""
from __future__ import annotations

import ast
import json
import os
import subprocess
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch

import core_shadow_freeze as freeze
import core_ui_successor as ui


def evaluation_tree() -> ast.Module:
    source = Path(__file__).with_name('core_gate_checks.sh').read_text()
    code = source.split("<<'PYEVAL'\n", 1)[1].split('\nPYEVAL', 1)[0]
    return ast.parse(code)


def performance_callsite() -> ast.stmt:
    for node in evaluation_tree().body:
        if isinstance(node, (ast.Assign, ast.If)) and any(
            isinstance(child, ast.Assign) and any(
                isinstance(target, ast.Name) and target.id == 'misses'
                for target in child.targets)
            for child in ast.walk(node)
        ):
            return node
    raise AssertionError('performance result callsite missing')


class UISuccessorDispatchTests(unittest.TestCase):
    @patch.dict(os.environ, {'CORE_UI_CONFIG': '/sealed/successor.json', 'CORE_P7_RUN': ''})
    def test_extracted_legacy_function_is_standalone_under_ui_environment(self) -> None:
        node = next(node for node in evaluation_tree().body
                    if isinstance(node, ast.FunctionDef) and node.name == 'performance_gate')
        namespace = {}
        exec(compile(ast.Module(body=[node], type_ignores=[]), '<standalone-performance>', 'exec'), namespace)
        check = namespace['performance_gate']
        receipt = {'status': 'PASS', 'failures': [], 'saturation_companion': {'status': 'MEASURED'}}
        self.assertEqual(check(receipt, 0), [])
        misses = ['N1000 mind/old decision>0.5', 'N1000 tick ratio exceeds accepted arithmetic bar']
        self.assertEqual(check({**receipt, 'status': 'FAIL', 'failures': misses}, 1), misses)
        for failure in ('standing C3 rss_per_agent', 'n1000/tick_mean_us>1.1', 'unknown failure'):
            with self.subTest(failure=failure), self.assertRaises(ValueError):
                check({**receipt, 'status': 'FAIL', 'failures': [failure]}, 1)

    def test_real_callsite_delegates_only_ui_branch(self) -> None:
        for enabled in (True, False):
            with self.subTest(ui_enabled=enabled), tempfile.TemporaryDirectory() as directory:
                output = Path(directory).resolve() / 'evaluation'
                output.mkdir()
                receipt = {'status': 'FAIL', 'failures': ['historical miss']}
                (output / 'performance.json').write_text(json.dumps(receipt))
                root = Path('/owned')
                legacy = Mock(return_value=['legacy miss'])
                namespace = {'root': root, 'output': output, 'freeze': freeze,
                             'perf_exit': 1, 'performance_gate': legacy}
                with patch.dict(os.environ, {'CORE_UI_CONFIG': '/sealed/successor.json' if enabled else ''}), \
                     patch.object(ui, 'performance', return_value=['historical miss']) as successor:
                    exec(compile(ast.Module(body=[performance_callsite()], type_ignores=[]),
                                 '<performance-dispatch>', 'exec'), namespace)
                if enabled:
                    successor.assert_called_once_with(root, receipt, 1, output.parent)
                    legacy.assert_not_called()
                    self.assertEqual(namespace['misses'], [])
                else:
                    successor.assert_not_called()
                    legacy.assert_called_once_with(receipt, 1)
                    self.assertEqual(namespace['misses'], ['legacy miss'])

    @patch.dict(os.environ, {'CORE_UI_CONFIG': '/sealed/successor.json'})
    def test_ui_validation_failure_propagates_without_legacy_fallback(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory).resolve()
            (output / 'performance.json').write_text('{}')
            legacy = Mock()
            namespace = {'root': Path('/owned'), 'output': output, 'freeze': freeze,
                         'perf_exit': 1, 'performance_gate': legacy}
            with patch.object(ui, 'performance', side_effect=freeze.FreezeError('fresh comparison failed')):
                with self.assertRaisesRegex(freeze.FreezeError, 'fresh comparison failed'):
                    exec(compile(ast.Module(body=[performance_callsite()], type_ignores=[]),
                                 '<performance-dispatch>', 'exec'), namespace)
            legacy.assert_not_called()


class UIAdmissionScopeTests(unittest.TestCase):
    def test_ui_timing_children_get_admission_without_enabling_parent_p7(self) -> None:
        source = Path(__file__).with_name('gate.sh').read_text()
        blocks = []
        for marker in ('    step 12 perf ', '    step 27 nav_bench '):
            stop = source.index('\nfi', source.index(marker)) + len('\nfi')
            start = source.rfind('\nif ', 0, source.index(marker)) + 1
            blocks.append(source[start:stop])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / 'rust').mkdir()
            (root / 'bin').mkdir()
            tools = root / 'tools/v8'
            tools.mkdir(parents=True)
            log = root / 'events'
            stub = '#!/bin/bash\nprintf "%s:%s:%s\\n" "$KIND" "${CORE_P7_RUN-unset}" "${CORE_P7_DEADLINE_EPOCH-unset}" >> "$EVENTS"\n'
            cargo = root / 'bin/cargo'
            cargo.write_text(stub.replace('$KIND', 'build'))
            cargo.chmod(0o755)
            (tools / 'perf.sh').write_text(stub.replace('$KIND', 'perf'))
            python_stub = "import os\nwith open(os.environ['EVENTS'],'a') as f: f.write(KIND+':'+os.environ.get('CORE_P7_RUN','unset')+':'+os.environ.get('CORE_P7_DEADLINE_EPOCH','unset')+'\\n')\n"
            (tools / 'perf_summary.py').write_text(python_stub.replace('KIND', repr('summary')))
            (tools / 'core_p7_admission.py').write_text(python_stub.replace('KIND', repr('nav')))
            environment = {**os.environ, 'CORE_UI_CONFIG': '/sealed/successor.json',
                           'CORE_UI_DEADLINE_EPOCH': '123456789', 'EVENTS': str(log),
                           'PATH': str(root / 'bin') + os.pathsep + os.environ['PATH']}
            environment.pop('CORE_P7_RUN', None)
            environment.pop('CORE_P7_DEADLINE_EPOCH', None)
            script = 'set -euo pipefail\nLOG=/out\nBASE_DATA=/base\nstep() { shift 2; "$@"; }\n'
            script += '\n'.join(blocks)
            script += '\nprintf "parent:%s:%s\\n" "${CORE_P7_RUN-unset}" "${CORE_P7_DEADLINE_EPOCH-unset}" >> "$EVENTS"\n'
            result = subprocess.run(['bash', '-c', script], cwd=root, env=environment, check=False,
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(log.read_text().splitlines(), [
                'build:unset:unset', 'perf:1:123456789', 'summary:unset:unset',
                'nav:unset:123456789', 'parent:unset:unset'])
