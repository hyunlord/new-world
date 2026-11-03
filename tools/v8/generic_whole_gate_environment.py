# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate
"""Frozen whole-gate process environment shared by real shell and direct producers."""
from pathlib import Path

from generic_gate import require
from mux2_gate_inputs import Node

from generic_process import ProcessSpec
from generic_whole_gate_contract import hashes


def process_spec(session: Node, arm: str, argv: tuple[str, ...], output: Path,
                 additions: dict[str, str], descriptor: int | None) -> ProcessSpec:
    product = session.field('arms').field(arm)
    root = Path(product.field('root').text())
    common = hashes(session.field('environment'))
    arm_environment = hashes(product.field('environment'))
    require(all(key == 'PATH' or common[key] == arm_environment[key]
                for key in set(common) & set(arm_environment)),
            '/environment', 'common environment changes product build identity')
    environment = arm_environment | common
    if 'source_sha' in product.mapping():
        environment['WORLDSIM_GIT_SHA'] = product.field('source_sha').text()
    environment['WORLDSIM_TEST_SETTINGS_DIR'] = str(Path(session.field('output').text())/arm/'settings')
    require(not set(additions) & set(environment), '/environment', 'observer override collision')
    environment.update(additions)
    if descriptor is not None:
        environment['WORLDSIM_MUX2_LANE_FD'] = str(descriptor)
    external = {Path(path): digest for path, digest in hashes(session.field('external_inputs')).items()}
    inputs = hashes(product.field('source_inputs'))
    for path in tuple(external):
        if path.is_relative_to(root):
            name = path.relative_to(root).as_posix()
            require(name not in inputs or inputs[name] == external[path], '/inputs', 'conflicting binding')
            inputs[name] = external.pop(path)
    bash = Path(session.field('bash').field('path').text())
    return ProcessSpec((str(bash), '-c', 'exec "$@" 2>&1', 'whole-step', *argv), root, bash,
                       session.field('bash').field('sha256').text(), root, tuple(sorted(inputs.items())),
                       output, tuple(sorted(environment.items())), tuple(sorted(external.items())),
                       pass_fds=() if descriptor is None else (descriptor,))
