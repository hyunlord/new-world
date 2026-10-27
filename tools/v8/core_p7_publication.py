"""P7 tool-only publication adapter; original826 verification stays immutable."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

import core_shadow_freeze as freeze

BASE = '826a7ecd9e38ca3fc1fff253baa0933aee593022'
DELTA = 'docs/v8/core/CORE_P7_TOOL_DELTA.json'
LOCK = 'ddc99ac9fc161d9b610c0f5d8dbf5e279c8908c10717647ba50ab2fef70c36d1'


def need(condition: bool, message: str) -> None:
    if not condition:
        raise freeze.FreezeError(message)


def git(repo: Path, *args: str) -> bytes:
    return subprocess.check_output(['git', '-C', str(repo), *args])


def committed(repo: Path, revision: str, scope: str) -> dict[str, str]:
    """Exact regular-file inventory; no provenance fields are stripped."""
    entries = []
    for record in git(repo, 'ls-tree', '-rz', revision, '--', scope).split(b'\0'):
        if not record:
            continue
        header, name = record.split(b'\t', 1)
        mode, kind, blob = header.split()
        need(mode in (b'100644', b'100755') and kind == b'blob', 'nonregular committed input')
        entries.append((os.fsdecode(name), blob))
    if not entries:
        return {}
    data = subprocess.check_output(['git', '-C', str(repo), 'cat-file', '--batch'],
                                   input=b''.join(blob + b'\n' for _, blob in entries))
    result = {}
    offset = 0
    for name, blob in entries:
        end = data.index(b'\n', offset)
        object_id, kind, size = data[offset:end].split()
        need(object_id == blob and kind == b'blob', 'unexpected Git object response')
        start = end + 1
        stop = start + int(size)
        need(data[stop:stop + 1] == b'\n', 'truncated Git object response')
        result[name] = hashlib.sha256(data[start:stop]).hexdigest()
        offset = stop + 1
    need(offset == len(data), 'extra Git object response')
    return result


def live(repo: Path, scope: str) -> dict[str, str]:
    """Include untracked and ignored files so hidden verifier inputs fail closed."""
    result = {}
    for path in (repo / scope).rglob('*'):
        need(not path.is_symlink(), 'symlink input')
        if path.is_file():
            result[path.relative_to(repo).as_posix()] = freeze.digest(path)
    return result


def validate_delta(old: dict[str, str], new: dict[str, str], declared: freeze.Document) -> None:
    expected = {name: {'before_sha256': old.get(name), 'after_sha256': new.get(name)}
                for name in sorted(set(old) | set(new)) if old.get(name) != new.get(name)}
    need(declared == expected and bool(expected), 'unapproved or inaccurate complete tool delta')
    need(all(name.startswith('tools/v8/') for name in declared), 'delta outside tools/v8')


def publication(repo: Path) -> str:
    """Verify the exact committed publication manifest and unchanged measured Rust."""
    head = git(repo, 'rev-parse', 'HEAD').decode().strip()
    git(repo, 'merge-base', '--is-ancestor', BASE, head)
    rust = committed(repo, BASE, 'rust')
    need(committed(repo, head, 'rust') == rust, 'P7 cannot change Rust')
    # Runtime target/build caches are not source inputs; source_hash and git inventory
    # enforce the same measured domain as the original verifier.
    import core_perf_check as perf
    import core_shadow_successor as prior
    need(perf.source_hash() == prior.source_digest(rust), 'live Rust source differs')
    need(freeze.digest(repo / 'rust/Cargo.lock') == LOCK, 'live lock differs')
    current = committed(repo, head, 'tools/v8')
    need(live(repo, 'tools/v8') == current, 'live tools differ from HEAD')
    raw = git(repo, 'show', head + ':' + DELTA)
    need((repo / DELTA).read_bytes() == raw, 'tool delta must be committed and unchanged')
    declared = json.loads(raw, object_pairs_hook=freeze.unique_pairs)
    need(type(declared) is dict, 'tool delta must be a mapping')
    validate_delta(committed(repo, BASE, 'tools/v8'), current, declared)
    return head


def verify_view(repo: Path, view: Path) -> dict[str, str]:
    need(git(view, 'rev-parse', 'HEAD').decode().strip() == BASE, 'immutable verifier HEAD differs')
    expected = {**committed(repo, BASE, 'rust'), **committed(repo, BASE, 'tools/v8'),
                'rust/Cargo.lock': LOCK}
    actual = {**live(view, 'rust'), **live(view, 'tools/v8')}
    need(actual == expected, 'immutable826 verifier contents differ/extra files')
    return expected


def check(repo: Path, receipt: Path, checksum: str) -> freeze.Document:
    """Delegate full historical proof without modifying the original source guard."""
    repo = freeze.canonical(repo)
    need(bool(os.environ.get('CORE_P7_RUN')), 'P7 run required')
    view_name = os.environ.get('CORE_P7_VERIFIER_VIEW')
    need(bool(view_name), 'CORE_P7_VERIFIER_VIEW required')
    view = freeze.external(repo, Path(view_name))
    head = publication(repo)
    expected = verify_view(repo, view)
    freeze.pin(receipt, checksum)
    need(freeze.load(receipt)['candidate_commit'] == BASE, 'P7 receipt must retain826 source')
    script = ('from pathlib import Path; import sys,json; sys.path.insert(0,"tools/v8"); '
              'import core_p3d_successor as p; '
              'r=p.check(Path.cwd(),Path(sys.argv[1]),sys.argv[2]); '
              'Path(sys.argv[3]).write_text(json.dumps(r,sort_keys=True))')
    with tempfile.TemporaryDirectory(prefix='core-p7-original-') as temporary:
        output = Path(temporary).resolve() / 'endpoint.json'
        subprocess.run([sys.executable, '-B', '-c', script, str(receipt), checksum, str(output)],
                       cwd=view, check=True, env={**os.environ, 'PYTHONDONTWRITEBYTECODE': '1'})
        result = freeze.load(output)
    need(verify_view(repo, view) == expected, 'verifier view changed during execution')
    need(publication(repo) == head, 'publication changed during verification')
    freeze.pin(receipt, checksum)
    need(result['source_commit'] == BASE, 'wrong verified endpoint')
    return result


P7_JUSTIFICATION = (
    'P-7 side64 justification: n200_side64 tick mean and p99 regression is accepted '
    'for this M-CORE pass only when every other standing scene remains within 1.10, '
    'all C3 ceilings pass, and the fresh full gate satisfies the owner rules; track '
    'the unresolved density cost in ISS-342/#648 for M7 node consolidation.'
)


def adjudication(repo: Path, config: Path, receipt: Path,
                 initial_sha256: str | None = None) -> str:
    """Bind step45 to this gate's successful step25; re-pin at completion.

    The gate log is append-only during execution, so its head line is checked
    rather than hashing its changing tail. The completed adjudication and all
    four paired inputs and their reconstructed comparison ledger must remain byte-identical.
    """
    scope = freeze.external(repo, Path(os.environ['CORE_P7_RUN']))
    bench = scope / 'full-gate/bench'
    session = bench / 'perf-session'
    need(config == session / 'config.json' and receipt == session / 'receipt.json',
         'P7 adjudication session differs')
    path = bench / 'p7-adjudication.json'
    checksum = freeze.digest(path)
    if initial_sha256 is not None:
        need(checksum == initial_sha256, 'P7 adjudication changed')
    data = freeze.load(path)
    need(type(data.get('schema_version')) is int and data['schema_version'] == 1,
         'P7 adjudication schema differs')
    need(data.get('status') == 'JUSTIFIED', 'P7 step25 was not justified')
    head = git(repo, 'rev-parse', 'HEAD').decode().strip()
    need(data.get('publication_head') == head, 'P7 adjudication publication differs')
    gate = scope / 'full-gate/gate.txt'
    need(data.get('gate') == str(gate), 'P7 adjudication gate differs')
    freeze.canonical(gate)
    with gate.open() as stream:
        need(stream.readline().rstrip('\n') == 'head ' + head, 'P7 gate head differs')
    need(type(data.get('ledger_exit_code')) is int and data['ledger_exit_code'] == 0,
         'P7 ledger did not pass')
    need('paired_error' in data and data['paired_error'] is None, 'P7 paired error present/missing')
    need(data.get('justified_scenes') == ['n200_side64'], 'P7 justified scenes differ')
    need(data.get('justification') == P7_JUSTIFICATION
         and os.environ.get('BENCH_JUSTIFIED') == P7_JUSTIFICATION,
         'P7 justification quote differs')
    changelog = git(repo, 'show', head + ':CHANGELOG.md')
    need((repo / 'CHANGELOG.md').read_bytes() == changelog
         and ('- ' + P7_JUSTIFICATION) in changelog.decode().splitlines(),
         'P7 justification missing from committed CHANGELOG')
    expected = {'paired_config': config, 'paired_receipt': receipt,
                'candidate_summary': session / 'new-summary.json',
                'base_summary': session / 'base-summary.json'}
    for name, expected_path in expected.items():
        need(freeze.read_reference(repo, data[name]) == expected_path,
             'P7 adjudication reference differs: ' + name)
    need(data.get('comparison_basis') == 'same-session-interleaved-base',
         'P7 comparison basis differs')
    comparison = bench / 'comparison-ledger.md'
    need(freeze.read_reference(repo, data['comparison_ledger']) == comparison,
         'P7 comparison ledger path differs')
    from core_p7_ledger import ledger_text
    need(comparison.read_bytes() == ledger_text(expected['base_summary']).encode('utf-8'),
         'P7 comparison ledger differs from pinned same-session BASE')
    freeze.read_reference(repo, data['base_summary'])
    freeze.read_reference(repo, data['comparison_ledger'])
    need(data.get('raw_failure_labels') == freeze.load(receipt).get('failures'),
         'P7 adjudication raw failures differ')
    freeze.pin(path, checksum)
    return checksum
