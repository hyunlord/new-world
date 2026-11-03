# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_source
"""Materialize named H/O layers over admitted P; this module never builds a binary."""
import hashlib
import shutil
from pathlib import Path

from generic_build import derived_snapshot, overlay_files
from generic_cli_prepare import PreparedCli
from generic_gate import document, require
from generic_sources import freeze_source, git, safe_path
from mux2_gate_inputs import Json, Node

from generic_process import atomic_json, file_hash


def materialize(prepared: PreparedCli, role_manifests: tuple[Path, Path]) -> Node:
    """Apply exact approved overlays once, retaining three distinct source identities."""
    output = prepared.selectors.output
    require(not output.exists(), '/materialize', 'fresh output required')
    harness = document(prepared.harness_json)
    require(freeze_source(prepared.selectors.harness_root, prepared.selectors.harness_sha) == harness.mapping(),
            '/harness', 'harness changed after source admission')
    role_anchors = tuple((file_hash(path), path.lstat().st_mode) for path in role_manifests)
    arms: dict[str, Json] = {}
    output.mkdir(mode=0o700)
    for label, plan, role_path in zip(('base', 'candidate'), prepared.arms, role_manifests, strict=True):
        source = document(plan.baseline_json)
        require(freeze_source(plan.debug.root, plan.debug.source_sha) == source.mapping(),
                '/product', 'product changed before materialization')
        roles = document(role_path.read_text())
        roles.field('schema').equals('whole_gate_layers.v1')
        roles.field('product_sha').equals(plan.debug.source_sha)
        roles.field('harness_sha').equals(prepared.selectors.harness_sha)
        index = git(plan.debug.root, 'ls-files', '--stage', '-z')
        require(plan.debug.overlay is not None, '/layers', 'explicit composed overlay required')
        assert plan.debug.overlay is not None
        declared = document(plan.debug.overlay.manifest.read_text())
        expected = {row.field('path').text(): row.field('observer_sha256').text()
                    for row in declared.field('files').array()}
        rows = roles.field('files').array()
        require(len(rows) == len(expected) and {row.field('path').text() for row in rows} == set(expected),
                '/layers', 'role map differs from exact overlay paths')
        for row in rows:
            name = row.field('path').text()
            role = row.field('role').text()
            require(role in {'H', 'O'}, name, 'product data replacement is not a tooling role')
            row.field('post_sha256').equals(expected[name])
            if role == 'H':
                require(name.startswith('tools/v8/') and (name.endswith(('.py', '.sh')) or name == 'tools/v8/gate_f10_no_kill.json'),
                        name, 'H is named executable tooling, not fixture/product data')
                require(file_hash(safe_path(prepared.selectors.harness_root, name)) == expected[name],
                        name, 'H postimage differs from admitted harness')
            else:
                _ = row.field('qualification').text()
        changed = overlay_files(plan.debug)
        effective = derived_snapshot(plan.debug.root, source.mapping(), changed, index)
        arms[label] = {'root': str(plan.debug.root), 'source_sha': plan.debug.source_sha,
                       'source_before': source.value, 'effective_source': effective,
                       'source_inputs': {**{name: digest for name, digest in plan.original_inputs
                                           if name not in source.field('tracked_manifest').mapping()},
                                         **{name: Node(value).array()[1].text()
                                            for name, value in Node(effective).field('manifest').mapping().items()
                                            if Node(value).array()[0].text() != '120000'}},
                       'index_sha256': hashlib.sha256(index).hexdigest(),
                       'layer_manifest_path': str(role_path), 'layer_manifest_sha256': file_hash(role_path),
                       'overlay_manifest': str(plan.debug.overlay.manifest),
                       'overlay_manifest_sha256': plan.debug.overlay.manifest_sha256,
                       'environment': dict(plan.environment), 'cargo_home': str(plan.debug.cargo_home),
                       'generated_roots': [str(plan.debug.root/'rust/target'), str(plan.debug.root/'.godot')]}
    d = prepared.declarations
    declarations: dict[str, Json] = {
        'candidate': d.candidate, 'base': d.base, 'purpose': d.purpose,
        'known_fixtures': list(d.known_fixtures), 'ancestors': list(d.ancestors),
        'files': [list(row) for row in d.files], 'ignored_historical': list(d.ignored_historical),
        'source_manifest_sha256': d.source_manifest_sha256,
        'changes': [{'fixture_id': c.fixture_id, 'base': c.base, 'behavior_commit': c.behavior_commit,
                     'reason': c.reason, 'declaration_path': c.declaration_path,
                     'declaration_commit': c.declaration_commit, 'behavior_paths': list(c.behavior_paths)}
                    for c in d.changes]}
    receipt: dict[str, Json] = {'schema': 'whole_gate_materialization.v1',
        'classification': 'PRODUCT_PLUS_HARNESS_PLUS_OBSERVER', 'build_status': 'NOT_RUN',
        'observer_semantics': 'REQUIRES_SEPARATE_QUALIFICATION', 'harness': harness.value,
        'binding': document(prepared.binding_json).value, 'arms': arms, 'declarations': declarations}
    require(tuple((file_hash(path), path.lstat().st_mode) for path in role_manifests) == role_anchors,
            '/layers', 'original role manifests changed during materialization')
    atomic_json(output/'materialization.json', receipt)
    return Node(receipt)


def check_materialization(session: Node) -> None:
    """Recheck retained Git/index and effective source, never refresh the original layer map."""
    receipt = Path(session.field('materialization').field('path').text())
    require(file_hash(receipt) == session.field('materialization').field('sha256').text(),
            '/materialization', 'materialization receipt changed')
    original = document(receipt.read_text())
    original.field('arms').equals(session.field('arms').value)
    original.field('declarations').equals(session.field('declarations').value)
    original.field('binding').equals(session.field('binding').value)
    harness = session.field('harness')
    require(freeze_source(Path(harness.field('root').text()), harness.field('source_sha').text())
            == original.field('harness').mapping(), '/harness', 'original harness changed')
    for raw in session.field('arms').mapping().values():
        arm = Node(raw)
        root = Path(arm.field('root').text())
        index = git(root, 'ls-files', '--stage', '-z')
        require(hashlib.sha256(index).hexdigest() == arm.field('index_sha256').text(),
                '/source', 'original product index changed')
        manifest = Path(arm.field('overlay_manifest').text())
        require(file_hash(manifest) == arm.field('overlay_manifest_sha256').text(),
                '/overlay', 'overlay manifest changed')
        changed: dict[str, tuple[str | None, str]] = {}
        for row in document(manifest.read_text()).field('files').array():
            before = row.field('base0_sha256')
            changed[row.field('path').text()] = (None if before.value is None else before.text(),
                                                row.field('observer_sha256').text())
        require(derived_snapshot(root, arm.field('source_before').mapping(), changed, index)
                == arm.field('effective_source').mapping(), '/source', 'effective product source changed')


def prepare_session(config: Path) -> Path:
    """Parse a closed source selector and materialize; no build or runtime acceptance implied."""
    from typing import Literal

    from generic_cli_execute import arm, digest, path, tool
    from generic_cli_prepare import CliSelectors, prepare_cli

    config_hash = file_hash(config)
    config_mode = config.lstat().st_mode
    node = document(config.read_text())
    node.field('schema').equals('whole_gate.v1')
    require(set(node.mapping()) == {'schema', 'selectors', 'output', 'layers', 'python', 'bash',
            'environment', 'external_inputs', 'acceptance', 'observations'}, '/config', 'unknown or missing field')
    selected = node.field('selectors')
    require(set(selected.mapping()) == {'harness_root', 'harness_sha', 'published_sha', 'purpose',
            'base', 'candidate', 'cargo', 'rustc'}, '/selectors', 'unknown or missing selector field')
    purpose = selected.field('purpose').text()
    checked: Literal['publication', 'self_proof', 'defect_proof']
    match purpose:
        case 'publication': checked = 'publication'
        case 'self_proof': checked = 'self_proof'
        case 'defect_proof': checked = 'defect_proof'
        case _: raise ValueError('unsupported source purpose')
    cargo, rustc = tool(selected.field('cargo')), tool(selected.field('rustc'))
    selection = CliSelectors(path(selected.field('harness_root')), digest(selected.field('harness_sha'), 40),
        digest(selected.field('published_sha'), 40), checked, arm(selected.field('base')),
        arm(selected.field('candidate')), cargo.path, cargo.sha256, rustc.path, rustc.sha256,
        path(node.field('output')))
    environment = {name: Node(value).text() for name, value in node.field('environment').mapping().items()}
    from generic_whole_gate_fixture_inputs import FIXTURE_ENV, validate_fixture_inputs

    allowed_environment = {'PATH', 'LANG', 'LC_ALL', 'GODOT_BIN', 'CARGO_BUILD_JOBS',
                           'PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE', 'HOME', 'TMPDIR'} | FIXTURE_ENV
    require(set(environment) <= allowed_environment, '/environment', 'unregistered global environment override')
    forbidden = {'M6A_GATE_CONFIG', 'READ_GATE_CONFIG', 'CORE_UI_CONFIG', 'CORE_P7_RUN',
                 'WORLDSIM_MUX2_LANE_FD', 'WORLDSIM_TEST_SETTINGS_DIR'}
    require(not set(environment) & forbidden, '/environment', 'legacy pass selector or owner override')
    require('GODOT_BIN' in environment and 'PATH' in environment, '/environment', 'explicit Godot and tool PATH required')
    for command, expected in (('python3', path(node.field('python').field('path'))),
                              ('bash', path(node.field('bash').field('path'))),
                              ('cargo', selection.cargo), ('rustc', selection.rustc)):
        resolved = shutil.which(command, path=environment['PATH'])
        require(resolved is not None and Path(resolved).resolve() == expected, command, 'PATH tool differs from selected tool')
    prepared = prepare_cli(selection)
    external_original = {name: Node(value).text() for name, value in node.field('external_inputs').mapping().items()}
    require(environment['GODOT_BIN'] in external_original, '/godot', 'Godot requires explicit executable hash')
    external_modes = {name: Path(name).lstat().st_mode for name in external_original}
    for name, expected in external_original.items():
        require(file_hash(Path(name)) == expected, name, 'selected external input changed')
    validate_fixture_inputs(environment, external_original)
    role_paths = tuple(path(row) for row in node.field('layers').array())
    require(len(role_paths) == 2, '/layers', 'base and candidate role manifests required')
    materialized = materialize(prepared, (role_paths[0], role_paths[1]))
    external = {name: Node(value).text() for name, value in node.field('external_inputs').mapping().items()}
    external[str(config)] = config_hash
    external[str(selection.cargo)] = selection.cargo_sha256
    external[str(selection.rustc)] = selection.rustc_sha256
    for binary in ('python', 'bash'):
        value = node.field(binary)
        require(file_hash(path(value.field('path'))) == digest(value.field('sha256')), '/tools', 'tool changed')
        external[value.field('path').text()] = value.field('sha256').text()
    for arm_plan, role in zip(prepared.arms, role_paths, strict=True):
        external[str(role)] = file_hash(role)
        overlay = arm_plan.debug.overlay
        require(overlay is not None, '/layers', 'missing actual overlay')
        assert overlay is not None
        external[str(overlay.patch)] = overlay.patch_sha256
        external[str(overlay.manifest)] = overlay.manifest_sha256
    receipt = selection.output/'materialization.json'
    external[str(receipt)] = file_hash(receipt)
    source = document(prepared.harness_json)
    inputs = {name: Node(value).array()[2].text()
              for name, value in source.field('tracked_manifest').mapping().items()
              if Node(value).array()[0].text() != '120000'}
    session: dict[str, Json] = {'schema': 'whole_gate_session.v1', 'purpose': checked,
        'selector_path': str(config), 'selector_sha256': config_hash, 'qualification': 'PRODUCT_UNQUALIFIED',
        'harness': {'root': str(selection.harness_root), 'source_sha': selection.harness_sha,
                    'inputs': {name: value for name, value in inputs.items()}},
        'binding': materialized.field('binding').value, 'arms': materialized.field('arms').value,
        'declarations': materialized.field('declarations').value,
        'materialization': {'path': str(receipt), 'sha256': file_hash(receipt)},
        'output': str(selection.output), 'python': node.field('python').value, 'bash': node.field('bash').value,
        'environment': node.field('environment').value,
        'external_inputs': {name: value for name, value in external.items()},
        'external_modes': {name: Path(name).lstat().st_mode for name in external},
        'registry': {'path': str(selection.harness_root/'tools/v8/gate_fixtures.json'),
                     'sha256': file_hash(selection.harness_root/'tools/v8/gate_fixtures.json')},
        'acceptance': node.field('acceptance').value, 'observations': node.field('observations').value}
    require(all(Path(name).lstat().st_mode == mode for name, mode in external_modes.items()),
            '/external_inputs', 'external input mode changed during materialization')
    require(config.lstat().st_mode == config_mode, '/config', 'selector mode changed during materialization')
    require(file_hash(config) == config_hash, '/config', 'selector changed during materialization')
    target = selection.output/'session.json'
    atomic_json(target, session)
    return target
