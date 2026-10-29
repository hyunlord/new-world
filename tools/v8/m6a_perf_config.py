"""Strict arm-specific provenance for fresh M6a standing performance."""
from __future__ import annotations
import os
from pathlib import Path
from typing import Final
from core_perf_check import Document, Json, ROOT, git, require, sha, source_hash, write
from m6a_evidence import Node, Reference, reference_json
from m6a_gate_source import BASE, CONFLICTING, SCOPES, source_digest
import core_p7_publication as inventory

KIND: Final = 'm6a_fresh_performance_v1'
FIELDS: Final = frozenset(('schema_version', 'kind', 'base_head', 'candidate_head',
    'candidate_source_sha256', 'candidate_runtime_sha256', 'deadline_epoch',
    'base', 'new', 'base_build', 'new_build', 'commands'))
BUILD_FIELDS: Final = frozenset(('schema_version', 'source_head', 'runtime_sha256',
    'lock_sha256', 'profile', 'exit_code', 'command', 'probe', 'perf', 'log', 'lock'))

class StrictDocument(Document):
    """Retain existing arithmetic API after strict duplicate/nonfinite parsing."""
    @classmethod
    def read(cls, path: Path) -> StrictDocument:
        parsed = Document.parse(Node.load(path).value)
        return cls(parsed.values)


def reference(row: Document) -> Path:
    """Authenticate canonical bytes, size and path of one input."""
    return Path(Reference.parse(Node(dict(row.values))).path)


def reject_waivers() -> None:
    require(not any(os.environ.get(name) for name in CONFLICTING), 'M6a perf rejects inherited waivers')
    require(not any(value for name, value in os.environ.items() if name.startswith('CORE_P7_')),
            'M6a perf rejects P7 overrides')


def verify_config(config: Document) -> None:
    """Each binary binds its own source; unequal simulation trees are expected."""
    reject_waivers()
    require(set(config.values) == FIELDS, 'unknown/missing M6a perf config field')
    require(type(config.values['schema_version']) is int and config.values['schema_version'] == 1,
            'wrong perf schema')
    require(config.text('kind') == KIND and config.text('base_head') == BASE, 'wrong perf kind/BASE')
    head = config.text('candidate_head')
    require(head == git('rev-parse', 'HEAD'), 'perf candidate is not exact current HEAD')
    inventory.git(ROOT, 'merge-base', '--is-ancestor', BASE, head)
    require(config.text('candidate_source_sha256') == source_hash(), 'live Rust input changed')
    require(not git('status', '--porcelain', '--untracked-files=all', '--', 'rust'), 'dirty Rust input')
    require(not git('status', '--porcelain', '--untracked-files=all', '--', *SCOPES), 'dirty runtime/tool input')
    require(config.text('candidate_runtime_sha256') == source_digest(ROOT, head), 'candidate runtime digest changed')
    require(type(config.values['deadline_epoch']) is int and config.number('deadline_epoch') > 0,
            'invalid hard deadline')
    commands = reference(config.child('commands'))
    require(sha(commands) == sha(ROOT / 'tools/v8/bench_script.json'), 'changed bench command script')
    for role, revision in (('base', BASE), ('new', head)):
        binary = reference(config.child(role))
        worktrees = [Path(line.removeprefix('worktree ')) for line in git('worktree', 'list', '--porcelain').splitlines()
                     if line.startswith('worktree ')]
        require(not any(binary.is_relative_to(path) for path in worktrees), 'perf binary must be frozen outside mutable worktrees')
        proof = StrictDocument.read(reference(config.child(role + '_build')))
        require(set(proof.values) == BUILD_FIELDS, 'unknown/missing build field')
        require(type(proof.values['schema_version']) is int and proof.values['schema_version'] == 1,
                'wrong build schema')
        require(type(proof.values['exit_code']) is int and proof.values['exit_code'] == 0
                and proof.text('profile') == 'release' and bool(proof.text('command').strip()),
                'unsuccessful/unattested release build')
        build_head = proof.text('source_head')
        require(git('rev-parse', build_head + ':rust') == git('rev-parse', revision + ':rust'),
                'build compiled tree differs from its arm')
        require(proof.text('runtime_sha256') == source_digest(ROOT, build_head), 'build own source digest differs')
        lock = reference(proof.child('lock'))
        require(not any(lock.is_relative_to(path) for path in worktrees),
                'build lock must be frozen outside mutable worktrees')
        require(sha(lock) == proof.text('lock_sha256'), 'frozen build Cargo.lock differs')
        if role == 'new':
            require(sha(ROOT / 'rust/Cargo.lock') == proof.text('lock_sha256'),
                    'live candidate Cargo.lock differs from build')
        require(reference(proof.child('perf')) == binary, 'build identifies different perf executable')
        reference(proof.child('probe'))
        reference(proof.child('log'))
        for expected in (build_head, revision):
            script = inventory.committed(ROOT, expected, 'tools/v8/bench_script.json')['tools/v8/bench_script.json']
            require(script == sha(commands), 'bench input differs from built arm')


def freeze(base_build: Path, new_build: Path, deadline: int, output: Path) -> None:
    """Seal existing independently attested builds after the candidate commit."""
    base, new = StrictDocument.read(base_build), StrictDocument.read(new_build)
    config: dict[str, Json] = {'schema_version': 1, 'kind': KIND, 'base_head': BASE,
        'candidate_head': git('rev-parse', 'HEAD'), 'candidate_source_sha256': source_hash(),
        'candidate_runtime_sha256': source_digest(ROOT, git('rev-parse', 'HEAD')),
        'deadline_epoch': deadline, 'base': dict(base.child('perf').values),
        'new': dict(new.child('perf').values), 'base_build': reference_json(base_build),
        'new_build': reference_json(new_build), 'commands': reference_json(ROOT / 'tools/v8/bench_script.json')}
    verify_config(Document.parse(config))
    write(output, config, exclusive=True)
