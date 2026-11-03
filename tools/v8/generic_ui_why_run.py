"""Paired original Why input fixtures with an explicit direct-event mechanism.

No exact-horizon companion is supplied: unequal actual horizons or intervention
arguments remain unresolved. Caller owns qualified products and the native lane.
"""
import hashlib
import json
from dataclasses import dataclass
from itertools import pairwise
from pathlib import Path
from typing import Final

from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, boundary, document, integer, require
from generic_observations import keys, ordinary_pair
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_sources import inventory
from generic_ui_companion import native_commands
from generic_ui_history import UIEvidence
from generic_ui_observations import (
    UIInstance,
    parse_native,
    require_fresh_history,
    unsigned_string,
)
from generic_ui_visibility_run import freeze
from mux2_gate_inputs import BRIDGE, Node
from mux2_gate_visual import validate_why_checks

from generic_ui_receipts import CheckpointPlan, InstancePlan, UIPlan, validate_ui

PARENT: Final = 'tools/v8/mux2_why_input_test.gd'
LOCALE: Final = 'tools/v8/mux21_why_locale_test.gd'
HELPER: Final = 'res://tools/v8/gate_ui_observer.gd'
VARIANTS: Final = ('base','en','ko')
PINS: Final[dict[str,str]] = {'tools/v8/mux2_why_input_test.gd': '74ba3b67886de41174aba198e1624f5b77974d34c2a71bb9beede0c19f076e27', 'tools/v8/mux21_why_locale_test.gd': 'ed533bfea8a73778d68a8e46f5340a5d7f1194ca2e100846be8e59bd9f725f7a', 'tools/v8/gate_ui_observer.gd': 'b0f2ef8c577d777aff9442a55bf3f8ad29c5a5898011d0b331a24db4c9dba967', 'scripts/core/main_bootstrap.gd': '1801e3bb6cac5d4e6085ae804648d86bca17d073f2d09d17d882dec719d9e1f5', 'scripts/ui/mux/settings_store.gd': '70ced6c709f8d63f60a3d3e04127db6432eaf86972a69a3a4a9f618e642907ec', 'rust/crates/sim-bridge/src/ffi/engine_build.rs': '7b90e7dd8d2ae713bf88721fabffc478df1cfe46adbed0ac83d667da5129b892', 'rust/crates/sim-systems/src/worldgen/mod.rs': '11525a81664d3206c72754f3772d1ba4e61cd3f9dbe2ab611be4e4124378b601'}


def check_product(product: ProbeProduct) -> None:
    inputs=dict(product.input_files)
    required=set(PINS)|{BRIDGE,'project.godot','sim_bridge.gdextension','scenes/main.tscn'}
    require(len(inputs)==len(product.input_files) and required<=inputs.keys(),'/inputs','incomplete Why inventory')
    require(all(inputs[name]==digest for name,digest in PINS.items()),'/inputs','unreviewed Why semantics')
    require(inventory(product.input_root,list(inputs))==inputs and file_hash(product.executable)==product.executable_sha256,
            '/inputs','product changed')


def process_spec(product: ProbeProduct, variant: str, output: Path) -> ProcessSpec:
    require(variant in VARIANTS,'/variant','unsupported Why invocation')
    check_product(product)
    argv=(str(product.executable),'--path',str(product.input_root),'--resolution','1920x1080',
          '--script','res://'+(PARENT if variant=='base' else LOCALE))
    if variant!='base':argv+=('--','--locale='+variant)
    environment=(('HOME',str(Path.home())),('WORLDSIM_GATE_OBSERVER','1'),
                 ('WORLDSIM_GATE_INVOCATION_ID','why.'+variant+':'+str(output)),('WORLDSIM_GATE_UI_OUT',str(output)),
                 ('WORLDSIM_TEST_SETTINGS_DIR',str(output/'settings')),('WORLDSIM_GIT_SHA',product.source_sha))
    return ProcessSpec(argv,product.input_root,product.executable,product.executable_sha256,product.input_root,
                       product.input_files,output,environment,fresh_directories=('settings',),timeout_seconds=180)


@boundary
def mechanism(raw: Node) -> tuple[int,tuple[int,int,int]]:
    checkpoints=raw.field('checkpoints').array()
    require(len(checkpoints)==6,'/checkpoints','all six Why checkpoints required')
    before=checkpoints[2].field('context')
    keys(before,{'method','args'});before.field('method').equals('on_building_placed')
    values=before.field('args').array()
    require(len(values)==3,'/intervention','three exact event arguments required')
    args=tuple(integer(v,2**31-1) for v in values)
    require(args[2]==1,'/intervention','unexpected building fixture kind')
    queued=checkpoints[3].field('context')
    keys(queued,{'method','args','accepted'})
    queued.field('method').equals('on_building_placed');queued.field('args').equals(before.field('args').value)
    queued.field('accepted').equals(True)
    return integer(checkpoints[1].field('clock').field('tick')),(args[0],args[1],args[2])


def lifecycle_plan(horizon: int, args: tuple[int,int,int]) -> tuple[tuple[CheckpointPlan, ...], tuple[tuple[InstancePlan, ...], ...]]:
    """Existing exact Why intervention lifecycle independent of native transport."""
    require(type(horizon) is int and horizon>=0,'/plan','invalid Why plan')
    context={'method':'on_building_placed','args':list(args)}
    checkpoints=(CheckpointPlan('bootstrap',0,1,0),CheckpointPlan('ordinary_prefix',0,1,horizon),
                 CheckpointPlan('before_intervention',0,1,horizon,json.dumps(context)),
                 CheckpointPlan('intervention_queued',0,1,horizon,json.dumps({**context,'accepted':True})),
                 CheckpointPlan('after_intervention_tick',0,1,horizon+1),
                 CheckpointPlan('terminal',0,1,horizon+1,releases_node=True))
    return checkpoints, ((InstancePlan('bootstrap',20261010,0,horizon+1),),)


def plan(spec: ProcessSpec, variant: str, horizon: int, args: tuple[int,int,int]) -> UIPlan:
    require(variant in VARIANTS,'/plan','invalid Why variant')
    checkpoints, instances = lifecycle_plan(horizon,args)
    return UIPlan('why','why.'+variant+':'+str(spec.output_root),spec,'res://'+PARENT,HELPER,
                  checkpoints,instances)


@boundary
def commands(raw: Node) -> None:
    checkpoints=raw.field('checkpoints').array()
    transcripts=tuple(native_commands(cp.field('commands'),integer(cp.field('clock').field('tick')),index==0)
                      for index,cp in enumerate(checkpoints))
    require(len(transcripts)==6,'/commands','incomplete command coverage')
    for before,after in pairwise(transcripts):
        require(after[:len(before)]==before,'/commands','command prefix changed')
    require(transcripts[1]==transcripts[2]==transcripts[3] and transcripts[4]==transcripts[5],
            '/commands','UI issued commands while paused')


@boundary
def domain(spec: ProcessSpec, variant: str) -> None:
    stdout=(spec.output_root/'stdout.log').read_bytes().decode('utf-8')
    stderr=(spec.output_root/'stderr.log').read_bytes().decode('utf-8')
    require(not any(word in stdout+stderr for word in ('SCRIPT ERROR','Parse Error','ERROR:')),'/oracle','native runtime error')
    validate_why_checks(stdout,53 if variant=='base' else 64)
    outputs={line.removeprefix('MUX2_WHY_OUTPUT ') for line in stdout.splitlines() if line.startswith('MUX2_WHY_OUTPUT ')}
    require(outputs=={str(spec.output_root/'why')},'/images','foreign or absent capture directory')
    if variant!='base':
        require([line for line in stdout.splitlines() if line.startswith('MUX21_WHY_LOCALE ')]==['MUX21_WHY_LOCALE '+variant],
                '/locale','wrong or missing locale')
    images=[spec.output_root/'why'/('why-'+scale+'.png') for scale in (('1.0','1.25') if variant=='base' else ('1.0','1.25','1.5'))]
    images.extend(spec.output_root/'placement'/('why-placement-'+scale+'.png') for scale in ('1.0','1.25'))
    for image in images:
        _=file_hash(image)
        require(image.read_bytes().startswith(b'\x89PNG\r\n\x1a\n'),'/images','missing PNG capture')


@dataclass(frozen=True)
class WhyEvidence:
    evidence: UIEvidence
    prefix: UIInstance
    terminal: UIInstance
    args: tuple[int,int,int]


@boundary
def inspect(spec: ProcessSpec, result: ProcessResult, variant: str) -> WhyEvidence:
    path=spec.output_root/f'why-{result.pid}.json'
    data=path.read_bytes();digest=hashlib.sha256(data).hexdigest();raw=document(data.decode('utf-8'))
    horizon,args=mechanism(raw)
    required=plan(spec,variant,horizon,args)
    proof=UIEvidence(path,digest,required,result,file_hash(result.receipt))
    valid=validate_ui(path,digest,required,result,proof.process_sha256)
    commands(raw);domain(spec,variant)
    cp=raw.field('checkpoints').array()[1]
    prefix=parse_native(json.dumps(cp.field('observation').value),unsigned_string(cp.field('node_instance_id')))[0]
    terminal=valid.instances[0][0]
    for instance in (prefix,terminal):
        state=require_fresh_history(instance)
        require(state.founders==10,'/census','unexpected founder census')
    return WhyEvidence(proof,prefix,terminal,args)


def compare(base: WhyEvidence, candidate: WhyEvidence) -> None:
    require(base.prefix.state.terminal_tick==candidate.prefix.state.terminal_tick and base.args==candidate.args,
            '/pair','UNRESOLVED: unmatched Why horizon or direct-event input; no qualified companion')
    for left,right in ((base.prefix,candidate.prefix),(base.terminal,candidate.terminal)):
        _=ordinary_pair(left.state,right.state)


def run_pair(products: Products, variant: str, output: Path) -> ComponentResult:
    require(variant in VARIANTS,'/variant','unsupported Why invocation')
    require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh canonical output required')
    roots=(products.base.input_root,products.candidate.input_root)
    require(all(r==r.resolve(strict=True) and not output.is_relative_to(r) and not r.is_relative_to(output) for r in roots),'/output','product/output overlap')
    require(not roots[0].is_relative_to(roots[1]) and not roots[1].is_relative_to(roots[0]),'/products','independent product roots required')
    require(products.base.executable_sha256==products.candidate.executable_sha256,'/products','different Godot versions')
    specs=tuple(process_spec(product,variant,output/name) for name,product in (('base',products.base),('candidate',products.candidate)))
    output.mkdir()
    frozen: dict[Path,dict[str,str]]={}
    evidence: list[WhyEvidence]=[]
    errors: list[str]=[]
    for spec in specs:
        try:
            result=execute(spec)
            freeze(spec.output_root,frozen)
            evidence.append(inspect(spec,result,variant))
        except (GateContractError,OSError,UnicodeError) as error:errors.append(spec.output_root.name+': '+str(error))
    try:
        require(len(evidence)==2,'/pair','missing completed arm')
        compare(evidence[0],evidence[1])
    except GateContractError as error:errors.append(str(error))
    for folder,hashes in frozen.items():
        try:
            require(inventory(folder,[p.name for p in folder.iterdir()])==hashes,str(folder),'artifacts changed during pair')
        except (GateContractError,OSError) as error:errors.append(str(error))
    for product in (products.base,products.candidate):
        try:check_product(product)
        except (GateContractError,OSError) as error:errors.append(str(error))
    status='FAIL' if errors else 'PASS'
    receipt=output/'why.json'
    atomic_json(receipt,{'schema_version':1,'variant':variant,'status':status,'errors':[e for e in errors],
        'classification':'ordinary_boot_prefix_then_direct_building_placed_event_mechanism',
        'mechanisms':[{'tick':e.prefix.state.terminal_tick,'method':'on_building_placed','args':list(e.args),'accepted':True,
                       'sidecar':str(e.evidence.sidecar),'sha256':e.evidence.sidecar_sha256} for e in evidence],
        'artifacts':{str(folder):{name:digest for name,digest in hashes.items()} for folder,hashes in frozen.items()},
        'build_provenance':'caller_owned_not_certified_here','full_gate_claim':False})
    return ComponentResult(receipt,status)
