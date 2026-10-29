"""Replay frozen CORE fixtures as history, with their original reviewed policy."""
from __future__ import annotations

import hashlib
import subprocess
from pathlib import Path

import core_perf_check as perf
import core_shadow_freeze as freeze
import core_ui_successor as old
from m6a_evidence import Node, write_json
from m6a_gate_source import Gate, need


def run(root: Path, command: list[str | Path], log: Path) -> None:
    with log.open('x') as stream:
        subprocess.run([str(arg) for arg in command], cwd=root, stdout=stream,
                       stderr=subprocess.STDOUT, check=True)


def historical(gate: Gate) -> tuple[freeze.Document, freeze.Document]:
    """Authenticate fixed published pins, never invoke UI successor source checks."""
    return old.history(gate.root, freeze.load(gate.path))


def replay(gate: Gate, output: Path) -> None:
    _, config = historical(gate)
    manifest_path = Path(str(config['manifest']))
    freeze.pin(manifest_path, config['manifest_sha256'])
    manifest = freeze.load(manifest_path)
    shadow = freeze.read_reference(gate.root, manifest['shadow'])
    control = freeze.read_reference(gate.root, manifest['control'])
    policy = old.inventory.git(gate.root, 'show',
        old.RUNTIME + ':rust/crates/sim-core/data/colony/mind.ron')
    policy_sha = hashlib.sha256(policy).hexdigest()
    calibration = Node(config['calibration'])
    need({key for key, _ in calibration.fields()} == {'on', 'off'}, 'missing historical modes')
    helper = gate.root / 'tools/v8/core_gate_checks.sh'
    for steward in ('off', 'on'):
        approved = calibration.get(steward)
        paths = {name: freeze.read_reference(gate.root, value.value)
                 for name, value in approved.fields()}
        need(set(paths) == {'shadow', 'control', 'feature_off', 'review'}, 'historical arms missing')
        run(gate.root, ['bash', helper, 'shadow', paths['shadow'], paths['control'],
                        paths['feature_off'], paths['review']], output / f'calibration-{steward}.log')
        review = Node.load(paths['review'])
        inputs = Node.load(paths['shadow']).get('inputs')
        need(review.get('policy_sha256').text() == policy_sha, 'historical policy changed')
        need(inputs.get('steward').boolean() == (steward == 'on'), 'wrong historical steward')
        flags = ['--scene', 'colony', '--seeds', '0..1', '--days', '20', '--workers', '1',
                 '--steward', steward]
        if inputs.get('defence').value is not None:
            flags += ['--defence', 'on' if inputs.get('defence').boolean() else 'off']
        for name, allowed in [('story', ('on', 'off', 'no-beasts')), ('neighbours', ('on', 'off'))]:
            value = inputs.get(name).text()
            need(value in allowed, 'invalid historical world input')
            flags += ['--' + name, value]
        mode = output / steward
        mode.mkdir()
        for name, binary, prefix in [('observed', shadow, ['mind-shadow', '--observe', 'on']),
                ('control', shadow, ['mind-shadow', '--observe', 'off']), ('feature-off', control, [])]:
            historical(gate)
            run(gate.root, [binary, *prefix, *flags, '--out', mode / name], mode / f'{name}.log')
        observed = mode / 'observed/mind_shadow.json'
        twin = mode / 'control/mind_shadow.json'
        off = mode / 'feature-off/summary.json'
        categories: freeze.Document = {}
        for world in Node.load(observed).get('worlds').rows():
            for key, count in world.get('observation').get('disagreement_categories').fields():
                category = {name: item.value for name, item in review.get('categories').get(key).fields()}
                category['count'] = count.integer()
                categories[key] = category
        fixture_review = {'schema_version': 1, 'fixture_run': True,
            'reviewer': review.get('reviewer').value, 'categories': categories,
            'feature_off_inputs': inputs.value, 'policy_sha256': policy_sha,
            'shadow_sha256': freeze.digest(observed), 'control_sha256': freeze.digest(twin),
            'feature_off_sha256': freeze.digest(off), 'category_review_origin': approved.get('review').value,
            'review_method': 'Historical frozen policy; fresh fixture counts, no M6a equivalence claim.',
            'freeze_manifest_sha256': config['manifest_sha256'], 'fixture_config_sha256': gate.checksum}
        review_path = mode / 'fixture-review.json'
        write_json(review_path, fixture_review)
        run(gate.root, ['bash', helper, 'fixture', observed, twin, off, review_path], mode / 'validation.log')
    historical(gate)


def recompute(gate: Gate, output: Path) -> None:
    """Preserve historical raw failures exactly, without adopting their exceptions."""
    evaluation, _ = historical(gate)
    original_path = freeze.read_reference(gate.root, evaluation['perf_config'])
    original = perf.Document.read(original_path)
    expected = old.inventory.committed(gate.root, old.BASE, 'tools/v8')['tools/v8/core_perf_check.py']
    need(freeze.digest(Path(perf.__file__).resolve()) == expected, 'historical raw verifier changed')
    original_verify = perf.verify_config
    original_source = perf.source_hash
    measured = original.text('candidate_head')
    need(measured == old.RUNTIME, 'historical measured source differs')
    inventory = old.inventory.committed(gate.root, measured, 'rust')
    digest = hashlib.sha256()
    for name, checksum in sorted(inventory.items()):
        raw = name.encode()
        digest.update(len(raw).to_bytes(8, 'little'))
        digest.update(raw)
        digest.update(bytes.fromhex(checksum))
    historical_source = digest.hexdigest()
    need(historical_source == original.text('candidate_source_sha256'), 'historical committed source differs')

    def mapped(config: perf.Document) -> None:
        need(config.values == original.values, 'historical mapping input changed')
        value = dict(config.values)
        value['repo'] = str(gate.root)
        # Recompute historical evidence against its actual committed source, not
        # today's behavior-changing M6a source. Every other old check is retained.
        perf.source_hash = lambda: historical_source
        try:
            original_verify(perf.Document.parse(value))
        finally:
            perf.source_hash = original_source

    perf.verify_config = mapped
    try:
        code = perf.check(original_path.parent, output)
    finally:
        perf.verify_config = original_verify
    receipt = freeze.read_reference(gate.root, evaluation['perf_receipt'])
    need(output.read_bytes() == receipt.read_bytes(), 'historical raw result changed')
    old.same_historical_result(freeze.load(receipt), freeze.load(output), code)
    print('CORE_HISTORY original_verifier=NOT_RERUN; frozen raw result retained, not M6a acceptance')
