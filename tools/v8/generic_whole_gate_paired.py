# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -B -m unittest test_generic_whole_gate_paired
"""Consume actual paired producers beneath one candidate whole-shell stage.

The stage owner authenticates the enclosing process and source. This module
keeps the original acceptance closure and joins its real BASE/candidate children;
it never manufactures a BASE whole-stage or a ProcessResult.
"""
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_direct_run import partition, step_fixtures
from generic_fixture_registry import ExpansionContext, Invocation, Registry, expand
from generic_gate import boundary, document, integer, require, text
from generic_process import file_hash, real_path
from mux2_gate_inputs import Node

PAIRED: Final = frozenset({44, 47, 48, 50})


@dataclass(frozen=True, slots=True)
class Closure:
    root: Path
    acceptance: Path
    files: tuple[tuple[str, str], ...]
    modes: tuple[tuple[str, int], ...]


@boundary
def recheck(original: Closure) -> None:
    """Compare complete original names, modes and bytes without rebaselining."""
    real_path(original.root, directory=True)
    actual = {str(p) for p in (original.root, *original.root.rglob('*')) if p != original.acceptance}
    require(actual == dict(original.modes).keys(), '/paired', 'original namespace changed')
    files = dict(original.files)
    for name, mode in original.modes:
        path = Path(name)
        require(path.is_relative_to(original.root) and path == path.resolve(strict=True), name, 'foreign or linked evidence')
        require(path.lstat().st_mode == mode, name, 'original mode changed')
        require(stat.S_ISDIR(mode) or stat.S_ISREG(mode), name, 'special evidence')
        require((name in files) == stat.S_ISREG(mode), name, 'incomplete file binding')
        if name in files:
            require(file_hash(path) == files[name], name, 'original bytes changed')
    require(files.keys() <= dict(original.modes).keys(), '/paired', 'unbound file mode')


def closure(node: Node, acceptance: Path) -> Closure:
    """Decode B's original post-return snapshot; acceptance itself is anchored by A."""
    files = tuple((name, text(node.field('original_artifacts').field(name)))
                  for name in sorted(node.field('original_artifacts').mapping()))
    modes = tuple((name, integer(node.field('original_modes').field(name)))
                  for name in sorted(node.field('original_modes').mapping()))
    result = Closure(acceptance.parent, acceptance, files, modes)
    recheck(result)
    return result


@dataclass(frozen=True, slots=True)
class Paired:
    step: int
    closure: Closure
    receipt: Node
    plans: tuple[Invocation, ...]


def _read(path: Path, original: Closure) -> Node:
    digest = dict(original.files).get(str(path))
    require(digest is not None and file_hash(path) == digest, str(path), 'receipt lacks original binding')
    return document(path.read_text())


@boundary
def authenticate(session: Node, stage: Node, registry: Registry) -> Paired:
    """Join source-owned direct component plans to genuine returned child receipts."""
    step = integer(stage.field('step'))
    require(step in PAIRED, '/paired', 'unsupported paired step')
    stage.field('arm').equals('candidate')
    stage.field('state').equals('RETURNED')
    stage.field('native_exit').equals(0)
    process = Path(text(stage.field('process_receipt')))
    acceptance = process.parent.parent / 'acceptance/acceptance.json'
    expected_hash = text(stage.field('observer_artifacts').field(str(acceptance)))
    require(file_hash(acceptance) == expected_hash, '/paired', 'original acceptance changed')
    node = document(acceptance.read_text())
    node.field('schema').equals('whole_gate_acceptance.v1')
    node.field('step').equals(step)
    node.field('status').equals('COMPONENT_PASS')
    node.field('component_status').equals('PASS')
    original = closure(node, acceptance)
    receipt_path = acceptance.parent / 'component/receipt.json'
    node.field('component_receipt').equals(str(receipt_path))
    node.field('component_receipt_sha256').equals(dict(original.files)[str(receipt_path)])
    receipt = _read(receipt_path, original)
    receipt.field('schema').equals('direct_step_components_v1')
    receipt.field('step').equals(step)
    receipt.field('status').equals('PASS')
    receipt.field('errors').equals([])
    receipt.field('registry_sha256').equals(registry.sha256)
    required = step_fixtures(step)
    receipt.field('fixtures').equals(list(required))
    plans: list[Invocation] = []
    for arm_name in ('base', 'candidate'):
        arm = session.field('arms').field(arm_name)
        root = Path(text(arm.field('root')))
        receipt.field(arm_name).equals(arm.field('source_sha').value)
        node.field(arm_name).equals(arm.field('source_sha').value)
        product = receipt.field('products').field(arm_name)
        product.field('root').equals(str(root))
        executable = Path(text(product.field('executable')))
        require(executable.is_relative_to(root), '/product', 'foreign executable root')
        require(file_hash(executable) == text(product.field('executable_sha256')), '/product', 'executable changed')
        candidates = expand(registry, ExpansionContext(root, root, receipt_path.parent / 'runs', executable, executable))
        plans.extend(p for p in candidates if p.arm == arm_name and p.fixture_id in required)
    expected = {p.id: p for p in plans}
    seen: set[str] = set()
    groups = {name: ids.intersection(required) for name, ids in partition(registry).items() if ids.intersection(required)}
    seen_groups: set[str] = set()
    for component in receipt.field('components').array():
        name = text(component.field('component'))
        require(name in groups and name not in seen_groups, '/paired', 'foreign or duplicate component')
        seen_groups.add(name)
        component.field('status').equals('PASS')
        child_path = Path(text(component.field('receipt')))
        require(child_path == receipt_path.parent / (name + '.json'), '/paired', 'foreign component receipt')
        child = _read(child_path, original)
        component.field('receipt_sha256').equals(dict(original.files)[str(child_path)])
        child.field('status').equals('PASS')
        child.field('errors').equals([])
        for invocation in component.field('invocations').array():
            identity = text(invocation.field('id'))
            require(identity in expected and identity not in seen, '/paired', 'foreign or duplicate invocation')
            seen.add(identity)
            plan = expected[identity]
            require(plan.fixture_id in groups[name], identity, 'invocation assigned to wrong component')
            for field, value in (('fixture', plan.fixture_id), ('arm', plan.arm), ('repeat', plan.repeat),
                                 ('out', str(plan.out)), ('cwd', str(plan.cwd))):
                invocation.field(field).equals(value)
            invocation.field('seeds').equals(list(plan.seeds))
            invocation.field('roles').equals(list(plan.roles))
            invocation.field('argv').equals(None if plan.argv is None else list(plan.argv))
            native = _read(plan.out / 'process.json', original)
            native.field('status').equals('PASS')
            native.field('native_exit').equals(0)
            native.field('errors').equals([])
            product = receipt.field('products').field(plan.arm)
            native.field('executable').equals(product.field('executable').value)
            for key in ('expected_executable_sha256', 'before_executable_sha256', 'after_executable_sha256'):
                native.field(key).equals(product.field('executable_sha256').value)
            native.field('cwd').equals(str(plan.cwd))
            native.field('input_root').equals(str(plan.cwd))
            native.field('output_root').equals(str(plan.out))
            require(integer(native.field('pid')) > 0, '/paired', 'missing native PID')
            require(plan.argv is not None, identity, 'missing original command')
            if plan.argv is None:
                raise AssertionError('validated command')
            argv = tuple(text(v) for v in native.field('argv').array())
            require(argv in (plan.argv, (*plan.argv, '--gate-observations')), identity, 'actual command differs from registered invocation')
            native.field('environment').field('WORLDSIM_GATE_OBSERVER').equals('1')
            native.field('environment').field('WORLDSIM_GIT_SHA').equals(session.field('arms').field(plan.arm).field('source_sha').value)
            for stream in ('stdout', 'stderr'):
                native.field(stream + '_sha256').equals(dict(original.files)[str(plan.out / (stream + '.log'))])
    require(seen_groups == groups.keys(), '/paired', 'missing component')
    require(seen == expected.keys(), '/paired', 'missing paired invocation')
    recheck(original)
    require(file_hash(acceptance) == expected_hash, '/paired', 'acceptance changed during evaluation')
    return Paired(step, original, receipt, tuple(plans))
