"""Exact embedded Cargo occurrences; downstream wrapper/workload authority is pending."""
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

from gate_oracle_child import import_generated
from generic_gate import document, require
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
)
from generic_setup_bridge import (
    CargoWaitEvidence,
    held_lane,
    immutable_inputs,
    qualify_wait,
)
from generic_setup_clippy import ClippyEndpoint, check_clippy_endpoint, clippy_command
from generic_setup_prepare import PreparedSetup
from generic_workspace_cases import ListingProcess, process_output
from generic_workspace_endpoints import (
    EvidenceAnchor,
    GeneratedState,
    anchor,
    generated,
    state_json,
)
from mux2_gate_inputs import Json

Step = Literal[10, 11, 12, 18]
PROBE: Final = 'cargo build --release -p sim-test --bin worldsim_probe'
PACKAGE: Final = 'cargo build --release -p sim-test'


@dataclass(frozen=True, slots=True)
class EmbeddedRequest:
    prepared: PreparedSetup
    step: Step
    predecessor: 'ClippyEndpoint | EmbeddedEndpoint'
    wait: CargoWaitEvidence
    output: Path
    descriptor: int


@dataclass(frozen=True, slots=True)
class EmbeddedEndpoint:
    request: EmbeddedRequest
    receipt: Path
    receipt_sha256: str
    pid: int
    before: GeneratedState
    after: GeneratedState
    history: tuple[EvidenceAnchor, ...]
    process: ListingProcess
    status: Literal['PASS_EMBEDDED_BUILD_ONLY'] = 'PASS_EMBEDDED_BUILD_ONLY'
    chronology: Literal['INTERVENING_COMPONENT_HANDOFF_PENDING'] = 'INTERVENING_COMPONENT_HANDOFF_PENDING'
    build_product_authority: Literal[False] = False
    publication_authority: Literal[False] = False


class EmbeddedFailure(RuntimeError):
    def __init__(self, primary: BaseException, secondary: BaseException | None, receipt: Path | None) -> None:
        super().__init__('embedded producer failed; original exception retained')
        self.primary: BaseException = primary
        self.secondary: BaseException | None = secondary
        self.receipt: Path | None = receipt


class EmbeddedOwnershipUnknown(EmbeddedFailure):
    """Actual caller lane must remain held; no downstream or inferred close."""


def prior_history(request: EmbeddedRequest) -> tuple[EvidenceAnchor, ...]:
    prior = request.predecessor
    match prior:
        case ClippyEndpoint():
            require(prior.request.prepared is request.prepared, '/prior', 'foreign setup')
            check_clippy_endpoint(prior)
        case EmbeddedEndpoint():
            require(prior.request.prepared is request.prepared and prior.request.step < request.step,
                    '/prior', 'foreign or reordered embedded predecessor')
            check_endpoint(prior)
    return prior.history


def check_endpoint(endpoint: EmbeddedEndpoint) -> None:
    """Retained immutable history only, not today's evolving generated tree."""
    request = endpoint.request
    historical = prior_history(request)
    immutable_inputs(request.prepared)
    require(len(endpoint.history) == len(historical)+1 and endpoint.history[:-1] == historical
            and endpoint.history[-1].root == request.output, '/history', 'missing producer/history anchor')
    spec = endpoint.process.spec
    prepared = request.prepared
    require(spec.output_root == request.output/'process' and spec.input_root == prepared.inputs.root
            and spec.inputs == prepared.materialized and spec.environment == prepared.inputs.environment,
            '/process', 'retained request binding differs')
    expected_inputs = {str(prepared.inputs.root/name): digest for name,digest in prepared.materialized}
    expected_inputs.update({str(path):digest for path,digest in prepared.external})
    expected_inputs.update({str(path):digest for path,digest in qualify_wait(prepared,request.wait)})
    bash = next(tool for tool in prepared.inputs.tools if tool.name == 'bash')
    expected_inputs[str(bash.path)] = bash.sha256
    _ = process_output(endpoint.process,command(request),prepared.inputs.root,environment_for(spec),expected_inputs)
    require(endpoint.receipt == request.output/'receipt.json' and not (request.output/'failure.json').exists()
            and endpoint.pid == endpoint.process.result.pid and endpoint.process.result.status == 'PASS'
            and endpoint.process.result.native_exit == 0 and type(endpoint.pid) is int and endpoint.pid > 0
            and not endpoint.publication_authority and not endpoint.build_product_authority,
            '/endpoint', 'invalid retained producer')
    require(file_hash(endpoint.receipt) == endpoint.receipt_sha256, '/endpoint', 'receipt changed')
    row = document(endpoint.receipt.read_text())
    expected: dict[str, Json] = {'schema':'embedded_build.v1','step':request.step,'status':endpoint.status,
                      'pid':endpoint.pid,'native_exit':0,'ownership':'NORMAL_CARGO_RETURN',
                      'chronology':endpoint.chronology,'build_product_authority':False,'publication_authority':False,
                      'wrapper_workload':'NOT_RUN','original_argv':list(command(request)),
                      'source_sha':request.prepared.inputs.source_sha,
                      'predecessor_sha256':request.predecessor.receipt_sha256,
                      'generated_before':state_json(endpoint.before),'generated_after':state_json(endpoint.after),
                      'process_receipt':str(endpoint.process.result.receipt),
                      'process_receipt_sha256':endpoint.process.receipt_sha256}
    for key,value in expected.items(): row.field(key).equals(value)
    require(endpoint.before == request.predecessor.after and endpoint.before.cache == endpoint.after.cache
            and endpoint.before.settings == endpoint.after.settings, '/endpoint', 'transition differs')
    for item in endpoint.history: item.recheck()


def command(request: EmbeddedRequest) -> tuple[str, ...]:
    # Reuse original setup tool/environment checks without the stale initial recipe.
    shell = clippy_command(request.prepared)[0]
    root = request.prepared.inputs.root
    require(type(request.step) is int and request.step in (10, 11, 12, 18), '/step', 'unsupported embedded occurrence')
    source = root/('tools/v8/identity.sh' if request.step == 10 else 'tools/v8/gate.sh')
    require(dict(request.prepared.materialized).get(str(source.relative_to(root))) == file_hash(source),
            '/source', 'wrapper is not an immutable input')
    lines = source.read_text().splitlines()
    expected = SOURCES[request.step]
    selected = tuple(line for line in lines if ('cargo build' in line if request.step == 10 else line.lstrip().startswith(f'step {request.step} ')))
    require(selected == expected
            and all(lines.count(line) == 1 for line in expected), '/source', 'original wrapper command changed')
    if request.step == 10:
        # identity.sh owns a local ROOT; positional binding avoids shell interpolation/export.
        return (shell, '-c', 'ROOT="$1"; '+expected[0], 'embedded-build', str(root))
    body = PACKAGE if request.step == 12 else PROBE
    body = f'(cd rust && {body})' if request.step == 12 else f'cd rust && {body}'
    return (shell, '-c', body)


def recheck(request: EmbeddedRequest, state: GeneratedState) -> None:
    _ = prior_history(request)
    immutable_inputs(request.prepared)
    require(generated(request.prepared) == state, '/generated', 'state changed outside actual producer')


def run_owned(request: EmbeddedRequest) -> EmbeddedEndpoint:
    output = request.output; prepared = request.prepared
    receipt: Path | None = None; in_flight = False; result: ProcessResult | None = None
    row: dict[str, Json] = {'schema':'embedded_build.v1','step':request.step,'status':'FAIL',
        'ownership':'NOT_STARTED','build_product_authority':False,'publication_authority':False,
        'chronology':'INTERVENING_COMPONENT_HANDOFF_PENDING','wrapper_workload':'NOT_RUN'}
    try:
        held_lane(request.descriptor)
        argv = command(request)
        history = prior_history(request)
        before = request.predecessor.after
        recheck(request, before)
        waits = qualify_wait(prepared, request.wait)
        require(output.is_absolute() and output == output.resolve() and output.parent.is_dir()
                and not output.exists() and not output.is_symlink(), '/output', 'fresh canonical output required')
        for protected in (prepared.inputs.root, prepared.inputs.settings, prepared.inputs.cargo_home,
                          *(item.root for item in history)):
            require(not output.is_relative_to(protected) and not protected.is_relative_to(output), '/output', 'overlap')
        external = dict(prepared.external)
        for path, digest in waits:
            require(not path.is_relative_to(prepared.inputs.root) and (path not in external or external[path] == digest),
                    '/wait', 'conflicting wait evidence')
            external[path] = digest
        bash = next(tool for tool in prepared.inputs.tools if tool.name == 'bash')
        spec = ProcessSpec(argv,prepared.inputs.root,bash.path,bash.sha256,prepared.inputs.root,
                           prepared.materialized,output/'process',prepared.inputs.environment,tuple(sorted(external.items())))
        output.mkdir(); receipt = output/'receipt.json'
        row.update(source_sha=prepared.inputs.source_sha,original_argv=list(argv),
                   predecessor_sha256=request.predecessor.receipt_sha256,generated_before=state_json(before))
        started_text = json.dumps(row,sort_keys=True,indent=2)+'\n'
        empty_output = import_generated(output)
        atomic_json(output/'started.json',row)
        require((output/'started.json').read_text() == started_text, '/started', 'publication content differs')
        started_state = import_generated(output)
        require(set(started_state) == {'.','started.json'} and started_state['.'] == empty_output['.'],
                '/started', 'publication namespace changed')
        recheck(request,before)
        require(import_generated(output) == started_state, '/started', 'prelaunch evidence changed')
        in_flight = True
        result = execute(spec)
        require(result.status == 'PASS' and result.native_exit == 0 and type(result.pid) is int and result.pid > 0,
                '/wait', 'actual normal Cargo return not established')
        assert result.pid is not None
        after = generated(prepared); process_anchor = anchor(spec.output_root); output_state = import_generated(output)
        require({key:value for key,value in output_state.items() if key != 'process' and not key.startswith('process/')} == started_state,
                '/started', 'producer changed initial evidence or output namespace')
        require(before.cache == after.cache and before.settings == after.settings, '/generated', 'build changed Godot/settings')
        binding = ListingProcess(spec,result,file_hash(result.receipt))
        required = {str(prepared.inputs.root/name): digest for name,digest in prepared.materialized}
        required.update({str(path):digest for path,digest in external.items()}); required[str(bash.path)] = bash.sha256
        _ = process_output(binding,argv,spec.cwd,environment_for(spec),required)
        require(command(request) == argv and qualify_wait(prepared,request.wait) == waits, '/wait', 'source profile changed')
        recheck(request,after); process_anchor.recheck()
        in_flight = False
        row.update(status='PASS_EMBEDDED_BUILD_ONLY',ownership='NORMAL_CARGO_RETURN',pid=result.pid,native_exit=0,
                   process_receipt=str(result.receipt),process_receipt_sha256=binding.receipt_sha256,
                   generated_after=state_json(after))
        require(import_generated(output) == output_state, '/output', 'late output mutation')
        expected = json.dumps(row,sort_keys=True,indent=2)+'\n'
        atomic_json(receipt,row)
        require(receipt.read_text() == expected, '/receipt', 'publication mutation')
        require({key:value for key,value in import_generated(output).items() if key != 'receipt.json'} == output_state,
                '/output', 'publication membership/mode mutation')
        endpoint = EmbeddedEndpoint(request,receipt,file_hash(receipt),result.pid,before,after,(*history,anchor(output)),binding)
        check_endpoint(endpoint); recheck(request,after); process_anchor.recheck(); held_lane(request.descriptor)
        # Use retained complete anchors after all authentication/lane callbacks; never refresh them.
        for item in endpoint.history: item.recheck()
        return endpoint
    except BaseException as primary:
        row['ownership'] = 'UNKNOWN_KEEP_CALLER_LANE' if in_flight else 'NO_INFLIGHT_OWNED_CALL'
        row['status'] = 'FAIL'
        row['generated_after'] = None
        if result is not None: row.update(native_exit=result.native_exit,pid=result.pid,process_status=result.status)
        failure = EmbeddedOwnershipUnknown if in_flight else EmbeddedFailure
        if receipt is not None:
            try:
                if receipt.exists(): receipt = output/'failure.json'
                atomic_json(receipt,row)
            except BaseException as diagnostic:
                raise failure(primary,diagnostic,receipt) from diagnostic
        raise failure(primary,None,receipt) from primary


SOURCES: Final[dict[int, tuple[str, ...]]] = {10: ('(cd "$ROOT/rust" && cargo build --release -p sim-test --bin worldsim_probe)',), 11: ('step 11 determinism bash -c \'cd rust && cargo build --release -p sim-test --bin worldsim_probe && ./target/release/worldsim_probe --scene baseline --seeds 0..4 --days 120 --workers 4 --check-determinism --out "$0/determinism_probe" && echo "determinism checked on seeds 0..4"\' "$LOG"',), 12: ('    step 12 perf bash -c \'(cd rust && cargo build --release -p sim-test) && CORE_P7_RUN=1 CORE_P7_DEADLINE_EPOCH="${M6A_DEADLINE_EPOCH:?}" bash tools/v8/perf.sh rust/target/release "$0/perf" candidate > /dev/null && echo "M6A report-only historical unpaired comparison; paired acceptance step25/45" && python3 tools/v8/perf_summary.py --compare "$1/perf/perf_summary.json" "$0/perf/perf_summary.json"\' "$LOG" "$BASE_DATA"', '    step 12 perf bash -c \'(cd rust && cargo build --release -p sim-test) && CORE_P7_RUN=1 CORE_P7_DEADLINE_EPOCH="${CORE_UI_DEADLINE_EPOCH:?}" bash tools/v8/perf.sh rust/target/release "$0/perf" candidate > /dev/null && python3 tools/v8/perf_summary.py --compare "$1/perf/perf_summary.json" "$0/perf/perf_summary.json"\' "$LOG" "$BASE_DATA"', '    step 12 perf bash -c \'(cd rust && cargo build --release -p sim-test) && bash tools/v8/perf.sh rust/target/release "$0/perf" candidate > /dev/null && echo "P7 report-only natural timings; no unpaired historical comparison; paired acceptance is step25/45"\' "$LOG"', '    step 12 perf bash -c \'(cd rust && cargo build --release -p sim-test) && echo "READ report-only; paired acceptance is step 25 (interleaved bench)"\'', '    step 12 perf bash -c \'(cd rust && cargo build --release -p sim-test) && bash tools/v8/perf.sh rust/target/release "$0/perf" candidate > /dev/null && python3 tools/v8/perf_summary.py --compare "$1/perf/perf_summary.json" "$0/perf/perf_summary.json"\' "$LOG" "$BASE_DATA"'), 18: ('step 18 colony_determinism bash -c \'cd rust && cargo build --release -p sim-test --bin worldsim_probe && ./target/release/worldsim_probe --scene colony --seeds 0..4 --days 10 --workers 4 --commands ../tools/v8/u_script.json --check-determinism --out "$0/colony_determinism"\' "$LOG"',)}
