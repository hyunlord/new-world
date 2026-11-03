# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/v8/generic_whole_gate_play.py SESSION SHA ARM OUTPUT
"""Run original six play cases with real native waits and explicitly pending qualification."""
import os
import signal
import sys
from pathlib import Path

from generic_gate import require
from generic_play_owner import PlayOwner, PlayOwnershipUnknown
from generic_process import atomic_json, file_hash
from generic_whole_gate_contract import load_session
from mux2_gate_lock import FD_ENV

from generic_whole_gate_play_plan import build_plan

QUALIFICATION_INPUTS = (
    'main: same session BASE+O bridge control/census and boot lifecycle at 300 frames',
    'm1: exact original built bridge observer-off/on comparison and qualified legacy census',
    'v7: exact original built bridge observer-off/on comparison and qualified legacy census',
    'all: actual import/headless generated-runtime namespace closure and per-invocation settings',
)


def run(session_path: Path, session_sha256: str, arm: str, output: Path, descriptor: int) -> int:
    plan=build_plan(session_path,session_sha256,arm,output)
    owner=PlayOwner(plan,descriptor)
    try:
        owner.launch();outcome=owner.wait()
    except PlayOwnershipUnknown:
        try:
            atomic_json(output/'play-result.json',{'schema':'whole_gate_play_result.v1','settlement':'UNKNOWN',
                'qualification':'INCOMPLETE','owner_output':str(plan.output),'owner_pid':os.getpid(),
                'retained_owner_fd':owner.lane_fd,'pending_native_qualification':list(QUALIFICATION_INPUTS)})
        finally:
            # Diagnostic I/O must never unwind an owner whose children may remain live.
            while True:signal.pause()
    _=load_session(session_path,session_sha256)
    for path,digest in plan.inputs:
        require(file_hash(path)==digest,str(path),'plan input changed after original waits')
    terminal=plan.output/'terminal.json'
    atomic_json(output/'play-result.json',{'schema':'whole_gate_play_result.v1','settlement':outcome.settlement,
        'original_exit':outcome.outer_exit,'original_case_exits':list(outcome.case_exits),
        'qualification':'INCOMPLETE','pending_native_qualification':list(QUALIFICATION_INPUTS),
        'session':str(session_path),'session_sha256':session_sha256,'arm':arm,
        'plan':str(output/'plan.json'),'plan_sha256':file_hash(output/'plan.json'),
        'owner_terminal':str(terminal),'owner_terminal_sha256':file_hash(terminal),
        'projections':{version:projection.declaration() for version,projection in owner.projections.items()}})
    for name,stream in (('stdout.log',sys.stdout),('stderr.log',sys.stderr)):
        _=stream.write((plan.output/name).read_text())
        _=stream.flush()
    return outcome.outer_exit


def main(argv: list[str]) -> int:
    require(len(argv) in {3,4},'/argv','expected SESSION SHA ARM [OUTPUT]')
    raw=os.environ.get(FD_ENV,'');require(raw.isdecimal(),'/owner','inherited actual lane FD required')
    session=load_session(Path(argv[0]),argv[1])
    output=Path(argv[3]) if len(argv)==4 else Path(session.field('output').text())/'steps'/argv[2]/'21-play_check/play-owned'
    return run(Path(argv[0]),argv[1],argv[2],output,int(raw))


if __name__=='__main__':
    raise SystemExit(main(sys.argv[1:]))
