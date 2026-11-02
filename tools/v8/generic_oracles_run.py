"""Original step38 shell and all sixteen oracles with retained live ownership.

Source/resource binding is not build provenance, GPU acceptance or fullgate authority.
Only the caller that retains OracleSuiteOwnershipUnknown may own the enclosing lane.
"""
import ast
import fcntl
import io
import json
import os
import re
import select
import shutil
import subprocess
import sys
from contextlib import redirect_stdout
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType
from typing import BinaryIO, Final, Literal

import gate_oracle_child as child
import mux2_gate_oracle_log as checker
from gate_stall_child import range_hash
from generic_fixtures import ProbeProduct
from generic_gate import document, integer, require
from generic_m6a_structural import Tooling, import_closure, resolution_paths
from generic_process import atomic_json, file_hash, real_path
from generic_sources import inventory
from mux2_gate_inputs import EvidenceError, Json, Node
from mux2_gate_lock import FD_ENV


def eager_sources() -> dict[str, tuple[ModuleType, Path, str]]:
    """Capture the eager graph once; lazy unused modules remain fully file-sealed.

    Selected suite call paths do not invoke excluded legacy lazy helpers. Future
    use requires explicit pre-use provenance, never refreshing this snapshot.
    """
    entry = Path(__file__).resolve()
    roots = (entry.parent, Path(child.__file__).resolve().parent)
    pending = [('generic_oracles_run', entry)]
    found: dict[str, tuple[ModuleType, Path, str]] = {}
    while pending:
        name, source = pending.pop()
        if name in found:
            continue
        key = __name__ if name == 'generic_oracles_run' else name
        module = sys.modules.get(key)
        require(module is not None and module.__file__ is not None
                and Path(module.__file__).resolve() == source,
                '/imports', 'missing or displaced eager module')
        if module is None:
            raise AssertionError('validated eager module')
        found[name] = (module, source, file_hash(source))
        nodes: list[ast.AST] = list(ast.parse(source.read_text()).body)
        while nodes:
            node = nodes.pop()
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
                continue
            imports: list[str] = []
            if isinstance(node, ast.Import):
                imports = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom) and node.module is not None:
                require(node.level == 0, '/imports', 'relative tooling import requires review')
                imports = [node.module]
            for imported in imports:
                relative = imported.replace('.', '/')+'.py'
                for root in roots:
                    candidate = root/relative
                    if candidate.is_file():
                        pending.append((imported, candidate.resolve()))
                        break
            nodes.extend(ast.iter_child_nodes(node))
    return dict(sorted(found.items()))


EAGER_SOURCES: Final = eager_sources()
LOADED: Final = {name: (path, digest) for name, (_, path, digest) in EAGER_SOURCES.items()}
NO_WORLD_PINS: Final = {'project.godot': 'cd16a69aa33fbde80fb64c2bf8fb584f6d7aaef0f4b135ed92c8a35cb6061c98', 'scripts/core/locale.gd': 'b92c662d50966ce294be3596d91ef7530a3defd59010beee2c898e2e85ae50e3', 'scripts/core/settlement_selection.gd': '2789c540c8fbc119ec35f36599739534e5db093290d6d909d1c001e3bcd356db', 'scripts/ui/content_context.gd': 'e8cdac02c9d5241464a2d153e51c1c1db3a30696198d255d791213f0ac31ce10', 'scripts/ui/economy_text.gd': 'a6360920baa63e9d83d4d30c7e211dc7ff9e97fcc1be5e1a0172074586947c5f', 'scripts/ui/mux/settings_store.gd': '70ced6c709f8d63f60a3d3e04127db6432eaf86972a69a3a4a9f618e642907ec', 'scripts/ui/person_names.gd': 'cd8afd961c3eb31fc6b7706ea1bdb4ea0c9334e229cff590974aa6e2de422791', 'scripts/ui/story_text.gd': '99dccbc516edea653b0ffb52eb691995d1aadd6c21eac0494cebb719797e3ffd', 'scripts/world/art/art_data.gd': 'fd3d27da528819c6ace5ad04824c1c7f8b8a5232b9f2535f40c3d7b1160fa919', 'scripts/world/art/material_resolver.gd': '2bcd0f504dc03a0d96ac611894008f720d8ecddcfedaaeccdc782794993f876e', 'scripts/world/art/site_ghost.gd': 'd091fa865f86beef20367b240bb1e330b8dae9484b54c2b0ef75b47ef2d1f82e', 'scripts/world/art/tone.gd': '6475fa1835d88942d22b6effd2b96ccf7d5456bca8b35103cf1c20fb6d80635c', 'scripts/world/art/wall_baker.gd': '2e18a31d168ea00a9bc2eb7a1c3e4f151a81c1f86b1fb2f3e60192d02dd17150', 'scripts/world/art_hash.gd': 'e37c1a4553175450dbf598bfc3b7a0d86c51016bc2c781a6e4ef82a93e288afe', 'scripts/world/colony/agent_layer.gd': 'd1489bb5ac61c85b668809eb26b3f3faa1f71b4ec3fdbc0a0d2c517683136aea', 'scripts/world/colony/lod_markers.gd': '803c4fece49f5565d8e1f35b0084ed13fb6a7aabd669353941eca32190a72c6a', 'scripts/world/colony/marker_layout.gd': '85a3b63cd7267972297f4106cbf9d37ed16ba1472b6ba4bf639d8d6391056526', 'scripts/world/colony/overlay_layer.gd': '7ebc9a67fff0aace359820932fecebba4b38031d9256f4a7f0dade1053c05e93', 'scripts/world/colony/room_layer.gd': '229009ec88c5041b946ed845832e449e12bc5c6409ba57c03411aa8b718576c5', 'scripts/world/colony/view.gd': 'f67192ce8caefbff8971a449f1b5e001c3b787cfb72807b8546c51f42f8e95ab', 'tools/v8/fixtures/mux2_badge_grid_reference.gd': '5d0828663ac6c85d5831eb8bb779fcbc8e8e39af0e88b6f324ce8563f933e39a', 'tools/v8/fixtures/mux2_badge_local_reference.gd': '88ef729b1b278daf02dac90563b17b506d218898b02c30b601935572dfcd5ca9', 'tools/v8/fixtures/mux2_extent_owner_reference.gd': 'ceade2af68339098372f8a47596a66e01e407854d9d5dd383ccda198b2c22df6', 'tools/v8/fixtures/mux2_person_cache_reference.gd': '9c398f81a26a1f469798864513a8f26b21ffaa1dd662a04613e4e8bfa0a94bd5', 'tools/v8/gate_ui_observer.gd': 'b0f2ef8c577d777aff9442a55bf3f8ad29c5a5898011d0b331a24db4c9dba967', 'tools/v8/mart2_fps.gd': '391559cb812518d3e90a1d831ee874e09bb6b5a381db66501c7abe5766f540e2', 'tools/v8/mux2_badge_aggregate_test.gd': 'e835cdf7d8533393491d97e447bf7e7350dc414b5030b0860e104d00ae983557', 'tools/v8/mux2_badge_extent_test.gd': '65884d303fc7aebfbcb1653582ac2bd54f7bac64bc45b9419d1957962007965c', 'tools/v8/mux2_badge_grid_test.gd': 'c0235c182b5b658e15195f1910cf55005eea7bb2522c0df873f77825a0749fc7', 'tools/v8/mux2_badge_index_test.gd': '55e3739cd4c5fbdfc1206ffc0eb28824c4865eaa9dcf630576472759f6747e23', 'tools/v8/mux2_badge_local_test.gd': '14db211a09ed72fcd58f5e1dc68144d44ae732b9ef15bd490ff7151893a274de', 'tools/v8/mux2_badge_packing_test.gd': 'ce2593544fef33cb3a16a9ca08c9c8ffc00620fa34cedf3fdf016d17445a7789', 'tools/v8/mux2_bake_receipt_test.gd': '69bf9cb14d73babcd4e2b4fd9bad705fc5bf944725b9aac6c63732b4434ce40c', 'tools/v8/mux2_camp_framing_test.gd': 'ea85a891c0c82ee1ff7b1c2453da89d9fccb6c71a2b8bf4894fbc8c42506092d', 'tools/v8/mux2_coincident_test.gd': '117f5701637a09f835e47d81b0df03a27b33c162c045ee9539b03ee021839ada', 'tools/v8/mux2_extent_owner_test.gd': '188ef706f38589fc2f1665750dc21cf1fb3058c4aae7c4a8233ae5391fa411f4', 'tools/v8/mux2_marker_layout_test.gd': '08fe98ddcf555f0a1e9e53b42dd12a7a96173d44d1d46183fed4ecd28d89da09', 'tools/v8/mux2_overlap_test.gd': '135ad5f12356af60da64946f50e89012166151336f8cae3e0b3d9a389f2c3f2d', 'tools/v8/mux2_perf_monitor.gd': '5b61ab5907d03c6902515b686b616885fa4f38f372256fc0fedc0a9775e3974b', 'tools/v8/mux2_person_cache_test.gd': '21546f987f288668d95f26abc2c19c9f9396461735fda6d43c728c498061e2b4', 'tools/v8/mux2_stall_test.gd': 'cd171933e176b10cdd5530707259643891fe217cd8f0075bff080942775ea5a4', 'tools/v8/mux2_visibility_metrics.gd': 'a1c332af8d78cf1bf198544bf430a90b8cd9f306700d5234ea5db8485d453fa2', 'tools/v8/mux2_visibility_provenance.gd': '2198ea956808547d9c4c86b7ba98c1da6815c092363a78744bf13fef6293806b', 'tools/v8/mux2_visibility_validator_test.gd': 'dade5ae5f059e66544c17daa022ee08697ddf74ed4d2aace3e30397e06f0c0f2', 'tools/v8/mux2_wall_cache_test.gd': '1143ebce1a777bb3981e61d5c5e199f6023c7d0d8a9d9e78495422f00b2d3e91'}
PROTOCOL_PINS: Final = {'mux2_gate_checks.sh': '527313c36d44e0f462a1986d181718c370ca819e282fbbf7e6f029982222471c', 'godot_bin.sh': '04b137eff9db572813d48f0f1c50f383f61581a1b701b1f8a1f6082b64ecc500', 'run_bounded.sh': '64b0d037c1a854add96cccf25ae7a0bcfc40be78ab4406f092514e4891009bf7', 'gate_oracle_child.py': 'af1d16e2774710ad2800c3ebd6479454657a460067651cec1f0f5a11f43ad785', 'mux2_gate_oracle_log.py': 'fa9b6a13f85e419a8cf700421c5d8227240cef89e04677eb125755753a10bf1d'}


@dataclass(frozen=True, slots=True)
class OracleInputs:
    product: ProbeProduct
    tools: Tooling


@dataclass(frozen=True, slots=True)
class OracleResult:
    status: Literal['PASS', 'FAIL']
    receipt: Path
    sha256: str


@dataclass(slots=True)
class LiveInvocation:
    """Mutable retained handles; never reconstructed from recorded process IDs."""
    plan: child.SuitePlan
    lane_fd: int
    channels: child.SuiteChannels
    stdout: BinaryIO
    stderr: BinaryIO
    shell: subprocess.Popen[bytes] | None = None
    controller: child.SuiteController | None = None
    launch_intent: bool = False
    request_write_closed: bool = False
    control_read_closed: bool = False
    control_write_closed: bool = False


class OracleSuiteOwnershipUnknown(RuntimeError):
    def __init__(self, live: LiveInvocation, failures: tuple[BaseException, ...]) -> None:
        super().__init__('oracle suite ownership unresolved; retain caller lane and live handles')
        self.live: LiveInvocation = live
        self.failures: tuple[BaseException, ...] = failures


def inputs_current(inputs: OracleInputs) -> tuple[tuple[Path, str], ...]:
    """Reconstruct effective scope and bind actual imported code to materialized tools."""
    product, tools = inputs.product, inputs.tools
    real_path(product.input_root, directory=True)
    real_path(tools.root, directory=True)
    require(re.fullmatch('[0-9a-f]{40}', product.source_sha) is not None, '/source', 'full source identity required')
    resources = set(NO_WORLD_PINS) | {'sim_bridge.gdextension', '.godot/extension_list.cfg',
        '.godot/global_script_class_cache.cfg', '.godot/uid_cache.bin',
        'rust/target/debug/libsim_bridge.dylib', 'rust/target/release/libsim_bridge.dylib', 'tools/v8/gate.sh'}
    for directory in ('scripts', 'assets', 'localization', 'shaders', '.godot/imported'):
        real_path(product.input_root/directory, directory=True)
        resources.update(p.relative_to(product.input_root).as_posix()
                         for p in (product.input_root/directory).rglob('*') if p.is_file())
    resources.update(resolution_paths(product.input_root))
    bound = dict(product.input_files)
    require(len(bound) == len(product.input_files) and resources <= bound.keys(), '/inputs', 'incomplete resources')
    require(all(bound.get(name) == digest for name, digest in NO_WORLD_PINS.items()), '/worlds', 'unreviewed no-world source')
    require(inventory(product.input_root, list(bound)) == bound, '/inputs', 'product input changed')
    files = dict(tools.files)
    closure = import_closure(tools.root, 'generic_oracles_run.py')
    require(len(files) == len(tools.files) and set(closure) <= files.keys(), '/tools', 'incomplete tooling closure')
    require(all(files.get(name) == digest for name, digest in PROTOCOL_PINS.items()), '/tools', 'unreviewed suite protocol')
    require(inventory(tools.root, list(files)) == files, '/tools', 'tool source changed')
    for name, (admitted, path, digest) in EAGER_SOURCES.items():
        key = __name__ if name == 'generic_oracles_run' else name
        module = sys.modules.get(key)
        require(module is not None and module is admitted and module.__file__ is not None and Path(module.__file__).resolve() == path
                and files.get(name.replace('.', '/')+'.py') == digest and file_hash(path) == digest,
                '/imports', 'loaded coordinator differs from sealed source')
    for name, digest in files.items():
        materialized = product.input_root/'tools/v8'/name
        require(bound.get('tools/v8/'+name) == digest and file_hash(materialized) == digest,
                '/tools', 'materialized helper differs')
    sources = {product.input_root/name: digest for name, digest in bound.items()}
    sources.update({tools.root/name: digest for name, digest in files.items()})
    for executable, digest in ((product.executable, product.executable_sha256),
                               (tools.python, tools.python_sha256), (tools.bash, tools.bash_sha256)):
        real_path(executable, directory=False)
        require(file_hash(executable) == digest, '/tools', 'executable changed')
        sources[executable] = digest
    require(tools.python == Path(sys.executable).resolve() and tools.bash == Path('/bin/bash').resolve(),
            '/tools', 'suite interpreters differ')
    for name in ('python3', 'bash', 'dirname', 'mkdir', 'grep'):
        resolved = shutil.which(name, path=tools.path)
        require(resolved is not None, '/PATH', 'missing suite tool')
        if resolved is None:
            raise AssertionError('validated tool')
        path = Path(resolved).resolve()
        expected = tools.python if name == 'python3' else tools.bash if name == 'bash' else (Path('/usr/bin')/name).resolve() if name in ('dirname', 'grep') else (Path('/bin')/name).resolve()
        require(path == expected, '/PATH', 'unbound suite tool')
        sources[path] = file_hash(path)
    return tuple(sorted(sources.items()))


def audit_log(name: str, path: Path) -> str:
    """Use the original checker, and the original final grep predicates verbatim."""
    if name == 'mux2_stall_test':
        raw = path.read_bytes()
        require(b'MUX2_STALL_SELF_TEST failures=[]' in raw.split(b'\n') and b'SCRIPT ERROR' not in raw,
                '/stall', 'original stall self-test grep failed')
        return ''
    with redirect_stdout(io.StringIO()) as output:
        checker.validate_log(name, path)
    return output.getvalue()


def launch(plan: child.SuitePlan, lane_fd: int, output: Path) -> tuple[LiveInvocation, child.SuiteResult]:
    """Cross Popen intent only with a retained record; all ambiguous exits retain it."""
    child.suite_lane(lane_fd)
    fcntl.flock(lane_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    stdout = (output/'shell.stdout').open('xb')
    try:
        stderr = (output/'shell.stderr').open('xb')
    except BaseException:
        stdout.close()
        raise
    try:
        channels = child.prepare_suite(plan, lane_fd)
    except BaseException:
        stdout.close()
        stderr.close()
        raise
    live = LiveInvocation(plan, lane_fd, channels, stdout, stderr)
    environment = dict(plan.environment)
    environment.update({child.SUITE_PLAN: str(plan.path), child.SUITE_HASH: plan.digest,
        child.SUITE_REQUEST: str(channels.request_write), child.SUITE_CONTROL: str(channels.control_read), FD_ENV: str(lane_fd)})
    argv = ('bash', 'tools/v8/mux2_gate_checks.sh', 'oracles', str(plan.logs))
    try:
        live.launch_intent = True
        live.shell = subprocess.Popen(argv, cwd=plan.root, env=environment, stdin=subprocess.DEVNULL,
            stdout=stdout, stderr=stderr, pass_fds=(lane_fd, channels.request_write, channels.control_read))
        os.close(channels.request_write)
        live.request_write_closed = True
        os.close(channels.control_read)
        live.control_read_closed = True
        live.controller = child.SuiteController(plan, live.shell, lane_fd, channels)
        live.controller.start()
        while True:
            live.controller.service_ready()
            if live.shell.poll() is not None:
                # A finished frame may be queued just before the shell's observed exit.
                # Drain complete frames until service_ready no longer changes state.
                while True:
                    before = (len(live.controller.completed), live.controller.buffer, live.controller.active)
                    live.controller.service_ready()
                    if before == (len(live.controller.completed), live.controller.buffer, live.controller.active):
                        break
                _ = live.shell.wait()
                result = live.controller.close_settled()
                stdout.close()
                stderr.close()
                atomic_json(output/'caller.json', {'argv': list(argv), 'cwd': str(plan.root),
                    'environment': {k: v for k, v in environment.items()},
                    'pass_fds': [lane_fd, channels.request_write, channels.control_read],
                    'shell_pid': live.shell.pid, 'shell_exit': result.shell_exit,
                    'shell_wait_observed': True, 'close_settled_returned': True})
                return live, result
            _ = select.select((channels.request_read,), (), (), 0.05)
    except BaseException as failure:  # noqa: BLE001 -- Essential live ownership boundary; no exception rendering.
        cleanup: tuple[BaseException, ...] = ()
        try:
            if live.controller is not None:
                live.controller.cancel('caller operation failed')
            elif not live.control_write_closed:
                os.close(channels.control_write)
                live.control_write_closed = True
        except BaseException as cancel_failure:  # noqa: BLE001 -- Failed diagnostics/close cannot discard actual handles.
            cleanup = (cancel_failure,)
        raise OracleSuiteOwnershipUnknown(live, (failure, *cleanup)) from None


def output_snapshot(output: Path) -> dict[str, str]:
    return inventory(output, [p.name for p in output.iterdir()])


def suite_inventory(plan: child.SuitePlan, completed: tuple[str, ...]) -> None:
    """The original complete prefix owns exactly its native and log artifacts."""
    expected = {'suite-prepared.json', 'suite-start.json', 'suite-terminal.json'}
    expected.update(row.evidence.name for row in plan.rows[:len(completed)])
    require({p.name for p in plan.evidence.iterdir()} == expected, '/evidence', 'unjoined producer evidence')
    require({p.name for p in plan.logs.iterdir()} == {row.log.name for row in plan.rows[:len(completed)]},
            '/logs', 'unjoined producer logs')
    for row in plan.rows[:len(completed)]:
        require({p.name for p in row.evidence.iterdir()} == {'request.json', 'wrapper-start.json', 'native-start.json', 'native-terminal.json'},
                '/row', 'unexpected producer artifact')


def suite_audit(plan: child.SuitePlan, result: child.SuiteResult, output: Path) -> tuple[Literal['PASS', 'FAIL'], list[Json]]:
    """Shared original log semantics; this neither launches children nor settles ownership."""
    audits: list[Json] = []
    markers = ''
    passed = result.shell_exit == 0 and len(result.completed) == 16
    for row in plan.rows:
        if row.identity not in result.completed:
            audits.append({'identity': row.identity, 'status': 'NOT_RUN', 'reason': 'original shell stopped before admitted native'})
            continue
        try:
            marker = audit_log(row.identity, row.log)
            markers += marker
            native = document((row.evidence/'native-terminal.json').read_text())
            native_ok = integer(native.field('native_exit')) == 0 and native.field('timed_out').value is False
            passed = passed and native_ok
            audits.append({'identity': row.identity, 'status': 'PASS' if native_ok else 'FAIL', 'oracle_output': marker})
        except (ValueError, OSError, EvidenceError):
            passed = False
            audits.append({'identity': row.identity, 'status': 'FAIL', 'reason': 'original log oracle rejected'})
    actual_markers = ''.join(line+'\n' for line in (output/'shell.stdout').read_text().splitlines() if line.startswith('MUX2_ORACLE_PASS '))
    if result.shell_exit == 0:
        passed = passed and actual_markers == markers
    return ('PASS' if passed else 'FAIL'), audits


def finish(inputs: OracleInputs, live: LiveInvocation, result: child.SuiteResult, output: Path) -> OracleResult:
    """Only a returned helper settlement reaches frozen, in-process oracle audit."""
    plan = live.plan
    require(live.shell is not None and live.controller is not None and live.controller.closed,
            '/owner', 'actual closed live owner required')
    if live.shell is None or live.controller is None:
        raise AssertionError('validated live owner')
    require(live.controller.shell is live.shell and live.controller.plan is plan
            and live.controller.snapshot() == result and live.shell.returncode == result.shell_exit,
            '/owner', 'foreign suite completion')
    caller = document((output/'caller.json').read_text())
    caller.field('shell_pid').equals(live.shell.pid)
    caller.field('shell_exit').equals(result.shell_exit)
    caller.field('shell_wait_observed').equals(True)
    caller.field('close_settled_returned').equals(True)
    require(result.settlement == 'SETTLED' and result.shell_exit is not None and result.shell_exit >= 0,
            '/settlement', 'unsettled suite')
    require(result.completed == child.SUITE_NAMES[:len(result.completed)], '/coverage', 'non-prefix suite')
    suite_inventory(plan, result.completed)
    frozen = output_snapshot(output)
    status, audits = suite_audit(plan, result, output)
    _ = inputs_current(inputs)
    child.suite_current(plan)
    require(output_snapshot(output) == frozen, '/outputs', 'original artifacts changed during audit')
    receipt = output/'result.json'
    atomic_json(receipt, {'schema': 'generic_oracles_v1', 'status': status, 'source_sha': inputs.product.source_sha,
        'plan': str(plan.path), 'plan_sha256': plan.digest, 'shell_pid': result.shell_pid, 'shell_exit': result.shell_exit,
        'completed': list(result.completed), 'oracles': audits, 'artifacts': {k: v for k, v in frozen.items()},
        'classification': 'NO_REGISTERED_SIMULATION_WORLD', 'native_worlds': [],
        'build_qualification': 'PENDING', 'gpu_acceptance': 'PENDING', 'publication_eligible': False})
    digest = file_hash(receipt)
    _ = inputs_current(inputs)
    child.suite_current(plan)
    require(output_snapshot(output) == {**frozen, 'result.json': digest}, '/outputs', 'final artifacts changed')
    return OracleResult(status, receipt, digest)


def component_environment(inputs: OracleInputs, output: Path) -> dict[str, str]:
    """Exact source-owned suite environment, also used for historical binding."""
    return {'PATH': inputs.tools.path, 'HOME': str(output/'home'), 'LANG': 'C', 'LC_ALL': 'C',
        'PYTHONNOUSERSITE': '1', 'PYTHONDONTWRITEBYTECODE': '1', 'PYTHONCOERCECLOCALE': '0',
        '__CF_USER_TEXT_ENCODING': f'0x{os.getuid():X}:0x3:0x33', 'GODOT_BIN': str(inputs.product.executable)}


def run_component(inputs: OracleInputs, output: Path, lane_fd: int) -> OracleResult:
    """Caller owns the lane through return or retains the typed UNKNOWN indefinitely."""
    sources = inputs_current(inputs)
    child.suite_lane(lane_fd)
    fcntl.flock(lane_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    require(not output.exists() and output == output.resolve(), '/output', 'fresh canonical output required')
    real_path(output.parent, directory=True)
    require(all(not output.is_relative_to(root) and not root.is_relative_to(output)
                for root in (inputs.product.input_root, inputs.tools.root)), '/roots', 'output overlaps source')
    output.mkdir()
    evidence, logs, home = output/'evidence', output/'logs', output/'home'
    for directory in (evidence, logs, home):
        directory.mkdir()
    environment = component_environment(inputs, output)
    planfile = output/'plan.json'
    atomic_json(planfile, {'schema': child.SUITE_SCHEMA, 'root': str(inputs.product.input_root),
        'evidence': str(evidence), 'logs': str(logs), 'godot': str(inputs.product.executable),
        'inputs': {str(path): digest for path, digest in sources}, 'environment': {k: v for k, v in environment.items()}})
    plan = child.load_suite(planfile, file_hash(planfile))
    live, settled = launch(plan, lane_fd, output)
    return finish(inputs, live, settled, output)


class OracleSettledFailure(RuntimeError):
    """Source control flow proves no unresolved children, not a fabricated receipt."""
    def __init__(self, output: Path, primary: BaseException) -> None:
        super().__init__('source-owned oracle preparation or settled validation failed')
        self.output: Path = output
        self.primary: BaseException = primary


def run_owned_component(inputs: OracleInputs, output: Path, lane_fd: int) -> OracleResult:
    """Only launch crosses process intent and it converts all later faults to UNKNOWN."""
    try:
        return run_component(inputs, output, lane_fd)
    except OracleSuiteOwnershipUnknown:
        raise
    except BaseException as failure:  # noqa: BLE001 -- Source-owned prelaunch/post-settlement boundary; retain primary without rendering.
        # Do not adopt an existing result file on stale-output/preflight rejection.
        raise OracleSettledFailure(output, failure) from None


def fields(node: Node, expected: set[str]) -> None:
    require(set(node.mapping()) == expected, node.label, 'receipt fields differ')


def signed_exit(node: Node) -> int:
    value = node.value
    require(type(value) is int and -255 <= value <= 255, node.label, 'raw native exit required')
    if type(value) is not int:
        raise AssertionError('validated raw exit')
    return value


def historical_row(plan: child.SuitePlan, row: child.SuiteRow, shell_pid: int, owner_pid: int) -> None:
    """Join native wait evidence without constructing a live controller or inspecting PIDs."""
    request = document((row.evidence/'request.json').read_text())
    fields(request, {'schema', 'plan_sha256', 'ordinal', 'identity', 'wrapper_pid', 'parent_pid',
                     'cwd', 'argv', 'limit', 'log_device', 'log_inode'})
    expected: dict[str, Json] = {'schema': child.SUITE_SCHEMA, 'plan_sha256': plan.digest, 'ordinal': row.ordinal,
        'identity': row.identity, 'parent_pid': shell_pid, 'cwd': str(plan.root), 'argv': list(row.argv), 'limit': row.limit}
    for name, value in expected.items():
        request.field(name).equals(value)
    for name in ('ordinal', 'parent_pid', 'limit', 'log_device', 'log_inode'):
        _ = integer(request.field(name))
    wrapper_pid = integer(request.field('wrapper_pid'))
    require(wrapper_pid > 0 and wrapper_pid not in (shell_pid, owner_pid), '/wrapper', 'invalid wrapper identity')
    log = row.log.stat()
    request.field('log_device').equals(log.st_dev)
    request.field('log_inode').equals(log.st_ino)
    wrapper = document((row.evidence/'wrapper-start.json').read_text())
    fields(wrapper, {'schema', 'request', 'wrapper_wait_observed'})
    wrapper.field('schema').equals(child.SUITE_SCHEMA)
    require(json.dumps(wrapper.field('request').value, sort_keys=True) == json.dumps(request.value, sort_keys=True),
            '/wrapper', 'typed request differs')
    require(wrapper.field('wrapper_wait_observed').value is False, '/wrapper', 'invented wrapper wait')
    start = document((row.evidence/'native-start.json').read_text())
    end = document((row.evidence/'native-terminal.json').read_text())
    fields(start, {'schema', 'request', 'native_pid', 'environment', 'stream_start'})
    fields(end, {'schema', 'request', 'native_pid', 'native_exit', 'timed_out', 'settlement', 'error',
                 'stream_start', 'stream_end', 'stream_sha256', 'started_monotonic_ns', 'ended_monotonic_ns'})
    for node in (start, end):
        node.field('schema').equals(child.SUITE_SCHEMA)
        require(json.dumps(node.field('request').value, sort_keys=True) == json.dumps(request.value, sort_keys=True),
                '/native', 'typed request differs')
    pid = integer(start.field('native_pid'))
    require(pid > 0 and pid not in (wrapper_pid, shell_pid, owner_pid), '/native', 'invalid native identity')
    require(integer(end.field('native_pid')) == pid, '/native', 'native identity differs')
    start.field('environment').equals({k: v for k, v in child.suite_environment(plan).items()})
    end.field('settlement').equals('SETTLED')
    end.field('error').equals('')
    _ = signed_exit(end.field('native_exit'))
    require(type(end.field('timed_out').value) is bool, '/native', 'raw timeout boolean required')
    first = integer(start.field('stream_start'))
    require(integer(end.field('stream_start')) == first, '/stream', 'stream start differs')
    last = integer(end.field('stream_end'))
    end.field('stream_sha256').equals(range_hash(row.log, first, last))
    require(integer(end.field('ended_monotonic_ns')) > integer(end.field('started_monotonic_ns')),
            '/native', 'invalid native interval')


def historical_suite(plan: child.SuitePlan, output: Path, shell_pid: int, shell_exit: int,
                     completed: tuple[str, ...]) -> child.SuiteResult:
    """Parse historical joins only; returned data cannot release any live ownership."""
    caller = document((output/'caller.json').read_text())
    fields(caller, {'argv', 'cwd', 'environment', 'pass_fds', 'shell_pid', 'shell_exit',
                    'shell_wait_observed', 'close_settled_returned'})
    argv: list[Json] = ['bash', 'tools/v8/mux2_gate_checks.sh', 'oracles', str(plan.logs)]
    caller.field('argv').equals(argv)
    caller.field('cwd').equals(str(plan.root))
    require(integer(caller.field('shell_pid')) == shell_pid and integer(caller.field('shell_exit'), 255) == shell_exit,
            '/caller', 'outer shell differs')
    require(caller.field('shell_wait_observed').value is True and caller.field('close_settled_returned').value is True,
            '/caller', 'missing returned settlement')
    descriptors = tuple(integer(node) for node in caller.field('pass_fds').array())
    require(len(descriptors) == 3 and len(set(descriptors)) == 3 and min(descriptors) > 2,
            '/caller', 'invalid recorded descriptor set')
    lane_fd, request_fd, control_fd = descriptors
    environment = dict(plan.environment)
    environment.update({child.SUITE_PLAN: str(plan.path), child.SUITE_HASH: plan.digest,
        child.SUITE_REQUEST: str(request_fd), child.SUITE_CONTROL: str(control_fd), FD_ENV: str(lane_fd)})
    caller.field('environment').equals({k: v for k, v in environment.items()})
    prepared = document((plan.evidence/'suite-prepared.json').read_text())
    fields(prepared, {'schema', 'plan_sha256', 'owner_pid', 'nonce', 'request_inode', 'control_inode'})
    prepared.field('schema').equals(child.SUITE_SCHEMA)
    prepared.field('plan_sha256').equals(plan.digest)
    require(re.fullmatch('[0-9a-f]{64}', prepared.field('nonce').text()) is not None, '/prepared', 'invalid nonce')
    owner_pid = integer(prepared.field('owner_pid'))
    require(owner_pid > 0 and owner_pid != shell_pid, '/prepared', 'invalid owner identity')
    _ = integer(prepared.field('request_inode'))
    _ = integer(prepared.field('control_inode'))
    start = document((plan.evidence/'suite-start.json').read_text())
    fields(start, {'schema', 'plan_sha256', 'shell_pid', 'owner_pid', 'argv', 'cwd'})
    expected_start: dict[str, Json] = {'schema': child.SUITE_SCHEMA, 'plan_sha256': plan.digest, 'argv': argv, 'cwd': str(plan.root)}
    for name, value in expected_start.items():
        start.field(name).equals(value)
    require(integer(start.field('shell_pid')) == shell_pid and integer(start.field('owner_pid')) == owner_pid,
            '/start', 'suite identity differs')
    terminal = document((plan.evidence/'suite-terminal.json').read_text())
    fields(terminal, {'schema', 'plan_sha256', 'shell_pid', 'shell_exit', 'settlement', 'completed',
                     'closed_log_sha256', 'wrapper_waits_observed', 'oracle_admission', 'error'})
    for name, value in {'schema': child.SUITE_SCHEMA, 'plan_sha256': plan.digest, 'settlement': 'SETTLED',
                        'oracle_admission': 'CALLER_REQUIRED', 'error': ''}.items():
        terminal.field(name).equals(value)
    require(integer(terminal.field('shell_pid')) == shell_pid and integer(terminal.field('shell_exit'), 255) == shell_exit,
            '/terminal', 'suite exit differs')
    terminal.field('completed').equals(list(completed))
    require(terminal.field('wrapper_waits_observed').value is False, '/terminal', 'invented wrapper waits')
    terminal.field('closed_log_sha256').equals({str(row.log): file_hash(row.log) for row in plan.rows[:len(completed)]})
    for row in plan.rows[:len(completed)]:
        historical_row(plan, row, shell_pid, owner_pid)
        if row.ordinal < len(completed):
            native = document((row.evidence/'native-terminal.json').read_text())
            require(signed_exit(native.field('native_exit')) == 0 and native.field('timed_out').value is False,
                    '/prefix', 'original shell cannot continue after producer failure')
            _ = audit_log(row.identity, row.log)  # Original set -e also stops on checker failure.
    return child.SuiteResult('SETTLED', shell_pid, shell_exit, completed, '')


def authenticate_result(inputs: OracleInputs, receipt: Path, expected_sha256: str) -> OracleResult:
    """Read-only complete evidence joins; no build, live settlement or PID-recovery authority."""
    real_path(receipt, directory=False)
    require(receipt.name == 'result.json' and re.fullmatch('[0-9a-f]{64}', expected_sha256) is not None
            and file_hash(receipt) == expected_sha256, '/receipt', 'terminal binding differs')
    output = receipt.parent
    frozen = output_snapshot(output)
    node = document(receipt.read_text())
    fields(node, {'schema', 'status', 'source_sha', 'plan', 'plan_sha256', 'shell_pid', 'shell_exit', 'completed',
                  'oracles', 'artifacts', 'classification', 'native_worlds', 'build_qualification', 'gpu_acceptance', 'publication_eligible'})
    for name, value in {'schema': 'generic_oracles_v1', 'source_sha': inputs.product.source_sha,
                        'classification': 'NO_REGISTERED_SIMULATION_WORLD', 'build_qualification': 'PENDING', 'gpu_acceptance': 'PENDING'}.items():
        node.field(name).equals(value)
    node.field('native_worlds').equals([])
    require(node.field('publication_eligible').value is False, '/receipt', 'publication authority forbidden')
    node.field('artifacts').equals({k: v for k, v in frozen.items() if k != 'result.json'})
    node.field('plan').equals(str(output/'plan.json'))
    plan = child.load_suite(output/'plan.json', node.field('plan_sha256').text())
    document(plan.path.read_text()).field('godot').equals(str(inputs.product.executable))
    require(all(not output.is_relative_to(root) and not root.is_relative_to(output)
                for root in (inputs.product.input_root, inputs.tools.root)), '/roots', 'output overlaps source')
    require(plan.root == inputs.product.input_root and plan.evidence == output/'evidence' and plan.logs == output/'logs',
            '/plan', 'foreign product/output scope')
    require(dict(plan.environment) == component_environment(inputs, output), '/plan', 'suite environment differs')
    require(dict(plan.inputs) == dict(inputs_current(inputs)), '/inputs', 'complete suite input map differs')
    completed = tuple(row.text() for row in node.field('completed').array())
    require(completed == child.SUITE_NAMES[:len(completed)], '/coverage', 'not exact completed prefix')
    suite_inventory(plan, completed)
    pid = integer(node.field('shell_pid'))
    require(pid > 0, '/pid', 'invalid shell identity')
    code = integer(node.field('shell_exit'), 255)
    history = historical_suite(plan, output, pid, code, completed)
    status, audits = suite_audit(plan, history, output)
    node.field('status').equals(status)
    node.field('oracles').equals(audits)
    _ = inputs_current(inputs)
    child.suite_current(plan)
    require(output_snapshot(output) == frozen and file_hash(receipt) == expected_sha256,
            '/outputs', 'historical artifacts changed during authentication')
    return OracleResult(status, receipt, expected_sha256)
