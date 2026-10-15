#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/revitalize/compare_identity_pass17.py REFERENCE.json CANDIDATE.json
"""Strict probe identity: complete JSON structure, exact values, timing excluded."""

from collections.abc import Iterator, Sequence
from dataclasses import dataclass
import json
from pathlib import Path
import sys
from typing import Final, NoReturn, TypeAlias, assert_never

JsonValue: TypeAlias = None | bool | int | float | str | list['JsonValue'] | dict[str, 'JsonValue']
TIMING_KEYS: Final = frozenset({'timing', 'wall_seconds', 'ms_per_agent_tick'})


@dataclass(frozen=True, slots=True)
class Difference:
    path: str
    reference: str | None
    candidate: str | None


class NonFiniteJson(ValueError):
    token: str

    def __init__(self, token: str) -> None:
        self.token = token
        super().__init__(f'nonfinite JSON number: {token}')


def reject_constant(token: str) -> NoReturn:
    raise NonFiniteJson(token)


def tokens(value: JsonValue, path: str = '$') -> Iterator[tuple[str, str]]:
    """Include container structure and f64 sign bits, not just scalar leaves."""
    match value:
        case dict() as members:
            keys = sorted(members.keys() - TIMING_KEYS)
            yield path, 'object:' + json.dumps(keys, ensure_ascii=True)
            for key in keys:
                escaped = key.replace('~', '~0').replace('/', '~1')
                yield from tokens(members[key], f'{path}/{escaped}')
        case list() as items:
            yield path, f'array:{len(items)}'
            for index, child in enumerate(items):
                yield from tokens(child, f'{path}/{index}')
        case bool() as scalar:
            yield path, f'boolean:{scalar}'
        case int() as scalar:
            yield path, f'integer:{scalar}'
        case float() as scalar:
            yield path, f'float:{scalar.hex()}'
        case str() as scalar:
            yield path, 'string:' + json.dumps(scalar, ensure_ascii=True)
        case None:
            yield path, 'null'
        case unreachable:
            assert_never(unreachable)


def compare(reference: JsonValue, candidate: JsonValue) -> list[Difference]:
    """Return every mismatch, including missing keys and empty containers."""
    left, right = dict(tokens(reference)), dict(tokens(candidate))
    return [Difference(path, left.get(path), right.get(path))
            for path in sorted(left.keys() | right.keys())
            if left.get(path) != right.get(path)]


def main(argv: Sequence[str]) -> int:
    if len(argv) != 3:
        print('usage: compare_identity_pass17.py REFERENCE.json CANDIDATE.json', file=sys.stderr)
        return 2
    try:
        reference: JsonValue = json.loads(Path(argv[1]).read_text(), parse_constant=reject_constant)
        candidate: JsonValue = json.loads(Path(argv[2]).read_text(), parse_constant=reject_constant)
    except (OSError, json.JSONDecodeError, NonFiniteJson) as error:
        print(f'identity input error: {error}', file=sys.stderr)
        return 2
    differences = compare(reference, candidate)
    print(f'{"FAIL" if differences else "PASS"}: {len(differences)} differences; only {sorted(TIMING_KEYS)} excluded')
    for difference in differences[:40]:
        print(f'{difference.path}: reference={difference.reference!r} candidate={difference.candidate!r}')
    return 1 if differences else 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv))
