"""Whole-gate measurement admission only; never launches, terminates or invents children."""
import json
import math
import os
import sys
import time
from collections.abc import Callable
from pathlib import Path

from core_p7_admission import AdmissionError, Loads, admit
from generic_gate import document, require
from generic_process import file_hash
from mux2_gate_inputs import Node

DEADLINE_ENV = 'WORLDSIM_MEASUREMENT_DEADLINE_EPOCH'


def deadline_epoch(node: Node) -> float:
    value = node.value
    require(type(value) in (int, float), '/measurement_deadline_epoch', 'finite numeric deadline required, not bool')
    assert isinstance(value, (int, float))
    require(math.isfinite(value) and value > 0, '/measurement_deadline_epoch', 'finite positive deadline required')
    return float(value)


def session_deadline(session: Node) -> float:
    """Rebind the reflected deadline to the exact original selector bytes."""
    deadline = deadline_epoch(session.field('measurement_deadline_epoch'))
    path = Path(session.field('selector_path').text())
    require(file_hash(path) == session.field('selector_sha256').text(), '/selector', 'deadline selector changed')
    document(path.read_text()).field('measurement_deadline_epoch').equals(session.field('measurement_deadline_epoch').value)
    return deadline


def admit_measurement(log: Path, label: str, deadline: float, *,
                      sample: Callable[[], Loads] = os.getloadavg,
                      clock: Callable[[], float] = time.time,
                      sleep: Callable[[float], None] = time.sleep) -> Loads:
    """Reuse fixed <8/window policy; final deadline check prevents a late-sample launch."""
    _ = deadline_epoch(Node(deadline))
    def checked_sample() -> Loads:
        loads = sample()
        if len(loads) != 3 or any(type(value) not in (int, float) or not math.isfinite(value) or value < 0 for value in loads):
            raise AdmissionError('malformed 1/5/15 load sample')
        return loads
    loads = admit(log, label, deadline=deadline, sample=checked_sample, clock=clock, sleep=sleep)
    now = clock()
    if now >= deadline:
        with log.open('a') as stream:
            _ = stream.write(json.dumps({'label': label, 'utc': now, 'loads_1_5_15': loads,
                'status': 'NOT_RUN', 'reason': 'owner hard stop reached after admission sample'})+'\n')
        raise AdmissionError('owner hard stop reached after admission sample')
    return loads


def main(arguments: list[str]) -> int:
    """Admission-only shell boundary: caller runs its unchanged command after exit zero."""
    require(len(arguments) == 3, '/argv', 'expected LOG LABEL DEADLINE_EPOCH; commands are not accepted')
    log, label, raw_deadline = arguments
    try:
        deadline = deadline_epoch(Node(float(raw_deadline)))
        _ = admit_measurement(Path(log), label, deadline)
    except (AdmissionError, ValueError, OSError) as error:
        print(f'MEASUREMENT NOT_RUN: {error}', flush=True)
        return 75
    return 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
