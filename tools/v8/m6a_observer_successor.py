"""Exact P9 frozen-evidence reuse across the seven reviewed observer-tool paths."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
from typing import Final

import core_p7_publication as inventory
from m6a_evidence import (INPUT_ROOTS, EvidenceError, Json, Node, Reference, digest,
                          git, reference_json, write_json)

P9: Final = '5c7e05e677cebda16fea5825f88d83561acd9d2d'
DEADLINE: Final = 1793335103
KIND: Final = 'm6a_observer_successor_v1'
ALLOWED: Final = frozenset('tools/v8/' + name for name in (
    'm6a_base72_entry.rs', 'm6a_base72.py', 'test_m6a_base72.py',
    'm6a_observer_successor.py', 'test_m6a_observer_successor.py',
    'm6a_gate_source.py', 'm6a_gate_evidence.py'))
PINS: Final = {
    'freeze': ('p9-policy-freeze.json', 'baf1d707f83bdfe44ff08bf453a5e63199de93d56281395498172007e9f9368b'),
    'register': ('calibration-register.jsonl', 'cf276dc39744f81ac184ebb41d14059b123a5e7e9259c6dcf4718bc0095d621e'),
    'calibration': ('calibration-4/receipt.json', 'c71d8e9b3df3eca95163657ed2dff45212af452b5cb66834f5749a35338132a5'),
    'candidate': ('p9-heldout-candidate/receipt.json', 'bf8eb48c96caea61ba90202291d36d75fb2f255dc2cf95a75956bd9d9421b05e'),
    'candidate_build': ('candidate-p9-binaries/build-receipt.json', '7f5b21deca7ab91ea5d5774f28822f47a42da9a55294d04ea5569f9b018ef7a3'),
}
FIELDS: Final = frozenset(('schema_version', 'kind', 'original_head', 'successor_head',
    'original_source_hash', 'successor_source_hash', 'changes', *PINS))


def need(condition: bool, message: str) -> None:
    if not condition:
        raise EvidenceError(message)


def committed_inputs(root: Path, head: str) -> dict[str, str]:
    result: dict[str, str] = {}
    for scope in INPUT_ROOTS:
        result.update(inventory.committed(root, head, scope))
    need(bool(result), 'empty committed observer input inventory')
    return result


def inventory_hash(files: dict[str, str]) -> str:
    seal = hashlib.sha256()
    for name, checksum in sorted(files.items()):
        seal.update(name.encode() + b'\0' + checksum.encode() + b'\n')
    return seal.hexdigest()


def changes(old: dict[str, str], new: dict[str, str]) -> list[dict[str, Json]]:
    changed = {name for name in old.keys() | new.keys() if old.get(name) != new.get(name)}
    need(changed == ALLOWED, 'observer successor differs outside exact seven reviewed tool changes')
    need(all(name in new for name in changed), 'observer successor deletes an allowed tool')
    return [{'path': name, 'before': old.get(name), 'after': new[name]} for name in sorted(changed)]


def verify_live(root: Path, head: str, expected: dict[str, str]) -> None:
    need(git(root, 'rev-parse', 'HEAD') == head, 'observer successor is not current HEAD')
    need(not git(root, 'status', '--porcelain', '--untracked-files=all', '--', *INPUT_ROOTS),
         'dirty/untracked observer successor inputs')
    names = inventory.git(root, 'ls-files', '-z', '--cached', '--others',
                          '--exclude-standard', '--', *INPUT_ROOTS).split(b'\0')
    actual: dict[str, str] = {}
    for raw in filter(None, names):
        name = os.fsdecode(raw)
        path = root / name
        need(path.is_file() and not path.is_symlink(), 'nonregular live observer input')
        actual[name] = digest(path)
    need(actual == expected, 'live observer inputs differ from committed successor')


def pinned_evidence(root: Path, proof: Node, original_hash: str) -> None:
    refs: dict[str, Reference] = {}
    for key, (_, checksum) in PINS.items():
        reference = Reference.parse(proof.get(key))
        need(reference.sha256 == checksum, 'changed original P9 evidence: ' + key)
        need(not Path(reference.path).is_relative_to(root.resolve()), 'P9 evidence must be external')
        refs[key] = reference
    frozen = refs['freeze'].read()
    need(frozen.get('source_head').text() == P9
         and frozen.get('source_hash').text() == original_hash, 'original frozen source differs')
    need(frozen.get('calibration_register').value == proof.get('register').value,
         'original register reference differs')
    rows = [Node.parse(line) for line in Path(refs['register'].path).read_text().splitlines()]
    need(len(rows) == 8, 'exactly four closed original rounds required')
    for index, row in enumerate(rows):
        need(row.get('event').text() == ('started' if index % 2 == 0 else 'finished')
             and row.get('round').integer() == index // 2 + 1, 'invalid original round order')
        if index % 2:
            Reference.parse(row.get('receipt'))
    need(rows[-1].get('receipt').value == proof.get('calibration').value
         and rows[-1].get('exit_code').integer() == 0, 'original final calibration differs')
    for key in ('calibration', 'candidate'):
        cohort = refs[key].read()
        need(cohort.get('source_head').text() == P9
             and cohort.get('source_hash').text() == original_hash
             and cohort.get('exit_code').integer() == 0
             and cohort.get('source_unchanged').boolean(), 'original cohort changed: ' + key)
        for tool in ('observer', 'scorer'):
            Reference.parse(cohort.get(tool))
    candidate = refs['candidate'].read()
    need(candidate.get('freeze').value == proof.get('freeze').value, 'candidate freeze replaced')
    build = refs['candidate_build'].read()
    need(build.get('source_head').text() == P9
         and candidate.get('binary').value == build.get('probe').value, 'candidate build replaced')
    probe = Reference.parse(build.get('probe'))
    need(frozen.get('binary_sha256').text() == probe.sha256, 'original binary changed')
    Reference.parse(build.get('lock'))
    need(digest(root/'rust/Cargo.lock') == build.get('lock_sha256').text(), 'live lock changed')
    need(frozen.get('scorer_sha256').text() == digest(root/'tools/v8/m6a_score.py'),
         'frozen scorer changed')


def validate(root: Path, path: Path, head: str, binding: Node) -> str:
    proof = Node.load(path)
    need({name for name, _ in proof.fields()} == FIELDS, 'unexpected successor proof fields')
    need(proof.get('schema_version').integer() == 1 and proof.get('kind').text() == KIND,
         'wrong observer successor proof schema')
    need(proof.get('original_head').text() == P9
         and proof.get('successor_head').text() == head, 'wrong observer successor heads')
    inventory.git(root, 'merge-base', '--is-ancestor', P9, head)
    old, new = committed_inputs(root, P9), committed_inputs(root, head)
    verify_live(root, head, new)
    delta = changes(old, new)
    changed_paths = set(git(root, 'diff', '--name-only', '--no-renames', P9, head,
                            '--', *INPUT_ROOTS).splitlines())
    need(changed_paths == ALLOWED, 'unreviewed file mode/path change in successor')
    need(proof.get('changes').value == delta, 'reviewed tool delta differs')
    original_hash = inventory_hash(old)
    need(proof.get('original_source_hash').text() == original_hash
         and proof.get('successor_source_hash').text() == inventory_hash(new), 'source proof hash differs')
    for name in ('M6A_GOAL.md', 'M6A_DESIGN.md'):
        relative = 'docs/v8/m6a/' + name
        need(inventory.committed(root, P9, relative) == inventory.committed(root, head, relative),
             'frozen protocol changed: ' + name)
    pinned_evidence(root, proof, original_hash)
    need(binding.get('deadline_epoch').integer() == DEADLINE, 'original deadline changed')
    for key in ('freeze', 'register'):
        need(binding.get('calibration').get(key).value == proof.get(key).value,
             'gate replaced original ' + key)
    need(binding.get('heldout').get('candidate').value == proof.get('candidate').value
         and binding.get('candidate_build').value == proof.get('candidate_build').value,
         'gate replaced original candidate evidence')
    return original_hash


def prepare(root: Path, evidence: Path, output: Path) -> None:
    """Create a new proof only after the exact tooling successor is committed."""
    head = git(root, 'rev-parse', 'HEAD')
    old, new = committed_inputs(root, P9), committed_inputs(root, head)
    verify_live(root, head, new)
    data: dict[str, Json] = {'schema_version': 1, 'kind': KIND, 'original_head': P9,
        'successor_head': head, 'original_source_hash': inventory_hash(old),
        'successor_source_hash': inventory_hash(new), 'changes': changes(old, new)}
    data.update({key: reference_json(evidence/name) for key, (name, _) in PINS.items()})
    pinned_evidence(root, Node(data), inventory_hash(old))
    write_json(output, data)
