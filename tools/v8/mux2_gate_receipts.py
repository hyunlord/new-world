#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/mux2_gate_receipts.py --help
"""Capture or verify serialized GPU evidence, bound to source content and binary."""
import argparse
import json
import math
import os
import subprocess
import sys
import time
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING, Final, Literal, TypeAlias, assert_never

if TYPE_CHECKING:
    from gate_stall_child import ChildPlan
from generic_stall_build import BuildEvidenceReference, derived_source_binding
from mux2_gate_inputs import (
    BRIDGE,
    EvidenceError,
    Json,
    Node,
    check_hash,
    clean_sources,
    digest,
    external,
    git,
    read_json,
    require,
    same_inputs,
    snapshot,
    validate_capture_status,
)
from mux2_gate_lock import FD_ENV, lane
from mux2_gate_stall import validate_stall
from mux2_gate_visual import validate_visibility, validate_why

Kind: TypeAlias = Literal['stall', 'visibility', 'why']
KINDS: Final = ('stall', 'visibility', 'why')


def now() -> str:
    return datetime.now(UTC).isoformat()


def write(path: Path, value: Json) -> None:
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')
    temporary.replace(path)


def stall_timeout() -> int:
    if 'CORE_P7_RUN' not in os.environ:
        return 960
    deadline = float(os.environ.get('CORE_P7_DEADLINE_EPOCH', 'nan'))
    require(math.isfinite(deadline), 'NOT_RUN: P7 requires finite CORE_P7_DEADLINE_EPOCH')
    remaining = math.floor(deadline - time.time())
    require(remaining >= 650, 'NOT_RUN: P7 deadline cannot fit both stall arms')
    return min(960 + 2 * 8 * 900, remaining)


def validate_p7_admission(report: Node) -> None:
    policy = report.field('p7_admission')
    deadline = policy.field('deadline_epoch').number()
    require(policy.field('wait_seconds').number() == 900, 'P7 wait policy must be 900 seconds')
    policy.field('max_waits').equals(8)
    require(policy.field('load1_limit_exclusive').number() == 8, 'P7 load policy must be below 8')
    for arm in report.field('arms').array():
        admission = arm.field('admission')
        admission.field('status').equals('ADMITTED')
        samples = admission.field('samples').array()
        require(1 <= len(samples) <= 9, 'P7 admission sample count')
        for index, sample in enumerate(samples):
            loads = [value.number() for value in sample.field('loads_1_5_15').array()]
            require(len(loads) == 3 and all(value >= 0 for value in loads), 'P7 load triplet')
            sample.field('waits').equals(index)
            sample.field('sysctl_exit').equals(0)
            final = index == len(samples) - 1
            sample.field('action').equals('ADMITTED' if final else 'WAIT')
            require(loads[0] < 8 if final else loads[0] >= 8, 'P7 admission load threshold')
            if index:
                require(sample.field('epoch').number() - samples[index - 1].field('epoch').number() >= 900,
                        'P7 admission windows shorter than 900 seconds')
        admitted = samples[-1].field('epoch').number()
        started = arm.field('sampling_started_epoch').number()
        require(admitted <= started < deadline, 'P7 admission/start ordering')


def commands(root: Path, output: Path, kind: Kind, godot: Path) -> list[list[str]]:
    base = ['bash', str(root / 'tools/v8/run_bounded.sh')]
    engine = [str(godot), '--path', str(root), '--resolution', '1920x1080']
    match kind:
        case 'stall':
            return [base + [str(stall_timeout())] + engine + ['--always-on-top', '--windowed', '--script', 'res://tools/v8/mux2_stall_test.gd',
                                              '--', str(output / 'run')]]
        case 'visibility':
            return [base + ['1800'] + engine + ['--headless', '--script',
                    'res://tools/v8/mux2_visibility_saves.gd', '--', str(output / 'saves')],
                    base + ['1200'] + engine + ['--script', 'res://tools/v8/mux2_visibility_test.gd',
                    '--', '--manifest', str(output / 'saves/manifest.json'), '--out', str(output / 'run')]]
        case 'why':
            return [base + ['180'] + engine + ['--script', 'res://tools/v8/mux2_why_input_test.gd']]
        case unreachable:
            assert_never(unreachable)


def validate_output(root: Path, out: Path, kind: Kind) -> list[Path]:
    log = out / 'command.log'
    require('SCRIPT ERROR' not in log.read_text(), 'capture script error')
    extra: list[Path] = []
    match kind:
        case 'stall':
            report = read_json(out / 'run/stall-report.json')
            validate_stall(report.value)
            if 'CORE_P7_RUN' in os.environ or 'p7_admission' in report.mapping():
                validate_p7_admission(report)
            report.field('bridge').field('sha256').equals(digest(root / BRIDGE))
            for name, value in report.field('source_sha256').mapping().items():
                check_hash(root / name, Node(value).text())
        case 'visibility':
            validate_visibility(root, out)
        case 'why':
            extra = validate_why(log)
        case unreachable:
            assert_never(unreachable)
    return extra


def capture(root: Path, output: Path, kind: Kind, godot: Path) -> None:
    child_chain = 'WORLDSIM_STALL_CHILD_PLAN' in os.environ or 'WORLDSIM_STALL_CHILD_PLAN_SHA256' in os.environ
    if child_chain:
        require(kind == 'stall' and 'WORLDSIM_STALL_CHILD_PLAN' in os.environ
                and 'WORLDSIM_STALL_CHILD_PLAN_SHA256' in os.environ, 'incomplete or foreign child plan')
    out = external(output, root)
    require(not out.exists(), 'capture output must be new')
    clean_sources(root)
    require(godot.is_file(), 'Godot executable missing')
    out.mkdir(parents=True)
    with lane() as descriptor:
        before = snapshot(root)
        receipt: dict[str, Json] = {'schema': 1, 'kind': kind, 'complete': False,
            'exit_code': -1, 'started_utc': now(), 'source_head': git(root, 'rev-parse', 'HEAD'),
            'root': str(root), 'inputs': before, 'godot': str(godot),
            'godot_sha256': digest(godot), 'artifacts': {}}
        receipt_path = out / 'receipt.json'
        write(receipt_path, receipt)
        env = dict(os.environ, WORLDSIM_TEST_SETTINGS_DIR=str(out / 'settings'),
                   WORLDSIM_MUX2_PROFILE='1', **{FD_ENV: str(descriptor)})
        (out / 'settings').mkdir()
        code = -1
        try:
            with (out / 'command.log').open('w') as log:
                for command in commands(root, out, kind, godot):
                    log.write(json.dumps(command) + '\n')
                    log.flush()
                    code = _capture_command(command, root, env, log.fileno(), descriptor, child_chain)
                    require(code == 0, f'{kind} capture exited {code}')
            same_inputs(before, snapshot(root))
            clean_sources(root)
            require(git(root, 'rev-parse', 'HEAD') == receipt['source_head'], 'commit changed during capture')
            check_hash(godot, Node(receipt['godot_sha256']).text())
            extra = validate_output(root, out, kind)
            artifacts = [p for p in out.rglob('*') if p.is_file() and p != receipt_path]
            receipt['artifacts'] = {str(p): digest(p) for p in sorted(set(artifacts + extra))}
            receipt['complete'] = True
        finally:
            try:
                if child_chain:
                    from gate_stall_child import close_capture
                    close_capture(env, out / 'command.log')
            finally:
                receipt['exit_code'] = code
                receipt['finished_utc'] = now()
                receipt['inputs_end'] = snapshot(root)
                write(receipt_path, receipt)
    print(f'MUX2_CAPTURE_PASS {receipt_path}')


def verify(root: Path, receipt_path: Path, kind: Kind) -> None:
    path = external(receipt_path, root)
    receipt = read_json(path)
    validate_capture_status(receipt.value, kind)
    receipt.field('root').equals(str(root))
    clean_sources(root)
    before = receipt.field('inputs').mapping()
    after = receipt.field('inputs_end').mapping()
    same_inputs(before, after)
    same_inputs(before, snapshot(root))
    head = receipt.field('source_head').text()
    subprocess.run(['git', '-C', str(root), 'merge-base', '--is-ancestor', head, 'HEAD'], check=True)
    check_hash(Path(receipt.field('godot').text()), receipt.field('godot_sha256').text())
    artifacts = receipt.field('artifacts').mapping()
    require(bool(artifacts), 'missing raw artifact hashes')
    for name, value in artifacts.items():
        check_hash(external(Path(name), root), Node(value).text())
    extra = validate_output(root, path.parent, kind)
    actual = [p for p in path.parent.rglob('*') if p.is_file() and p != path]
    same_inputs(artifacts, {str(p): digest(p) for p in sorted(set(actual + extra))})
    print(f'MUX2_RECEIPT_PASS kind={kind} capture_head={head} gate_head={git(root, "rev-parse", "HEAD")}')


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=('capture', 'verify'))
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument('--kind', choices=KINDS, required=True)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--receipt', type=Path)
    parser.add_argument('--godot', type=Path,
                        default=Path(os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot')))
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        match args.action:
            case 'capture':
                require(args.out is not None, '--out required')
                capture(root, args.out, args.kind, args.godot.resolve())
            case 'verify':
                require(args.receipt is not None, '--receipt required')
                verify(root, args.receipt, args.kind)
            case unreachable:
                assert_never(unreachable)
        return 0
    except (EvidenceError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'MUX2_RECEIPT_FAIL {error}', file=sys.stderr)
        return 1





def _capture_command(command: list[str], root: Path, environment: dict[str, str],
                     log_fd: int, lane_fd: int, child_chain: bool) -> int:
    """Share the unchanged original command and descriptor launch path."""
    if child_chain:
        from gate_stall_child import capture_wrapper
        return capture_wrapper(command, root, environment, log_fd, lane_fd)
    return subprocess.run(command, cwd=root, env=environment, stdout=log_fd,
        stderr=subprocess.STDOUT, pass_fds=(lane_fd,), check=False).returncode


def _derived_child_plan(root: Path, out: Path, godot: Path, reference: BuildEvidenceReference) -> 'ChildPlan':
    from gate_stall_child import ChildPlan
    from generic_gate import document as parse
    from generic_process import real_path
    require('WORLDSIM_STALL_CHILD_PLAN' in os.environ and 'WORLDSIM_STALL_CHILD_PLAN_SHA256' in os.environ,
            'both bounded child plan bindings required')
    require(not any(key.startswith('CORE_P7_') for key in os.environ), 'only NONE supported')
    path = Path(os.environ['WORLDSIM_STALL_CHILD_PLAN'])
    sha = os.environ['WORLDSIM_STALL_CHILD_PLAN_SHA256']
    real_path(path, directory=False); check_hash(path, sha)
    node = parse(path.read_text())
    require(set(node.mapping()) == {'schema','protocol','root','capture','evidence','argv','inputs'}, 'invalid child plan keys')
    node.field('schema').equals('stall_bounded_child_v1'); node.field('protocol').equals('NONE')
    node.field('root').equals(str(root)); node.field('capture').equals(str(out))
    argv = tuple(commands(root,out,'stall',godot)[0][3:])
    node.field('argv').equals(list(argv))
    evidence = Path(node.field('evidence').text()); real_path(evidence,directory=True)
    require(all(not a.is_relative_to(b) and not b.is_relative_to(a)
                for a,b in ((root,out),(root,evidence),(out,evidence))), 'overlapping roots')
    inputs = tuple((Path(name),Node(value).text()) for name,value in node.field('inputs').mapping().items())
    required = {(root/name,Node(value).text()) for name,value in snapshot(root).items()}
    required.update({(godot,digest(godot)),(reference.path,reference.sha256)})
    require(required <= set(inputs), 'child plan lacks runtime/reference binding')
    for source, value in inputs:
        real_path(source,directory=False)
        require(not source.is_relative_to(out) and not source.is_relative_to(evidence), 'evidence input overlap')
        check_hash(source,value)
    return ChildPlan(path,sha,root,out,evidence,argv,inputs)


def _derived_chain(plan: 'ChildPlan', environment: dict[str,str], code: int) -> tuple[tuple[str,str],...]:
    """Require terminal evidence before anchoring the completed chain."""
    from gate_stall_child import bind_chain
    from generic_gate import document
    from generic_sources import inventory
    require(code in (0,1), 'unconfirmed native completion')
    for name in ('native-terminal.json','wrapper-terminal.json'):
        terminal = document((plan.evidence/name).read_text())
        terminal.field('native_exit').equals(code); terminal.field('errors').equals([])
        require(terminal.field('duration_ns').number()>0, 'missing terminal duration')
    native = document((plan.evidence/'native-terminal.json').read_text())
    native.field('timed_out').equals(False); native.field('wrapper_exit').equals(code)
    require(native.field('native_pid').number()>0, 'missing native PID')
    chain = tuple(inventory(plan.evidence,[p.name for p in plan.evidence.iterdir()]).items())
    _ = bind_chain(plan,environment,code,chain)
    return chain


def _capture_artifacts(out: Path) -> dict[str,Json]:
    from generic_process import real_path
    artifacts: dict[str,Json] = {}
    for path in sorted(out.rglob('*')):
        real_path(path,directory=path.is_dir())
        if path.is_file() and path != out/'receipt.json':
            artifacts[str(path)] = digest(path)
    return artifacts


@dataclass(frozen=True, slots=True)
class DerivedCaptureCompletion:
    """Live synchronous wrapper/child chain confirmation, not historical authority."""
    child: 'ChildPlan'
    environment: tuple[tuple[str,str], ...]
    chain_files: tuple[tuple[str,str], ...]
    native_exit: int


@dataclass(frozen=True, slots=True)
class OwnedDerivedCapture:
    completion: DerivedCaptureCompletion
    receipt: Path
    sha256: str
    files: tuple[tuple[str,str], ...]


@dataclass(frozen=True, slots=True)
class DerivedCaptureSettledFailure(Exception):
    original: BaseException
    diagnostics: tuple[BaseException, ...]
    completion: DerivedCaptureCompletion


@dataclass(frozen=True, slots=True)
class DerivedCaptureUnknown(Exception):
    original: BaseException
    diagnostics: tuple[BaseException, ...]


@dataclass(slots=True)
class _DerivedCaptureState:
    """Mutable live state advances only after the actual source-owned chain joins."""
    completion: DerivedCaptureCompletion | None = None
    original: BaseException | None = None
    diagnostics: list[BaseException] = field(default_factory=list)


def _derived_error(error: BaseException, state: _DerivedCaptureState | None, *, diagnostic: bool = False) -> str:
    if state is None:
        return str(error)
    if diagnostic:
        state.diagnostics.append(error)
    elif state.original is None:
        state.original=error
    return 'owned derived capture diagnostic failure' if diagnostic else 'owned derived capture operation failed'


def capture_derived_stall_owned(root: Path, output: Path, godot: Path,
                                reference: BuildEvidenceReference) -> OwnedDerivedCapture:
    """Expose live settlement without changing legacy strict1 failure semantics."""
    from generic_sources import inventory
    state=_DerivedCaptureState()
    unclassified: BaseException | None=None
    try:
        _capture_derived_stall(root,output,godot,reference,state)
        require(state.completion is not None, 'owned capture returned without chain confirmation')
        assert state.completion is not None
        files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
        return OwnedDerivedCapture(state.completion,output/'receipt.json',digest(output/'receipt.json'),files)
    except BaseException as error:
        unclassified=error
        raise
    finally:
        if unclassified is not None:
            original=state.original if state.original is not None else unclassified
            diagnostics=tuple(state.diagnostics)+((unclassified,) if unclassified is not original and not any(unclassified is d for d in state.diagnostics) else ())
            if state.completion is None:
                raise DerivedCaptureUnknown(original,diagnostics) from original
            raise DerivedCaptureSettledFailure(original,diagnostics,state.completion) from original


def capture_derived_stall(root: Path, output: Path, godot: Path, reference: BuildEvidenceReference) -> None:
    _capture_derived_stall(root,output,godot,reference)


def _capture_derived_stall(root: Path, output: Path, godot: Path, reference: BuildEvidenceReference,
                           owned: _DerivedCaptureState | None = None) -> None:
    """Reuse caller-owned lane; INCOMPLETE leaves its recovery obligation with the caller."""
    from gate_stall_child import bind_chain, close_capture
    from generic_gate import document
    from generic_sources import inventory
    from mux2_gate_stall import validate_stall_capture
    require(FD_ENV in os.environ, 'derived capture requires a caller-owned inherited lane')
    out = external(output,root)
    require(not out.exists(), 'capture output must be new')
    plan = _derived_child_plan(root,out,godot,reference)
    source = derived_source_binding(root,reference)
    out.mkdir(parents=True)
    with lane() as descriptor:
        before = snapshot(root)
        godot_sha = digest(godot)
        receipt: dict[str, Json] = {'schema':'derived_stall_capture.v1','kind':'stall','complete':False,
            'exit_code':-1,'started_utc':now(),'source_head':source['origin_sha'],'root':str(root),
            'inputs':before,'godot':str(godot),'godot_sha256':godot_sha,'artifacts':{},'derived_binding':source}
        receipt_path = out/'receipt.json'
        env = dict(os.environ,WORLDSIM_TEST_SETTINGS_DIR=str(out/'settings'),WORLDSIM_MUX2_PROFILE='1',**{FD_ENV:str(descriptor)})
        code = -1; primary: str | None = None; cleanup: str | None = None
        postflight: dict[str,Json] = {}; launched = False; confirmed = False; timing_failure = False; oracles_complete = False
        chain: tuple[tuple[str,str],...] = (); artifacts: dict[str,Json] = {}
        write(receipt_path,receipt)
        try:
            require(derived_source_binding(root,reference)==source, 'derived source changed under lane')
            require(_derived_child_plan(root,out,godot,reference)==plan, 'child plan changed under lane')
            (out/'settings').mkdir()
            try:
                with (out/'command.log').open('w') as log:
                    command = commands(root,out,'stall',godot)[0]
                    _ = log.write(json.dumps(command)+'\n');log.flush()
                    launched = True
                    code = _capture_command(command,root,env,log.fileno(),descriptor,True)
            except BaseException as error:
                if owned is not None:
                    _ = _derived_error(error,owned)
                raise
            finally:
                try:
                    if launched:
                        close_capture(env,out/'command.log')
                except BaseException as error:
                    if owned is None and not isinstance(error,(OSError,ValueError)):
                        raise
                    cleanup = _derived_error(error,owned,diagnostic=True)
            chain = _derived_chain(plan,env,code)
            confirmed = True
            if owned is not None:
                owned.completion=DerivedCaptureCompletion(plan,tuple(env.items()),chain,code)
            # The first producer anchor precedes every timing/output oracle.
            artifacts = _capture_artifacts(out)
            arms = validate_stall_capture(document((out/'run/stall-report.json').read_text()).value, strict=False)
            require(code == (0 if all(arm.strict_pass for arm in arms) else 1), 'strict timing/native exit differ')
            timing_failure = code == 1
            if code == 0:
                _ = validate_output(root,out,'stall')
            oracles_complete = True
        except (OSError,ValueError,subprocess.SubprocessError) as error:
            primary = _derived_error(error,owned)
        except BaseException as error:
            primary = type(error).__name__+': '+str(error) if owned is None else _derived_error(error,owned)
            raise
        finally:
            # Independent endpoint checks cannot erase the original operational failure.
            try:
                require(derived_source_binding(root,reference)==source, 'derived evidence changed')
            except (OSError,ValueError) as error:
                postflight['source'] = _derived_error(error,owned,diagnostic=True)
            try:
                receipt['inputs_end'] = snapshot(root)
                same_inputs(before,Node(receipt['inputs_end']).mapping())
            except (OSError,ValueError) as error:
                postflight['runtime'] = _derived_error(error,owned,diagnostic=True)
            try:
                check_hash(godot,godot_sha)
            except (OSError,ValueError) as error:
                postflight['godot'] = _derived_error(error,owned,diagnostic=True)
            if confirmed:
                try:
                    require(_capture_artifacts(out)==artifacts, 'capture changed during validators')
                    _ = bind_chain(plan,env,code,chain)
                    require(derived_source_binding(root,reference)==source, 'derived evidence changed after chain')
                    require(_capture_artifacts(out)==artifacts, 'capture changed during final checks')
                    require(inventory(plan.evidence,[p.name for p in plan.evidence.iterdir()])==dict(chain), 'chain changed during final checks')
                    same_inputs(before,snapshot(root)); check_hash(godot,godot_sha)
                    if oracles_complete and primary is None and cleanup is None and not postflight and code==0:
                        receipt['artifacts']=artifacts;receipt['complete']=True
                except (OSError,ValueError) as error:
                    postflight['final'] = _derived_error(error,owned,diagnostic=True)
            receipt['integrity'] = {'status':'INCOMPLETE' if not confirmed else ('FAILED' if not oracles_complete or primary or cleanup or postflight else 'VERIFIED'),
                'child_completion':'CONFIRMED' if confirmed else 'UNCONFIRMED','expected_timing_failure':timing_failure,
                'primary_error':primary,'cleanup_error':cleanup,'postflight_errors':postflight}
            receipt['exit_code']=code;receipt['finished_utc']=now()
            write(receipt_path,receipt)
        require(confirmed, 'INCOMPLETE: caller retains lane/child recovery obligation; no downstream continuation')
        require(primary is None and cleanup is None and not postflight, 'derived capture integrity failed')
        if owned is None:
            require(code==0, 'stall capture exited 1 (provisional diagnostic only)')
    if code==0:
        print(f'MUX2_CAPTURE_PASS {receipt_path}')


if __name__ == '__main__':
    raise SystemExit(main())
