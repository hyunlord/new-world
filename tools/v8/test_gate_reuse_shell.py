# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_gate_reuse_shell.py
"""Exercise the production debug-step shell without starting cargo or Godot."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


class DebugReuseShellTests(unittest.TestCase):
    def run_debug(self, *, pregate=False, verify=0, snapshot=0, create=0, suite=0):
        # Given: the production function and shell fakes at the process boundary.
        source = Path(__file__).with_name('gate.sh').read_text()
        self.assertIn('debug_step() {\n', source)
        start = source.index('debug_step() {\n')
        function = source[start:source.index('\n}\n', start) + 3]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            script = '''set -uo pipefail
ROOT="$LOG"
SUMMARY="$LOG/gate.txt"
status=0
python3() {
    echo "helper $2" >> "$LOG/calls"
    case "$2" in
        verify) echo 'PASS_REUSED /original/receipt.json abc123 12000'; return "$VERIFY_CODE" ;;
        snapshot) return "$SNAPSHOT_CODE" ;;
        create) return "$CREATE_CODE" ;;
    esac
}
step() {
    echo "suite $*" >> "$LOG/calls"
    echo 'actual debug log' > "$LOG/test_debug.log"
    if (( SUITE_CODE != 0 )); then status=1; fi
    return "$SUITE_CODE"
}
'''
            script += function + '\ndebug_step\necho "status=$status" >> "$LOG/calls"\n'
            env = dict(os.environ, LOG=directory, C1_PREGATE_ONLY=str(int(pregate)),
                       GATE_DEBUG_RECEIPT='/original/receipt.json', VERIFY_CODE=str(verify),
                       SNAPSHOT_CODE=str(snapshot), CREATE_CODE=str(create), SUITE_CODE=str(suite))
            # When: execute only the debug-step coordinator, never the gate.
            result = subprocess.run(['bash', '-c', script], env=env, text=True,
                                    capture_output=True, check=False)
            # Then: the callers can distinguish reuse, fallback and suite failure.
            self.assertEqual(result.returncode, 0, result.stderr)
            calls = (root / 'calls').read_text()
            summary = (root / 'gate.txt').read_text() if (root / 'gate.txt').exists() else ''
            log = (root / 'test_debug.log').read_text() if (root / 'test_debug.log').exists() else ''
            return calls, summary, log

    def test_full_reuses_verified_receipt_with_original_provenance(self):
        calls, summary, log = self.run_debug()
        self.assertNotIn('suite ', calls)
        self.assertIn('PASS_REUSED /original/receipt.json abc123 12000', summary)
        self.assertRegex(summary, r'4 test_debug\s+exit\s+0\s+\d+s\s+started')
        self.assertIn('PASS_REUSED', log)
        self.assertIn('status=0', calls)

    def test_full_verification_failure_runs_complete_suite(self):
        calls, summary, _ = self.run_debug(verify=1)
        self.assertIn('suite 4 test_debug bash -c cd rust && cargo test --workspace --no-fail-fast -- --test-threads=8', calls)
        self.assertNotIn('PASS_REUSED', summary)
        self.assertNotIn('helper create', calls)

    def test_pregate_always_runs_suite_and_only_then_creates_receipt(self):
        calls, _, _ = self.run_debug(pregate=True)
        self.assertNotIn('helper verify', calls)
        self.assertLess(calls.index('helper snapshot'), calls.index('suite '))
        self.assertLess(calls.index('suite '), calls.index('helper create'))

    def test_failed_suite_does_not_issue_receipt(self):
        calls, _, _ = self.run_debug(pregate=True, suite=7)
        self.assertNotIn('helper create', calls)
        self.assertIn('status=1', calls)

    def test_snapshot_failure_keeps_fresh_suite_without_receipt(self):
        calls, _, _ = self.run_debug(pregate=True, snapshot=1)
        self.assertIn('suite ', calls)
        self.assertNotIn('helper create', calls)

    def test_receipt_write_failure_does_not_replace_suite_verdict(self):
        calls, summary, _ = self.run_debug(pregate=True, create=1)
        self.assertIn('status=0', calls)
        self.assertIn('receipt unavailable', summary)


if __name__ == '__main__':
    unittest.main()
