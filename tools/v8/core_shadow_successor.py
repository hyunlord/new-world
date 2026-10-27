"""Explicit measured runtime succession; never claims frozen/current byte equality.

Proofs may be captured from staged source. The later receipt binds their exact
source inventory to a commit, without rewriting original capture source labels.
Provenance exclusions use only the committed core_equivalence contract.
"""
from __future__ import annotations

import hashlib
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tarfile
import tempfile

import core_shadow_freeze as frozen
import core_shadow_test_migration as migration
import core_equivalence as equivalence

POLICY = 'rust/crates/sim-core/data/colony/mind.ron'
POLICY_SHA = '7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8'
PREDECESSOR = '44e0a0266c085d409c16c77f4a2a0181b8a2a750'
FROZEN_SHA = 'ad4bf65d0bf1747f8e2572c9d630df48b36c2a14c7df1d80017aa243a1f80002'
ON_SHA = 'cad9811728445bba5d8aa9027d6450ceb9aec3b40635258572577b711c387958'
OUTCOMES_SHA = '42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed'
WHY_SHA = '552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8'
OFF_SHAS = {
    'soak': '203d098630d061261c76e83f6b1d3951d662884846758004ccd0c52bc4ca2b5f',
    'story': '94f177b3ef924a1ab59357ded06a5e0a7fed22ac02b5646eaabed6040e149d28',
    'neighbours': '0281170659f77c905ff7b9a10b63550587c9881e688521d70e45020f66493267',
    'knowledge_f10': 'fc48d580d159a03d83d00125d2f2d2d9e41bbbb606930c071a185c9f280ea985',
    'm55_pressure': '4a0cda9ea85041210834e8124e04005b2cad3b5c1cd5e636ae897b638f674a36',
    'm55_nodraft': '1f319da7c4b76ad59861c810862667e02e0afa0fbdee6d87345f917b83b9adb2',
    'm55_storyoff': '139682522580c142f1d5f715efdf33b5a9ed724ee4d099c1642ab257c7cb4f93',
    'm55_noneighbours': 'bc651e11dda612aa54f3e349ccc56c26e35980ba36adbedbdd183f28710b3f41',
    'm55_quiet': 'a08bbb39f93a889c4a1cbece1879220cddddf2809b056eaa44910e8fdffacb33',
}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise frozen.FreezeError(message)


def fields(value: frozen.Document, names: str) -> None:
    require(isinstance(value, dict) and set(value) == set(names.split()), 'unexpected successor fields')


def source_digest(inventory: dict[str, str]) -> str:
    """Use the existing perf source_hash encoding for an explicit full inventory."""
    digest = hashlib.sha256()
    for name, sha in sorted(inventory.items(), key=lambda item: os.fsencode(item[0])):
        path = PurePosixPath(name)
        require(name.startswith('rust/') and '..' not in path.parts and str(path) == name,
                'unsafe source path')
        require(type(sha) is str and re.fullmatch('[0-9a-f]{64}', sha) is not None, 'invalid source hash')
        raw = os.fsencode(name)
        digest.update(len(raw).to_bytes(8, 'little'))
        digest.update(raw)
        digest.update(bytes.fromhex(sha))
    return digest.hexdigest()


def check_delta(before: dict[str, str], after: dict[str, str], changes: frozen.Document) -> None:
    """Reject even explicitly listed policy mutations and unlisted source deltas."""
    require(after.get(POLICY) == before.get(POLICY) == POLICY_SHA, 'frozen policy changed')
    actual = {name: {'before_sha256': before.get(name), 'after_sha256': after.get(name)}
              for name in set(before) | set(after) if before.get(name) != after.get(name)}
    require(actual == changes, 'source delta differs from exact reviewed paths/hashes')


def comparison_contract(contract: frozen.Document) -> None:
    require(type(contract.get('schema_version')) is int and contract['schema_version'] == 1, 'contract version')
    require(contract.get('mode') in ('literal', 'canonical_payload_v1'), 'unrecognized/pending provenance authorization')
    names = 'schema_version mode'
    if contract['mode'] == 'canonical_payload_v1':
        names += ' keys_sha256 checker_sha256 reference_outcomes reference_why'
    fields(contract, names)


def git(repo: Path, *args: str) -> bytes:
    return subprocess.check_output(['git', '-C', str(repo), *args])


def documentation_links(repo: Path, links: list[str]) -> str:
    """Allow only enumerated single-parent Markdown commits before optimization."""
    require(type(links) is list, 'documentation links must be a list')
    parent = PREDECESSOR
    for commit in links:
        require(type(commit) is str and re.fullmatch('[0-9a-f]{40}', commit) is not None,
                'full documentation commit required')
        require(git(repo, 'show', '-s', '--format=%P', commit).decode().strip() == parent,
                'documentation ancestry differs')
        names = git(repo, 'diff', '--name-only', '-z', parent, commit).split(b'\0')
        names = list(filter(None, names))
        require(bool(names) and all(name.endswith(b'.md') for name in names), 'non-documentation change')
        require(git(repo, 'rev-parse', parent + ':rust') == git(repo, 'rev-parse', commit + ':rust'),
                'documentation Rust tree differs')
        parent = commit
    return parent


def evidence_links(repo: Path, parent: str, links: list[frozen.Document]) -> str:
    """Only exact listed verifier/contract changes may precede measured Rust work."""
    allowed = {equivalence.CONTRACT_PATH, equivalence.SCRIPT_PATH, 'tools/v8/test_core_equivalence.py',
               'tools/v8/core_shadow_successor.py', 'tools/v8/test_core_shadow_successor.py',
               'tools/v8/core_gate_checks.sh', 'docs/v8/core/CORE_DECISIONS.md'}
    require(type(links) is list, 'evidence links must be a list')
    for link in links:
        fields(link, 'commit changes')
        commit = link['commit']
        require(type(commit) is str and re.fullmatch('[0-9a-f]{40}', commit) is not None, 'full evidence commit required')
        require(git(repo, 'show', '-s', '--format=%P', commit).decode().strip() == parent, 'evidence ancestry differs')
        names = {os.fsdecode(name) for name in git(repo, 'diff', '--name-only', '-z', parent, commit).split(b'\0') if name}
        require(bool(names) and names <= allowed and names == set(link['changes']), 'unapproved evidence paths')
        for name in names:
            hashes = []
            for revision in (parent, commit):
                row = git(repo, 'ls-tree', revision, '--', name)
                require(not row or row.startswith((b'100644 blob ', b'100755 blob ')), 'nonregular evidence input')
                hashes.append(hashlib.sha256(git(repo, 'show', revision + ':' + name)).hexdigest() if row else None)
            require(link['changes'][name] == dict(zip(('before_sha256', 'after_sha256'), hashes)), 'evidence file hashes differ')
        require(git(repo, 'rev-parse', parent + ':rust') == git(repo, 'rev-parse', commit + ':rust'), 'evidence Rust tree differs')
        parent = commit
    return parent


def committed_inventory(repo: Path, commit: str, inventory: dict[str, str]) -> None:
    """Bind staged bytes to a later commit; Cargo.lock retains its explicit hash."""
    require(type(commit) is str and re.fullmatch('[0-9a-f]{40}', commit) is not None, 'full commit required')
    source_digest(inventory)
    rows = git(repo, 'ls-tree', '-r', '-z', commit, '--', 'rust').split(b'\0')
    tracked = {}
    for row in filter(None, rows):
        meta, raw = row.split(b'\t', 1)
        mode, kind, blob = meta.split()
        require(kind == b'blob' and mode in (b'100644', b'100755'), 'nonregular committed source')
        tracked[raw.decode()] = hashlib.sha256(git(repo, 'cat-file', 'blob', blob.decode())).hexdigest()
    require(set(inventory) - set(tracked) <= {'rust/Cargo.lock'} and bool(tracked), 'uncommitted source input')
    require(all(inventory.get(name) == sha for name, sha in tracked.items()), 'commit differs from measured source')


def pinned_tree(repo: Path, reference: frozen.Document) -> Path:
    """A complete external directory manifest binds raw proof files, not a PASS label."""
    manifest = frozen.load(frozen.read_reference(repo, reference))
    fields(manifest, 'root files')
    root = frozen.external(repo, Path(manifest['root']))
    actual = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), 'symlink in proof tree')
        if path.is_file():
            actual[path.relative_to(root).as_posix()] = frozen.digest(path)
    require(actual == manifest['files'] and bool(actual), 'proof tree inventory differs')
    return root


def run(repo: Path, *args: str | Path) -> None:
    subprocess.run([str(arg) for arg in args], cwd=repo, check=True)


def check_proof(repo: Path, proof_ref: frozen.Document, binary_sha: str, build: frozen.Document,
                reference_safety: Path, contract: frozen.Document) -> None:
    proof = frozen.load(frozen.read_reference(repo, proof_ref))
    fields(proof, 'safety steward heldout capture_receipt')
    safety, steward, heldout = (pinned_tree(repo, proof[key]) for key in ('safety', 'steward', 'heldout'))
    receipt = frozen.load(frozen.read_reference(repo, proof['capture_receipt']))
    fields(receipt, 'binary_sha256 pre_source_sha256 post_source_sha256 steward_command steward_exit')
    require(receipt['binary_sha256'] == binary_sha
            and receipt['pre_source_sha256'] == receipt['post_source_sha256'] == build['rust_source_sha256'],
            'capture/build source binding differs')
    require(receipt['steward_exit'] == 0 and type(receipt['steward_exit']) is int, 'ON capture incomplete')
    require(receipt['steward_command'] == ['--scene', 'colony', '--seeds', '0..2', '--days', '24',
            '--workers', '2', '--steward', 'on', '--check-determinism'], 'wrong ON command')
    require(frozen.digest(safety / 'bin/worldsim_probe') == binary_sha, 'safety binary differs')
    run(repo, 'bash', repo / 'tools/v8/mux_safety.sh', 'compare', reference_safety, safety)
    require(frozen.digest(steward / 'summary.json') == ON_SHA, 'ON summary differs')
    log = (steward / 'probe.log').read_text()
    require('DETERMINISM PASS' in log and 'DETERMINISM FAIL' not in log, 'ON determinism incomplete')
    mind = frozen.load(heldout / 'mind-receipt.json')
    require(mind['binary_sha256'] == binary_sha and mind['source_sha'] == build['source_sha']
            and mind['policy_sha256'] == POLICY_SHA and type(mind['probe_exit']) is int
            and mind['probe_exit'] == 0, 'heldout binary/source/exit mismatch')
    with tempfile.TemporaryDirectory(prefix='core-successor-score-') as temporary:
        output = Path(temporary).resolve()
        run(repo, sys.executable, repo / 'tools/v8/core_score.py', heldout / 'mind-probe/summary.json',
            '--receipt', heldout / 'mind-receipt.json', '--phase', 'heldout',
            '--base-summary', heldout / 'base-probe/summary.json', '--base-receipt', heldout / 'base-receipt.json',
            '--output', output / 'outcomes.json')
        run(repo, sys.executable, repo / 'tools/v8/core_why_check.py', '--summary', heldout / 'mind-probe/summary.json',
            '--receipt', heldout / 'mind-receipt.json', '--output', output / 'why.json')
        for artifact, expected in (('outcomes', OUTCOMES_SHA), ('why', WHY_SHA)):
            if contract['mode'] == 'literal':
                require(frozen.digest(output / (artifact + '.json')) == expected, 'literal heldout equivalence differs')
            else:
                reference = frozen.read_reference(repo, contract['reference_' + artifact])
                require(frozen.digest(reference) == expected, 'original heldout artifact differs')
                result = equivalence.compare_bound(repo, reference, output / (artifact + '.json'), artifact)
                require(result['equal'], 'normalized heldout payload differs')
                print('SUCCESSOR_PAYLOAD ' + artifact + ' ' + result['reference_payload_sha256'])
    for key in ('safety', 'steward', 'heldout'):
        pinned_tree(repo, proof[key])


def check(repo: Path, receipt_path: Path, receipt_sha: str) -> frozen.Document:
    """Verify frozen predecessor, each measured successor, and the live endpoint."""
    repo = frozen.canonical(repo)
    import core_perf_check as perf
    initial_source = perf.source_hash()
    frozen.external(repo, receipt_path)
    frozen.pin(receipt_path, receipt_sha)
    data = frozen.load(receipt_path)
    fields(data, 'schema_version kind predecessor_archive predecessor_inventory original_config original_config_sha256 comparison_contract reference_safety documentation_links evidence_links steps')
    frozen.version(data)
    require(data['kind'] == 'measured_runtime_successor', 'wrong successor kind')
    contract = frozen.load(frozen.read_reference(repo, data['comparison_contract']))
    comparison_contract(contract)
    if contract['mode'] == 'canonical_payload_v1':
        frozen.pin(repo / equivalence.CONTRACT_PATH, contract['keys_sha256'])
        frozen.pin(repo / equivalence.SCRIPT_PATH, contract['checker_sha256'])
    config_path = frozen.read_reference(repo, data['original_config'])
    require(frozen.digest(config_path) == data['original_config_sha256'], 'predecessor config pin differs')
    config = frozen.load(config_path)
    require(config['manifest_sha256'] == FROZEN_SHA, 'not original freeze4')
    frozen.pin(repo / 'tools/v8/core_shadow_test_migration.py', config['test_migration_checker_sha256'])
    before = data['predecessor_inventory']
    committed_inventory(repo, PREDECESSOR, before)
    archive = frozen.read_reference(repo, data['predecessor_archive'])
    with tempfile.TemporaryDirectory(prefix='core-successor-predecessor-') as temporary:
        projection = Path(temporary).resolve()
        with tarfile.open(archive, 'r:gz') as tar:
            members = tar.getmembers()
            require(len({item.name for item in members}) == len(members), 'duplicate archive members')
            for name, sha in before.items():
                member = tar.getmember(name)
                require(member.isfile(), 'nonregular archived source')
                stream = tar.extractfile(member)
                require(stream is not None, 'missing archived source')
                with stream:
                    raw = stream.read()
                require(hashlib.sha256(raw).hexdigest() == sha, 'predecessor archive differs')
                path = projection / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(raw)
        migration.check(projection, Path(config['manifest']), FROZEN_SHA,
                        frozen.read_reference(repo, config['test_migration']), config['test_migration']['sha256'])
    reference_safety = pinned_tree(repo, data['reference_safety'])
    for name, sha in OFF_SHAS.items():
        require(frozen.digest(reference_safety / 'colony' / name / 'summary.json') == sha,
                'reference differs from fixed CORE_REPORT fixture')
    require(type(data['steps']) is list and bool(data['steps']), 'missing successor steps')
    parent = documentation_links(repo, data['documentation_links'])
    parent = evidence_links(repo, parent, data['evidence_links'])
    endpoint = {}
    for step in data['steps']:
        fields(step, 'parent_commit candidate_commit inventory changes build_receipt binary proof')
        require(step['parent_commit'] == parent, 'broken successor chain')
        require(git(repo, 'show', '-s', '--format=%P', step['candidate_commit']).decode().strip() == parent, 'not direct commit successor')
        committed_inventory(repo, step['candidate_commit'], step['inventory'])
        check_delta(before, step['inventory'], step['changes'])
        build = frozen.load(frozen.read_reference(repo, step['build_receipt']))
        binary_sha = frozen.digest(frozen.read_reference(repo, step['binary']))
        source_sha = source_digest(step['inventory'])
        require(build['rust_source_sha256'] == build['pre_build_rust_source_sha256']
                == build['post_build_rust_source_sha256'] == source_sha and build['policy_sha256'] == POLICY_SHA
                and build['binary_sha256'] == build['binaries']['worldsim_probe'] == binary_sha
                and type(build['exit_code']) is int and build['exit_code'] == 0, 'build binding differs')
        check_proof(repo, step['proof'], binary_sha, build, reference_safety, contract)
        before, parent = step['inventory'], step['candidate_commit']
        endpoint = {'source_commit': parent, 'rust_source_sha256': source_sha,
                    'binary_sha256': binary_sha, 'policy_sha256': POLICY_SHA}
    require(perf.source_hash() == initial_source == endpoint['rust_source_sha256'], 'live successor source differs')
    run(repo, 'git', 'merge-base', '--is-ancestor', parent, 'HEAD')
    require(git(repo, 'rev-parse', 'HEAD:rust') == git(repo, 'rev-parse', parent + ':rust'), 'publication Rust differs')
    frozen.read_reference(repo, data['predecessor_archive'])
    frozen.read_reference(repo, data['original_config'])
    frozen.read_reference(repo, data['comparison_contract'])
    pinned_tree(repo, data['reference_safety'])
    for step in data['steps']:
        frozen.read_reference(repo, step['binary'])
        frozen.read_reference(repo, step['build_receipt'])
        proof = frozen.load(frozen.read_reference(repo, step['proof']))
        frozen.read_reference(repo, proof['capture_receipt'])
        for key in ('safety', 'steward', 'heldout'):
            pinned_tree(repo, proof[key])
    frozen.pin(receipt_path, receipt_sha)
    print('CURRENT_SUCCESSOR_EQUIVALENCE PASS ' + parent)
    return endpoint
