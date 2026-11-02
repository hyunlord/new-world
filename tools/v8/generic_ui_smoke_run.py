"""Original 900-frame smoke oracles and authenticated exact-horizon safety.

Caller supplies independently qualified products and owns the native lane.
Companions qualify native history only, never the original label oracle.
"""
import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, boundary, document, integer, require
from generic_observations import Observation, ordinary_pair
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_sources import inventory
from generic_ui_companion import native_commands, validate_smoke_companion
from generic_ui_history import UIEvidence
from generic_ui_observations import require_fresh_history
from generic_ui_receipts import CheckpointPlan, InstancePlan, UIPlan, validate_ui
from generic_ui_visibility_run import freeze
from mux2_gate_inputs import BRIDGE, Node

HELPER: Final = 'res://tools/v8/gate_ui_observer.gd'
SCRIPTS: Final = {'smoke.boot':'res://scripts/core/main_bootstrap.gd',
                 'smoke.labels':'res://tools/v8/smoke.gd',
                 'smoke.companion':'res://tools/v8/gate_ui_companion.gd'}
PINS: Final[dict[str,str]] = {'tools/v8/smoke.gd': 'a35860cd99ae462d7c9cda66a007fc6ac84c383742ddbd8445acc8c4237b3f56', 'tools/v8/smoke.sh': '2302f20b4ec3173fe8b41512b5d25209ef44f0d96b1af9333fd5a36c29c25c0f', 'tools/v8/smoke_base_counts.txt': 'a859aa81ac57e51d6761fd4dcfdc0487770505b015b7e4f4496ff52bc297021f', 'tools/v8/gate_ui_observer.gd': 'b0f2ef8c577d777aff9442a55bf3f8ad29c5a5898011d0b331a24db4c9dba967', 'tools/v8/gate_ui_companion.gd': '3580932ddee258113b1d02d563effbb508ba09f946e948b2a0dc8c9e4ac0620c', 'scripts/core/main_bootstrap.gd': '1801e3bb6cac5d4e6085ae804648d86bca17d073f2d09d17d882dec719d9e1f5', 'scripts/ui/mux/settings_store.gd': '70ced6c709f8d63f60a3d3e04127db6432eaf86972a69a3a4a9f618e642907ec', 'rust/crates/sim-bridge/src/ffi/engine_build.rs': '7b90e7dd8d2ae713bf88721fabffc478df1cfe46adbed0ac83d667da5129b892', 'rust/crates/sim-systems/src/worldgen/mod.rs': '11525a81664d3206c72754f3772d1ba4e61cd3f9dbe2ab611be4e4124378b601'}


def check_product(product: ProbeProduct) -> None:
    inputs=dict(product.input_files)
    require(len(inputs)==len(product.input_files) and (set(PINS)|{BRIDGE,"project.godot","sim_bridge.gdextension","scenes/main.tscn"})<=inputs.keys(),'/inputs','incomplete smoke inventory')
    require(all(inputs[name]==digest for name,digest in PINS.items()),'/inputs','unreviewed smoke semantics')
    require(inventory(product.input_root,list(inputs))==inputs and file_hash(product.executable)==product.executable_sha256,
            '/inputs','product changed')


def process_spec(product: ProbeProduct, producer: str, output: Path, original: UIEvidence | None = None) -> ProcessSpec:
    check_product(product)
    require(producer in SCRIPTS,'/producer','unsupported smoke producer')
    argv=(str(product.executable),'--headless','--path',str(product.input_root))
    if producer=='smoke.boot':argv+=('--quit-after','900')
    elif producer=='smoke.labels':argv+=('--script',SCRIPTS[producer],'--','--frames','900')
    else:
        require(original is not None,'/companion','authenticated original required')
        assert original is not None
        target=original.plan.instances[0][0].terminal_tick
        require(0<=target<=200000,'/companion','unsupported exact horizon')
        argv+=('--script',SCRIPTS[producer],'--','--ticks',str(target),'--original-producer',original.plan.producer,
               '--original-sidecar-sha256',original.sidecar_sha256)
    env=(('HOME',str(Path.home())),('WORLDSIM_GATE_OBSERVER','1'),
         ('WORLDSIM_GATE_INVOCATION_ID',producer+':'+str(output)),('WORLDSIM_GATE_UI_OUT',str(output)),
         ('WORLDSIM_TEST_SETTINGS_DIR',str(output/'settings')),('WORLDSIM_GIT_SHA',product.source_sha))
    if producer=='smoke.boot':env+=(('WORLDSIM_GATE_UI_BOOT','1'),)
    return ProcessSpec(argv,product.input_root,product.executable,product.executable_sha256,product.input_root,
                       product.input_files,output,env,fresh_directories=('settings',),timeout_seconds=600)


@boundary
def plan(spec: ProcessSpec, producer: str, raw: Node, original: UIEvidence | None = None) -> UIPlan:
    labels=('bootstrap','native_tree_exit') if producer=='smoke.boot' else (
        ('bootstrap','initial_frames_complete','terminal') if producer=='smoke.labels' else ('bootstrap','companion_terminal'))
    rows=raw.field('checkpoints').array()
    require(len(rows)==len(labels),'/checkpoints','incomplete smoke lifecycle')
    ticks=tuple(integer(row.field('clock').field('tick')) for row in rows)
    require(ticks[0]==0 and tuple(sorted(ticks))==ticks and ticks[-1]<=200000,'/ticks','invalid actual smoke horizon')
    checkpoints: list[CheckpointPlan]=[]
    for index,label in enumerate(labels):
        context='{}'
        if label=='initial_frames_complete':context='{"requested_frames":900}'
        if label=='companion_terminal':
            require(original is not None,'/companion','missing original binding')
            assert original is not None
            target=original.plan.instances[0][0].terminal_tick
            require(ticks[-1]==target,'/companion','wrong exact horizon')
            context=json.dumps({'requested_tick':target,'original_producer':original.plan.producer,
                                'original_sidecar_sha256':original.sidecar_sha256})
        checkpoints.append(CheckpointPlan(label,0,1,ticks[index],context,index==len(labels)-1))
    return UIPlan(producer,producer+':'+str(spec.output_root),spec,SCRIPTS[producer],HELPER,tuple(checkpoints),
                  ((InstancePlan('bootstrap',20261010,0,ticks[-1]),),),
                  completion='native_tree_exit' if producer=='smoke.boot' else 'producer_finish')


@dataclass(frozen=True)
class SmokeEvidence:
    evidence: UIEvidence
    state: Observation


@boundary
def inspect(spec: ProcessSpec, result: ProcessResult, producer: str, original: UIEvidence | None = None) -> SmokeEvidence:
    path=spec.output_root/f'{producer}-{result.pid}.json'
    data=path.read_bytes();digest=hashlib.sha256(data).hexdigest();raw=document(data.decode('utf-8'))
    required=plan(spec,producer,raw,original)
    proof=UIEvidence(path,digest,required,result,file_hash(result.receipt))
    valid=validate_ui(path,digest,required,result,proof.process_sha256)
    require(len(valid.instances)==1 and len(valid.instances[0])==1 and not valid.loaded_unqualified,
            '/instances','smoke requires one fresh native instance')
    instance=valid.instances[0][0];state=require_fresh_history(instance)
    require(state.founders==10 and not instance.closed,'/census','unexpected smoke lifecycle')
    previous: tuple[tuple[int,str,int],...]=()
    for index,cp in enumerate(raw.field('checkpoints').array()):
        commands=native_commands(cp.field('commands'),integer(cp.field('clock').field('tick')),index==0)
        require(commands[:len(previous)]==previous,'/commands','command prefix changed')
        previous=commands
    return SmokeEvidence(proof,state)


@boundary
def oracle(boot: SmokeEvidence, labels: SmokeEvidence) -> dict[str,int]:
    logs: list[str]=[]
    for proof in (boot,labels):
        folder=proof.evidence.plan.process.output_root
        logs.append((folder/'stdout.log').read_bytes().decode('utf-8')+'\n'+(folder/'stderr.log').read_bytes().decode('utf-8'))
    eligible=[line for log in logs for line in log.splitlines() if not re.search('alsa|audio',line,re.IGNORECASE)]
    counts={name:sum(pattern in line for line in eligible) for name,pattern in
            (('script_errors','SCRIPT ERROR'),('errors','ERROR:'),('locale_warnings','Locale:'))}
    done=[line for line in logs[1].splitlines() if line.startswith('SMOKE_DONE')]
    require(len(done)==1,'/oracle','exactly one original SMOKE_DONE required')
    match=re.fullmatch(r'SMOKE_DONE frames=900 labels=([0-9]+) raw_keys=([0-9]+)',done[0])
    require(match is not None,'/oracle','original 900-frame scan did not complete')
    assert match is not None
    counts['raw_keys']=int(match[2])
    baseline=boot.evidence.plan.process.input_root/'tools/v8/smoke_base_counts.txt'
    expected: dict[str,int]={}
    for line in baseline.read_bytes().decode('utf-8').splitlines():
        if not line or line.startswith('#'):continue
        key,value=line.split();require(key not in expected and value.isdecimal(),'/oracle','invalid historical counts')
        expected[key]=int(value)
    require(expected.keys()==counts.keys(),'/oracle','historical count schema differs')
    require(all(counts[key]<=value for key,value in expected.items()),'/oracle','original smoke count exceeds historical baseline')
    return counts


def run_pair(products: Products, output: Path) -> ComponentResult:
    require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh canonical output required')
    roots=(products.base.input_root,products.candidate.input_root)
    require(all(r==r.resolve(strict=True) and not output.is_relative_to(r) and not r.is_relative_to(output) for r in roots),'/output','input/output overlap')
    require(not roots[0].is_relative_to(roots[1]) and not roots[1].is_relative_to(roots[0]),'/products','independent roots required')
    require(products.base.executable_sha256==products.candidate.executable_sha256,'/products','different Godot binaries')
    for product in (products.base,products.candidate):check_product(product)
    output.mkdir()
    frozen: dict[Path,dict[str,str]]={};errors: list[str]=[]
    originals: dict[tuple[str,str],SmokeEvidence]={};counts: dict[str,dict[str,int]]={}
    links: list[dict[str,str|int]]=[]
    def run(product: ProbeProduct, producer: str, folder: Path, original: UIEvidence | None = None) -> SmokeEvidence:
        spec=process_spec(product,producer,folder,original);result=execute(spec)
        freeze(folder,frozen)
        return inspect(spec,result,producer,original)
    for arm,product in (('base',products.base),('candidate',products.candidate)):
        for kind in ('boot','labels'):
            try:
                actual=run(product,'smoke.'+kind,output/(arm+'-'+kind));originals[arm,kind]=actual
                companion=run(product,'smoke.companion',output/(arm+'-'+kind+'-own'),actual.evidence)
                _=validate_smoke_companion(actual.evidence,companion.evidence)
            except (GateContractError,OSError,UnicodeError) as error:errors.append(arm+' '+kind+': '+str(error))
        try:counts[arm]=oracle(originals[arm,'boot'],originals[arm,'labels'])
        except (GateContractError,OSError,UnicodeError,KeyError,ValueError) as error:errors.append(arm+' oracle: '+str(error))
    for kind in ('boot','labels'):
        try:
            base=originals['base',kind];candidate=originals['candidate',kind]
            if base.state.terminal_tick==candidate.state.terminal_tick:_=ordinary_pair(base.state,candidate.state)
            else:
                # Each arm has already qualified its exact-horizon adapter against its own original.
                for origin,product,arm in ((base,products.candidate,'candidate'),(candidate,products.base,'base')):
                    other=run(product,'smoke.companion',output/(arm+'-'+kind+'-matched'),origin.evidence)
                    left,right=(origin.state,other.state) if arm=='candidate' else (other.state,origin.state)
                    _=ordinary_pair(left,right)
                    links.append({'original':str(origin.evidence.sidecar),'original_sha256':origin.evidence.sidecar_sha256,
                                  'companion':str(other.evidence.sidecar),'companion_sha256':other.evidence.sidecar_sha256,
                                  'tick':origin.state.terminal_tick})
        except (GateContractError,OSError,UnicodeError,KeyError) as error:errors.append(kind+' safety: '+str(error))
    for folder,hashes in frozen.items():
        try:require(inventory(folder,[p.name for p in folder.iterdir()])==hashes,str(folder),'artifacts changed during pair')
        except (GateContractError,OSError) as error:errors.append(str(error))
    for product in (products.base,products.candidate):
        try:check_product(product)
        except (GateContractError,OSError) as error:errors.append(str(error))
    status='FAIL' if errors else 'PASS';receipt=output/'smoke.json'
    atomic_json(receipt,{'schema_version':1,'status':status,'errors':[e for e in errors],'counts':{arm:{k:v for k,v in values.items()} for arm,values in counts.items()},'matched_companions':[{k:v for k,v in link.items()} for link in links],
        'originals':[{'arm':arm,'kind':kind,'sidecar':str(e.evidence.sidecar),'sha256':e.evidence.sidecar_sha256,
                      'actual_tick':e.state.terminal_tick} for (arm,kind),e in originals.items()],
        'artifacts':{str(folder):{k:v for k,v in hashes.items()} for folder,hashes in frozen.items()},
        'build_provenance':'caller_owned_not_certified_here','full_gate_claim':False})
    return ComponentResult(receipt,status)
