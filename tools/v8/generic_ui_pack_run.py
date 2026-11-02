"""Authenticate the original selected seven-family pack UI, not its Rust children.

One continuous ordinary native world per arm. Catalogue-negative DTOs remain
presentation controls; no partial-world or rewritten-clock safety exemption.
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
from generic_observations import ordinary_pair
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
from generic_ui_receipts import CheckpointPlan, InstancePlan, UIPlan, validate_ui
from generic_ui_visibility_run import freeze
from mux2_gate_inputs import BRIDGE, Node

SCRIPT: Final = 'tools/v8/data_only_ext_test.gd'
HELPER: Final = 'res://tools/v8/gate_ui_observer.gd'
PACK: Final = 'rust/crates/sim-test/data/ext_pack/pack.ron'
# The exact manifest references these eleven immutable selected fixture sources.
PACK_FILES: Final = ('pack.ron','items.ron','recipes.ron','blueprints.ron','knowledge.ron','pressures.ron',
                    'animals.ron','alerts.json','lessons.json','locale_en.json','locale_ko.json','bindings.json')
PINS: Final[dict[str,str]] = {'tools/v8/data_only_ext_test.gd': '6911b4488f26d064f217b7d39033e3489bf2f122b12359b9f17286c3ebae6b0f', 'tools/v8/gate_ui_observer.gd': 'b0f2ef8c577d777aff9442a55bf3f8ad29c5a5898011d0b331a24db4c9dba967', 'scripts/core/main_bootstrap.gd': '1801e3bb6cac5d4e6085ae804648d86bca17d073f2d09d17d882dec719d9e1f5', 'scripts/ui/mux/settings_store.gd': '70ced6c709f8d63f60a3d3e04127db6432eaf86972a69a3a4a9f618e642907ec', 'rust/crates/sim-bridge/src/ffi/engine_build.rs': '7b90e7dd8d2ae713bf88721fabffc478df1cfe46adbed0ac83d667da5129b892', 'rust/crates/sim-systems/src/worldgen/mod.rs': '11525a81664d3206c72754f3772d1ba4e61cd3f9dbe2ab611be4e4124378b601', 'rust/crates/sim-test/data/ext_pack/alerts.json': '30c4dc958ddcaf7bc632ea96b18e4202561ca1f8756de99c681193fb5af685ca', 'rust/crates/sim-test/data/ext_pack/animals.ron': 'be03cb0c8451455b3089891d224144bc66ca64b370d69da9278a1c2fd9559aa8', 'rust/crates/sim-test/data/ext_pack/bindings.json': 'c04221eea234b51593d03faa4e6b944b9e2b4fd93698b841e7b8da07a220e80f', 'rust/crates/sim-test/data/ext_pack/blueprints.ron': '2a7fe272276f62b72c0a1f2ae8ac2cc6b3dcabd96228e14ed1b67146b56dfc32', 'rust/crates/sim-test/data/ext_pack/items.ron': '8c9756806f5ab8431151b880d426f47587359103d5e15df777e6d1ee8d291073', 'rust/crates/sim-test/data/ext_pack/knowledge.ron': '7fd3f51f40fe504abede14a39f74629ec759d23d744a4523f3efdae1ee31b342', 'rust/crates/sim-test/data/ext_pack/lessons.json': '427f7f73fa9bf647e5aa87779f4477c3414cbbd45f0c16e25f264b60d661c90f', 'rust/crates/sim-test/data/ext_pack/locale_en.json': 'adaf1ae4ced47d3ff46b6b99e3e76a7c92088c4fd930f302bcdc9362b027d404', 'rust/crates/sim-test/data/ext_pack/locale_ko.json': '7b12b155c5eb7f83f3505e1e0dc90e00cc138e2c3db3c7d947035bdafe89a410', 'rust/crates/sim-test/data/ext_pack/pack.ron': '2e6f9269d0a72360e2615928cbb6ea29ba3943e7a3b2992c9f5a2c1c0143841f', 'rust/crates/sim-test/data/ext_pack/pressures.ron': '606a5fa90dfd011f7d4e5b09dc4ee7be382933bf0cdd7afdfbef1e49b08c9778', 'rust/crates/sim-test/data/ext_pack/recipes.ron': '5f15efd2dc3de49d3a4ffb9bcf076de3d4c490d1a7d41da0a06ccb028025112f'}


def check_product(product: ProbeProduct) -> None:
    inputs=dict(product.input_files)
    required=set(PINS)|{BRIDGE,'project.godot','sim_bridge.gdextension','scenes/main.tscn'}
    require(len(inputs)==len(product.input_files) and required<=inputs.keys(),'/inputs','incomplete selected-pack inputs')
    require(all(inputs[name]==digest for name,digest in PINS.items()),'/inputs','changed selected pack or unreviewed UI semantics')
    require(inventory(product.input_root,list(inputs))==inputs and file_hash(product.executable)==product.executable_sha256,
            '/inputs','product changed')


def process_spec(product: ProbeProduct, output: Path) -> ProcessSpec:
    check_product(product)
    argv=(str(product.executable),'--headless','--path',str(product.input_root),'--script','res://'+SCRIPT,'--',str(output/'ui'))
    environment=(('HOME',str(Path.home())),('WORLDSIM_CONTENT_PACK',str(product.input_root/PACK)),
                 ('WORLDSIM_GATE_OBSERVER','1'),('WORLDSIM_GATE_INVOCATION_ID','pack.ui:'+str(output)),
                 ('WORLDSIM_GATE_UI_OUT',str(output)),('WORLDSIM_TEST_SETTINGS_DIR',str(output/'settings')),
                 ('WORLDSIM_GIT_SHA',product.source_sha))
    return ProcessSpec(argv,product.input_root,product.executable,product.executable_sha256,product.input_root,
                       product.input_files,output,environment,fresh_directories=('settings',),timeout_seconds=600)


def plan(spec: ProcessSpec, horizon: int) -> UIPlan:
    require(type(horizon) is int and horizon>=0,'/horizon','unknown native horizon')
    context={'manifest_path':str(spec.input_root/PACK),'manifest_sha256':dict(spec.inputs)[PACK],
             'fixture_classification':'ordinary_selected_pack','synthetic_DTO_checks':'presentation_validation_only'}
    checkpoints=(CheckpointPlan('bootstrap',0,1,0),CheckpointPlan('paused_selected_pack',0,1,horizon,json.dumps(context)),
                 CheckpointPlan('before_explicit_tick',0,1,horizon),CheckpointPlan('after_explicit_tick',0,1,horizon+1),
                 CheckpointPlan('terminal',0,1,horizon+1,releases_node=True))
    return UIPlan('pack.ui','pack.ui:'+str(spec.output_root),spec,'res://'+SCRIPT,HELPER,checkpoints,
                  ((InstancePlan('bootstrap',20261010,0,horizon+1),),))


@boundary
def commands(raw: Node) -> None:
    cps=raw.field('checkpoints').array()
    rows=tuple(native_commands(cp.field('commands'),integer(cp.field('clock').field('tick')),index==0) for index,cp in enumerate(cps))
    require(len(rows)==5,'/commands','all selected-pack checkpoints required')
    require(rows[1]==rows[2] and rows[3]==rows[4],'/commands','unexpected command during paused UI checks')
    for before,after in pairwise(rows):require(after[:len(before)]==before,'/commands','native command prefix changed')


@boundary
def domain(spec: ProcessSpec) -> None:
    stdout=(spec.output_root/'stdout.log').read_bytes().decode('utf-8')
    stderr=(spec.output_root/'stderr.log').read_bytes().decode('utf-8')
    require(not any(word in stdout+stderr for word in ('SCRIPT ERROR','Parse Error','ERROR:','DATA_ONLY_EXT_FAIL')),
            '/oracle','selected-pack runtime/assertion error')
    report=document((spec.output_root/'ui/data-only-ext-results.json').read_bytes().decode('utf-8'))
    report.field('failures').equals(0)
    report.field('selected_pack').equals(str(spec.input_root/PACK))
    report.field('rendered').equals(False)
    checks=report.field('checks').array()
    assertions=[row for row in checks if 'passed' in row.mapping()]
    require(len(assertions)>=20,'/oracle','original minimum assertion coverage missing')
    for row in assertions:row.field('passed').equals(True)
    require(len([row for row in checks if row.field('check').value=='rendered capture' and
                 row.field('status').value=='NOT_RUN_HEADLESS'])==1,'/oracle','headless capture status missing')


@dataclass(frozen=True)
class PackEvidence:
    evidence: UIEvidence
    prefix: UIInstance
    terminal: UIInstance


@boundary
def inspect(spec: ProcessSpec, result: ProcessResult) -> PackEvidence:
    path=spec.output_root/f'pack.ui-{result.pid}.json'
    data=path.read_bytes();raw=document(data.decode('utf-8'))
    cps=raw.field('checkpoints').array()
    require(len(cps)==5,'/checkpoints','all selected-pack native checkpoints required')
    required=plan(spec,integer(cps[1].field('clock').field('tick')))
    proof=UIEvidence(path,hashlib.sha256(data).hexdigest(),required,result,file_hash(result.receipt))
    actual=validate_ui(path,proof.sidecar_sha256,required,result,proof.process_sha256)
    commands(raw);domain(spec)
    cp=cps[1]
    prefix=parse_native(json.dumps(cp.field('observation').value),unsigned_string(cp.field('node_instance_id')))[0]
    terminal=actual.instances[0][0]
    for instance in (prefix,terminal):
        state=require_fresh_history(instance)
        require(state.founders==10,'/census','unexpected selected-pack initial census')
    return PackEvidence(proof,prefix,terminal)


def compare(base: PackEvidence, candidate: PackEvidence) -> None:
    require(base.prefix.state.terminal_tick==candidate.prefix.state.terminal_tick,'/pair',
            'UNRESOLVED: unequal selected-pack prefix horizons; no qualified selected-pack companion')
    for left,right in ((base.prefix,candidate.prefix),(base.terminal,candidate.terminal)):
        _=ordinary_pair(left.state,right.state)


def run_pair(products: Products, output: Path) -> ComponentResult:
    require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh canonical output required')
    roots=(products.base.input_root,products.candidate.input_root)
    require(all(r==r.resolve(strict=True) and not output.is_relative_to(r) and not r.is_relative_to(output) for r in roots),'/output','product/output overlap')
    require(not roots[0].is_relative_to(roots[1]) and not roots[1].is_relative_to(roots[0]),'/products','independent product roots required')
    require(products.base.executable_sha256==products.candidate.executable_sha256,'/products','different Godot versions')
    specs=tuple(process_spec(product,output/name) for name,product in (('base',products.base),('candidate',products.candidate)))
    output.mkdir()
    frozen: dict[Path,dict[str,str]]={}
    evidence: list[PackEvidence]=[]
    errors: list[str]=[]
    for spec in specs:
        try:
            result=execute(spec);freeze(spec.output_root,frozen)
            evidence.append(inspect(spec,result))
        except (GateContractError,OSError,UnicodeError) as error:errors.append(spec.output_root.name+': '+str(error))
    try:
        require(len(evidence)==2,'/pair','missing completed arm');compare(evidence[0],evidence[1])
    except GateContractError as error:errors.append(str(error))
    for folder,hashes in frozen.items():
        try:
            require(inventory(folder,[p.name for p in folder.iterdir()])==hashes,str(folder),'artifacts changed during pair')
        except (GateContractError,OSError) as error:errors.append(str(error))
    for product in (products.base,products.candidate):
        try:check_product(product)
        except (GateContractError,OSError) as error:errors.append(str(error))
    status='FAIL' if errors else 'PASS'
    receipt=output/'pack-ui.json'
    atomic_json(receipt,{'schema_version':1,'status':status,'errors':[e for e in errors],
        'classification':'ordinary_selected_pack_continuous_world','negative_DTO_checks':'presentation_validation_only',
        'selected_pack_inputs':{name:digest for name,digest in PINS.items() if name.startswith(str(Path(PACK).parent)+'/')},
        'worlds':[{'prefix_tick':e.prefix.state.terminal_tick,'terminal_tick':e.terminal.state.terminal_tick,
                   'sidecar':str(e.evidence.sidecar),'sha256':e.evidence.sidecar_sha256} for e in evidence],
        'artifacts':{str(folder):{name:digest for name,digest in hashes.items()} for folder,hashes in frozen.items()},
        'rendered_capture':'NOT_RUN_HEADLESS','rust_mechanism_children':'outside_this_component',
        'build_provenance':'caller_owned_not_certified_here','full_gate_claim':False})
    return ComponentResult(receipt,status)
