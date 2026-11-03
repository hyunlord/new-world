"""Persistent owner for original launcher cases; runtime qualification stays incomplete."""
from __future__ import annotations

import fcntl
import os
import select
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, BinaryIO, Literal, final

from gate_oracle_child import suite_lane
from generic_gate import document, integer, require, text
from generic_process import atomic_json, file_hash, real_path
from mux2_gate_inputs import Json, Node
from mux2_gate_lock import FD_ENV

import gate_play_child as child
import gate_play_protocol as protocol

if TYPE_CHECKING:
    from generic_play_projection import PlayProjection
    from generic_play_sources import HistoricalBindings, SourceGrants


@dataclass(frozen=True, slots=True)
class PlayPlan:
    root: Path
    output: Path
    suite: Path
    suite_sha256: str
    inputs: tuple[tuple[Path, str], ...]
    environment: tuple[tuple[str, str], ...]
    # Exact source-owned projections supplied by caller, not an environment waiver.
    child_environments: tuple[tuple[str, str, tuple[tuple[str, str], ...]], ...]
    historical: HistoricalBindings | None = None
    legacy_observer: tuple[Path,str] | None = None
    projection_root: Path | None = None
    whole_session: tuple[Path,str] | None = None


def case_argv(root: Path, index: int) -> tuple[str, ...]:
    play = str(root / 'tools/play.sh')
    return (('/bin/bash', play, 'no-such-version-v8play'),
        ('env', 'GODOT_BIN=/nonexistent', '/bin/bash', play, 'm1', '--headless-check'),
        ('/bin/bash', play, 'm1', '--speed', '7'),
        ('/bin/bash', play, 'main', '--headless-check'),
        ('/bin/bash', play, 'm1', '--headless-check'),
        ('/bin/bash', play, 'v7', '--headless-check'))[index]


def current(plan: PlayPlan) -> None:
    require(file_hash(plan.suite) == plan.suite_sha256, '/suite', 'suite changed')
    for path, digest in plan.inputs:
        require(file_hash(path) == digest, str(path), 'bound input changed')


def evidence_membership(root: Path) -> dict[str, str]:
    real_path(root, directory=True)
    members = {'.': 'directory'}
    for path in root.rglob('*'):
        require(not path.is_symlink(), str(path), 'symlink in native evidence')
        require(path.is_file() or path.is_dir(), str(path), 'unsupported native evidence entry')
        members[str(path.relative_to(root))] = 'directory' if path.is_dir() else 'file'
    return members


def validate(plan: PlayPlan) -> None:
    real_path(plan.root, directory=True); real_path(plan.suite, directory=False)
    require(plan.output.is_absolute() and plan.output == plan.output.resolve()
            and not plan.output.exists() and not plan.output.is_relative_to(plan.root)
            and not plan.root.is_relative_to(plan.output), '/output', 'fresh separate output required')
    required = {plan.root / 'tools/v8' / name for name in ('test_play.sh', 'gate_play_child.py',
        'gate_play_protocol.py', 'generic_play_owner.py')}
    if plan.historical is not None:
        required |= {plan.root / 'tools/v8' / name for name in ('generic_play_sources.py', 'generic_sources.py')}
    required |= {plan.root / 'tools/play.sh', Path('/bin/bash').resolve(), Path(sys.executable).resolve()}
    require(required <= {p for p, _ in plan.inputs}, '/inputs', 'helper/launcher/tool bindings missing')
    env = dict(plan.environment)
    require(len(env) == len(plan.environment) and not any(k.startswith('WORLDSIM_GATE_PLAY_') or k == FD_ENV for k in env),
        '/environment', 'caller protocol override')
    require('PATH' in env and 'PLAY_ROOT' in env and 'PLAY_DATA' in env, '/environment', 'isolated launcher roots required')
    for key in ('PLAY_ROOT', 'PLAY_DATA'):
        path = Path(env[key]); require(path.is_absolute() and path == path.resolve()
            and not path.exists() and not path.is_relative_to(plan.root)
            and not plan.root.is_relative_to(path), '/roots', 'fresh isolated launcher path required')
    require('PYTHONPATH' not in env and 'PYTHONHOME' not in env, '/environment', 'import overrides forbidden')
    require(shutil.which('python3', path=env['PATH']) is not None
        and Path(shutil.which('python3', path=env['PATH']) or '').resolve() == Path(sys.executable).resolve(),
        '/python', 'unbound helper interpreter')
    for path, _ in plan.inputs:
        require(not path.is_relative_to(plan.output) and not plan.output.is_relative_to(path), '/inputs', 'output overlap')
    require((plan.legacy_observer is None)==(plan.projection_root is None),'/projection','incomplete observer export binding')
    if plan.legacy_observer is not None:
        require(plan.historical is not None,'/projection','historical grants required')
        if plan.whole_session is not None:
            require(plan.whole_session in plan.inputs,'/projection','whole session lacks frozen input binding')
        assert plan.projection_root is not None
        require(plan.projection_root.is_absolute() and plan.projection_root==plan.projection_root.resolve()
                and not plan.projection_root.exists() and not plan.projection_root.is_relative_to(plan.root),
                '/projection','fresh isolated observer export root required')
        require(plan.legacy_observer in plan.inputs,'/projection','observer lacks frozen plan input')
        helpers={plan.root/'tools/v8'/name for name in ('generic_play_projection.py','generic_legacy_overlay.py',
                 'generic_legacy_play_build.py','gate_play_protocol.py')}
        if plan.whole_session is not None:
            helpers |= {plan.root/'tools/v8'/name for name in ('generic_play_main_projection.py','generic_whole_gate_play_environment.py')}
        require(helpers<={path for path,_ in plan.inputs},'/projection','new projection helpers lack source bindings')
        roots=(Path(env['PLAY_ROOT']),Path(env['PLAY_DATA']),plan.projection_root)
        require(all(path.parent==plan.output.parent for path in roots) and len(set(roots))==3,
                '/projection','launcher/export roots must be distinct siblings under the owned output parent')
    current(plan)
    for case in (child.CASES[0], *child.CASES[3:]):
        _ = child.select_case(plan.suite, plan.suite_sha256, case, 'version')


@final
class PlayOwnershipUnknown(RuntimeError):
    def __init__(self, owner: PlayOwner, reason: str) -> None:
        super().__init__('play ownership UNKNOWN; retain live owner and lane: ' + reason)
        self.owner = owner


@dataclass(frozen=True, slots=True)
class PlayOutcome:
    settlement: Literal['SETTLED']
    outer_exit: int
    case_exits: tuple[int, ...]
    qualification: Literal['INCOMPLETE'] = 'INCOMPLETE'


@final
class PlayOwner:
    """Owns a duplicated real lane FD until every source-owned wait is joined."""
    def __init__(self, plan: PlayPlan, lane_fd: int) -> None:
        validate(plan)
        self.source_grants: SourceGrants | None = None
        self.projections: dict[str,PlayProjection] = {}
        if plan.historical is not None:
            from generic_play_sources import SourceGrants as HistoricalGrants
            self.source_grants = HistoricalGrants(plan.historical, plan.root, Path(dict(plan.environment)['PLAY_ROOT']))
        suite_lane(lane_fd)
        fcntl.flock(lane_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        self.plan = plan; self.lane_fd = os.dup(lane_fd)
        self.shell: subprocess.Popen[bytes] | None = None
        self.request_read = -1; self.request_write = -1; self.control_read = -1; self.control_write = -1
        self.stdout: BinaryIO | None = None; self.stderr: BinaryIO | None = None
        self.active_case: dict[str, Json] | None = None; self.active_child: dict[str, Json] | None = None
        self.case_exits: list[int] = []; self.roles: list[str] = []; self.anchors: dict[Path, str] = {}
        self.native_memberships: dict[Path, dict[str, str]] = {}
        self.buffer = b''; self.unknown = ''; self.closed = False
        self.diagnostic_error: BaseException | None = None

    def check(self) -> None:
        suite_lane(self.lane_fd); current(self.plan)
        for projection in self.projections.values():projection.recheck()
        if self.source_grants is not None:
            self.source_grants.check()
        for root, members in self.native_memberships.items():
            require(evidence_membership(root) == members, str(root), 'native evidence membership changed')
        for path, digest in self.anchors.items():
            require(file_hash(path) == digest, str(path), 'prior evidence changed')

    def launch(self) -> None:
        finished = False
        try:
            require(self.shell is None and not self.unknown, '/owner', 'already launched/unknown')
            self.plan.output.mkdir(parents=True, exist_ok=False)
            self.request_read, self.request_write = os.pipe(); self.control_read, self.control_write = os.pipe()
            self.stdout = (self.plan.output / 'stdout.log').open('xb'); self.stderr = (self.plan.output / 'stderr.log').open('xb')
            env = {**dict(self.plan.environment), protocol.REQUEST: str(self.request_write), protocol.CONTROL: str(self.control_read),
                protocol.EVIDENCE: str(self.plan.output / 'protocol'), FD_ENV: str(self.lane_fd),
                'WORLDSIM_GATE_PLAY_RECEIPTS': '1', 'WORLDSIM_GATE_PLAY_SUITE_PLAN': str(self.plan.suite),
                'WORLDSIM_GATE_PLAY_SUITE_PLAN_SHA256': self.plan.suite_sha256}
            argv = ('/bin/bash', 'tools/v8/test_play.sh')
            atomic_json(self.plan.output / 'intent.json', {'argv': list(argv), 'cwd': str(self.plan.root),
                'environment': {k: v for k, v in env.items()}, 'owner_pid': os.getpid(),
                'inputs': {str(p): digest for p, digest in self.plan.inputs},
                **({'historical_bindings': self.source_grants.declaration()} if self.source_grants is not None else {}),
                'child_environment_projections': [{'case': case, 'role': role, 'environment': {k: v for k, v in values}}
                    for case, role, values in self.plan.child_environments]})
            self.shell = subprocess.Popen(argv, cwd=self.plan.root, env=env, stdout=self.stdout, stderr=self.stderr,
                pass_fds=(self.lane_fd, self.request_write, self.control_read))
            atomic_json(self.plan.output / 'started.json', {'pid': self.shell.pid, 'owner_pid': os.getpid()})
            for name in ('intent.json', 'started.json'):
                self.anchors[self.plan.output / name] = file_hash(self.plan.output / name)
            os.close(self.request_write); self.request_write = -1; os.close(self.control_read); self.control_read = -1
            finished = True
        finally:
            failure = sys.exception()
            if not finished and not self.unknown:
                self.retain_failure(failure if failure is not None else RuntimeError('launch did not finish'))

    def fail(self, reason: str) -> None:
        self.retain_failure(RuntimeError(reason))

    def retain_failure(self, primary: BaseException) -> None:
        """Every arbitrary lifecycle fault keeps the object/FD and original cause."""
        self.unknown = self.unknown or type(primary).__name__
        diagnostic_finished = False
        try:
            if self.plan.output.exists() and not (self.plan.output / 'unknown.json').exists():
                atomic_json(self.plan.output / 'unknown.json', {'settlement': 'UNKNOWN', 'reason': self.unknown,
                    'owner_pid': os.getpid(), 'outer_pid': self.shell.pid if self.shell is not None else None,
                    'lane_retained': True, 'completed_cases': list(self.case_exits)})
            diagnostic_finished = True
        finally:
            secondary = sys.exception()
            if not diagnostic_finished and secondary is not None:
                self.diagnostic_error = secondary
            raise PlayOwnershipUnknown(self, self.unknown) from primary

    def retain(self, path: Path) -> Node:
        digest = file_hash(path); row = document(path.read_text())
        self.anchors[path] = digest
        return row

    def message(self, notification: Node) -> None:
        self.check(); require(set(notification.mapping()) == {'path', 'sha256'}, '/frame', 'unexpected frame')
        path = Path(text(notification.field('path')))
        require(path.is_relative_to(self.plan.output / 'protocol') and file_hash(path) == text(notification.field('sha256')),
            '/frame', 'unbound record')
        require(path not in self.anchors, '/frame', 'duplicate event')
        real_path(path, directory=False)
        row = self.retain(path)
        require(set(row.mapping()) == {'event', 'kind', 'case', 'role', 'wrapper_pid', 'parent_pid', 'payload'}, '/record', 'unexpected fields')
        event = text(row.field('event')); kind = text(row.field('kind'))
        index = len(self.case_exits); require(index < 6, '/case', 'extra case')
        row.field('case').equals(child.CASES[index]); payload = row.field('payload')
        expected_path = self.plan.output / 'protocol' / ('case-' + str(index))
        if kind == 'child': expected_path /= text(row.field('role'))
        require(path == expected_path / (event + '.json'), '/path', 'event outside exact case/role path')
        pid = integer(row.field('wrapper_pid')); require(pid > 0, '/pid', 'invalid wrapper')
        if kind == 'case':
            row.field('role').equals('case')
            if event == 'request':
                require(self.active_case is None and self.active_child is None, '/case', 'overlapping case')
                payload.field('argv').equals(list(case_argv(self.plan.root, index)))
                payload.field('cwd').equals(str(self.plan.root))
                assert self.shell is not None
                row.field('parent_pid').equals(self.shell.pid)
                self.active_case = row.mapping(); self.roles = []
            elif event == 'returned':
                require(self.active_case is not None and self.active_child is None, '/case', 'case lacks nested completion')
                assert self.active_case is not None
                row.field('wrapper_pid').equals(self.active_case['wrapper_pid'])
                row.field('parent_pid').equals(self.active_case['parent_pid'])
                code = integer(payload.field('native_exit')); require(code >= 0, '/case', 'case shell signaled')
                started = self.retain(path.parent / 'case-started.json')
                started.field('wrapper_pid').equals(pid)
                started.field('pid').equals(payload.field('pid').value); require(integer(started.field('pid')) > 0, '/pid', 'invalid case shell')
                expected = ['version'] if index == 0 else ([] if index in (1, 2) else ['version', 'build', 'import-1', 'import-2', 'headless'])
                require(self.roles == expected, '/coverage', 'missing/extra original children')
                self.case_exits.append(code); self.active_case = None
            else:
                require(False, '/event', 'unexpected case event')
        elif kind == 'child':
            require(self.active_case is not None, '/child', 'outside case')
            role = text(row.field('role'))
            if event == 'request':
                require(self.active_child is None and role not in self.roles, '/child', 'overlapping/repeated child')
                sequence = ('version',) if index == 0 else ('version', 'build', 'import-1', 'import-2', 'headless')
                require(index not in (1, 2) and len(self.roles) < len(sequence) and role == sequence[len(self.roles)],
                    '/order', 'child outside original order')
                selected = child.select_case(self.plan.suite, self.plan.suite_sha256, child.CASES[index], role)
                payload.field('plan').equals(str(selected.plan)); payload.field('plan_sha256').equals(selected.plan_sha256)
                entry = document(selected.plan.read_text()).field('roles').field(role)
                payload.field('argv').equals(entry.field('argv').value); payload.field('cwd').equals(entry.field('cwd').value)
                expected = [values for case, name, values in self.plan.child_environments if (case, name) == (child.CASES[index], role)]
                require(len(expected) == 1, '/environment', 'missing/duplicate bound child projection')
                observed = dict(payload.field('environment').mapping())
                for key in (protocol.REQUEST, protocol.CONTROL, protocol.EVIDENCE, FD_ENV):
                    _ = observed.pop(key, None)
                if self.plan.whole_session is not None:
                    from generic_whole_gate_play_environment import validate_environment
                    validate_environment(payload.field('environment'), dict(expected[0]),
                        Path(text(payload.field('cwd'))), self.plan.root, Path(dict(self.plan.environment)['PLAY_ROOT']))
                else:
                    require(observed == dict(expected[0]), '/environment', 'child environment differs')
                if self.source_grants is not None:
                    self.source_grants.check_environment(payload)
                if self.source_grants is not None and index >= 3 and role != 'version':
                    source_record = self.source_grants.request(('main', 'm1', 'v7')[index - 3], role, entry, payload)
                    source_path = path.with_name('source-grant.json')
                    atomic_json(source_path, source_record)
                    self.anchors[source_path] = file_hash(source_path)
                projection=self.projections.get(('main','m1','v7')[index-3]) if index>=3 else None
                if projection is not None and role in {'import-1','import-2','headless'}:
                    payload.field('projection').equals(projection.declaration())
                    _=projection.command(role,tuple(text(v) for v in payload.field('argv').array()),Path(text(payload.field('cwd'))))
                else:
                    require('projection' not in payload.mapping(),'/projection','ungranted runtime export')
                self.active_child = row.mapping()
            elif event == 'returned':
                require(self.active_child is not None, '/child', 'no admitted child')
                assert self.active_child is not None
                for key in ('wrapper_pid', 'case', 'role'):
                    row.field(key).equals(self.active_child[key])
                selected = child.select_case(self.plan.suite, self.plan.suite_sha256, child.CASES[index], role)
                entry = document(selected.plan.read_text()).field('roles').field(role)
                receipt = Path(text(payload.field('receipt')))
                require(receipt == Path(text(entry.field('output'))) / 'receipt.json', '/receipt', 'foreign child output')
                require(file_hash(receipt) == text(payload.field('sha256')), '/receipt', 'terminal changed')
                require({p.name for p in receipt.parent.iterdir()} == {'receipt.json', 'started.json', 'stdout.log', 'stderr.log'},
                    '/output', 'unexpected child evidence')
                terminal = self.retain(receipt)
                terminal.field('plan_sha256').equals(selected.plan_sha256)
                terminal.field('suite_sha256').equals(self.plan.suite_sha256)
                request_payload = Node(self.active_child['payload'])
                if 'projection' in request_payload.mapping():
                    projection=self.projections[('main','m1','v7')[index-3]]
                    terminal.field('requested_argv').equals(request_payload.field('argv').value)
                    terminal.field('requested_cwd').equals(request_payload.field('cwd').value)
                    terminal.field('projection').equals(projection.declaration())
                    terminal.field('argv').equals(request_payload.field('argv').value)
                    terminal.field('cwd').equals(str(projection.export_root))
                    terminal.field('environment').equals(request_payload.field('environment').value)
                else:
                    for key in ('argv', 'cwd', 'environment'):
                        terminal.field(key).equals(request_payload.field(key).value)
                terminal.field('executable').equals(entry.field('executable').value)
                terminal.field('executable_sha256').equals(entry.field('executable_sha256').value)
                terminal.field('parent_pid').equals(self.active_child['parent_pid'])
                terminal.field('parent_pid_after_wait').equals(self.active_child['parent_pid'])
                terminal.field('wrapper_pid').equals(pid); terminal.field('case').equals(child.CASES[index])
                terminal.field('native_exit').equals(payload.field('native_exit').value)
                require(integer(terminal.field('native_exit')) >= 0 and terminal.field('timed_out').value is False,
                    '/wait', 'signal/timeout descendant completion unresolved')
                terminal.field('errors').equals([])
                expected_inputs = dict(entry.field('inputs').mapping())
                expected_inputs[str(self.plan.suite)] = self.plan.suite_sha256
                terminal.field('expected_inputs').equals(expected_inputs)
                terminal.field('after_inputs').equals(expected_inputs)
                declared = [Path(text(value)) for value in entry.field('artifacts').array()]
                terminal.field('declared_artifacts').equals([str(value) for value in declared])
                artifact_hashes = terminal.field('artifact_hashes').mapping()
                require(set(artifact_hashes) == {str(value) for value in declared}, '/artifacts', 'missing generated artifact')
                for artifact in declared:
                    digest = text(Node(artifact_hashes[str(artifact)]))
                    require(file_hash(artifact) == digest, '/artifact', 'generated artifact changed')
                    self.anchors[artifact] = digest
                start = self.retain(receipt.parent / 'started.json')
                for key in ('argv', 'cwd', 'environment', 'plan_sha256', 'executable_sha256', 'parent_pid'):
                    start.field(key).equals(terminal.field(key).value)
                require(integer(terminal.field('pid')) > 0, '/pid', 'invalid native process')
                start.field('pid').equals(terminal.field('pid').value); start.field('wrapper_pid').equals(pid)
                for name in ('stdout', 'stderr'):
                    log = receipt.parent / (name + '.log')
                    require(file_hash(log) == text(terminal.field(name + '_sha256')), '/logs', 'child output changed')
                    self.anchors[log] = file_hash(log)
                if self.source_grants is not None and role == 'build':
                    build_record = self.source_grants.returned(('main', 'm1', 'v7')[index - 3], receipt)
                    build_path = path.with_name('build-join.json')
                    atomic_json(build_path, build_record)
                    self.anchors[build_path] = file_hash(build_path)
                    version=('main','m1','v7')[index-3]
                    if self.plan.legacy_observer is not None and (version in {'m1','v7'} or self.plan.whole_session is not None):
                        assert self.plan.projection_root is not None
                        self.plan.projection_root.mkdir(parents=True,exist_ok=True)
                        if version=='main':
                            assert self.plan.whole_session is not None
                            from generic_play_main_projection import (
                                prepare_main_projection,
                            )
                            session_path,session_sha=self.plan.whole_session
                            projection=prepare_main_projection(self.source_grants,build_path,self.anchors[build_path],
                                session_path,session_sha,self.plan.projection_root/version)
                        else:
                            observer,observer_hash=self.plan.legacy_observer
                            from generic_play_projection import (
                                prepare_legacy_projection,
                            )
                            projection=prepare_legacy_projection(self.source_grants,version,build_path,self.anchors[build_path],
                                self.plan.projection_root/version,observer,observer_hash)
                        self.projections[version]=projection
                        manifest=self.plan.projection_root/(version+'.json')
                        atomic_json(manifest,projection.declaration());self.anchors[manifest]=file_hash(manifest)
                members = evidence_membership(receipt.parent)
                require(members == {'.': 'directory', 'receipt.json': 'file', 'started.json': 'file',
                    'stdout.log': 'file', 'stderr.log': 'file'}, '/output', 'unexpected child evidence membership')
                self.native_memberships[receipt.parent] = members
                self.roles.append(role); self.active_child = None
            else:
                require(False, '/event', 'unexpected child event')
        else:
            require(False, '/kind', 'unexpected role kind')
        self.check()
        grant = path.with_name(event + '-grant.json')
        require(not grant.exists(), '/grant', 'duplicate grant')
        atomic_json(grant, {'request_sha256': self.anchors[path], 'owner_pid': os.getpid(),
            'wrapper_pid': pid, 'event': event, 'kind': kind, 'case': child.CASES[index]})
        self.anchors[grant] = file_hash(grant)
        self.check(); require(os.write(self.control_write, b'G') == 1, '/grant', 'short grant')

    def wait(self) -> PlayOutcome:
        finished = False
        try:
            require(self.shell is not None and not self.unknown, '/owner', 'not live')
            assert self.shell is not None
            while True:
                ready, _, _ = select.select((self.request_read,), (), (), 0.05)
                if ready:
                    chunk = os.read(self.request_read, 4096)
                    self.buffer += chunk
                    while b'\n' in self.buffer:
                        raw, self.buffer = self.buffer.split(b'\n', 1)
                        require(len(raw) < 4096, '/frame', 'oversized frame'); self.message(document(raw.decode()))
                    require(len(self.buffer) < 4096, '/frame', 'oversized partial frame')
                    if not chunk:
                        break
                if self.shell.poll() is not None and not ready:
                    self.fail('outer ended without protocol EOF')
            code = self.shell.wait(); self.check()
            require(code >= 0 and len(self.case_exits) == 6 and self.active_case is None
                and self.active_child is None and not self.buffer, '/settlement', 'missing nested waits')
            expected_files = {self.plan.output / name for name in ('stdout.log', 'stderr.log')} | {p for p in self.anchors if p.is_relative_to(self.plan.output)}
            require({p for p in self.plan.output.rglob('*') if p.is_file()} == expected_files, '/outputs', 'unaccounted owner evidence')
            outer_logs = {name: file_hash(self.plan.output / name) for name in ('stdout.log', 'stderr.log')}
            atomic_json(self.plan.output / 'terminal.json', {'settlement': 'SETTLED', 'outer_pid': self.shell.pid,
                'outer_exit': code, 'case_exits': list(self.case_exits), 'qualification': 'INCOMPLETE',
                'outer_log_hashes': {k: v for k, v in outer_logs.items()},
                'retained_files': {str(p): digest for p, digest in self.anchors.items()},
                'native_output_membership': {str(root): dict(members) for root, members in self.native_memberships.items()},
                'wrapper_wait_authority': 'SOURCE_FOREGROUND_CASE_SHELL_WAIT', 'native_waits': 'OBSERVED'})
            self.check()
            require(outer_logs == {name: file_hash(self.plan.output / name) for name in outer_logs}, '/logs', 'outer logs changed')
            result = PlayOutcome('SETTLED', code, tuple(self.case_exits))
            if self.stdout is not None: self.stdout.close()
            if self.stderr is not None: self.stderr.close()
            for fd in (self.request_read, self.control_write):
                os.close(fd)
            os.close(self.lane_fd)
            self.closed = True; finished = True
            return result
        finally:
            failure = sys.exception()
            if not finished and not self.unknown:
                self.retain_failure(failure if failure is not None else RuntimeError('wait did not finish'))


def run_owned(plan: PlayPlan, lane_fd: int) -> PlayOutcome:
    owner = PlayOwner(plan, lane_fd); owner.launch(); return owner.wait()


def case_main(argv: tuple[str, ...]) -> int:
    _ = protocol.exchange('request', 'case', 'case', {'argv': list(argv), 'cwd': str(Path.cwd())})
    index = protocol.case_index(os.environ['WORLDSIM_GATE_PLAY_CASE'])
    output = Path(os.environ[protocol.EVIDENCE]) / ('case-' + str(index))
    passed = tuple(int(os.environ[key]) for key in (protocol.REQUEST, protocol.CONTROL, FD_ENV))
    process = subprocess.Popen(argv, pass_fds=passed)
    atomic_json(output / 'case-started.json', {'pid': process.pid, 'wrapper_pid': os.getpid()})
    code = process.wait()
    _ = protocol.exchange('returned', 'case', 'case', {'pid': process.pid, 'native_exit': code})
    return 128-code if code < 0 else code


if __name__ == '__main__':
    require(len(sys.argv) > 2 and sys.argv[1] == '--case', '/argv', 'expected --case ORIGINAL_COMMAND')
    raise SystemExit(case_main(tuple(sys.argv[2:])))
