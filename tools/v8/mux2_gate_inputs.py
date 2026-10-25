#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by mux2_gate_receipts.py.
"""Content-bound evidence inputs, independent of report-only commit identities."""
from collections.abc import Mapping
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import subprocess
from typing import Final, TypeAlias, assert_never

Json: TypeAlias = None | bool | int | float | str | list['Json'] | dict[str, 'Json']
INPUT_DIRS: Final = ('rust/crates', 'scripts', 'scenes', 'shaders', 'assets', 'localization', 'tools/v8')
INPUT_FILES: Final = ('rust/Cargo.toml', 'rust/Cargo.lock', 'rust/rust-toolchain.toml',
                      'project.godot', 'sim_bridge.gdextension', 'tools/play.sh')
BRIDGE: Final = 'rust/target/debug/libsim_bridge.dylib'


@dataclass(frozen=True, slots=True)
class EvidenceError(ValueError):
    reason: str

    def __str__(self) -> str:
        return self.reason


def require(condition: bool, reason: str) -> None:
    if not condition:
        raise EvidenceError(reason)


@dataclass(frozen=True, slots=True)
class Node:
    value: Json
    label: str = 'report'

    def mapping(self) -> dict[str, Json]:
        match self.value:
            case dict() as value:
                return value
            case None | bool() | int() | float() | str() | list():
                raise EvidenceError(f'{self.label}: expected object')
            case unreachable:
                assert_never(unreachable)

    def field(self, key: str) -> 'Node':
        value = self.mapping()
        require(key in value, f'{self.label}: missing {key}')
        return Node(value[key], f'{self.label}.{key}')

    def array(self) -> list['Node']:
        match self.value:
            case list() as value:
                return [Node(row, f'{self.label}[{i}]') for i, row in enumerate(value)]
            case None | bool() | int() | float() | str() | dict():
                raise EvidenceError(f'{self.label}: expected array')
            case unreachable:
                assert_never(unreachable)

    def text(self) -> str:
        match self.value:
            case str() as value:
                return value
            case None | bool() | int() | float() | list() | dict():
                raise EvidenceError(f'{self.label}: expected string')
            case unreachable:
                assert_never(unreachable)

    def number(self) -> float:
        import math
        require(type(self.value) in (int, float), f'{self.label}: expected number')
        match self.value:
            case int() | float() as value:
                require(math.isfinite(value), f'{self.label}: non-finite number')
                return float(value)
            case None | str() | list() | dict():
                raise EvidenceError(f'{self.label}: expected number')
            case unreachable:
                assert_never(unreachable)

    def equals(self, expected: Json) -> None:
        require(type(self.value) is type(expected) and self.value == expected,
                f'{self.label}: expected {expected!r}')


def read_json(path: Path) -> Node:
    return Node(json.loads(path.read_text()), str(path))


def digest(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def check_hash(path: Path, expected: str) -> None:
    require(len(expected) == 64 and path.is_file() and digest(path) == expected,
            f'missing or changed artifact: {path}')


def same_inputs(before: Mapping[str, Json], after: Mapping[str, Json]) -> None:
    require(bool(before) and before == after, 'input content or inventory changed')
    require(all(isinstance(v, str) and len(v) == 64 for v in before.values()), 'invalid input hash')


def git(root: Path, *args: str) -> str:
    return subprocess.run(['git', '-C', str(root), *args], check=True,
                          capture_output=True, text=True).stdout.strip()


def snapshot(root: Path) -> dict[str, Json]:
    """Hash actual source/data/art plus ignored lock/binary; omit docs and generated caches."""
    paths: set[Path] = set()
    for folder in INPUT_DIRS:
        directory = root / folder
        require(directory.is_dir(), f'missing input directory {folder}')
        paths.update(p for p in directory.rglob('*') if p.is_file()
                     and '__pycache__' not in p.parts and p.suffix != '.pyc')
    imported = root / '.godot/imported'
    require(imported.is_dir(), 'missing imported render inputs')
    paths.update(p for p in imported.rglob('*') if p.is_file())
    for name in (*INPUT_FILES, BRIDGE):
        path = root / name
        if name == 'rust/rust-toolchain.toml' and not path.exists():
            continue
        require(path.is_file(), f'missing input {name}')
        paths.add(path)
    return {p.relative_to(root).as_posix(): digest(p) for p in sorted(paths)}


def clean_sources(root: Path) -> None:
    names = [*INPUT_DIRS, *INPUT_FILES, 'tools/play.sh']
    require(not git(root, 'diff', 'HEAD', '--', *names), 'uncommitted source changes')
    require(not git(root, 'ls-files', '--others', '--exclude-standard', '--', *names),
            'untracked source inputs must be committed before capture')


def external(path: Path, root: Path) -> Path:
    resolved = path.resolve()
    require(path.is_absolute() and not resolved.is_relative_to(root.resolve()),
            f'evidence must be absolute and external: {path}')
    return resolved


def validate_capture_status(raw: Json, kind: str) -> None:
    receipt = Node(raw, 'receipt')
    receipt.field('schema').equals(1)
    receipt.field('kind').equals(kind)
    receipt.field('complete').equals(True)
    receipt.field('exit_code').equals(0)
