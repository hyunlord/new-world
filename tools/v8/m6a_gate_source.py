"""M6a gate source binding; historical CORE equality is deliberately not used."""
from __future__ import annotations

import hashlib
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Mapping

import core_p7_publication as inventory
import core_shadow_freeze as freeze
import core_ui_successor as history
from m6a_evidence import EvidenceError, Node, Reference

BASE: Final = '5b0c88812374b87e7854cc9203f144ad0ccd9009'
SCOPES: Final = (*history.LIVE_SCOPES, 'tools', 'AGENTS.md', '.claude/skills')
FIELDS: Final = frozenset(('schema_version', 'kind', 'base_head', 'final_head',
    'runtime_sha256', 'gate_dir', 'deadline_epoch', 'history', 'candidate_build',
    'base_build', 'heldout', 'calibration', 'story', 'perf', 'protocol'))
CONFLICTING: Final = ('CORE_UI_CONFIG', 'CORE_UI_CONFIG_SHA256', 'CORE_P7_RUN',
    'CORE_P7_ADJUDICATION', 'BENCH_JUSTIFIED', 'BENCH_JUSTIFIED_SCENES',
    'CORE_P7_VERIFIER', 'MUX2_STALL_RECEIPT', 'MUX21_STALL_WAIVER',
    'WORLDSIM_CONTENT_PACK', 'WORLDSIM_MIND_HISTORY_SATURATE')


def need(condition: bool, message: str) -> None:
    if not condition:
        raise EvidenceError(message)


def keys(node: Node, expected: set[str] | frozenset[str]) -> None:
    need({name for name, _ in node.fields()} == expected, 'unexpected/missing fields')


def ref(root: Path, node: Node) -> Path:
    result = Reference.parse(node)
    return freeze.external(root, Path(result.path))


def source_digest(root: Path, head: str) -> str:
    """Hash every committed runtime/input/tool byte, independently of live state."""
    files: dict[str, str] = {}
    for scope in SCOPES:
        files.update(inventory.committed(root, head, scope))
    need(bool(files), 'empty committed input manifest')
    seal = hashlib.sha256()
    for name, checksum in sorted(files.items()):
        seal.update(name.encode() + b'\0' + checksum.encode() + b'\n')
    return seal.hexdigest()


def environment(values: Mapping[str, str]) -> None:
    need(not any(values.get(name) for name in CONFLICTING), 'M6a rejects inherited waiver/mode')
    need(not any(value for name, value in values.items()
                 if name.startswith('CORE_P7_') and name != 'CORE_P7_DEADLINE_EPOCH'),
         'M6a rejects P7 overrides')
    need(bool(values.get('M6A_GATE_CONFIG')) and bool(values.get('M6A_GATE_CONFIG_SHA256')),
         'M6a requires independently pinned config')


@dataclass(frozen=True, slots=True)
class Gate:
    root: Path
    path: Path
    checksum: str
    data: Node
    head: str
    output: Path

    @classmethod
    def load(cls, root: Path) -> Gate:
        environment(os.environ)
        path = freeze.external(root, Path(os.environ['M6A_GATE_CONFIG']))
        checksum = os.environ['M6A_GATE_CONFIG_SHA256']
        freeze.pin(path, checksum)
        data = Node.load(path)
        keys(data, FIELDS)
        need(data.get('schema_version').integer() == 1
             and data.get('kind').text() == 'm6a_simulation_successor_v1'
             and data.get('base_head').text() == BASE, 'wrong M6a protocol/base')
        gate = cls(root, path, checksum, data, data.get('final_head').text(),
                   freeze.external(root, Path(data.get('gate_dir').text())))
        gate.verify()
        return gate

    def verify(self) -> None:
        freeze.pin(self.path, self.checksum)
        need(inventory.git(self.root, 'rev-parse', 'HEAD').decode().strip() == self.head,
             'gate HEAD changed')
        inventory.git(self.root, 'merge-base', '--is-ancestor', BASE, self.head)
        for scope in SCOPES:
            names = inventory.git(self.root, 'ls-files', '-z', '--cached', '--others',
                                  '--exclude-standard', '--', scope)
            actual: dict[str, str] = {}
            for name in filter(None, names.split(b'\0')):
                relative = os.fsdecode(name)
                path = self.root / relative
                need(path.is_file() and not path.is_symlink(), 'missing/symlink live gate input')
                actual[relative] = freeze.digest(path)
            need(actual == inventory.committed(self.root, self.head, scope),
                 'dirty/untracked gate input: ' + scope)
        need(source_digest(self.root, self.head) == self.data.get('runtime_sha256').text(),
             'committed input digest differs')
        need(self.data.get('deadline_epoch').integer() > 0, 'missing hard deadline')
        for name in ('candidate_build', 'base_build', 'perf'):
            ref(self.root, self.data.get(name))
        need(os.environ.get('M6A_PERF_CONFIG') == str(ref(self.root, self.data.get('perf'))),
             'performance dispatch differs from sealed config')
        self.build('candidate_build', self.head)
        self.build('base_build', BASE)
        protocol = self.data.get('protocol')
        keys(protocol, {'goal', 'design'})
        for name, filename in [('goal', 'M6A_GOAL.md'), ('design', 'M6A_DESIGN.md')]:
            frozen = ref(self.root, protocol.get(name))
            relative = 'docs/v8/m6a/' + filename
            expected = inventory.committed(self.root, self.head, relative)[relative]
            need(freeze.digest(frozen) == expected == freeze.digest(self.root / relative),
                 'protocol differs from committed input')

    def build(self, name: str, expected_head: str) -> Node:
        row = Node.load(ref(self.root, self.data.get(name)))
        keys(row, {'schema_version', 'source_head', 'runtime_sha256', 'lock_sha256',
                   'profile', 'exit_code', 'command', 'probe', 'perf', 'log', 'lock'})
        need(row.get('schema_version').integer() == 1 and row.get('exit_code').integer() == 0
             and row.get('profile').text() == 'release' and bool(row.get('command').text().strip()),
             'unsuccessful/unattested release build')
        source_head = row.get('source_head').text()
        need(source_digest(self.root, source_head) == row.get('runtime_sha256').text(),
             'build source inventory differs')
        for scope in ('rust', 'tools/v8/bench_script.json'):
            need(inventory.committed(self.root, source_head, scope)
                 == inventory.committed(self.root, expected_head, scope),
                 'binary compiled input does not match its arm: ' + scope)
        lock = ref(self.root, row.get('lock'))
        need(freeze.digest(lock) == row.get('lock_sha256').text(), 'frozen build lock differs')
        if name == 'candidate_build':
            need(freeze.digest(self.root / 'rust/Cargo.lock') == row.get('lock_sha256').text(),
                 'live candidate lock differs from build')
        worktrees = [Path(line.removeprefix('worktree ')) for line in
            inventory.git(self.root, 'worktree', 'list', '--porcelain').decode().splitlines()
            if line.startswith('worktree ')]
        for binary in ('probe', 'perf', 'log', 'lock'):
            path = ref(self.root, row.get(binary))
            need(not any(path.is_relative_to(tree) for tree in worktrees),
                 'build evidence must not be in another mutable checkout')
        return row

    def binary(self, arm: str = 'candidate_build') -> Path:
        expected = self.head if arm == 'candidate_build' else BASE
        return ref(self.root, self.build(arm, expected).get('probe'))

    def destination(self, path: Path, name: str) -> Path:
        need(freeze.external(self.root, path) == self.output / name, 'wrong gate output directory')
        path.mkdir(parents=True, exist_ok=False)
        return path
