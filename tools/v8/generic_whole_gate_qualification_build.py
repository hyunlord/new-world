# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_qualification_build
"""Recheck generic_build's actual release build contract for qualification."""
import json
import os
import tomllib
from pathlib import Path

from generic_gate import document, require
from generic_process import file_hash
from generic_sources import git, inventory
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Node


def authenticate(session: Node, build: Node, process: Node, product: str, mode: str) -> None:
    selector_path = Path(session.field('selector_path').text())
    require(file_hash(selector_path) == session.field('selector_sha256').text(), '/selector', 'original selector changed')
    selected = document(selector_path.read_text()).field('selectors')
    cargo = selected.field('cargo')
    rustc = selected.field('rustc')
    for name, tool in (('cargo', cargo), ('rustc', rustc)):
        path = Path(tool.field('path').text())
        digest = tool.field('sha256').text()
        session.field('external_inputs').field(str(path)).equals(digest)
        require(file_hash(path) == digest and os.access(path, os.X_OK), name, 'selected build tool changed or not executable')
        build.field(name+'_sha256').equals(digest)
    require(product in {'sim_bridge', 'worldsim_probe', 'worldsim_perf'}, '/product', 'unsupported qualification product')
    bridge = product == 'sim_bridge'
    profile = 'debug' if bridge else 'release'
    package = 'sim-bridge' if bridge else 'sim-test'
    expected = [cargo.field('path').text(), 'build', '--locked', '--message-format=json-render-diagnostics', *(() if bridge else ('--release',)), '-p', package]
    build.field('argv').equals([arg for arg in expected])
    process.field('argv').equals([arg for arg in expected])
    process.field('errors').equals([])
    build.field('profile').equals(profile)
    build.field('packages').equals([package])
    build.field('features').equals([])
    build.field('source_kind').equals('PRISTINE_COMMIT' if mode == 'original_off' else 'DERIVED_OBSERVER')
    root = Path(build.field('root').text())
    require(root == root.resolve(strict=True), '/root', 'noncanonical build root')
    require(git(root, 'rev-parse', 'HEAD').decode().strip() == build.field('source_sha').text(), '/source', 'build source HEAD changed')
    process.field('cwd').equals(str(root/'rust'))
    process.field('input_root').equals(str(root))
    process.field('executable').equals(cargo.field('path').text())
    process.field('expected_executable_sha256').equals(cargo.field('sha256').text())
    inputs = inventory(root, [*INPUT_DIRS, *INPUT_FILES])
    require(bool(inputs), '/build/inputs', 'empty original build closure')
    build.field('effective_inputs_before').equals({name: digest for name, digest in inputs.items()})
    process.field('expected_inputs').equals({name: digest for name, digest in inputs.items()})
    environment = process.field('environment')
    names = {'PATH','CARGO_HOME','CARGO_TARGET_DIR','RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER'}
    require(set(build.field('environment').mapping()) == names, '/build/environment', 'unreviewed build override')
    require(set(environment.mapping()) == names | {'LANG','LC_ALL'}, '/build/environment', 'unreviewed process override')
    environment.field('LANG').equals('C')
    environment.field('LC_ALL').equals('C')
    environment.field('PATH').equals(str(Path(rustc.field('path').text()).parent)+':/usr/bin:/bin:/usr/sbin:/sbin')
    for key, value in build.field('environment').mapping().items():
        environment.field(key).equals(value)
    cargo_home = Path(environment.field('CARGO_HOME').text())
    config_paths = {parent/'.cargo'/name for parent in (root/'rust', *(root/'rust').parents) for name in ('config','config.toml')}
    config_paths.update(cargo_home/name for name in ('config','config.toml'))
    configs = {str(path): file_hash(path) for path in config_paths if path.exists() or path.is_symlink()}
    build.field('cargo_configs').equals({name: digest for name, digest in configs.items()})
    environment.field('RUSTC').equals(rustc.field('path').text())
    environment.field('RUSTC_WRAPPER').equals('')
    environment.field('RUSTC_WORKSPACE_WRAPPER').equals('')
    target = root/'rust/target'
    build.field('target_dir').equals(str(target))
    environment.field('CARGO_TARGET_DIR').equals(str(target))
    binary = build.field('artifacts').field(product)
    path = target/profile/('libsim_bridge.dylib' if bridge else product)
    binary.field('path').equals(str(path))
    require(os.access(path, os.X_OK), '/binary', 'selected compiled artifact is not executable')
    require(file_hash(path) == binary.field('sha256').text(), '/binary', 'selected compiled artifact changed')
    output = Path(process.field('output_root').text())
    receipt = Path(build.field('process_receipt').text())
    require(receipt.name == 'process.json' and receipt.parent.name == 'cargo-build', '/build/process', 'not original cargo-build receipt')
    require(output == receipt.parent, '/build/process', 'foreign build output')
    messages = [document(line) for line in (output/'stdout.log').read_text().splitlines()]
    terminals = [row for row in messages if row.field('reason').text() == 'build-finished']
    require(len(terminals) == 1 and messages[-1].value == terminals[0].value, '/build', 'missing or repeated final Cargo terminal')
    terminals[0].field('success').equals(True)
    matches = [row for row in messages if row.field('reason').text() == 'compiler-artifact'
               and row.field('target').field('name').text() == product]
    require(len(matches) == 1, '/binary', 'missing or repeated actual Cargo compiler artifact')
    artifact = matches[0]
    artifact.field('fresh').equals(False)
    artifact.field('target').field('kind').equals(['cdylib', 'rlib'] if bridge else ['bin'])
    artifact.field('target').field('src_path').equals(str(root/'rust/crates/sim-bridge/src/lib.rs') if bridge else str(root/'rust/crates/sim-test/src/bin'/f'{product}.rs'))
    workspace = document(json.dumps(tomllib.loads((root/'rust/Cargo.toml').read_text())))
    version = workspace.field('workspace').field('package').field('version').text()
    artifact.field('package_id').equals('path+'+(root/'rust/crates'/package).as_uri()+'#'+version)
    artifact.field('profile').field('debug_assertions').equals(bridge)
    artifact.field('profile').field('test').equals(False)
    artifact.field('executable').equals(None if bridge else str(path))
    require(str(path) in [item.text() for item in artifact.field('filenames').array()], '/binary', 'compiled file differs')
