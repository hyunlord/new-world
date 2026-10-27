"""P3d behavioral succession: historical equivalence, then fresh behavioral evidence.

No receipt status can replace recomputation. The host parity verifier is a required,
committed dependency; observation-only diagnostics cannot satisfy this protocol.
"""
from __future__ import annotations

from datetime import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

import core_shadow_freeze as freeze
import core_shadow_successor as prior
import core_perf_check as perf
import mux_safety_validate as safety
import core_p3d_fixtures as fixtures
from compare_summaries import removed_keys

P12 = '1ca297b82f738cdcd881a344bc863bde669abe1a'
VIEW_SHA = 'd9ae619f7229d5da96cafeb8daffca8fe67be3c836e76899a36cb61d021bad3e'
HOST_CHECKER = 'tools/v8/core_p3d_host_check.py'
PROTOCOL = 'p3d_rethink_v1'
need = prior.require
fields = prior.fields


def tree(root: Path, directories: tuple[str, ...]) -> dict[str, str]:
    result = {}
    for name in directories:
        for path in (root / name).rglob('*'):
            need(not path.is_symlink(), 'symlink in source/tool inventory')
            if path.is_file():
                result[path.relative_to(root).as_posix()] = freeze.digest(path)
    return result


def committed_tools(repo: Path, commit: str) -> dict[str, str]:
    result = {}
    for line in prior.git(repo, 'ls-tree', '-rz', commit, '--', 'tools/v8').split(b'\0'):
        if not line:
            continue
        metadata, name = line.split(b'\t', 1)
        mode, kind, blob = metadata.split()
        need(mode in (b'100644', b'100755') and kind == b'blob', 'nonregular committed verifier')
        result[name.decode()] = hashlib.sha256(prior.git(repo, 'cat-file', 'blob', blob.decode())).hexdigest()
    need(bool(result), 'missing committed tools')
    return result


def invoke(repo: Path, *args: str | Path) -> None:
    subprocess.run([str(arg) for arg in args], cwd=repo, check=True,
                   env={**os.environ, 'PYTHONDONTWRITEBYTECODE': '1'})


def historical(repo: Path, data: freeze.Document) -> freeze.Document:
    view_path = freeze.read_reference(repo, data['historical_view'])
    freeze.pin(view_path, VIEW_SHA)
    view = freeze.load(view_path)
    root = freeze.external(repo, Path(view['root']))
    need(view['source_commit'] == P12 and view['kind'] == 'immutable_p12_verifier_view', 'wrong historical view')
    need(tree(root, ('rust', 'tools/v8')) == view['files'], 'historical inventory changed/extra files')
    need(prior.git(root, 'rev-parse', 'HEAD').decode().strip() == P12, 'historical HEAD differs')
    receipt = freeze.read_reference(repo, data['preserving_parent'])
    script = ('import json,sys; from pathlib import Path; sys.path.insert(0,"tools/v8"); '
              'import core_shadow_successor as s; '
              'r=s.check(Path.cwd(),Path(sys.argv[1]),sys.argv[2]); '
              'Path(sys.argv[3]).write_text(json.dumps(r,sort_keys=True))')
    with tempfile.TemporaryDirectory(prefix='p3d-prefix-') as temporary:
        output = Path(temporary).resolve() / 'endpoint.json'
        invoke(root, sys.executable, '-B', '-c', script, receipt,
               data['preserving_parent']['sha256'], output)
        endpoint = freeze.load(output)
    need(endpoint['source_commit'] == P12, 'prefix does not end at p12')
    need(tree(root, ('rust', 'tools/v8')) == view['files'], 'historical verifier mutated its view')
    freeze.pin(view_path, VIEW_SHA)
    return endpoint


def fresh_contract(data: freeze.Document) -> None:
    fields(data, 'schema_version kind seed_protocol historical_view preserving_parent candidate_commit inventory changes build_receipt binary source_freeze host_contract host_evidence host_manifest calibration safety steward reference_safety capture_receipt heldout')
    freeze.version(data)
    need(data['kind'] == 'behavior_changing_p3d_successor' and data['seed_protocol'] == PROTOCOL,
         'not the explicit behavioral-change protocol')
    need(set(data['calibration']) == {'off', 'on'}, 'both fresh shadow modes required')


def default_build(build: freeze.Document) -> None:
    need(type(build.get('build_features')) is list and build['build_features'] == []
         and type(build.get('exit_code')) is int and build['exit_code'] == 0,
         'successful default-feature build required')


def source_binding(repo: Path, data: freeze.Document, endpoint: freeze.Document) -> tuple[str, str]:
    commit = data['candidate_commit']
    need(prior.git(repo, 'show', '-s', '--format=%P', commit).decode().strip() == P12,
         'P3d must be a separate direct successor of preserving p12')
    prior.committed_inventory(repo, commit, data['inventory'])
    parent_receipt = freeze.load(freeze.read_reference(repo, data['preserving_parent']))
    need(data['reference_safety'] == parent_receipt['reference_safety'], 'baseline reference differs from preserving parent')
    original = parent_receipt['steps'][-1]['inventory']
    prior.check_delta(original, data['inventory'], data['changes'])
    need(prior.source_digest(original) == endpoint['rust_source_sha256'], 'prefix inventory differs')
    source = prior.source_digest(data['inventory'])
    need(perf.source_hash() == source, 'live source differs from behavioral successor')
    prior.run(repo, 'git', 'merge-base', '--is-ancestor', commit, 'HEAD')
    need(prior.git(repo, 'rev-parse', 'HEAD:rust') == prior.git(repo, 'rev-parse', commit + ':rust'),
         'publication Rust changed')
    expected_tools = committed_tools(repo, commit)
    names = prior.git(repo, 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', 'tools/v8').split(b'\0')
    live_tools = {os.fsdecode(name): freeze.digest(repo / os.fsdecode(name)) for name in set(names) if name}
    need(live_tools == expected_tools, 'live verifier tools differ from candidate commit')
    binary = freeze.digest(freeze.read_reference(repo, data['binary']))
    build = freeze.load(freeze.read_reference(repo, data['build_receipt']))
    default_build(build)
    view = freeze.load(freeze.read_reference(repo, data['historical_view']))
    binary_path = freeze.read_reference(repo, data['binary'])
    need(build['cargo_lock_sha256'] == view['cargo_lock_sha256']
         == freeze.digest(repo / 'rust/Cargo.lock') == freeze.digest(binary_path.parent / 'Cargo.lock'),
         'independently pinned build/current/copied lock differs')
    need(build['source_sha'] == commit and build['rust_source_sha256'] == source
         and build['pre_build_rust_source_sha256'] == build['post_build_rust_source_sha256'] == source
         and build['binary_sha256'] == build['binaries']['worldsim_probe'] == binary
         and build['policy_sha256'] == prior.POLICY_SHA and type(build['exit_code']) is int
         and build['exit_code'] == 0, 'candidate committed build binding differs')
    frozen = freeze.load(freeze.read_reference(repo, data['source_freeze']))
    fields(frozen, 'schema_version kind candidate_commit rust_source_sha256 policy_sha256 binary_sha256 frozen_utc calibration host_contract host_evidence host_manifest tools')
    freeze.version(frozen)
    need(frozen['kind'] == 'p3d_pre_heldout_freeze' and frozen['candidate_commit'] == commit
         and frozen['rust_source_sha256'] == source and frozen['binary_sha256'] == binary
         and frozen['policy_sha256'] == prior.POLICY_SHA and frozen['tools'] == expected_tools,
         'fresh freeze source/build/policy/tools mismatch')
    for key in ('calibration', 'host_contract', 'host_evidence', 'host_manifest'):
        need(frozen[key] == data[key], 'fresh freeze does not bind completed shadow/parity evidence')
    return source, binary


def parity_counts(parity: freeze.Document) -> None:
    counts = [parity[key] for key in ('compared_person_ticks', 'retained_person_ticks', 'full_person_ticks', 'mismatches')]
    need(all(type(value) is int and value >= 0 for value in counts)
         and counts[0] == counts[1] + counts[2] and counts[1] > 0 and counts[2] > 0 and counts[3] == 0,
         'host parity is incomplete, diagnostic-only, or divergent')


def host_and_shadow(repo: Path, data: freeze.Document, source: str) -> None:
    # This fixed committed verifier is deliberately absent until real scheduling parity is implemented.
    need(HOST_CHECKER in committed_tools(repo, data['candidate_commit']), 'required host parity verifier is not committed')
    evidence = prior.pinned_tree(repo, data['host_evidence'])
    contract = freeze.read_reference(repo, data['host_contract'])
    manifest = freeze.read_reference(repo, data['host_manifest'])
    with tempfile.TemporaryDirectory(prefix='p3d-host-parity-') as temporary:
        result = Path(temporary).resolve() / 'parity.json'
        invoke(repo, sys.executable, '-B', repo / HOST_CHECKER, '--contract', contract,
               '--evidence', evidence, '--output', result)
        parity = freeze.load(result)
    fields(parity, 'schema_version kind candidate_source_sha256 host_manifest_sha256 contract_sha256 raw_evidence_manifest_sha256 calibration compared_person_ticks retained_person_ticks full_person_ticks mismatches')
    freeze.version(parity)
    need(parity['kind'] == 'executed_scheduling_adapter_parity'
         and parity['candidate_source_sha256'] == source
         and parity['host_manifest_sha256'] == freeze.digest(manifest)
         and parity['contract_sha256'] == freeze.digest(contract)
         and parity['raw_evidence_manifest_sha256'] == data['host_evidence']['sha256'], 'host parity binding differs')
    need(parity['calibration'] == data['calibration'], 'parity verifier did not bind exact raw calibration captures')
    parity_counts(parity)
    for mode, refs in data['calibration'].items():
        fields(refs, 'shadow control feature_off review')
        paths = {key: freeze.read_reference(repo, ref) for key, ref in refs.items()}
        observed = freeze.load(paths['shadow'])
        need(observed['inputs']['steward'] is (mode == 'on'), 'shadow mode differs')
        invoke(repo, 'bash', repo / 'tools/v8/core_gate_checks.sh', 'shadow',
               paths['shadow'], paths['control'], paths['feature_off'], paths['review'])


def raw_identity(root: Path, removed_path: Path) -> dict[str, str]:
    """Use identity.sh raw_digest semantics, including its standing empty-key rule."""
    removed = removed_keys(removed_path)
    result = {}
    for path in sorted(root.glob('*.jsonl')):
        freeze.canonical(path)
        digest = hashlib.sha256()
        with path.open() as stream:
            for line in stream:
                row = json.loads(line, object_pairs_hook=freeze.unique_pairs)
                row.pop('git_sha', None)
                for key in removed & set(row):
                    need(row[key] in (0, 0.0, False, None, '', [], {}),
                         'removed raw identity key has nonempty value: ' + key)
                    row.pop(key)
                digest.update(json.dumps(row, sort_keys=True).encode() + b'\n')
        result[path.name] = digest.hexdigest()
    need(bool(result), 'missing baseline raw sample inventory')
    return result


def preservation(repo: Path, data: freeze.Document, source: str, binary: str) -> None:
    candidate = prior.pinned_tree(repo, data['safety'])
    reference = prior.pinned_tree(repo, data['reference_safety'])
    steward = prior.pinned_tree(repo, data['steward'])
    capture = freeze.load(freeze.read_reference(repo, data['capture_receipt']))
    fields(capture, 'binary_sha256 pre_source_sha256 post_source_sha256 steward_command steward_exit fixture_commands')
    need(capture['binary_sha256'] == binary and capture['pre_source_sha256'] == capture['post_source_sha256'] == source,
         'capture source/build binding differs')
    need(type(capture['steward_exit']) is int and capture['steward_exit'] == 0
         and capture['steward_command'] == ['--scene', 'colony', '--seeds', '0..2', '--days', '24', '--workers', '2', '--steward', 'on', '--check-determinism'], 'ON capture command incomplete')
    need(freeze.digest(candidate / 'bin/worldsim_probe') == binary, 'safety capture binary differs')
    for root in (candidate, reference):
        safety.validate_identity(root)
        safety.validate_colonies(root)
        need((root / 'capture_complete.txt').is_file(), 'incomplete safety capture')
        for path in (root / 'soak.log', root / 'identity/determinism.txt'):
            text = path.read_text()
            need('DETERMINISM PASS' in text and 'DETERMINISM FAIL' not in text, 'determinism evidence fails')
    fixtures.capture_contract(candidate, capture['fixture_commands'], data['candidate_commit'])
    fixtures.existing_scorers(repo, data['candidate_commit'], candidate)
    need(tree(candidate, ('inputs',)) == tree(reference, ('inputs',)), 'fixture scripts changed')
    invoke(repo, sys.executable, repo / 'tools/v8/compare_chronicles.py',
           reference / 'identity/chronicle', candidate / 'identity/chronicle')
    removed = repo / 'tools/v8/identity_removed_keys.txt'
    need(raw_identity(reference / 'identity/probe', removed) == raw_identity(candidate / 'identity/probe', removed),
         'baseline raw sample inventory/content differs')
    invoke(repo, sys.executable, repo / 'tools/v8/compare_summaries.py', reference / 'identity/probe/summary.json',
           candidate / 'identity/probe/summary.json', '--removed-keys', repo / 'tools/v8/identity_removed_keys.txt')
    summary = freeze.load(steward / 'summary.json')
    need(set(summary) == {'scene', 'days', 'seeds'} and summary['scene'] == 'colony'
         and type(summary['days']) is int and summary['days'] == 24 and len(summary['seeds']) == 2, 'ON coverage differs')
    for index, row in enumerate(summary['seeds']):
        fixtures.complete_row(row, 24, steward=True)
        for key, expected in (('seed', index), ('ledger_violation_ticks', 0), ('reservation_violation_ticks', 0)):
            need(type(row[key]) is int and row[key] == expected, 'ON seed/invariant differs')
    text = (steward / 'probe.log').read_text()
    need('DETERMINISM PASS' in text and 'DETERMINISM FAIL' not in text, 'ON determinism incomplete')


def fresh_outcomes(repo: Path, data: freeze.Document, binary: str) -> None:
    root = prior.pinned_tree(repo, data['heldout'])
    frozen = freeze.load(freeze.read_reference(repo, data['source_freeze']))
    freeze_time = datetime.fromisoformat(frozen['frozen_utc'])
    need(freeze_time.tzinfo is not None, 'freeze timestamp must have timezone')
    for arm in ('mind', 'base'):
        receipt = freeze.load(root / (arm + '-receipt.json'))
        need(type(receipt.get('probe_exit')) is int and receipt['probe_exit'] == 0,
             'heldout probe did not exit successfully: ' + arm)
        log = (root / (arm + '-probe.log')).read_text()
        need('DETERMINISM PASS' in log and 'DETERMINISM FAIL' not in log,
             'heldout determinism incomplete or failed: ' + arm)
        need(receipt['schema_version'] == 2 and receipt['seed_protocol'] == PROTOCOL
             and receipt['policy_freeze_sha256'] == data['source_freeze']['sha256'], 'heldout protocol/freeze differs')
        start, end = (datetime.fromisoformat(receipt[key]) for key in ('started_utc', 'finished_utc'))
        need(start.tzinfo is not None and end.tzinfo is not None and freeze_time < start <= end,
             'heldout did not follow fresh freeze')
        if arm == 'mind':
            need(receipt['source_sha'] == data['candidate_commit'] and receipt['binary_sha256'] == binary,
                 'heldout candidate differs')
    with tempfile.TemporaryDirectory(prefix='p3d-score-') as temporary:
        output = Path(temporary).resolve()
        invoke(repo, sys.executable, repo / 'tools/v8/core_score.py', root / 'mind-probe/summary.json',
               '--receipt', root / 'mind-receipt.json', '--phase', 'heldout', '--seed-protocol', PROTOCOL,
               '--base-summary', root / 'base-probe/summary.json', '--base-receipt', root / 'base-receipt.json',
               '--output', output / 'outcomes.json')
        invoke(repo, sys.executable, repo / 'tools/v8/core_why_check.py', '--summary', root / 'mind-probe/summary.json',
               '--receipt', root / 'mind-receipt.json', '--seed-protocol', PROTOCOL, '--output', output / 'why.json')
        for name in ('outcomes.json', 'why.json'):
            need(freeze.digest(output / name) == freeze.digest(root / name), 'fresh recomputation differs from recorded result')


def check(repo: Path, receipt_path: Path, receipt_sha: str) -> freeze.Document:
    repo = freeze.canonical(repo)
    freeze.external(repo, receipt_path)
    freeze.pin(receipt_path, receipt_sha)
    data = freeze.load(receipt_path)
    fresh_contract(data)
    endpoint = historical(repo, data)
    source, binary = source_binding(repo, data, endpoint)
    host_and_shadow(repo, data, source)
    preservation(repo, data, source, binary)
    fresh_outcomes(repo, data, binary)
    # Recheck all captured trees and the current closure after subprocess validation.
    for key in ('host_evidence', 'safety', 'steward', 'reference_safety', 'heldout'):
        prior.pinned_tree(repo, data[key])
    source_binding(repo, data, endpoint)
    for key in ('historical_view', 'preserving_parent', 'source_freeze', 'host_contract', 'host_manifest', 'capture_receipt'):
        freeze.read_reference(repo, data[key])
    for refs in data['calibration'].values():
        for reference in refs.values():
            freeze.read_reference(repo, reference)
    freeze.pin(receipt_path, receipt_sha)
    return {'source_commit': data['candidate_commit'], 'rust_source_sha256': source,
            'binary_sha256': binary, 'policy_sha256': prior.POLICY_SHA}
