"""Pass-specific real-simulation successor for the M6a 48-step gate."""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from typing import Final

import core_shadow_freeze as freeze
from m6a_evidence import EvidenceError, Node, reference_json, write_json
from m6a_gate_evidence import calibration, final, seal
from m6a_gate_history import historical, recompute, replay, run
from m6a_gate_source import Gate, need, ref, source_digest
import core_p7_publication as inventory
from m6a_gate_story import story
from m6a_gate_art import structural

ROOT: Final = Path(__file__).resolve().parents[2]


def fixtures(gate: Gate, path: Path) -> None:
    output = gate.destination(path, 'core-shadow-fixture')
    replay(gate, output)
    binary = gate.binary()
    for mode in ('off', 'on'):
        flags = ['--scene', 'colony', '--seeds', '0..1', '--days', '20', '--workers', '1',
                 '--steward', mode, '--defence', 'on', '--story', 'on', '--neighbours', 'on',
                 '--check-determinism']
        for twin in ('a', 'b'):
            gate.verify()
            log = output / f'candidate-{mode}-{twin}.log'
            run(gate.root, [binary, *flags, '--out', output / f'candidate-{mode}-{twin}'], log)
            need('DETERMINISM PASS' in log.read_text(), 'candidate twin check did not finish')
        a = output / f'candidate-{mode}-a/summary.json'
        b = output / f'candidate-{mode}-b/summary.json'
        need(a.read_bytes() == b.read_bytes(), 'candidate lifecycle fixture is nondeterministic')
    seal(gate, output)
    print('CORE_SHADOW PASS historical frozen policy replay + fresh candidate twins; food invariants step47')


def food_fixture(gate: Gate, path: Path) -> None:
    output = gate.destination(path, 'm6a-food-fixture')
    run(gate.root, [sys.executable, 'tools/v8/m6a_run.py', 'cohort', '--binary', gate.binary(),
        '--source-root', gate.root, '--source-head', gate.head, '--seeds', '0..2', '--days', '72',
        '--purpose', 'fixture', '--protocol', gate.protocol().name, '--out', output / 'cohort'], output / 'run.log')
    run(gate.root, [sys.executable, 'tools/v8/m6a_score.py', 'fixture', '--receipt',
        output / 'cohort/receipt.json', '--out', output / 'score.json'], output / 'score.log')
    seal(gate, output)


def data_only(gate: Gate, path: Path) -> None:
    output = gate.destination(path, 'm6a-food-data-only')
    run(gate.root, [sys.executable, 'tools/v8/m6a_data_only.py', '--binary', gate.binary(),
        '--source-root', gate.root, '--source-head', gate.head, '--out', output / 'fixture'], output / 'run.log')
    structural(gate, output)
    seal(gate, output)


def evaluation(gate: Gate, path: Path) -> None:
    output = gate.destination(path, 'core-evaluation')
    historical(gate)
    calibration(gate)
    heldout = gate.data.get('heldout')
    run(gate.root, [sys.executable, 'tools/v8/m6a_score.py', 'score', '--candidate',
        ref(gate.root, heldout.get('candidate')), '--base', ref(gate.root, heldout.get('base')),
        '--out', output / 'outcomes.json'], output / 'outcomes.log')
    story(gate, output / 'story.json')
    recompute(gate, output / 'historical-performance.json')
    run(gate.root, [sys.executable, 'tools/v8/m6a_perf.py', 'check', ref(gate.root, gate.data.get('perf')),
                   gate.output / 'bench/perf-session'], output / 'performance.log')
    identity_log = gate.output / 'identity.log'
    text = identity_log.read_text()
    for prefix in ('summary: PASS:', 'chronicle: PASS:', 'raw samples: PASS:', 'determinism: PASS:'):
        need(prefix in text, 'fresh baseline identity incomplete: ' + prefix)
    need('FAIL' not in text, 'baseline identity failure retained')
    identity = gate.output / 'identity'
    files = sorted(path for path in identity.rglob('*') if path.is_file())
    need(bool(files), 'fresh baseline identity outputs absent')
    write_json(output / 'identity.json', {'schema_version': 1, 'head': gate.head,
        'log': reference_json(identity_log), 'files': [reference_json(path) for path in files]})
    seal(gate, output)
    print('CORE_PERF M6A_PASS fresh all-scene paired bars/C3; no historical exceptions')
    print('CORE_EVALUATION M6A_PASS fresh heldout observations, story, identity, performance')


def prepare(template: Path, output: Path) -> None:
    """Seal reviewed external input references after committing the gate source."""
    need(freeze.external(ROOT, template) != freeze.external(ROOT, output), 'prepare requires a new output')
    data = {name: value.value for name, value in Node.load(template).fields()}
    head = inventory.git(ROOT, 'rev-parse', 'HEAD').decode().strip()
    data['final_head'] = head
    data['runtime_sha256'] = source_digest(ROOT, head)
    write_json(output, data)
    print('M6A_GATE_CONFIG=' + str(output))
    print('M6A_GATE_CONFIG_SHA256=' + freeze.digest(output))
    print('Preparation seals inputs; preflight must pass before any gate step.')


def main() -> int:
    if len(sys.argv) == 4 and sys.argv[1] == 'prepare':
        prepare(Path(sys.argv[2]), Path(sys.argv[3]))
        return 0
    if len(sys.argv) != 3:
        raise EvidenceError('usage: m6a_gate.py preflight|fixtures|evaluation|food_fixture|data_only|final OUTPUT')
    operation, output = sys.argv[1], Path(sys.argv[2])
    gate = Gate.load(ROOT)
    match operation:
        case 'preflight':
            need(freeze.external(gate.root, output) == gate.output, 'gate directory differs from config')
            need(os.environ.get('M6A_DEADLINE_EPOCH') == str(gate.data.get('deadline_epoch').integer()),
                 'deadline differs from config')
            historical(gate)
            calibration(gate)
        case 'fixtures':
            fixtures(gate, output)
        case 'evaluation':
            evaluation(gate, output)
        case 'food_fixture':
            food_fixture(gate, output)
        case 'data_only':
            data_only(gate, output)
        case 'stall':
            need(output == gate.output / 'mux2-stall', 'wrong stall output')
            if gate.repair_r1():
                command = [sys.executable, 'tools/v8/m6a_r1_stall.py', 'capture',
                           '--root', str(gate.root), '--out', str(output),
                           '--godot', os.environ['GODOT_BIN']]
            else:
                command = ['bash', 'tools/v8/mux2_gate_checks.sh', 'stall', str(output)]
            subprocess.run(command, cwd=gate.root, check=True)
        case 'final':
            need(output == gate.output, 'wrong final output')
            final(gate)
        case _:
            raise EvidenceError('unknown gate operation: ' + operation)
    gate.verify()
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (EvidenceError, freeze.FreezeError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit('M6A_GATE FAIL: ' + str(error)) from error
