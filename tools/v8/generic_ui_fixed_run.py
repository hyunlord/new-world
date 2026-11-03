# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by the gate owner; test with unittest test_generic_ui_fixed_run.
"""Fixed UI component execution; caller owns authenticated builds and native lane.

Products contain Godot executables and complete effective project/input inventories,
including the bridge. This component never supplies build or publication authority.
"""
import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, boundary, document, integer, require, text
from generic_observations import keys, ordinary_pair
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_sources import inventory
from generic_ui_companion import native_commands
from generic_ui_observations import UIInstance, require_fresh_history
from mux2_gate_inputs import BRIDGE, Json, Node

from generic_ui_fixed_lifecycle import (
    BOOT_SEED,
    NO_NATIVE,
    CommandPlan,
    lifecycle_plan,
)
from generic_ui_receipts import UIPlan, validate_ui

__all__ = ['BOOT_SEED', 'NO_NATIVE', 'lifecycle_plan']

@dataclass(frozen=True, slots=True)
class Fixture:
    script: str
    prefix: str
    timeout: float


FIXTURES: Final = {
    'toast': Fixture('mux21_toast_test.gd', 'MUX21_TOAST', 180),
    'marker.layout': Fixture('mux21_layout_test.gd', 'MUX21_LAYOUT', 300),
    'render.ground': Fixture('mart2_ground_test.gd', 'MART2_GROUND', 600),
    'render.people': Fixture('mart2_people_test.gd', 'MART2_PEOPLE', 600),
    'render.trees': Fixture('mart21_trees_test.gd', 'MART21_TREES', 600),
}
HELPER: Final = 'res://tools/v8/gate_ui_observer.gd'
# Pins are inspected workload/observer semantics, not a caller-controlled PASS flag.
PINS: Final[dict[str, str]] = {'tools/v8/mux21_toast_test.gd': 'cfb838663d66f01093850284b1f7052dbeb48612b0e4bf21191f1ac7da7e9429', 'tools/v8/mux21_layout_test.gd': '18082c2a49bcdeb19acfd124b73417740ad4855ccfbc9782f0334f0f9dde0834', 'tools/v8/mart2_ground_test.gd': '168b8bf1fa9a26263a30e7310fa144a24bbcde2f8385bc83606fe96ecd293789', 'tools/v8/mart2_people_test.gd': '699fec4f7d08a416ce8faa73478df2dbb0f0cc06d58ce9de40064f86be519994', 'tools/v8/mart21_trees_test.gd': '32b9dbdb247170653de40faf25578cffc83f68f26155fba064745285c59f0271', 'tools/v8/gate_ui_observer.gd': 'b0f2ef8c577d777aff9442a55bf3f8ad29c5a5898011d0b331a24db4c9dba967', 'scripts/core/main_bootstrap.gd': '1801e3bb6cac5d4e6085ae804648d86bca17d073f2d09d17d882dec719d9e1f5', 'scripts/ui/mux/settings_store.gd': '70ced6c709f8d63f60a3d3e04127db6432eaf86972a69a3a4a9f618e642907ec', 'rust/crates/sim-bridge/src/ffi/engine_build.rs': '7b90e7dd8d2ae713bf88721fabffc478df1cfe46adbed0ac83d667da5129b892', 'rust/crates/sim-systems/src/worldgen/mod.rs': '11525a81664d3206c72754f3772d1ba4e61cd3f9dbe2ab611be4e4124378b601'}
CommandRows = tuple[tuple[int, str, int], ...]


def definition(producer: str) -> Fixture:
    require(producer in FIXTURES, '/producer', 'unsupported fixed UI fixture')
    return FIXTURES[producer]




def make_plan(producer: str, spec: ProcessSpec, bootstrap_tick: int) -> UIPlan:
    """Bind the unchanged fixed lifecycle to the original direct process."""
    fixture = definition(producer)
    plan = lifecycle_plan(producer, bootstrap_tick)
    return UIPlan(producer, producer+':'+str(spec.output_root), spec,
                  'res://tools/v8/'+fixture.script, HELPER, plan.checkpoints,
                  plan.instances, plan.classification, no_native_reason=plan.no_native_reason)


@boundary
def command_policy(plan: CommandPlan, logs: tuple[Node, ...]) -> tuple[CommandRows, ...]:
    """Check every checkpoint, including native factory ON and fixture OFF commands."""
    require(len(logs) == len(plan.checkpoints), '/commands', 'missing command checkpoint')
    output: list[CommandRows] = []
    previous: CommandRows = ()
    previous_instance = 0
    for cp, node in zip(plan.checkpoints, logs):
        keys(node, {'total', 'ticks', 'kinds', 'accepted'})
        ticks = tuple(integer(v) for v in node.field('ticks').array())
        kinds = tuple(text(v) for v in node.field('kinds').array())
        accepted = tuple(integer(v, 1) for v in node.field('accepted').array())
        require(len(ticks) == len(kinds) == len(accepted) == integer(node.field('total')),
                '/commands', 'truncated command columns')
        rows = tuple(zip(ticks, kinds, accepted))
        if cp.instance_count == 1:
            _ = native_commands(node, cp.tick, cp.label == 'bootstrap' or plan.producer == 'toast')
        else:
            instance = plan.instances[0][cp.instance_count-1]
            count = 0 if instance.reason == 'stress_reset' else (1 if cp.label == 'after_colony_reset' else 2)
            require(rows == ((0, 'SetSteward', 1),)*count, '/commands', 'unexpected reset command or rejected OFF')
        if previous_instance == cp.instance_count:
            require(rows[:len(previous)] == previous, '/commands', 'command prefix changed')
        previous, previous_instance = rows, cp.instance_count
        output.append(rows)
    return tuple(output)


@boundary
def domain(producer: str, stdout: str, stderr: str) -> None:
    """Preserve native exit separately and inspect the original completion oracle."""
    fixture = definition(producer)
    require(not any(word in stdout+stderr for word in ('SCRIPT ERROR', 'Parse Error', 'ERROR:')),
            '/oracle', 'script/runtime error despite completion')
    done = [line for line in stdout.splitlines() if line.startswith(fixture.prefix+'_DONE ')]
    require(len(done) == 1, '/oracle', 'missing or duplicate completion')
    if producer.startswith('render.'):
        require(done[0] == fixture.prefix+'_DONE failures=0' and
                any(line.startswith(fixture.prefix+' PASS ') for line in stdout.splitlines()) and
                not any(line.startswith(fixture.prefix+' FAIL ') for line in stdout.splitlines()),
                '/oracle', 'render completion/checks failed')
    else:
        row = document(done[0].split(' ', 1)[1])
        row.field('failures').equals(0)
        require(integer(row.field('checks')) > 0, '/oracle', 'no checks executed')
        if producer == 'marker.layout':
            require(integer(row.field('cases')) > 0 and text(row.field('display')) != 'headless',
                    '/oracle', 'layout requires actual renderer')


def process_spec(product: ProbeProduct, producer: str, output: Path) -> ProcessSpec:
    """Consume the owner's authenticated product; preserve original script workload."""
    fixture = definition(producer)
    inputs = dict(product.input_files)
    required = {'project.godot', 'sim_bridge.gdextension', BRIDGE, 'scenes/main.tscn'} | set(PINS)
    require(len(inputs) == len(product.input_files) and required <= inputs.keys(), '/inputs', 'incomplete fixed UI source inventory')
    require(all(inputs[name] == digest for name, digest in PINS.items()), '/inputs', 'unreviewed producer/helper/bootstrap semantics')
    require(all(file_hash(product.input_root/name) == digest for name,digest in product.input_files),
            '/inputs', 'product input bytes changed')
    require(file_hash(product.executable) == product.executable_sha256, '/godot', 'Godot binary changed')
    argv = (str(product.executable), '--path', str(product.input_root))
    argv += ('--rendering-method', 'gl_compatibility') if producer == 'marker.layout' else ('--headless',)
    argv += ('--script', 'res://tools/v8/'+fixture.script)
    if producer == 'marker.layout':
        argv += ('--', '--cache')
    # Godot macOS resolves user:// via HOME; application settings remain isolated.
    environment = (('HOME',str(Path.home())), ('WORLDSIM_GATE_OBSERVER','1'), ('WORLDSIM_GATE_INVOCATION_ID',producer+':'+str(output)),
                   ('WORLDSIM_GATE_UI_OUT',str(output)), ('WORLDSIM_TEST_SETTINGS_DIR',str(output/'settings')),
                   ('WORLDSIM_GIT_SHA',product.source_sha))
    return ProcessSpec(argv, product.input_root, product.executable, product.executable_sha256,
                       product.input_root, product.input_files, output, environment,
                       fresh_directories=('settings',), timeout_seconds=fixture.timeout)


@dataclass(frozen=True, slots=True)
class FixedEvidence:
    plan: UIPlan
    instances: tuple[UIInstance, ...]
    command_rows: tuple[CommandRows, ...]
    sidecar: Path
    sidecar_sha256: str
    process: ProcessResult
    process_sha256: str


@boundary
def inspect_output(producer: str, spec: ProcessSpec, result: ProcessResult) -> FixedEvidence:
    """Authenticate process/sidecar before allowing derived dynamic bootstrap input."""
    receipt_bytes = result.receipt.read_bytes()
    receipt_sha = hashlib.sha256(receipt_bytes).hexdigest()
    receipt = document(receipt_bytes.decode("utf-8"))
    receipt.field('timeout_seconds').equals(definition(producer).timeout)
    receipt.field('timed_out').equals(False)
    require(result.status == 'PASS' and result.native_exit == 0 and result.pid is not None,
            '/process', 'fixed UI child failed')
    sidecar = spec.output_root/f'{producer}-{result.pid}.json'
    sidecar_bytes = sidecar.read_bytes()
    digest = hashlib.sha256(sidecar_bytes).hexdigest()
    raw = document(sidecar_bytes.decode("utf-8"))
    checkpoints = raw.field('checkpoints').array()
    bootstrap_tick = integer(checkpoints[1].field('clock').field('tick')) if producer.startswith('render.') and len(checkpoints)>1 else 0
    plan = make_plan(producer, spec, bootstrap_tick)
    validated = validate_ui(sidecar, digest, plan, result, receipt_sha)
    require(not validated.loaded_unqualified, '/history', 'unexpected loaded history')
    logs = command_policy(plan, tuple(cp.field('commands') for cp in checkpoints))
    domain(producer, (spec.output_root/'stdout.log').read_text(), (spec.output_root/'stderr.log').read_text())
    instances = tuple(instance for node in validated.instances for instance in node)
    for instance in instances:
        _ = require_fresh_history(instance)
        require(instance.state.founders == (1000 if instance.reason == 'stress_reset' else 10),
                '/census', 'unexpected initial fixture population')
    return FixedEvidence(plan, instances, logs, sidecar, digest, result, receipt_sha)


@boundary
def compare(base: FixedEvidence, candidate: FixedEvidence) -> None:
    require(base.plan.producer == candidate.plan.producer and len(base.instances) == len(candidate.instances),
            '/pair', 'different fixture coverage')
    for left, right in zip(base.instances, candidate.instances):
        require(left.state.terminal_tick == right.state.terminal_tick, '/pair',
                'UNRESOLVED: unequal actual bootstrap horizons; no qualified matched companion')
        require((left.reason,left.seed,left.initial_agent_ids,left.terminal_agent_ids,left.state) ==
                (right.reason,right.seed,right.initial_agent_ids,right.terminal_agent_ids,right.state),
                '/pair', 'native census/history differs for fixed fixture')
        if right.reason == 'stress_reset':
            require(not right.state.knowledge_enabled and right.state.alive == 1000,
                    '/stress', 'stress render population/capability changed')
        else:
            _ = ordinary_pair(left.state, right.state)
    require(base.command_rows == candidate.command_rows, '/pair', 'native command transcript differs')


def run_pair(products: Products, producer: str, output: Path) -> ComponentResult:
    """Execute both original producers serially; retain failures, never retry or publish."""
    _ = definition(producer)
    require(not output.exists() and output.is_absolute() and output == output.resolve(), '/output', 'requires fresh external component directory')
    require(all(p.input_root == p.input_root.resolve(strict=True) and
                not output.is_relative_to(p.input_root) and not p.input_root.is_relative_to(output)
                for p in (products.base,products.candidate)),
            '/output', 'evidence must be outside canonical inputs')
    require(not products.base.input_root.is_relative_to(products.candidate.input_root) and
            not products.candidate.input_root.is_relative_to(products.base.input_root),
            '/products', 'independent nonoverlapping product roots required')
    specs = tuple(process_spec(product,producer,output/arm) for arm,product in
                  (('base',products.base),('candidate',products.candidate)))
    require(products.base.executable_sha256 == products.candidate.executable_sha256, '/godot', 'different Godot products')
    output.mkdir()
    evidence: list[FixedEvidence] = []
    errors: list[str] = []
    rows: list[Json] = []
    frozen: dict[Path, dict[str, str]] = {}
    for spec in specs:
        try:
            result = execute(spec)
            frozen[spec.output_root] = inventory(spec.output_root, [p.name for p in spec.output_root.iterdir()])
            row: dict[str, Json] = {'arm':spec.output_root.name,'receipt':str(result.receipt),
                                    'sha256':file_hash(result.receipt),'status':result.status,
                                    'outputs':{name:digest for name,digest in frozen[spec.output_root].items()}}
            rows.append(row)
            actual = inspect_output(producer,spec,result)
            evidence.append(actual)
            row.update(sidecar=str(actual.sidecar),sidecar_sha256=actual.sidecar_sha256)
        except (GateContractError, OSError, UnicodeError) as error:
            errors.append(f'{spec.output_root.name}: {error}')
    if len(evidence) == 2:
        try:
            compare(evidence[0],evidence[1])
        except GateContractError as error:
            errors.append(str(error))
    for folder, hashes in frozen.items():
        try:
            require(inventory(folder, [p.name for p in folder.iterdir()]) == hashes,
                    str(folder), 'child artifacts changed during pair')
        except (GateContractError, OSError) as error:
            errors.append(str(error))
    for product in (products.base, products.candidate):
        try:
            require(file_hash(product.executable) == product.executable_sha256 and
                    inventory(product.input_root, [name for name, _ in product.input_files]) == dict(product.input_files),
                    str(product.input_root), 'product changed during pair')
        except (GateContractError, OSError) as error:
            errors.append(str(error))
    status = 'PASS' if len(evidence) == 2 and not errors else 'FAIL'
    path = output/'fixed-ui.json'
    atomic_json(path, {'schema_version':1,'producer':producer,'status':status,'processes':rows,'errors':[error for error in errors],
                      'build_provenance':'caller_owned_not_certified_here','full_gate_claim':False,
                      'scope':'fixed UI domain plus exact native observations; headless render is not pixel acceptance'})
    return ComponentResult(path,status)
