"""Synthetic fail-closed boundaries; no historical gates or captures are run."""
from contextlib import ExitStack
import json
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import Mock, patch

import core_p7_publication as p
import core_perf_check as perf
import core_shadow_successor as prior


class PublicationTests(unittest.TestCase):
    def test_exact_add_edit_delete_delta(self):
        old = {'tools/v8/edit': 'a', 'tools/v8/delete': 'b'}
        new = {'tools/v8/edit': 'c', 'tools/v8/add': 'd'}
        delta = {key: {'before_sha256': old.get(key), 'after_sha256': new.get(key)}
                 for key in set(old) | set(new)}
        p.validate_delta(old, new, delta)
        for key in delta:
            with self.subTest(key=key), self.assertRaises(p.freeze.FreezeError):
                p.validate_delta(old, new, {k: v for k, v in delta.items() if k != key})
        delta['tools/v8/edit']['after_sha256'] = 'wrong'
        with self.assertRaises(p.freeze.FreezeError):
            p.validate_delta(old, new, delta)
        with self.assertRaises(p.freeze.FreezeError):
            p.validate_delta({}, {}, {})

    def publication(self, flaw=None):
        with tempfile.TemporaryDirectory() as directory, ExitStack() as stack:
            root = Path(directory).resolve()
            manifest = root / p.DELTA
            manifest.parent.mkdir(parents=True)
            old = {'tools/v8/check.py': 'a' * 64}
            new = {'tools/v8/check.py': 'b' * 64}
            delta = {'tools/v8/check.py': {'before_sha256': 'a' * 64, 'after_sha256': 'b' * 64}}
            raw = json.dumps({} if flaw == 'unapproved' else delta).encode()
            manifest.write_bytes(raw + (b' ' if flaw == 'manifest' else b''))
            rust = {'rust/lib.rs': 'c' * 64}
            def committed(repo, revision, scope):
                if scope == 'rust':
                    return {'rust/lib.rs': 'd' * 64} if flaw == 'committed Rust' and revision == 'head' else rust
                return old if revision == p.BASE else new
            def git(repo, *args):
                if args[0] == 'rev-parse':
                    return b'head\n'
                if args[0] == 'show':
                    return raw
                return b''
            stack.enter_context(patch.object(p, 'committed', side_effect=committed))
            stack.enter_context(patch.object(p, 'git', side_effect=git))
            stack.enter_context(patch.object(p, 'live', return_value=old if flaw == 'live tools' else new))
            stack.enter_context(patch.object(p.freeze, 'digest', return_value='wrong' if flaw == 'lock' else p.LOCK))
            stack.enter_context(patch.object(perf, 'source_hash', return_value='wrong' if flaw == 'live Rust' else prior.source_digest(rust)))
            return p.publication(root)

    def test_reviewed_committed_publication(self):
        self.assertEqual(self.publication(), 'head')

    def test_source_manifest_and_tool_boundaries(self):
        for flaw in ('committed Rust', 'live Rust', 'live tools', 'unapproved', 'manifest', 'lock'):
            with self.subTest(flaw=flaw), self.assertRaises(p.freeze.FreezeError):
                self.publication(flaw)

    def test_live_hidden_files_and_symlink(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / 'tools/v8').mkdir(parents=True)
            (root / 'tools/v8/.hidden').write_text('untracked')
            self.assertIn('tools/v8/.hidden', p.live(root, 'tools/v8'))
            (root / 'tools/v8/link').symlink_to('.hidden')
            with self.assertRaises(p.freeze.FreezeError):
                p.live(root, 'tools/v8')

    def test_view_exact_head_contents_and_extra_files(self):
        inventories = {'rust': {'rust/lib.rs': 'a'}, 'tools/v8': {'tools/v8/check.py': 'b'}}
        actual = {'rust': {**inventories['rust'], 'rust/Cargo.lock': p.LOCK},
                  'tools/v8': dict(inventories['tools/v8'])}
        with patch.object(p, 'git', return_value=p.BASE.encode()) as git, \
             patch.object(p, 'committed', side_effect=lambda repo, rev, scope: inventories[scope]), \
             patch.object(p, 'live', side_effect=lambda repo, scope: actual[scope]):
            p.verify_view(Path('/repo'), Path('/view'))
            git.return_value = b'other-head'
            with self.assertRaises(p.freeze.FreezeError):
                p.verify_view(Path('/repo'), Path('/view'))
            git.return_value = p.BASE.encode()
            actual['tools/v8']['extra'] = 'x'
            with self.assertRaises(p.freeze.FreezeError):
                p.verify_view(Path('/repo'), Path('/view'))
            actual['tools/v8'] = {'tools/v8/check.py': 'changed'}
            with self.assertRaises(p.freeze.FreezeError):
                p.verify_view(Path('/repo'), Path('/view'))

    def test_batch_inventory_hashes_exact_bytes(self):
        import hashlib
        tree = b'100644 blob aaaa\ttools/v8/a\0'
        body = b'first\nsecond\x00'
        response = b'aaaa blob ' + str(len(body)).encode() + b'\n' + body + b'\n'
        with patch.object(p, 'git', return_value=tree), \
             patch.object(p.subprocess, 'check_output', return_value=response):
            self.assertEqual(p.committed(Path('/repo'), 'head', 'tools/v8'),
                             {'tools/v8/a': hashlib.sha256(body).hexdigest()})

    def test_delegation_rechecks_view_and_publication(self):
        for flaw in (None, 'view', 'publication'):
            with self.subTest(flaw=flaw), ExitStack() as stack:
                stack.enter_context(patch.dict('os.environ',
                    {'CORE_P7_RUN': '/run', 'CORE_P7_VERIFIER_VIEW': '/view'}, clear=True))
                stack.enter_context(patch.object(p.freeze, 'canonical', side_effect=lambda path: path))
                stack.enter_context(patch.object(p.freeze, 'external', return_value=Path('/view')))
                stack.enter_context(patch.object(p.freeze, 'pin'))
                stack.enter_context(patch.object(p.freeze, 'load', side_effect=[
                    {'candidate_commit': p.BASE}, {'source_commit': p.BASE}]))
                pub = stack.enter_context(patch.object(p, 'publication',
                    side_effect=['head', 'changed' if flaw == 'publication' else 'head']))
                view = stack.enter_context(patch.object(p, 'verify_view',
                    side_effect=[{'a': 'hash'}, {'a': 'changed' if flaw == 'view' else 'hash'}]))
                run = stack.enter_context(patch.object(p.subprocess, 'run'))
                if flaw:
                    with self.assertRaises(p.freeze.FreezeError):
                        p.check(Path('/repo'), Path('/receipt'), 'hash')
                else:
                    self.assertEqual(p.check(Path('/repo'), Path('/receipt'), 'hash')['source_commit'], p.BASE)
                    self.assertEqual(pub.call_count, 2)
                self.assertEqual(view.call_count, 2)
                self.assertEqual(run.call_args.kwargs['cwd'], Path('/view'))
                self.assertIn('p.check(Path.cwd()', run.call_args.args[0][3])

    def test_adjudication_exact_session_and_negative_evidence(self):
        flaws = (None, 'status', 'publication_head', 'gate', 'gate_head', 'ledger_exit_code',
                 'paired_error', 'justified_scenes', 'justification', 'raw_failure_labels',
                 'paired_config', 'paired_receipt', 'candidate_summary', 'base_summary',
                 'changed_adjudication', 'changed_summary')
        for flaw in flaws:
            with self.subTest(flaw=flaw), tempfile.TemporaryDirectory() as directory:
                scope = Path(directory).resolve()
                repo, run = scope / 'repo', scope / 'run'
                repo.mkdir()
                session = run / 'full-gate/bench/perf-session'
                session.mkdir(parents=True)
                gate = run / 'full-gate/gate.txt'
                gate.write_text('head final-head\n')
                quote = '- ' + p.P7_JUSTIFICATION + '\n'
                (repo / 'CHANGELOG.md').write_text(quote)
                config, receipt = session / 'config.json', session / 'receipt.json'
                config.write_text('{}')
                receipt.write_text('{"failures": ["n200_side64/tick_mean_us>1.1"]}')
                (session / 'new-summary.json').write_text('{}')
                (session / 'base-summary.json').write_text('{}')
                def ref(path):
                    return {'path': str(path), 'sha256': p.freeze.digest(path), 'size': path.stat().st_size}
                data = {'schema_version': 1, 'status': 'JUSTIFIED',
                        'publication_head': 'final-head', 'gate': str(gate),
                        'ledger_exit_code': 0, 'paired_error': None,
                        'justified_scenes': ['n200_side64'], 'justification': p.P7_JUSTIFICATION,
                        'raw_failure_labels': ['n200_side64/tick_mean_us>1.1'],
                        'paired_config': ref(config), 'paired_receipt': ref(receipt),
                        'candidate_summary': ref(session / 'new-summary.json'),
                        'base_summary': ref(session / 'base-summary.json')}
                adjudication = session.parent / 'p7-adjudication.json'
                adjudication.write_text(json.dumps(data))
                def git(repo, *args):
                    return b'final-head' if args[0] == 'rev-parse' else quote.encode()
                with patch.object(p, 'git', side_effect=git), \
                     patch.dict('os.environ', {'CORE_P7_RUN': str(run),
                                              'BENCH_JUSTIFIED': p.P7_JUSTIFICATION}):
                    original = p.adjudication(repo, config, receipt)
                    if flaw is None:
                        self.assertEqual(p.adjudication(repo, config, receipt, original), original)
                        continue
                    if flaw in ('paired_config', 'paired_receipt', 'candidate_summary', 'base_summary'):
                        other = run / 'fake.json'
                        other.write_bytes(Path(data[flaw]['path']).read_bytes())
                        data[flaw] = ref(other)
                    elif flaw == 'gate_head':
                        gate.write_text('head stale-head\n')
                    elif flaw == 'changed_summary':
                        (session / 'new-summary.json').write_text('{"changed": true}')
                    elif flaw == 'changed_adjudication':
                        data['extra'] = 'tampered'
                    else:
                        data[flaw] = 'wrong'
                    adjudication.write_text(json.dumps(data))
                    with self.assertRaises(p.freeze.FreezeError):
                        p.adjudication(repo, config, receipt,
                                       original if flaw == 'changed_adjudication' else None)

    def test_gate_routes_and_default_side64_rejection(self):
        text = Path(__file__).with_name('core_gate_checks.sh').read_text()
        self.assertEqual(text.count("raise SystemExit('P7 requires the original P3d endpoint')"), 2)
        self.assertEqual(text.count('core_p7_publication.check('), 2)
        self.assertIn('endpoint = p3d.check(root, receipt,', text)
        self.assertIn("/ 'full-gate/bench/perf-session'", text)
        start = text.index('def performance_gate(')
        namespace = {}
        exec(text[start:text.index('\nprotocol_args', start)], namespace)
        receipt = {'failures': ['n200_side64/tick_mean_us>1.1'], 'status': 'FAIL',
                   'saturation_companion': {'status': 'MEASURED'}}
        with patch.dict('os.environ', {}, clear=True):
            with self.assertRaisesRegex(ValueError, 'standing performance/C3 failure'):
                namespace['performance_gate'](receipt, 1)
        checker = types.ModuleType('core_p7_acceptance')
        checker.check_paired = Mock()
        with patch.dict('sys.modules', {'core_p7_acceptance': checker}), \
             patch.dict('os.environ', {'CORE_P7_RUN': '/run'}, clear=True):
            self.assertEqual(namespace['performance_gate'](receipt, 1), receipt['failures'])
            checker.check_paired.assert_called_once_with(receipt)
            self.assertEqual(receipt['status'], 'FAIL')
            with self.assertRaisesRegex(ValueError, 'status/exit'):
                namespace['performance_gate'](receipt, 0)


if __name__ == '__main__':
    unittest.main()
