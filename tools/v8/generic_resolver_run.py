# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Original step32 only; caller retains its actual lane on unknown descendants."""
import fcntl
import os
import re
import shutil
import stat
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Final

from generic_fixture_run import ComponentResult
from generic_fixtures import ProbeProduct
from generic_gate import document, integer, require
from generic_m6a_structural import resolution_paths
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
    real_path,
)
from generic_sources import inventory, safe_path
from generic_static_run import ExecutionEnvironment
from generic_workspace_cases import ListingProcess, process_output
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json
from mux2_gate_lock import LOCK

COMMAND: Final = ('bash tools/v8/run_bounded.sh 300 "$GODOT_BIN" --headless --path . '
                  '--script res://tools/v8/resolver_parity.gd -- "$0/resolver_godot.json" '
                  '&& python3 tools/v8/resolver_parity.py --check "$0/resolver_godot.json"')
PINS: Final = {
    'tools/v8/gate.sh':'ba949251efa089536c44b9883515d017c7254122b4167756833dfd7070d8c523',
    'tools/v8/resolver_parity.gd':'3d437e0b7b114ef4d9a727937ed6dff2cea2965862fdb2ef95f6eba371b881ba',
    'tools/v8/resolver_parity.py':'42c95303ed1538aae3f77f684be2992eb281cb07d1d9bde60b8845409f805940',
    'tools/v8/run_bounded.sh':'64b0d037c1a854add96cccf25ae7a0bcfc40be78ab4406f092514e4891009bf7',
    'scripts/world/art/material_resolver.gd':'2bcd0f504dc03a0d96ac611894008f720d8ecddcfedaaeccdc782794993f876e',
    'scripts/world/art/flora_resolver.gd':'75cbb6f384bc7f59584d145f7bfdd8c9fd95dafa380d6e4c7d1eaa4e0ce47670',
    'scripts/world/art/art_data.gd':'fd3d27da528819c6ace5ad04824c1c7f8b8a5232b9f2535f40c3d7b1160fa919',
    'scripts/ui/content_context.gd':'e8cdac02c9d5241464a2d153e51c1c1db3a30696198d255d791213f0ac31ce10',
    'scripts/core/locale.gd':'b92c662d50966ce294be3596d91ef7530a3defd59010beee2c898e2e85ae50e3',
    'project.godot':'cd16a69aa33fbde80fb64c2bf8fb584f6d7aaef0f4b135ed92c8a35cb6061c98',
    'sim_bridge.gdextension':'6c47616cc217caf4b9dfcecfba4c6157f98b8ee8954c18602081ab8a63eb1b5a',
    'rust/crates/sim-bridge/src/lib.rs':'eff7aa23411792d0ef59fe6d440f31d74c44e9fde9bffa030f80af980e39f21c',
}


@dataclass(frozen=True)
class ResolverPlan:
    product: ProbeProduct
    context: ExecutionEnvironment
    output: Path
    process: ProcessSpec


class ResolverSettledFailure(RuntimeError):
    def __init__(self, original: BaseException, secondary: tuple[BaseException,...], receipt: Path | None) -> None:
        super().__init__('resolver failed after settled execution or before launch')
        self.original: BaseException=original
        self.secondary: tuple[BaseException,...]=secondary
        self.receipt: Path | None=receipt


class ResolverOwnershipUnknown(RuntimeError):
    def __init__(self, original: BaseException, secondary: tuple[BaseException,...], receipt: Path | None) -> None:
        super().__init__('resolver descendants unknown; caller must retain actual lane')
        self.original: BaseException=original
        self.secondary: tuple[BaseException,...]=secondary
        self.ownership_receipt: Path | None=receipt


def resource_names(root: Path) -> list[str]:
    names=[*INPUT_DIRS,*INPUT_FILES,'scripts','assets','localization','shaders','.godot/imported',
           '.godot/global_script_class_cache.cfg','.godot/extension_list.cfg','.godot/uid_cache.bin',
           'rust/target/debug/libsim_bridge.dylib','rust/target/release/libsim_bridge.dylib']
    names.extend(sorted(resolution_paths(root)))
    names.extend(PINS)
    names.extend('assets/v8art/'+name for name in ('manifest.json','bindings.json','render_params.json',
        'materials/construction_styles.json','materials/material_looks.json','materials/category_defaults.json',
        'flora/species.json','flora/habit_defaults.json'))
    names.append('localization/manifest.json')
    names.extend('tools/v8/'+name+'.json' for name in ('resolver_parity_fixture','flora_parity_fixture','resolver_parity_expected'))
    for row in document((root/'tools/v8/flora_parity_fixture.json').read_text()).field('cases').array():
        value=row.mapping().get('fixture')
        if value is not None:
            require(isinstance(value,str) and re.fullmatch('[A-Za-z0-9_-]+',value) is not None,'/fixture','unsafe flora fixture key')
            names.append('assets/v8art/flora/tests/'+str(value)+'.json')
    scopes=set(names)
    # Validate mandatory descendants before reducing covered scopes; absence is not optional.
    for name in sorted(scopes):
        path=safe_path(root,name)
        real_path(path,directory=path.is_dir())
    return sorted(name for name in scopes if not any(parent.as_posix() in scopes for parent in PurePosixPath(name).parents))


def bindings(product: ProbeProduct, context: ExecutionEnvironment) -> tuple[tuple[Path,str],...]:
    real_path(product.input_root,directory=True)
    require(re.fullmatch('[0-9a-f]{40}',product.source_sha) is not None,'/source','full source SHA required')
    expected=dict(product.input_files)
    require(len(expected)==len(product.input_files) and all(expected.get(name)==digest for name,digest in PINS.items()),
            '/source','unreviewed selected no-world chain')
    required=inventory(product.input_root,resource_names(product.input_root))
    require(all(expected.get(name)==digest for name,digest in required.items()),'/inputs','incomplete resource closure')
    require(inventory(product.input_root,list(expected))==expected,'/inputs','product drift')
    for name in ('override.cfg','addons/godot_test_mcp'):
        require(not (product.input_root/name).exists() and not (product.input_root/name).is_symlink(),name,'unreviewed local override')
    environment=dict(context.values)
    allowed={'PATH','HOME','TMPDIR','LANG','LC_ALL','GODOT_BIN','PYTHONNOUSERSITE','PYTHONDONTWRITEBYTECODE',
             'PYTHONCOERCECLOCALE','__CF_USER_TEXT_ENCODING'}
    require(len(environment)==len(context.values) and environment.keys()<=allowed,'/environment','unapproved runtime/protocol override')
    for key in ('PYTHONNOUSERSITE','PYTHONDONTWRITEBYTECODE'):require(environment.get(key)=='1',key,'Python startup policy required')
    for key in ('HOME','TMPDIR'):real_path(Path(environment[key]),directory=True)
    require(environment.get('GODOT_BIN')==str(product.executable),'/godot','wrong actual Godot')
    require(file_hash(product.executable)==product.executable_sha256,'/godot','binary drift')
    tools={name:(path,digest) for name,path,digest in context.tools}
    require(len(tools)==len(context.tools) and set(tools)=={'bash','python3'},'/tools','exact shell/Python tools required')
    external=dict(context.external_inputs)
    require(bool(external) and len(external)==len(context.external_inputs),'/runtime','explicit unique runtime inputs required')
    for name,(path,digest) in tools.items():
        real_path(path,directory=False)
        selected=shutil.which(name,path=environment['PATH'])
        require(selected is not None and Path(selected).resolve()==path and file_hash(path)==digest,'/PATH','tool differs')
        require(path not in external or external[path]==digest,'/tools','conflicting runtime binding');external[path]=digest
    for path,digest in (*context.prior_receipts,(product.executable,product.executable_sha256)):
        require(path not in external or external[path]==digest,'/inputs','conflicting evidence');external[path]=digest
    for path,digest in external.items():
        real_path(path,directory=False)
        require(not path.is_relative_to(product.input_root) and file_hash(path)==digest,str(path),'external input drift/overlap')
    return tuple(sorted(external.items()))


def prepare(product: ProbeProduct, context: ExecutionEnvironment, output: Path, *, historical: bool=False) -> ResolverPlan:
    external=bindings(product,context)
    require(output.is_absolute() and output==output.resolve() and not output.is_relative_to(product.input_root)
            and not product.input_root.is_relative_to(output),'/output','unsafe output root')
    require(historical or not output.exists(),'/output','fresh output required')
    require(all(not path.is_relative_to(output) for path,_ in external),'/output','runtime/output overlap')
    real_path(output.parent,directory=True)
    tools={name:(path,digest) for name,path,digest in context.tools};bash,digest=tools['bash']
    spec=ProcessSpec((str(bash),'-c',COMMAND,str(output)),product.input_root,bash,digest,product.input_root,
        product.input_files,output/'process',context.values,external)
    return ResolverPlan(product,context,output,spec)


def held_lane(context: ExecutionEnvironment) -> None:
    descriptor=context.held_lane_fd
    require(type(descriptor) is int and descriptor>=0,'/lane','caller-held descriptor required')
    if descriptor is None:raise ValueError('missing descriptor')
    held=os.fstat(descriptor);canonical=LOCK.stat()
    require(stat.S_ISREG(held.st_mode) and (held.st_dev,held.st_ino)==(canonical.st_dev,canonical.st_ino),'/lane','foreign lane')
    fcntl.flock(descriptor,fcntl.LOCK_EX|fcntl.LOCK_NB)


def completed(plan: ResolverPlan, result: ProcessResult, digest: str) -> bytes:
    require(result.status=='PASS' and result.native_exit==0 and type(result.pid) is int and result.pid>0,
            '/ownership','only actual returned successful chain settles')
    spec=plan.process
    required={str(spec.input_root/name):value for name,value in spec.inputs}
    required.update((str(path),value) for path,value in spec.external_inputs)
    required[str(spec.executable)]=spec.executable_sha256
    return process_output(ListingProcess(spec,result,digest),spec.argv,spec.cwd,environment_for(spec),required)


def checker_terminal(plan: ResolverPlan, stdout: bytes) -> dict[str,Json]:
    expected=document((plan.product.input_root/'tools/v8/resolver_parity_expected.json').read_text())
    material=len(expected.field('results').array());flora=len(expected.field('flora_results').array())
    require(material>0 and flora>0,'/checker','empty original expectation')
    terminal=f'resolver_parity: PASS ({material} material and {flora} flora cases, 0 differences)'
    lines=stdout.decode('utf-8').splitlines()
    require(bool(lines) and lines[-1]==terminal and lines.count(terminal)==1,'/checker','missing original terminal')
    # Original --check owns comparison/normalisation; no substitute comparator runs here.
    return {'material':material,'flora':flora,'checker':'ORIGINAL_SUBPROCESS_PASS','native_marker_diagnostic':
        any(line.startswith('RESOLVER_PARITY_DONE') for line in lines)}


def raw_inventory(output: Path) -> dict[str,str]:
    require({path.name for path in output.iterdir()} <= {'process','resolver_godot.json','receipt.json'},
            '/output','unexpected component output')
    return inventory(output,['process','resolver_godot.json'])


def run_owned(product: ProbeProduct, context: ExecutionEnvironment, output: Path) -> ComponentResult:
    in_flight=False;created=False
    try:
        held_lane(context);plan=prepare(product,context,output)
        output.mkdir();created=True
        in_flight=True
        result=execute(plan.process)
        digest=file_hash(result.receipt)
        stdout=completed(plan,result,digest)
        in_flight=False
        artifacts=raw_inventory(output)
        counts=checker_terminal(plan,stdout)
        _=bindings(product,context)
        require(raw_inventory(output)==artifacts,'/artifacts','late resolver output drift')
        receipt: dict[str,Json]={'schema':'generic_resolver_run.v1','status':'PASS','step':32,
            'source_sha':product.source_sha,'input_root':str(product.input_root),'process_sha256':digest,
            'parent_pid':result.pid,'native_pid':None,'native_exit':'NOT_RECORDED','checker':counts,
            'artifacts':{name:value for name,value in artifacts.items()},'source_build_admission':'PENDING',
            'simulation_classification':'NO_REGISTERED_SIMULATION_WORLD_SOURCE_DERIVED',
            'full_gate_claim':False,'historical_settlement_authority':False}
        atomic_json(output/'receipt.json',receipt)
        return ComponentResult(output/'receipt.json','PASS')
    except BaseException as original:
        receipt_path: Path | None=None
        if created:
            try:
                receipt_path=output/('ownership.json' if in_flight else 'settled-failure.json')
                atomic_json(receipt_path,{'schema':'resolver_failure.v1','status':'UNKNOWN' if in_flight else 'FAIL',
                    'source_build_admission':'PENDING','full_gate_claim':False})
            except BaseException as diagnostic:
                if in_flight:raise ResolverOwnershipUnknown(original,(diagnostic,),None) from diagnostic
                raise ResolverSettledFailure(original,(diagnostic,),None) from diagnostic
        if in_flight:raise ResolverOwnershipUnknown(original,(),receipt_path) from original
        raise ResolverSettledFailure(original,(),receipt_path) from original


def authenticate_result(product: ProbeProduct, context: ExecutionEnvironment, output: Path, digest: str) -> ComponentResult:
    """Historical endpoint consistency only; never settles an actual live owner."""
    plan=prepare(product,context,output,historical=True);path=output/'receipt.json'
    require(file_hash(path)==digest,'/receipt','component receipt drift')
    row=document(path.read_text())
    expected={'schema','status','step','source_sha','input_root','process_sha256','parent_pid','native_pid','native_exit',
              'checker','artifacts','source_build_admission','simulation_classification','full_gate_claim','historical_settlement_authority'}
    require(set(row.mapping())==expected,'/receipt','schema differs')
    for key,value in {'schema':'generic_resolver_run.v1','status':'PASS','step':32,'source_sha':product.source_sha,
                      'input_root':str(product.input_root),'native_pid':None,'native_exit':'NOT_RECORDED',
                      'source_build_admission':'PENDING','simulation_classification':'NO_REGISTERED_SIMULATION_WORLD_SOURCE_DERIVED',
                      'full_gate_claim':False,'historical_settlement_authority':False}.items():row.field(key).equals(value)
    pid=integer(row.field('parent_pid'));require(pid>0,'/process','positive parent PID required')
    result=ProcessResult(output/'process/process.json','PASS',0,pid)
    artifacts=raw_inventory(output)
    row.field('artifacts').equals({name:value for name,value in artifacts.items()})
    stdout=completed(plan,result,row.field('process_sha256').text())
    row.field('checker').equals(checker_terminal(plan,stdout))
    _=bindings(product,context)
    require(file_hash(path)==digest and raw_inventory(output)==artifacts,'/endpoint','late evidence drift')
    return ComponentResult(path,'PASS')
