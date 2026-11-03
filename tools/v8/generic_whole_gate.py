# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 generic_whole_gate.py step SESSION SHA ARM STEP NAME -- ARGV...
"""Actual whole-shell/stage execution. Aggregate waits are not native-world census."""
import os
import signal
import sys
from pathlib import Path

from generic_gate import document, require
from mux2_gate_inputs import Json, Node

from generic_process import (
    ProcessResult,
    atomic_json,
    environment_for,
    execute,
    file_hash,
)
from generic_whole_gate_contract import load_session, stage_root
from generic_whole_gate_environment import process_spec
from generic_whole_gate_journal import (
    ORDER,
    REFERENCE,
    anchored,
    begin,
    entries,
    publish,
    publish_terminal,
    recheck,
    snapshot,
)


def run_step(path: Path, digest: str, arm: str, step: int, name: str,
             argv: tuple[str, ...], descriptor: int | None = None) -> ProcessResult:
    """Retain original direct return and freeze outputs before collector evaluation."""
    session = load_session(path, digest)
    ordinal, previous = begin(session, arm, step)
    output = stage_root(session, arm, step, name)
    output.mkdir(parents=True, exist_ok=False)
    from generic_whole_gate_observations import route
    additions = route(session, step, output)
    for key in ('WORLDSIM_GATE_TEST_OUT', 'WORLDSIM_GATE_UI_OUT'):
        if key in additions:
            Path(additions[key]).mkdir(mode=0o700)
    spec = process_spec(session, arm, argv, output / 'process', additions, descriptor)
    result = execute(spec)
    artifacts, directories = snapshot(output)
    original_stage = Node(anchored(output))
    from generic_whole_gate_outputs import retain
    producer_anchors, producer_files, producer_modes = retain(session, arm, step)
    artifacts.update(producer_files)
    directories.update(producer_modes)
    _ = load_session(path, digest)
    from generic_whole_gate_products import publish_products, selected
    payload: dict[str, Json] = {
        'schema': 'whole_gate_step.v1', 'arm': arm, 'step': step, 'name': name,
        'argv': list(argv), 'cwd': str(spec.cwd), 'environment': {k:v for k,v in environment_for(spec).items()},
        'state': 'RETURNED' if result.native_exit is not None and result.native_exit >= 0 else 'UNKNOWN',
        'native_exit': result.native_exit, 'pid': result.pid, 'process_status': result.status,
        'process_receipt': str(result.receipt), 'process_receipt_sha256': file_hash(result.receipt),
        'observer_artifacts': dict(artifacts), 'observer_directories': dict(directories),
        'producer_anchors': producer_anchors, 'observer_modes': dict(directories),
        'descendant_coverage': 'REQUIRES_SOURCE_QUALIFIED_COLLECTOR',
        'selected_artifacts': selected(session, arm, step, result.native_exit),
    }
    recheck(output, original_stage)
    acceptance = output/'acceptance'
    if (acceptance/'acceptance.json').exists():
        payload['acceptance_anchor'] = {'path': str(acceptance/'acceptance.json'),
            'sha256': file_hash(acceptance/'acceptance.json'), **anchored(acceptance)}
    terminal = output/'stage.json'
    publish_terminal(output, terminal, payload, original_stage)
    publish(session, arm, step, terminal, ordinal, previous)
    publish_products(session, arm, terminal)
    for filename, expected in artifacts.items():
        require(file_hash(Path(filename)) == expected, filename, 'returned artifact changed')
    _ = sys.stdout.buffer.write((spec.output_root / 'stdout.log').read_bytes())
    _ = sys.stdout.buffer.flush()
    return result


def run_shell(path: Path, digest: str, script: Path, descriptor: int | None = None,
              arm: str = 'candidate') -> Path:
    """Run one actual candidate shell then evaluate its genuine complete stage journal."""
    session = load_session(path, digest)
    require(arm in {'base', 'candidate'}, '/arm', 'unknown shell arm')
    root = Path(session.field('arms').field(arm).field('root').text())
    require(script == root / 'tools/v8/gate.sh', '/shell', 'only materialized normal gate entrypoint')
    output = Path(session.field('output').text())
    spec = process_spec(session, arm, (str(script), '--whole-session', str(path), digest, arm), output / f'{arm}-shell', {}, descriptor)
    actual = execute(spec)
    if (actual.native_exit is None or actual.native_exit < 0 or actual.native_exit == 125) and descriptor is not None:
        retain_unknown(session, descriptor)
    ordered = entries(session, arm)
    original_outputs = Node(anchored(output))
    from generic_whole_gate_observations import evaluate
    evaluation: dict[str, Json] = ({'status': 'REFERENCE_ONLY', 'publication_authority': False}
        if arm == 'base' else evaluate(session, entries(session, 'base') + ordered, output, descriptor=descriptor))
    recheck(output, original_outputs)
    complete = tuple(int(row.field('step').number()) for row in ordered) == (REFERENCE if arm == 'base' else ORDER)
    from generic_whole_gate_outcome import arm_clean
    evaluation_node = Node(evaluation)
    if evaluation_node.field('status').text() == 'PASS' and 'waived_stage_exits' not in evaluation_node.mapping():
        evaluation['waived_stage_exits'] = []
    if evaluation_node.field('status').text() == 'PASS' and 'reference_stage_exits' not in evaluation_node.mapping():
        evaluation['reference_stage_exits'] = []
    clean = arm_clean(session, arm, ordered, document(actual.receipt.read_text()), Node(evaluation))
    _ = load_session(path, digest)
    base_ok = arm == 'base'
    if arm == 'candidate' and (output/'base-gate.json').exists():
        original_base = document((output/'base-gate.json').read_text())
        base_receipt = Path(original_base.field('shell_receipt').text())
        require(file_hash(base_receipt) == original_base.field('shell_receipt_sha256').text(), '/base', 'original BASE shell changed')
        base_ok = arm_clean(session, 'base', entries(session, 'base'), document(base_receipt.read_text()), Node(evaluation))
    final: dict[str, Json] = {
        'schema': 'whole_gate_result.v1', 'status': 'PASS' if session.field('qualification').text() == 'PRODUCT_QUALIFIED' and complete and clean
        and base_ok and evaluation.get('status') in {'PASS', 'REFERENCE_ONLY'} else 'FAIL', 'shell_exit': actual.native_exit,
        'shell_receipt': str(actual.receipt), 'shell_receipt_sha256': file_hash(actual.receipt),
        'complete_sequence': complete, 'evaluation': evaluation,
        'qualification': session.field('qualification').value, 'arm': arm,
        'qualification_gap': 'O neutrality must be bound by a separately reviewed qualification reader',
        'terminal_state': 'UNKNOWN' if actual.native_exit is None or actual.native_exit < 0 or actual.native_exit == 125 else 'RETURNED',
    }
    terminal = output / ('base-gate.json' if arm == 'base' else 'whole-gate.json')
    publish_terminal(output, terminal, final, original_outputs)
    return terminal


def retain_unknown(session: Node, descriptor: int | None) -> None:
    """Keep the real inherited lane owner alive until external ownership reconciliation."""
    require(descriptor is not None and descriptor >= 3, '/owner', 'UNKNOWN requires inherited live lane')
    assert descriptor is not None
    info = os.fstat(descriptor)
    atomic_json(Path(session.field('output').text())/'ownership-unknown.json', {
        'status': 'UNKNOWN', 'owner_pid': os.getpid(), 'descriptor': descriptor,
        'device': info.st_dev, 'inode': info.st_ino, 'mode': info.st_mode,
        'release_authority': False, 'reason': 'actual shell or acceptance descendant settlement unresolved'})
    while True:
        signal.pause()


def main() -> int:
    arguments = sys.argv[1:]
    require(bool(arguments), '/argv', 'operation required')
    operation, *rest = arguments
    descriptor_text = os.environ.get('WORLDSIM_MUX2_LANE_FD')
    descriptor = None if descriptor_text is None else int(descriptor_text)
    if operation == 'prepare':
        require(descriptor is not None, '/owner', 'prepare requires inherited owner; use bash tools/v8/gate.sh SELECTOR')
        from mux2_gate_lock import lane
        with lane() as held:
            require(held == descriptor, '/owner', 'inherited owner descriptor differs')
        require(len(rest) == 1, '/argv', 'prepare requires one selector')
        from generic_whole_gate_source import prepare_session
        path = prepare_session(Path(rest[0]))
        digest = file_hash(path)
        session = load_session(path, digest)
        # Missing observation routes must be resolved before spending a native run.
        from generic_fixture_registry import load_registry
        from generic_whole_gate_observations import routes
        missing = [row.fixture for row in routes(load_registry(Path(session.field('registry').field('path').text())))
                   if not row.supported]
        if missing:
            atomic_json(Path(session.field('output').text())/'preflight.json',
                        {'status': 'INCOMPLETE', 'missing_routes': list(missing), 'native_execution': 'NOT_RUN'})
            return 1
        reference = run_shell(path, digest,
            Path(session.field('arms').field('base').field('root').text())/'tools/v8/gate.sh', descriptor, 'base')
        if document(reference.read_text()).field('terminal_state').text() == 'UNKNOWN':
            retain_unknown(session, descriptor)
        terminal = run_shell(path, digest,
                             Path(session.field('arms').field('candidate').field('root').text())/'tools/v8/gate.sh', descriptor)
        if document(terminal.read_text()).field('terminal_state').text() == 'UNKNOWN':
            retain_unknown(session, descriptor)
        return 0 if document(terminal.read_text()).field('status').text() == 'PASS' else 1
    require(len(rest) >= 2, '/argv', 'session and hash required')
    path, digest = Path(rest[0]), rest[1]
    session = load_session(path, digest)
    if operation == 'finish':
        require(len(rest) == 5, '/argv', 'finish requires arm and two actual status fields')
        arm, postcheck, stages = rest[2:]
        require(arm in {'base', 'candidate'} and postcheck in {'0', '1'} and stages in {'0', '1'},
                '/summary', 'invalid actual shell status')
        atomic_json(Path(session.field('output').text())/arm/'shell-summary.json', {
            'arm': arm, 'postcheck_status': int(postcheck), 'stage_status': int(stages)})
        return 0
    if operation == 'companions':
        require(len(rest) == 2, '/argv', 'companions takes only original session identity')
        from generic_whole_gate_ui_companions import run_companions
        result = run_companions(path, digest, descriptor)
        _ = load_session(path, digest)
        if result.code == 2:
            retain_unknown(session, descriptor)
        root = Path(session.field('output').text())
        require(result.output == root/'auxiliaries/smoke', '/auxiliary', 'unexpected companion root')
        (root/'auxiliary-index').mkdir(exist_ok=True)
        atomic_json(root/'auxiliary-index/smoke.json', {
            'root': str(result.output), 'code': result.code, 'anchor': anchored(result.output)})
        return result.code
    if operation == 'info':
        require(len(rest) == 4, '/argv', 'info requires arm and key')
        arm, key = rest[2:]
        require(arm in {'base', 'candidate'}, '/arm', 'unknown arm')
        match key:
            case 'output': value = str(Path(session.field('output').text())/arm)
            case 'identity': value = str(Path(session.field('output').text())/'base/identity')
            case 'harness': value = session.field('harness').field('root').text()
            case _: raise ValueError('unknown info key')
        print(value)
        return 0
    require(operation == 'step' and len(rest) >= 6, '/argv', 'unsupported operation')
    arm, number, name, *command = rest[2:]
    if command and command[0] == '--':
        command = command[1:]
    result = run_step(path, digest, arm, int(number), name, tuple(command), descriptor)
    return 125 if result.native_exit is None or result.native_exit < 0 else result.native_exit


if __name__ == '__main__':
    raise SystemExit(main())
