"""Compare exact JSON payloads after the committed provenance-only exclusions.

Numbers are parsed as exact decimals, never binary floats. Canonical JSON uses
sorted object keys, compact separators, and coefficient/exponent decimal numbers.
Colony summaries have no exclusions and additionally require literal equality.
"""
from __future__ import annotations

import argparse
import copy
from decimal import Decimal
import hashlib
import json
from pathlib import Path
import subprocess
from typing import TypeAlias

JSON: TypeAlias = str | int | Decimal | bool | None | list['JSON'] | dict[str, 'JSON']
CONTRACT_PATH = 'docs/v8/core/CORE_PROVENANCE_KEYS.json'
SCRIPT_PATH = 'tools/v8/core_equivalence.py'
CONTRACT = {
    'schema_version': 1, 'canonicalization': 'sorted-keys-exact-decimal-v1',
    'artifacts': {
        'outcomes': ['/arms/mind/source_sha', '/arms/mind/binary_sha256',
                     '/input_hashes/mind/summary_sha256', '/input_hashes/mind/receipt_sha256',
                     '/input_hashes/base/summary_sha256', '/input_hashes/base/receipt_sha256'],
        'why': ['/source_sha', '/binary_sha256', '/input_hashes/summary_sha256',
                '/input_hashes/receipt_sha256'],
        'colony_summary': [],
    },
}


class EquivalenceError(ValueError):
    """The committed exclusion contract or payload is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise EquivalenceError(message)


def unique_pairs(pairs: list[tuple[str, JSON]]) -> dict[str, JSON]:
    result = {}
    for name, value in pairs:
        require(name not in result, 'duplicate JSON key: ' + name)
        result[name] = value
    return result


def reject_constant(value: str) -> None:
    raise EquivalenceError('nonfinite JSON number: ' + value)


def parse(raw: str) -> JSON:
    return json.loads(raw, object_pairs_hook=unique_pairs, parse_float=Decimal,
                      parse_int=Decimal, parse_constant=reject_constant)


def validate_contract(value: JSON) -> None:
    require(value == CONTRACT, 'contract differs from exact authorized provenance pointers')
    require(type(value['schema_version']) is int, 'contract version must be integer')


def number(value: Decimal) -> str:
    """Normalize decimal representation without context-dependent rounding."""
    require(value.is_finite(), 'nonfinite decimal')
    sign, digits, exponent = value.as_tuple()
    coefficient = ''.join(str(digit) for digit in digits).lstrip('0')
    if not coefficient:
        return '0'
    while coefficient.endswith('0'):
        coefficient = coefficient[:-1]
        exponent += 1
    return ('-' if sign else '') + coefficient + 'e' + str(exponent)


def canonical(value: JSON) -> str:
    """Emit one deterministic JSON representation without losing number precision."""
    match value:
        case None:
            return 'null'
        case bool():
            return 'true' if value else 'false'
        case int():
            return number(Decimal(value))
        case Decimal():
            return number(value)
        case str():
            return json.dumps(value, ensure_ascii=True)
        case list():
            return '[' + ','.join(canonical(item) for item in value) + ']'
        case dict():
            return '{' + ','.join(json.dumps(key, ensure_ascii=True) + ':' + canonical(value[key])
                                  for key in sorted(value)) + '}'
        case _:
            raise EquivalenceError('unsupported JSON value')


def payload(value: JSON, artifact: str) -> bytes:
    require(artifact in CONTRACT['artifacts'], 'unknown artifact schema')
    value = copy.deepcopy(value)
    for pointer in CONTRACT['artifacts'][artifact]:
        parts = pointer[1:].split('/')
        parent = value
        for part in parts[:-1]:
            require(isinstance(parent, dict) and part in parent, 'missing provenance pointer: ' + pointer)
            parent = parent[part]
        require(isinstance(parent, dict) and parts[-1] in parent, 'missing provenance pointer: ' + pointer)
        require(isinstance(parent[parts[-1]], str), 'provenance leaf must be a string: ' + pointer)
        del parent[parts[-1]]
    return canonical(value).encode('utf-8')


def compare(reference: Path, candidate: Path, artifact: str) -> dict[str, JSON]:
    """Pure comparison; callers must use compare_bound for production evidence."""
    before, after = reference.read_bytes(), candidate.read_bytes()
    left, right = (payload(parse(raw.decode('utf-8')), artifact) for raw in (before, after))
    equal = left == right and (artifact != 'colony_summary' or before == after)
    sha = lambda raw: hashlib.sha256(raw).hexdigest()
    return {'schema_version': 1, 'artifact': artifact, 'equal': equal,
            'comparison': 'literal' if artifact == 'colony_summary' else 'canonical_payload',
            'excluded_pointers': CONTRACT['artifacts'][artifact],
            'reference_sha256': sha(before), 'candidate_sha256': sha(after),
            'reference_payload_sha256': sha(left), 'candidate_payload_sha256': sha(right)}


def compare_bound(repo: Path, reference: Path, candidate: Path, artifact: str) -> dict[str, JSON]:
    """Require the exact script and exclusion file to be committed before comparing."""
    pins = {}
    for name in (SCRIPT_PATH, CONTRACT_PATH):
        raw = (repo / name).read_bytes()
        committed = subprocess.check_output(['git', '-C', str(repo), 'show', 'HEAD:' + name])
        require(raw == committed, 'comparison script/contract must be committed first: ' + name)
        pins[name] = raw
    contract = json.loads(pins[CONTRACT_PATH], object_pairs_hook=unique_pairs, parse_constant=reject_constant)
    validate_contract(contract)
    result = compare(reference, candidate, artifact)
    for name, raw in pins.items():
        require((repo / name).read_bytes() == raw, 'comparison input changed: ' + name)
    result['contract_sha256'] = hashlib.sha256(pins[CONTRACT_PATH]).hexdigest()
    result['checker_sha256'] = hashlib.sha256(pins[SCRIPT_PATH]).hexdigest()
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact', choices=tuple(CONTRACT['artifacts']), required=True)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--reference-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[2]
    require(hashlib.sha256(args.reference.read_bytes()).hexdigest() == args.reference_sha256,
            'reference artifact pin differs')
    require(not args.output.resolve().is_relative_to(repo), 'output must be external')
    result = compare_bound(repo, args.reference, args.candidate, args.artifact)
    require(result['reference_sha256'] == args.reference_sha256, 'reference changed during comparison')
    with args.output.open('x') as stream:
        json.dump(result, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write('\n')
    print('CORE_EQUIVALENCE ' + ('PASS' if result['equal'] else 'FAIL'))
    return 0 if result['equal'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
