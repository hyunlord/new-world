"""Actual independent build antecedents; no runtime, settlement or publication authority."""
import json
import os
import stat
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Literal

from generic_build import (
    BuildSpec,
    Overlay,
    WorkspaceBuildPair,
    build_workspace_pair,
    config_inventory,
    recheck_workspace_pair,
)
from generic_declarations import Declarations, read_declarations
from generic_fixture_registry import BACKLOG_IDS, Registry, load_registry
from generic_gate import document, require
from generic_process import file_hash, real_path
from generic_sources import freeze_binding, freeze_source, git, inventory
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES

Purpose = Literal['publication', 'self_proof', 'defect_proof']


@dataclass(frozen=True, slots=True)
class ArmSelector:
    root: Path
    source_sha: str
    cargo_home: Path
    configs: tuple[tuple[str, str], ...]
    environment: tuple[tuple[str, str], ...]
    overlay: Overlay | None = None


@dataclass(frozen=True, slots=True)
class CliSelectors:
    harness_root: Path
    harness_sha: str
    published_sha: str
    purpose: Purpose
    base: ArmSelector
    candidate: ArmSelector
    cargo: Path
    cargo_sha256: str
    rustc: Path
    rustc_sha256: str
    output: Path


@dataclass(frozen=True, slots=True)
class ArmPlan:
    debug: BuildSpec
    release: BuildSpec
    pair_output: Path
    baseline_json: str
    original_inputs: tuple[tuple[str, str], ...]
    environment: tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class PreparedCli:
    selectors: CliSelectors
    harness_json: str
    binding_json: str
    registry: Registry
    declarations: Declarations
    arms: tuple[ArmPlan, ArmPlan]


@dataclass(frozen=True, slots=True)
class _CliAnchor:
    prepared: PreparedCli
    base: WorkspaceBuildPair
    candidate: WorkspaceBuildPair
    closure: tuple[tuple[Path, bool, tuple[tuple[str, str], ...]], ...]


@dataclass(frozen=True, slots=True)
class CliBuilds:
    prepared: PreparedCli
    base: WorkspaceBuildPair
    candidate: WorkspaceBuildPair | None
    status: Literal['BUILD_ANTECEDENTS_ONLY', 'FAIL']
    runtime_qualification: Literal['NOT_RUN'] = 'NOT_RUN'
    original_gate_steps_executed: Literal[False] = False
    publication_authority: Literal[False] = False
    retained_anchor: _CliAnchor | None = None


def _environment(selection: CliSelectors, arm: ArmSelector) -> tuple[tuple[str, str], ...]:
    expected = {'LANG': 'C', 'LC_ALL': 'C', 'PATH': f'{selection.rustc.parent}:/usr/bin:/bin:/usr/sbin:/sbin',
                'CARGO_HOME': str(arm.cargo_home), 'CARGO_TARGET_DIR': str(arm.root/'rust/target'),
                'RUSTC': str(selection.rustc), 'RUSTC_WRAPPER': '', 'RUSTC_WORKSPACE_WRAPPER': ''}
    require(len(arm.environment) == len(expected) and dict(arm.environment) == expected,
            '/environment', 'selector differs from actual fixed builder environment')
    return tuple(sorted(expected.items()))


def _arm(selection: CliSelectors, arm: ArmSelector, name: str) -> ArmPlan:
    baseline = freeze_source(arm.root, arm.source_sha)
    real_path(arm.cargo_home, directory=True)
    require(not (arm.root/'rust/target').exists() and not (arm.root/'rust/target').is_symlink(),
            '/target', 'fresh arm target required')
    debug = BuildSpec(arm.root, arm.source_sha, selection.cargo, selection.cargo_sha256,
                      selection.rustc, selection.rustc_sha256, arm.cargo_home,
                      selection.output/f'{name}-debug', (), (), profile='debug', overlay=arm.overlay,
                      config_inputs=arm.configs, workspace_tests=True)
    _ = config_inventory(debug)
    expected = inventory(arm.root, [*INPUT_DIRS, *INPUT_FILES])
    if arm.overlay is not None:
        require(file_hash(arm.overlay.patch) == arm.overlay.patch_sha256
                and file_hash(arm.overlay.manifest) == arm.overlay.manifest_sha256, '/overlay', 'selected overlay changed')
    return ArmPlan(debug, replace(debug, profile='release', output=selection.output/f'{name}-release'),
                   selection.output/f'{name}-pair', json.dumps(baseline, sort_keys=True),
                   tuple(sorted(expected.items())), _environment(selection, arm))


def prepare_cli(selection: CliSelectors) -> PreparedCli:
    """Freeze committed registry/declarations and both pristine arms before any overlay/build.

    Published SHA is an explicit caller-observed selector; freeze_binding authenticates
    local ancestry only. This function does not fetch or claim remote publication proof.
    """
    roots = (selection.harness_root, selection.base.root, selection.candidate.root)
    for root in roots:
        real_path(root, directory=True)
    require(len(set(roots)) == 3 and all(not a.is_relative_to(b) for a in roots for b in roots if a != b),
            '/roots', 'three independent nonnested source roots required')
    for home in (selection.base.cargo_home, selection.candidate.cargo_home):
        real_path(home, directory=True)
        require(all(not home.is_relative_to(root) and not root.is_relative_to(home) for root in roots),
                '/cargo-home', 'cache overlaps source')
    require(selection.base.cargo_home != selection.candidate.cargo_home
            and not selection.base.cargo_home.is_relative_to(selection.candidate.cargo_home)
            and not selection.candidate.cargo_home.is_relative_to(selection.base.cargo_home),
            '/cargo-home', 'independent arm caches required')
    output = selection.output
    real_path(output.parent, directory=True)
    require(output.is_absolute() and not output.exists() and not output.is_symlink()
            and all(not output.is_relative_to(root) and not root.is_relative_to(output) for root in (*roots, selection.base.cargo_home, selection.candidate.cargo_home)),
            '/output', 'fresh independent output required')
    require(selection.base.source_sha == selection.published_sha, '/base', 'BASE must be selected published SHA')
    for path, digest in ((selection.cargo, selection.cargo_sha256), (selection.rustc, selection.rustc_sha256)):
        real_path(path, directory=False)
        require(file_hash(path) == digest, '/tool', 'selected tool changed')
    harness = freeze_source(selection.harness_root, selection.harness_sha)
    binding = freeze_binding(selection.harness_root, selection.harness_sha, selection.published_sha,
                             selection.purpose, candidate_sha=selection.candidate.source_sha)
    registry_path = selection.harness_root/'tools/v8/gate_fixtures.json'
    document(json.dumps(harness)).field('tracked_manifest').field('tools/v8/gate_fixtures.json').array()[2].equals(file_hash(registry_path))
    registry = load_registry(registry_path)
    known = tuple(sorted({row.id for row in registry.fixtures} | BACKLOG_IDS))
    declarations = read_declarations(selection.candidate.root, selection.candidate.source_sha,
                                     selection.published_sha, selection.purpose, known)
    arms = (_arm(selection, selection.base, 'base'), _arm(selection, selection.candidate, 'candidate'))
    require(freeze_source(selection.harness_root, selection.harness_sha) == harness
            and load_registry(registry_path) == registry, '/harness', 'harness changed during preparation')
    for arm in arms:
        require(json.dumps(freeze_source(arm.debug.root, arm.debug.source_sha), sort_keys=True) == arm.baseline_json,
                '/source', 'arm changed during preparation')
    return PreparedCli(selection, json.dumps(harness, sort_keys=True), json.dumps(binding, sort_keys=True), registry, declarations, arms)


def _check_pair(plan: ArmPlan, pair: WorkspaceBuildPair) -> None:
    require(recheck_workspace_pair(pair) == (plan.debug, plan.release), '/pair', 'unexpected actual build specs')
    for result in (pair.debug, pair.release):
        assert result is not None
        row = document(result.receipt.read_text())
        row.field('source_before').equals(document(plan.baseline_json).value)
        expected = dict(plan.original_inputs)
        # Only the actual authenticated producer may supply approved derived postimages.
        manifest = row.field('effective_source_before').field('manifest')
        for name in manifest.mapping():
            if name in expected or any(name.startswith(scope+'/') for scope in INPUT_DIRS) or name in INPUT_FILES:
                expected[name] = manifest.field(name).array()[1].text()
        require(row.field('effective_inputs_before').mapping() == expected, '/inputs', 'producer inputs differ from frozen selection')
        process = document(Path(row.field('process_receipt').text()).read_text())
        process.field('environment').equals(dict(plan.environment))


def _harness(prepared: PreparedCli) -> None:
    selection = prepared.selectors
    require(json.dumps(freeze_source(selection.harness_root, selection.harness_sha), sort_keys=True) == prepared.harness_json,
            '/harness', 'pristine harness changed during builds')
    binding = freeze_binding(selection.harness_root, selection.harness_sha, selection.published_sha,
                             selection.purpose, candidate_sha=selection.candidate.source_sha)
    require(json.dumps(binding, sort_keys=True) == prepared.binding_json, '/binding', 'local source relation changed')


def _raw(path: Path, source: bool) -> tuple[tuple[str, str], ...]:
    """Independent terminal bytes/kinds/modes anchor, including empty directories."""
    found: dict[str, str] = {}
    pending = [path]
    while pending:
        member = pending.pop()
        if not member.exists() and not member.is_symlink():
            found[str(member)] = 'ABSENT'; continue
        mode = member.lstat().st_mode
        if stat.S_ISDIR(mode):
            found[str(member)] = f'directory:{mode}'
            pending.extend(child for child in sorted(member.iterdir()) if not (source and member == path and child.name == '.git'))
        elif stat.S_ISREG(mode): found[str(member)] = f'file:{mode}:{file_hash(member)}'
        elif stat.S_ISLNK(mode): found[str(member)] = f'link:{mode}:{os.readlink(member)}'
        else: require(False, str(member), 'unsupported closure member')
    if source:
        found['HEAD'] = git(path, 'rev-parse', 'HEAD').decode()
        found['index'] = git(path, 'ls-files', '--stage', '-z').hex()
        found['frozen_main'] = git(path, 'for-each-ref', '--format=%(objectname)', 'refs/remotes/origin/v8/main').decode()
        found['git-marker'] = str((path/'.git').lstat().st_mode) + ((path/'.git').read_text() if (path/'.git').is_file() else '')
    return tuple(sorted(found.items()))


def _fixed_paths(prepared: PreparedCli) -> set[Path]:
    selected = prepared.selectors
    paths = {selected.cargo, selected.rustc}
    for plan in prepared.arms:
        spec = plan.debug
        paths.update(parent/'.cargo'/name for parent in (spec.root/'rust', *(spec.root/'rust').parents) for name in ('config', 'config.toml'))
        paths.update(spec.cargo_home/name for name in ('config', 'config.toml'))
        if spec.overlay is not None: paths.update((spec.overlay.patch, spec.overlay.manifest))
    return paths


def recheck_cli_builds(result: CliBuilds) -> None:
    """Authenticate retained live antecedents; never refresh after generated-state changes."""
    anchor = result.retained_anchor
    require(anchor is not None and result.status == 'BUILD_ANTECEDENTS_ONLY'
            and result.runtime_qualification == 'NOT_RUN' and result.original_gate_steps_executed is False
            and result.publication_authority is False, '/cli', 'actual successful live antecedent anchor required')
    assert anchor is not None
    require(result.prepared is anchor.prepared and result.base is anchor.base and result.candidate is anchor.candidate,
            '/cli', 'retained result identity replaced')
    prepared = anchor.prepared
    expected = {(path, False) for path in _fixed_paths(prepared)} | {(prepared.selectors.harness_root, True), (prepared.selectors.output, False)}
    for plan in prepared.arms:
        expected.update(((plan.debug.root, True), (plan.debug.output, False), (plan.release.output, False), (plan.pair_output, False)))
    require(len(anchor.closure) == len(expected) and {(path, source) for path, source, _ in anchor.closure} == expected,
            '/closure', 'retained closure coverage changed')
    _check_pair(prepared.arms[0], anchor.base)
    _harness(prepared)
    _check_pair(prepared.arms[1], anchor.candidate)
    require(all(_raw(path, source) == frozen for path, source, frozen in anchor.closure),
            '/closure', 'retained source/input/producer evidence changed after authentication')


def build_cli(prepared: PreparedCli) -> CliBuilds:
    """Run real build owners in order; unexpected exceptions propagate unchanged.

    This inherits their ownership limitations. A raised/interrupted build is not a
    settled FAIL: its producer evidence remains authoritative and no later arm runs.
    No runtime product or Rustdoc qualification is reconstructed from these results.
    """
    require(prepare_cli(prepared.selectors) == prepared, '/preparation', 'stale or reconstructed preparation')
    anchors = {(path, False): _raw(path, False) for path in _fixed_paths(prepared)}
    anchors[(prepared.selectors.harness_root, True)] = _raw(prepared.selectors.harness_root, True)
    prepared.selectors.output.mkdir()
    first, second = prepared.arms
    base = build_workspace_pair(first.debug, first.release, first.pair_output)
    if base.status != 'BUILD_ANTECEDENTS_ONLY':
        return CliBuilds(prepared, base, None, 'FAIL')
    for path, source in ((first.debug.root, True), (first.debug.output, False), (first.release.output, False), (first.pair_output, False)):
        anchors[(path, source)] = _raw(path, source)
    _check_pair(first, base)
    _harness(prepared)
    require(_arm(prepared.selectors, prepared.selectors.candidate, 'candidate') == second,
            '/candidate', 'candidate source/input/tool/config selection changed during BASE build')
    candidate = build_workspace_pair(second.debug, second.release, second.pair_output)
    for path, source in ((second.debug.root, True), (second.debug.output, False), (second.release.output, False), (second.pair_output, False)):
        anchors[(path, source)] = _raw(path, source)
    require(set(prepared.selectors.output.iterdir()) <= {plan.debug.output for plan in prepared.arms} | {plan.release.output for plan in prepared.arms} | {plan.pair_output for plan in prepared.arms}, '/output', 'unexpected producer output member')
    anchors[(prepared.selectors.output, False)] = _raw(prepared.selectors.output, False)
    if candidate.status != 'BUILD_ANTECEDENTS_ONLY':
        _check_pair(first, base); _harness(prepared)
        require(all(_raw(path, source) == expected for (path, source), expected in anchors.items()),
                '/closure', 'retained source/input/producer evidence changed after authentication')
        return CliBuilds(prepared, base, candidate, 'FAIL')
    anchor = _CliAnchor(prepared, base, candidate, tuple((path, source, expected) for (path, source), expected in anchors.items()))
    result = CliBuilds(prepared, base, candidate, 'BUILD_ANTECEDENTS_ONLY', retained_anchor=anchor)
    recheck_cli_builds(result)
    return result
