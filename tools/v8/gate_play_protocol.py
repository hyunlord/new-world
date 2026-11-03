"""Play-only foreground request/grant protocol; never grants qualification."""
import os
from pathlib import Path

from gate_oracle_child import require_connected, suite_send
from generic_gate import document, require
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json

REQUEST = 'WORLDSIM_GATE_PLAY_REQUEST_FD'
CONTROL = 'WORLDSIM_GATE_PLAY_CONTROL_FD'
EVIDENCE = 'WORLDSIM_GATE_PLAY_OWNER_OUT'


def enabled() -> bool:
    return REQUEST in os.environ


def exchange(event: str, kind: str, role: str, payload: dict[str, Json]) -> Path:
    """Caller cannot spawn until a durable request is granted by the owner."""
    case = os.environ['WORLDSIM_GATE_PLAY_CASE']
    output = Path(os.environ[EVIDENCE]) / ('case-' + str(case_index(case)))
    if kind == 'child':
        output /= role
    output.mkdir(parents=True, exist_ok=True)
    path = output / (event + '.json')
    require(not path.exists(), '/protocol', 'duplicate request/terminal')
    body: dict[str, Json] = {'event': event, 'kind': kind, 'case': case, 'role': role,
        'wrapper_pid': os.getpid(), 'parent_pid': os.getppid(), 'payload': payload}
    atomic_json(path, body)
    control = int(os.environ[CONTROL]); request = int(os.environ[REQUEST])
    require_connected(control)
    suite_send(request, {'path': str(path), 'sha256': file_hash(path)})
    require(os.read(control, 1) == b'G', '/protocol', 'owner grant/ack missing')
    grant = document(path.with_name(event + '-grant.json').read_text())
    grant.field('request_sha256').equals(file_hash(path))
    grant.field('wrapper_pid').equals(os.getpid())
    grant.field('event').equals(event); grant.field('kind').equals(kind); grant.field('case').equals(case)
    return path


def case_index(case: str) -> int:
    CASES = ('unknown version', 'GODOT_BIN=/nonexistent', 'bad --speed',
             '--headless-check main', '--headless-check m1', '--headless-check v7')
    require(case in CASES, '/case', 'unknown original case')
    return CASES.index(case)


def child_request(role: str, argv: tuple[str, ...], plan: Path, digest: str, projection: dict[str, Json] | None = None) -> None:
    _ = exchange('request', 'child', role, {'argv': list(argv), 'cwd': str(Path.cwd()),
        'plan': str(plan), 'plan_sha256': digest,
        'environment': {k: v for k, v in os.environ.items()},
        **({'projection':projection} if projection is not None else {})})


def child_returned(role: str, receipt: Path) -> None:
    row = document(receipt.read_text())
    _ = exchange('returned', 'child', role, {'receipt': str(receipt), 'sha256': file_hash(receipt),
        'native_exit': row.field('native_exit').value})
