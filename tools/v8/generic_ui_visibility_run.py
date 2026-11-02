"""Original visibility generation/capture, authenticated within each product arm.

Caller owns qualified builds and native lane. No caller-supplied PASS authority.
"""
import hashlib
import json
import math
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, boundary, document, integer, require, text
from generic_observations import Observation, keys, ordinary_pair
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_sources import inventory
from generic_ui_companion import native_commands
from generic_ui_history import SaveLink, UIEvidence, join_visibility_history
from generic_ui_observations import UIInstance, require_fresh_history
from generic_ui_receipts import CheckpointPlan, InstancePlan, UIPlan, validate_ui
from mux2_gate_inputs import BRIDGE, Json, Node
from mux2_gate_visual import validate_visibility

TARGETS: Final = (('day1_noon',1,12,240),('day1_night',1,22,840),
                 ('day20_noon',20,12,27600),('day20_night',20,22,28200))
GENERATOR: Final = 'tools/v8/mux2_visibility_saves.gd'
LOADER: Final = 'tools/v8/mux2_visibility_test.gd'
HELPER: Final = 'res://tools/v8/gate_ui_observer.gd'
# Exact inspected UI workload and calendar/bootstrap implementation, not caller knobs.
PINS: Final[dict[str,str]] = {'tools/v8/mux2_visibility_saves.gd': 'd019da58ce5d2d75c0607652fc649474ef40ba4540d5d95e4670230735707b4b', 'tools/v8/mux2_visibility_test.gd': '76ab91eb63da7a6814f549efc87e236ad0358d8883fe1f155e7edd226496e490', 'tools/v8/mux2_visibility_metrics.gd': 'a1c332af8d78cf1bf198544bf430a90b8cd9f306700d5234ea5db8485d453fa2', 'tools/v8/mux2_visibility_provenance.gd': '2198ea956808547d9c4c86b7ba98c1da6815c092363a78744bf13fef6293806b', 'tools/v8/gate_ui_observer.gd': 'b0f2ef8c577d777aff9442a55bf3f8ad29c5a5898011d0b331a24db4c9dba967', 'scripts/core/main_bootstrap.gd': '1801e3bb6cac5d4e6085ae804648d86bca17d073f2d09d17d882dec719d9e1f5', 'scripts/ui/mux/settings_store.gd': '70ced6c709f8d63f60a3d3e04127db6432eaf86972a69a3a4a9f618e642907ec', 'rust/crates/sim-bridge/src/ffi/engine_build.rs': '7b90e7dd8d2ae713bf88721fabffc478df1cfe46adbed0ac83d667da5129b892', 'rust/crates/sim-systems/src/worldgen/mod.rs': '11525a81664d3206c72754f3772d1ba4e61cd3f9dbe2ab611be4e4124378b601'}
Rows = tuple[tuple[int,str,int], ...]


def check_product(product: ProbeProduct) -> None:
    inputs=dict(product.input_files)
    required=set(PINS)|{BRIDGE,'project.godot','sim_bridge.gdextension','scenes/main.tscn'}
    require(len(inputs)==len(product.input_files) and required<=inputs.keys(),'/inputs','incomplete visibility inventory')
    require(all(inputs[name]==digest for name,digest in PINS.items()),'/inputs','unreviewed visibility semantics')
    require(inventory(product.input_root,list(inputs))==inputs and file_hash(product.executable)==product.executable_sha256,
            '/inputs','product changed')


def process_spec(product: ProbeProduct, arm: Path, generator: bool,
                 external: tuple[tuple[Path,str], ...] = ()) -> ProcessSpec:
    check_product(product)
    producer='visibility.generator' if generator else 'visibility.loader'
    output=arm/('generator-process' if generator else 'loader-process')
    script=GENERATOR if generator else LOADER
    argv=(str(product.executable),'--path',str(product.input_root),'--resolution','1920x1080')
    argv+=('--headless',) if generator else ()
    argv+=('--script','res://'+script,'--')
    argv+=(str(arm/'saves'),) if generator else ('--manifest',str(arm/'saves/manifest.json'),'--out',str(arm/'run'))
    environment=(('HOME',str(Path.home())),('WORLDSIM_GATE_OBSERVER','1'),
                 ('WORLDSIM_GATE_INVOCATION_ID',producer+':'+str(output)),('WORLDSIM_GATE_UI_OUT',str(output)),
                 ('WORLDSIM_TEST_SETTINGS_DIR',str(output/'settings')),('WORLDSIM_GIT_SHA',product.source_sha))
    return ProcessSpec(argv,product.input_root,product.executable,product.executable_sha256,product.input_root,
                       product.input_files,output,environment,external,fresh_directories=('settings',),
                       timeout_seconds=1800 if generator else 1200)


@boundary
def manifest(arm: Path, product: ProbeProduct) -> tuple[Node,...]:
    node=document((arm/'saves/manifest.json').read_bytes().decode('utf-8'))
    node.field('schema').equals(1);node.field('seed').equals(0);node.field('steward').equals('on')
    node.field('bridge_sha256').equals(dict(product.input_files)[BRIDGE])
    node.field('script_sha256').equals(dict(product.input_files)[GENERATOR])
    rows=tuple(node.field('scenes').array())
    require(len(rows)==4,'/manifest','four exact scenes required')
    for row,(name,day,hour,tick) in zip(rows,TARGETS):
        for field,value in (('id',name),('day',day),('hour',hour),('tick',tick),('save',str(arm/'saves'/(name+'.wsave')))):
            row.field(field).equals(value)
        row.field('sha256').equals(file_hash(arm/'saves'/(name+'.wsave')))
    return rows


def loader_context(value: Json) -> Json:
    """Match the pinned loader's parsed JSON numbers without relaxing context equality."""
    if isinstance(value, dict):
        return {key:loader_context(child) for key,child in value.items()}
    if isinstance(value, list):
        return [loader_context(child) for child in value]
    if type(value) is int:
        require(abs(value) <= (1 << 53) - 1,'/context','integer exceeds exact loader JSON range')
        return float(value)
    if isinstance(value, float):
        require(math.isfinite(value),'/context','nonfinite loader JSON number')
    return value


def plan(spec: ProcessSpec, rows: tuple[Node,...], generator: bool) -> UIPlan:
    require(len(rows)==4,'/plan','four scenes required')
    arm=spec.output_root.parent
    checkpoints: list[CheckpointPlan]=[]
    instances: list[tuple[InstancePlan,...]]=[]
    if generator:
        checkpoints=[CheckpointPlan('bootstrap',0,1,0),CheckpointPlan('after_reset',0,2,0)]
        for index,(row,target) in enumerate(zip(rows,TARGETS)):
            context: dict[str,Json]={'save_path':text(row.field('save')),'scene_id':target[0]}
            checkpoints.append(CheckpointPlan('save_prefix',0,2,target[3],json.dumps(context)))
            context.update(prefix_checkpoint=2+index*2,save_sha256=text(row.field('sha256')))
            checkpoints.append(CheckpointPlan('save_link',0,2,target[3],json.dumps(context)))
        checkpoints.append(CheckpointPlan('terminal',0,2,28200,releases_node=True))
        instances=[(InstancePlan('bootstrap',20261010,0,0),InstancePlan('reset',0,0,28200))]
    else:
        for index,(row,target) in enumerate(zip(rows,TARGETS)):
            tick=target[3]
            context={'save_path':text(row.field('save')),'save_sha256':text(row.field('sha256')),
                     'manifest_path':str(arm/'saves/manifest.json'),'manifest_sha256':file_hash(arm/'saves/manifest.json'),
                     'manifest_row':loader_context(row.value)}
            checkpoints.extend((CheckpointPlan('bootstrap',index,1,0),CheckpointPlan('loaded_initial',index,2,tick,json.dumps(context))))
            for zoom in (0.5,0.25):
                checkpoints.append(CheckpointPlan('zoom_complete',index,2,tick,json.dumps({'zoom':zoom,'saved_tick':float(tick),'scene_id':target[0]})))
            checkpoints.append(CheckpointPlan('loaded_terminal',index,2,tick,json.dumps(loader_context(row.value)),True))
            instances.append((InstancePlan('bootstrap',20261010,0,0),InstancePlan('load',0,tick,tick)))
    producer='visibility.generator' if generator else 'visibility.loader'
    return UIPlan(producer,producer+':'+str(spec.output_root),spec,'res://'+(GENERATOR if generator else LOADER),
                  HELPER,tuple(checkpoints),tuple(instances))


@boundary
def commands(raw: Node, generator: bool, prefixes: tuple[Rows,...] = ()) -> tuple[Rows,...]:
    result: list[Rows]=[]
    previous: Rows=()
    for index,checkpoint in enumerate(raw.field('checkpoints').array()):
        node=checkpoint.field('commands')
        keys(node,{'total','ticks','kinds','accepted'})
        ticks=tuple(integer(v) for v in node.field('ticks').array())
        kinds=tuple(text(v) for v in node.field('kinds').array())
        accepted=tuple(integer(v,1) for v in node.field('accepted').array())
        require(len(ticks)==len(kinds)==len(accepted)==integer(node.field('total')),'/commands','truncated transcript')
        rows=tuple(zip(ticks,kinds,accepted))
        tick=integer(checkpoint.field('clock').field('tick'))
        if (generator and index<2) or (not generator and index%5==0):
            _=native_commands(node,0,True)
        elif generator:
            require(rows[:2]==((0,'SetSteward',1),)*2 and all(k=='Steward' for _,k,_ in rows[2:]) and
                    ticks==tuple(sorted(ticks)) and all(t<=tick for t in ticks),'/commands','unexpected generator command')
            require(rows[:len(previous)]==previous,'/commands','generator prefix changed')
        else:
            require(len(prefixes)==4 and rows==prefixes[index//5],'/commands','loader changed saved command transcript')
        previous=rows
        result.append(rows)
    return tuple(result)


def inspect(spec: ProcessSpec, result: ProcessResult, required: UIPlan) -> tuple[UIEvidence,tuple[UIInstance,...],Node]:
    path=spec.output_root/f'{required.producer}-{result.pid}.json'
    data=path.read_bytes()
    evidence=UIEvidence(path,hashlib.sha256(data).hexdigest(),required,result,file_hash(result.receipt))
    valid=validate_ui(path,evidence.sidecar_sha256,required,result,evidence.process_sha256)
    return evidence,tuple(i for node in valid.instances for i in node),document(data.decode('utf-8'))


@boundary
def domain(arm: Path, root: Path, generator: bool) -> None:
    folder=arm/('generator-process' if generator else 'loader-process')
    stdout=(folder/'stdout.log').read_bytes().decode('utf-8')
    stderr=(folder/'stderr.log').read_bytes().decode('utf-8')
    require(not any(word in stdout+stderr for word in ('SCRIPT ERROR','Parse Error','ERROR:')),'/oracle','native runtime error')
    prefix='MUX2_VISIBILITY_SAVES_DONE' if generator else 'MUX2_VISIBILITY_DONE'
    lines=[line for line in stdout.splitlines() if line.startswith(prefix)]
    require(len(lines)==1,'/oracle','missing/duplicate completion')
    if generator:
        require(lines[0]=='MUX2_VISIBILITY_SAVES_DONE scenes=4','/oracle','incomplete saves')
    else:
        row=document(lines[0].removeprefix(prefix+' '))
        row.field('scenes').equals(8);row.field('failures').equals(0);row.field('pass').equals(True)
        validate_visibility(root,arm)


@dataclass(frozen=True)
class ArmEvidence:
    observations: tuple[Observation,...]
    native: tuple[UIInstance,...]
    generator: UIEvidence
    loader: UIEvidence
    chains: tuple[str,...]


def run_arm(product: ProbeProduct, arm: Path, frozen: dict[Path,dict[str,str]]) -> ArmEvidence:
    arm.mkdir()
    generator=process_spec(product,arm,True)
    result=execute(generator)
    freeze(generator.output_root,frozen)
    freeze(arm/'saves',frozen)
    rows=manifest(arm,product)
    source,g_native,raw=inspect(generator,result,plan(generator,rows,True))
    logs=commands(raw,True)
    domain(arm,product.input_root,True)
    external=tuple((arm/'saves'/(name+'.wsave'),text(row.field('sha256'))) for (name,_,_,_),row in zip(TARGETS,rows))
    external+=((arm/'saves/manifest.json',file_hash(arm/'saves/manifest.json')),)
    loader=process_spec(product,arm,False,external)
    result=execute(loader)
    freeze(loader.output_root,frozen)
    freeze(arm/'run',frozen)
    destination,l_native,raw=inspect(loader,result,plan(loader,rows,False))
    _=commands(raw,False,tuple(logs[2+i*2] for i in range(4)))
    domain(arm,product.input_root,False)
    joined=tuple(join_visibility_history(source,destination,SaveLink(path,digest,arm/'saves/manifest.json',external[-1][1],
                 TARGETS[i][0],2+i*2,3+i*2,1+i*5,BRIDGE)) for i,(path,digest) in enumerate(external[:-1]))
    fresh=tuple(require_fresh_history(i) for i in (*g_native,*l_native) if i.reason!='load')
    require(all(i.founders==10 for i in fresh),'/census','unexpected colony founder census')
    freeze(arm,frozen)
    return ArmEvidence(fresh+tuple(j.observation for j in joined),g_native+l_native,source,destination,tuple(j.evidence_json for j in joined))


def freeze(folder: Path, frozen: dict[Path,dict[str,str]]) -> None:
    frozen[folder]=inventory(folder,[p.name for p in folder.iterdir()])


def run_pair(products: Products, output: Path) -> ComponentResult:
    require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh canonical output required')
    roots=(products.base.input_root,products.candidate.input_root)
    require(all(r==r.resolve(strict=True) and not output.is_relative_to(r) and not r.is_relative_to(output) for r in roots),'/output','product/output overlap')
    require(not roots[0].is_relative_to(roots[1]) and not roots[1].is_relative_to(roots[0]),'/products','independent product roots required')
    require(products.base.executable_sha256==products.candidate.executable_sha256,'/products','different Godot versions')
    for product in (products.base,products.candidate):check_product(product)
    output.mkdir()
    frozen: dict[Path,dict[str,str]]={}
    evidence: list[ArmEvidence]=[]
    errors: list[str]=[]
    for name,product in (('base',products.base),('candidate',products.candidate)):
        try:
            evidence.append(run_arm(product,output/name,frozen))
        except (GateContractError,OSError,UnicodeError) as error:errors.append(name+': '+str(error))
    try:
        require(len(evidence)==2,'/pair','missing completed arm')
        require(len(evidence[0].observations)==len(evidence[1].observations)==10,'/pair','missing native coverage')
        for left,right in zip(evidence[0].observations,evidence[1].observations):_ = ordinary_pair(left,right)
    except GateContractError as error:errors.append(str(error))
    for folder,hashes in frozen.items():
        try:
            require(inventory(folder,[p.name for p in folder.iterdir()])==hashes,str(folder),'artifacts changed during pair')
        except (GateContractError,OSError) as error:errors.append(str(error))
    for product in (products.base,products.candidate):
        try:check_product(product)
        except (GateContractError,OSError) as error:errors.append(str(error))
    status='FAIL' if errors else 'PASS'
    receipt=output/'visibility.json'
    atomic_json(receipt,{'schema_version':1,'status':status,'errors':[e for e in errors],
        'artifacts':{str(folder):{name:digest for name,digest in hashes.items()} for folder,hashes in frozen.items()},
        'history_chains':[[chain for chain in arm.chains] for arm in evidence],
        'build_provenance':'caller_owned_not_certified_here','full_gate_claim':False})
    return ComponentResult(receipt,status)
