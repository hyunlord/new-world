# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by the gate owner; unittest test_generic_renderer_run.
"""Actual renderer producers plus native safety; no build/publication authority."""
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import (
    GateContractError,
    boundary,
    document,
    integer,
    require,
    typed_json,
)
from generic_observations import Observation, ProbeWorld, ordinary_pair, parse_probe
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_sources import inventory
from generic_ui_companion import native_commands
from generic_ui_observations import UIInstance
from generic_ui_receipts import (
    CheckpointPlan,
    InstancePlan,
    UIPlan,
    validate_ui,
)
from generic_ui_receipts import (
    bind_process as validate_process,
)
from mux2_gate_inputs import BRIDGE, Json, Node

PRODUCER: Final = 'renderer.agreement'
SCRIPT: Final = 'res://tools/v8/renderer_agreement.gd'
HELPER: Final = 'res://tools/v8/gate_ui_observer.gd'
# Source-reviewed workload and original domain oracle; not caller-provided PASS rows.
PINS: Final[dict[str, str]] = {'tools/v8/renderer_agreement.sh': '0d05f038764de1b611297445ae9e713d3829790e82d0670af9c7b04a03373be8', 'tools/v8/renderer_agreement.gd': '6fc92423e9ac3ccf63e1af37e51227791b84c0bcdc43806bc757e775113f391c', 'tools/v8/u_script.json': '32e9f6423867c858b678bcec9d1a14a170bba043917898c52434bcf00515f5b0', 'tools/v8/gate_ui_observer.gd': 'b0f2ef8c577d777aff9442a55bf3f8ad29c5a5898011d0b331a24db4c9dba967', 'scripts/core/main_bootstrap.gd': '1801e3bb6cac5d4e6085ae804648d86bca17d073f2d09d17d882dec719d9e1f5', 'rust/crates/sim-test/src/probe/ascii.rs': '01be9f8d756439c1bec2c74137148918d7258c03abdac60f90b0457ba0d3eecc'}


@dataclass(frozen=True, slots=True)
class RendererProducts:
    probes: Products
    godot: Products


@dataclass(frozen=True, slots=True)
class ArmEvidence:
    generator: Observation
    ascii: Observation
    bootstrap: UIInstance
    replay: UIInstance


def snapshot(path: Path) -> dict[str, str]:
    return inventory(path, [p.name for p in path.iterdir()])


def plan(spec: ProcessSpec, bootstrap_tick: int, commands: Path) -> UIPlan:
    require(bootstrap_tick >= 0, '/bootstrap', 'unknown actual horizon')
    context = json.dumps({'commands_path':str(commands), 'commands_sha256':file_hash(commands)})
    checkpoints = (CheckpointPlan('bootstrap',0,1,0),
                   CheckpointPlan('before_reset',0,1,bootstrap_tick,context),
                   CheckpointPlan('after_reset',0,2,0),
                   CheckpointPlan('replay_complete',0,2,900),
                   CheckpointPlan('terminal',0,2,900,releases_node=True))
    return UIPlan(PRODUCER, PRODUCER+':'+str(spec.output_root), spec, SCRIPT, HELPER, checkpoints,
                  ((InstancePlan('bootstrap',20261010,0,bootstrap_tick),InstancePlan('reset',100,0,900)),))


def spec_for(product: ProbeProduct, kind: str, output: Path, commands: Path) -> ProcessSpec:
    argv = (str(product.executable), '--scene','colony','--seeds','100..101')
    environment = (('WORLDSIM_GATE_OBSERVER','1'),('WORLDSIM_GIT_SHA',product.source_sha))
    external: tuple[tuple[Path,str], ...] = ()
    fresh: tuple[str, ...] = ()
    if kind == 'generator':
        argv += ('--days','1','--workers','1','--commands','tools/v8/u_script.json',
                 '--commands-out',str(output/'commands'),'--out',str(output),'--gate-observations')
    elif kind == 'ascii':
        argv += ('--ascii','--tick','900','--commands','tools/v8/u_script.json','--out',
                 str(output/'ascii_t900.txt'),'--people-json',str(output/'text_people.json'),'--gate-observations')
    else:
        require(kind == 'godot', '/producer', 'unknown renderer producer')
        argv = (str(product.executable),'--headless','--path',str(product.input_root),'--script',SCRIPT,'--',
                '--out',str(output/'godot_people.json'),'--commands',str(commands),'--seed','100','--tick','900')
        external = ((commands,file_hash(commands)),)
        fresh = ('settings',)
        environment += (('HOME',str(Path.home())),('WORLDSIM_GATE_INVOCATION_ID',PRODUCER+':'+str(output)),
                        ('WORLDSIM_GATE_UI_OUT',str(output)),('WORLDSIM_TEST_SETTINGS_DIR',str(output/'settings')))
    return ProcessSpec(argv,product.input_root,product.executable,product.executable_sha256,
                       product.input_root,product.input_files,output,environment,external,fresh,600)


@boundary
def bind_process(spec: ProcessSpec, result: ProcessResult) -> None:
    validate_process(UIPlan(PRODUCER,'',spec,SCRIPT,HELPER,(),()),result,file_hash(result.receipt))


@boundary
def inspect_probe(spec: ProcessSpec, result: ProcessResult, duration: int) -> Observation:
    bind_process(spec,result)
    directory = spec.output_root/'gate_observations'
    require({p.name for p in directory.iterdir()} == {'colony_s100_primary.json'}, '/probe', 'missing/extra world')
    observed = parse_probe((directory/'colony_s100_primary.json').read_bytes().decode('utf-8'),
                           ProbeWorld('colony',100,'primary',duration))
    require(observed.founders == 10, '/probe', 'changed colony founder census')
    return observed


@boundary
def command_policy(checkpoints: tuple[Node, ...], commands: Path) -> None:
    rows = document('{"rows":'+commands.read_bytes().decode('utf-8')+'}').field('rows').array()
    names = ('designate_chop_tiles','add_stockpile','place_blueprint')
    kinds = ('DesignateChopTiles','AddStockpile','PlaceBlueprint')
    require(len(rows) == 3, '/commands', 'changed U-script command count')
    accepted: list[int] = []
    for row,name in zip(rows,names):
        row.field('tick').equals(360)
        row.field('cmd').equals(name)
        value = row.field('result').field('accepted').value
        require(type(value) is bool, '/commands', 'missing native accepted result')
        accepted.append(int(value is True))
    for index,cp in enumerate(checkpoints):
        log = cp.field('commands')
        if index < 2:
            _ = native_commands(log,integer(cp.field('clock').field('tick')),index == 0)
        else:
            total = 1 if index == 2 else 5
            log.field('total').equals(total)
            log.field('ticks').equals([0] if index == 2 else [0,0,360,360,360])
            log.field('kinds').equals(['SetSteward'] if index == 2 else ['SetSteward','SetSteward',*kinds])
            log.field('accepted').equals([1] if index == 2 else [1,1,*accepted])


@boundary
def inspect_godot(spec: ProcessSpec, result: ProcessResult, commands: Path) -> tuple[UIInstance, UIInstance]:
    require(result.pid is not None, '/godot', 'missing native PID')
    path = spec.output_root/f'{PRODUCER}-{result.pid}.json'
    raw = document(path.read_bytes().decode('utf-8'))
    checkpoints = raw.field('checkpoints').array()
    require(len(checkpoints) == 5, '/godot', 'missing lifecycle/terminal observation')
    expected = plan(spec,integer(checkpoints[1].field('clock').field('tick')),commands)
    observed = validate_ui(path,file_hash(path),expected,result,file_hash(result.receipt))
    require(all(instance.state.founders == 10 for instance in observed.instances[0]), '/godot', 'changed founder census')
    command_policy(tuple(checkpoints),commands)
    for key in ('observation','clock','tallies','commands'):
        checkpoints[-1].field(key).equals(checkpoints[-2].field(key).value)
    stdout = (spec.output_root/'stdout.log').read_bytes().decode('utf-8')
    stderr = (spec.output_root/'stderr.log').read_bytes().decode('utf-8')
    require(not any(s in stdout+stderr for s in ('SCRIPT ERROR','Parse Error','ERROR:')) and
            sum(line.startswith('RENDERER_AGREEMENT_GODOT_DONE') for line in stdout.splitlines()) == 1,
            '/godot', 'original completion/error oracle failed')
    return observed.instances[0][0], observed.instances[0][1]


def oracle_source(product: ProbeProduct) -> str:
    source = (product.input_root/'tools/v8/renderer_agreement.sh').read_text()
    marker = '<<\'PY\'\n'
    require(source.count(marker) == 1 and source.endswith('PY\n'), '/oracle', 'changed source heredoc')
    return source.split(marker,1)[1][:-3]


def prepare(products: RendererProducts, output: Path) -> None:
    require(output.is_absolute() and output == output.resolve() and not output.exists(), '/output', 'fresh canonical output required')
    require(not products.probes.base.input_root.is_relative_to(products.probes.candidate.input_root) and
            not products.probes.candidate.input_root.is_relative_to(products.probes.base.input_root), '/products', 'independent arms required')
    for arm in ('base','candidate'):
        probe, godot = products.probes.arm(arm), products.godot.arm(arm)
        require(probe.source_sha == godot.source_sha and probe.input_root == godot.input_root and
                probe.input_files == godot.input_files, '/products', 'probe/Godot source inputs differ')
        for product in (probe,godot):
            require(not output.is_relative_to(product.input_root) and not product.input_root.is_relative_to(output), '/output', 'product overlap')
            require(bool(product.input_files) and len(dict(product.input_files)) == len(product.input_files), '/inputs', 'missing/duplicate inventory')
            require(inventory(product.input_root,[n for n,_ in product.input_files]) == dict(product.input_files) and
                    file_hash(product.executable) == product.executable_sha256, '/product', 'input/binary drift')
        inputs = dict(probe.input_files)
        require({'project.godot','sim_bridge.gdextension',BRIDGE,'scenes/main.tscn','.godot/extension_list.cfg'} <= inputs.keys(), '/inputs', 'bridge/import inputs missing')
        require(all(inputs.get(name) == digest for name,digest in PINS.items()), '/inputs', 'unreviewed renderer workload/helper')
    require(products.godot.base.executable_sha256 == products.godot.candidate.executable_sha256, '/godot', 'different Godot versions')


def run_pair(products: RendererProducts, output: Path) -> ComponentResult:
    """Caller owns source/build authentication and native lane; execute no builds/imports."""
    prepare(products,output)
    output.mkdir(parents=True)
    evidence: list[ArmEvidence] = []
    errors: list[str] = []
    pending: list[str] = []
    frozen: dict[Path,dict[str,str]] = {}
    executions: list[Json] = []
    for arm in ('base','candidate'):
        root = output/arm
        root.mkdir()
        probe, godot = products.probes.arm(arm), products.godot.arm(arm)
        commands = root/'generator/commands/colony_s100_commands.json'
        try:
            observations: list[Observation] = []
            instances: tuple[UIInstance, UIInstance] | None = None
            for kind,duration in (('generator',1440),('ascii',900),('godot',900)):
                spec = spec_for(godot if kind == 'godot' else probe,kind,root/kind,commands)
                result = execute(spec)
                frozen[spec.output_root] = snapshot(spec.output_root)
                executions.append({'arm':arm,'producer':kind,'receipt':str(result.receipt),'sha256':file_hash(result.receipt),'status':result.status})
                if kind == 'godot':
                    instances = inspect_godot(spec,result,commands)
                else:
                    observations.append(inspect_probe(spec,result,duration))
            python = Path(sys.executable).resolve(strict=True)
            inputs = tuple((p,file_hash(p)) for p in (root/'ascii/text_people.json',root/'godot/godot_people.json'))
            oracle_spec = ProcessSpec((str(python),'-c',oracle_source(probe),str(inputs[0][0]),str(inputs[1][0]),'900'),
                             probe.input_root,python,file_hash(python),probe.input_root,probe.input_files,root/'oracle',external_inputs=inputs,timeout_seconds=600)
            oracle = execute(oracle_spec)
            frozen[root/'oracle'] = snapshot(root/'oracle')
            executions.append({'arm':arm,'producer':'original_python_oracle','receipt':str(oracle.receipt),'sha256':file_hash(oracle.receipt),'status':oracle.status})
            bind_process(oracle_spec,oracle)
            require(oracle.status == 'PASS' and oracle.native_exit == 0, '/oracle', 'original renderer agreement failed')
            require(instances is not None, '/coverage', 'missing Godot instances')
            if instances is None:
                raise AssertionError('validated instances')
            evidence.append(ArmEvidence(observations[0],observations[1],*instances))
        except (GateContractError,OSError,UnicodeError) as error:
            errors.append(f'{arm}: {error}')
    safety: list[Json] = []
    if len(evidence) == 2:
        for name in ('generator','ascii','bootstrap','replay'):
            left,right = evidence
            a = left.generator if name == 'generator' else left.ascii if name == 'ascii' else left.bootstrap.state if name == 'bootstrap' else left.replay.state
            b = right.generator if name == 'generator' else right.ascii if name == 'ascii' else right.bootstrap.state if name == 'bootstrap' else right.replay.state
            try:
                if name == 'bootstrap' and a.terminal_tick != b.terminal_tick:
                    _ = ordinary_pair(a,a)
                    _ = ordinary_pair(b,b)
                    pending.append('bootstrap exact-horizon counterpart unresolved')
                else:
                    safety.append({'occurrence':name,**ordinary_pair(a,b)})
            except GateContractError as error:
                errors.append(f'{name}: {error}')
    for path,hashes in frozen.items():
        try:
            require(snapshot(path) == hashes,str(path),'completed producer artifacts mutated')
        except (GateContractError,OSError) as error:
            errors.append(str(error))
    for pair in (products.probes,products.godot):
        for product in (pair.base,pair.candidate):
            try:
                require(file_hash(product.executable) == product.executable_sha256 and inventory(product.input_root,[n for n,_ in product.input_files]) == dict(product.input_files), '/product', 'terminal input/binary drift')
            except (GateContractError,OSError) as error:
                errors.append(str(error))
    status = 'FAIL' if errors else 'PENDING' if pending else 'PASS'
    receipt = output/'renderer.json'
    atomic_json(receipt,{'schema_version':1,'status':status,'processes':executions,'safety':safety,'errors':list(errors),'pending':list(pending),
                        'outputs':typed_json({str(p):h for p,h in frozen.items()}),'sources':{'base':products.probes.base.source_sha,'candidate':products.probes.candidate.source_sha},
                        'full_gate_claim':False,'build_provenance':'caller_owned_not_certified_here'})
    return ComponentResult(receipt,status)
