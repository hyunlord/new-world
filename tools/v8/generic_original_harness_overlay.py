"""Write a source-only H overlay for later real generic_build execution."""
import json
from pathlib import Path

from generic_build import Overlay
from generic_gate import require
from generic_original_harness_projection import (
    OriginalHarnessContext,
    projected_manifest,
)
from generic_process import file_hash
from generic_sources import freeze_source, git
from mux2_gate_inputs import Json, Node


def prepare_overlay(context: OriginalHarnessContext, session: Node, root: Path, output: Path) -> Overlay:
    """No product mutation, build, build result, warm grant or qualification is created."""
    arm = session.field('arms').field(context.arm)
    original = Node(freeze_source(root, arm.field('source_sha').text()))
    harness = context.admitted_harness(session, original)
    expected = projected_manifest(harness, original)
    before = original.field('tracked_manifest').mapping()
    changes: list[Json] = []
    roles: list[Json] = []
    names: list[str] = []
    for name, raw in sorted(expected.items()):
        mode, digest = (field.text() for field in Node(raw).array())
        previous = None if name not in before else Node(before[name]).array()[2].text()
        if previous == digest:
            if name in before:
                Node(before[name]).array()[0].equals(mode)
            continue
        require(name.startswith('tools/v8/') and name.endswith(('.py', '.sh')),
                name, 'non-executable-H replacement')
        require(mode == ('100644' if name not in before else Node(before[name]).array()[0].text()),
                name, 'generic overlay cannot change original modes or add executable mode')
        names.append(name)
        changes.append({'path': name, 'base0_sha256': previous, 'observer_sha256': digest,
                        'operation': 'add' if previous is None else 'modify'})
        roles.append({'path': name, 'role': 'H', 'post_sha256': digest})
    require(bool(names), '/overlay', 'no H differences; use existing pristine build path, not a dummy overlay')
    hroot = Path(harness.field('root').text())
    protected = (root, hroot, *(Path(Node(value).field('root').text()) for value in session.field('arms').mapping().values()))
    require(output.is_absolute() and output.parent == output.parent.resolve(strict=True)
            and not output.exists() and not output.is_symlink()
            and all(not output.is_relative_to(path) and not path.is_relative_to(output) for path in protected),
            '/output', 'fresh external overlay output required')
    patch = git(hroot, 'diff', '--no-ext-diff', '--no-textconv', '--binary', '--full-index', '--no-renames',
                original.field('source_sha').text(), harness.field('source_sha').text(), '--', *names)
    require(bool(patch), '/overlay', 'missing actual H diff')
    output.mkdir(mode=0o700)
    patch_path, manifest_path = output / 'overlay.patch', output / 'overlay.json'
    _ = patch_path.write_bytes(patch)
    manifest: dict[str, Json] = {'schema_version': 1, 'product_base_sha': original.field('source_sha').value,
        'observer_patch_sha256': file_hash(patch_path), 'allowed_paths': [name for name in names], 'files': changes}
    _ = manifest_path.write_text(json.dumps(manifest, sort_keys=True))
    _ = (output / 'roles.json').write_text(json.dumps({'schema': 'whole_gate_layers.v1',
        'product_sha': original.field('source_sha').value, 'harness_sha': harness.field('source_sha').value,
        'files': roles}, sort_keys=True))
    _ = (output / 'projection.json').write_text(json.dumps({'schema': 'original_product_executable_h.v1',
        'session_path': str(context.session_path), 'session_sha256': context.session_sha256, 'arm': context.arm,
        'original_source': original.value, 'expected_manifest': expected,
        'build_status': 'NOT_RUN', 'qualification': 'UNAVAILABLE'}, sort_keys=True))
    require(freeze_source(root, original.field('source_sha').text()) == original.mapping(),
            '/source', 'original product changed while preparing H overlay')
    _ = context.admitted_harness(session, original)
    return Overlay(patch_path, file_hash(patch_path), manifest_path, file_hash(manifest_path))
