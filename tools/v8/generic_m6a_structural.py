"""Original no-world plant-art renderer; visual and native ancestry remain unqualified."""
import ast
import fcntl
import logging
import os
import re
import shutil
import signal
import stat
import subprocess
import sys
import uuid
from contextlib import suppress
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from types import ModuleType
from typing import Literal

import gate_oracle_child as oracle_child
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import document, integer, require, text
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
    real_path,
)
from generic_sources import inventory
from m6a_gate_art import structural_command, validate
from m6a_gate_history import run as original_run
from mux2_gate_inputs import Json
from mux2_gate_lock import FD_ENV, LOCK

LOGGER = logging.getLogger(__name__)
# Source-reviewed scene/autoload/rebuild paths: layers never enter the scene tree.
PINS: dict[str, str] = {'scripts/world/art_hash.gd': 'e37c1a4553175450dbf598bfc3b7a0d86c51016bc2c781a6e4ef82a93e288afe', 'scripts/ui/person_names.gd': 'cd8afd961c3eb31fc6b7706ea1bdb4ea0c9334e229cff590974aa6e2de422791', 'sim_bridge.gdextension': '6c47616cc217caf4b9dfcecfba4c6157f98b8ee8954c18602081ab8a63eb1b5a', 'project.godot': 'cd16a69aa33fbde80fb64c2bf8fb584f6d7aaef0f4b135ed92c8a35cb6061c98', 'tools/v8/m6a_plant_art_test.gd': 'c8deea193b8ffc26436213fa8bece13a912d2870dc4276ec52c3f6da634c2006', 'tools/v8/run_bounded.sh': '64b0d037c1a854add96cccf25ae7a0bcfc40be78ab4406f092514e4891009bf7', 'scripts/world/colony/plant_layer.gd': '84c7bb21b959d2511028b7df42d09b10cb16888b82c65aa59e849a66d95d0fc3', 'scripts/world/art/art_data.gd': 'fd3d27da528819c6ace5ad04824c1c7f8b8a5232b9f2535f40c3d7b1160fa919', 'scripts/ui/content_context.gd': 'e8cdac02c9d5241464a2d153e51c1c1db3a30696198d255d791213f0ac31ce10', 'scripts/core/locale.gd': 'b92c662d50966ce294be3596d91ef7530a3defd59010beee2c898e2e85ae50e3', 'scripts/core/sim_reads.gd': 'b50284a716ee296186fd007e78532a2c3c2569e7ce3e06f01b1b1428e6ba1128', 'scripts/world/colony/view.gd': 'f67192ce8caefbff8971a449f1b5e001c3b787cfb72807b8546c51f42f8e95ab'}
CLASS_CACHE_SHA256 = 'b82c2e7b1bd0f527aa0683cbc0225e49af2f5c54f20d5c9d603015e0d1236f6f'
ORACLE_PINS: dict[str, str] = {'m6a_gate_art.py': '05e6173b0142f1f1b2bbf78aab0170b07d6f1e9cc14675ae82996d622c7013d6', 'm6a_gate_history.py': '75b3893f7732caa48c670d5b9017d6cef1e1077b44249a63170d24b0510f2ff4'}


@dataclass(frozen=True, slots=True)
class Tooling:
    python: Path
    python_sha256: str
    bash: Path
    bash_sha256: str
    root: Path
    files: tuple[tuple[str, str], ...]
    path: str


@dataclass(frozen=True, slots=True)
class StructuralPlan:
    product: ProbeProduct
    tools: Tooling
    output: Path
    spec: ProcessSpec
    plan_path: Path
    plan_sha256: str


def import_closure(root: Path, entry: str) -> tuple[str, ...]:
    """Seal all repository-local static Python imports, including legacy oracle imports."""
    pending = [entry]
    found: set[str] = set()
    while pending:
        name = pending.pop()
        if name in found:
            continue
        found.add(name)
        for node in ast.walk(ast.parse((root/name).read_text())):
            modules: list[str] = []
            if isinstance(node, ast.Import):
                modules = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom) and node.module is not None:
                require(node.level == 0, '/imports', 'relative tooling import requires review')
                modules = [node.module]
            for module in modules:
                candidate = module.replace('.', '/')+'.py'
                if (root/candidate).is_file():
                    pending.append(candidate)
    return tuple(sorted(found))


def eager_sources() -> dict[str, tuple[ModuleType, Path, str]]:
    """Capture the used eager graph once; unused function-local imports stay file-sealed.

    Structural call paths do not invoke the legacy lazy imports. Adding a used lazy
    helper requires reviewed pre-use admission, not a refresh of this snapshot.
    """
    entry = Path(__file__).resolve()
    roots = (entry.parent, Path(original_run.__code__.co_filename).resolve().parent)
    pending = [('generic_m6a_structural', entry)]
    found: dict[str, tuple[ModuleType, Path, str]] = {}
    while pending:
        name, source = pending.pop()
        if name in found:
            continue
        key = __name__ if name == 'generic_m6a_structural' else name
        module = sys.modules.get(key)
        if module is None or module.__file__ is None:
            raise ValueError('missing eager coordinator module: '+name)
        actual = Path(module.__file__).resolve()
        found[name] = (module, actual, file_hash(actual))
        nodes: list[ast.AST] = list(ast.parse(source.read_text()).body)
        while nodes:
            node = nodes.pop()
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
                continue
            modules: list[str] = []
            if isinstance(node, ast.Import):
                modules = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom) and node.module is not None:
                require(node.level == 0, '/imports', 'relative tooling import requires review')
                modules = [node.module]
            for imported in modules:
                relative = imported.replace('.', '/')+'.py'
                for root in roots:
                    candidate = root/relative
                    if candidate.is_file():
                        pending.append((imported, candidate.resolve()))
                        break
            nodes.extend(ast.iter_child_nodes(node))
    return dict(sorted(found.items()))


EAGER_SOURCES = eager_sources()
LOADED_SOURCES = {name: (source, digest) for name, (_, source, digest) in EAGER_SOURCES.items()}


def coordinator_sources(tools: Tooling) -> None:
    """Authenticate every used helper; ambient late imports cannot alter admission."""
    sealed = dict(tools.files)
    for name, (admitted_module, source, digest) in EAGER_SOURCES.items():
        key = __name__ if name == 'generic_m6a_structural' else name
        module = sys.modules.get(key)
        if module is None or module is not admitted_module or module.__file__ is None or Path(module.__file__).resolve() != source:
            raise ValueError('unrecorded coordinator module: '+name)
        filename = name.replace('.', '/')+'.py'
        require(filename in sealed, '/tools', 'missing used coordinator source: '+name)
        if sealed[filename] != digest or file_hash(source) != digest:
            raise ValueError('coordinator source differs from sealed tooling: '+name)


def resource_path(value: str) -> str:
    """Only canonical project-local resource references are admitted."""
    if not value.startswith('res://'):
        raise ValueError('external resource resolution: '+value)
    name = value[6:]
    path = PurePosixPath(name)
    if not name or path.is_absolute() or '..' in path.parts or str(path) != name or '\\' in name:
        raise ValueError('noncanonical resource resolution: '+value)
    if name.endswith('.gdextension') and name != 'sim_bridge.gdextension':
        raise ValueError('unreviewed extension resource: '+name)
    return name


def uid_paths(data: bytes) -> set[str]:
    """Godot4.6 ResourceUID cache: u32 count, then u64 ID/u32 size/UTF8 path."""
    if len(data) < 4:
        raise ValueError('truncated UID cache')
    count = int.from_bytes(data[:4], 'little')
    offset = 4
    ids: set[int] = set()
    paths: set[str] = set()
    for _ in range(count):
        if offset + 12 > len(data):
            raise ValueError('truncated UID entry')
        uid = int.from_bytes(data[offset:offset+8], 'little')
        size = int.from_bytes(data[offset+8:offset+12], 'little')
        offset += 12
        if uid >= 2**63 or uid in ids or not size or offset+size > len(data):
            raise ValueError('invalid UID entry')
        ids.add(uid)
        name = resource_path(data[offset:offset+size].decode('utf-8'))
        paths.add(name)
        offset += size
    if offset != len(data):
        raise ValueError('trailing UID cache bytes')
    return paths


def resolution_paths(root: Path) -> set[str]:
    """Close the reviewed extension/class maps and every effective import/UID path."""
    extensions = (root/'.godot/extension_list.cfg').read_text().splitlines()
    if extensions != ['res://sim_bridge.gdextension']:
        raise ValueError('unreviewed extension set')
    # Exact reviewed descriptor includes only the original platform library mappings.
    if file_hash(root/'sim_bridge.gdextension') != PINS['sim_bridge.gdextension']:
        raise ValueError('unreviewed extension library mappings')
    cache = root/'.godot/global_script_class_cache.cfg'
    if file_hash(cache) != CLASS_CACHE_SHA256:
        raise ValueError('unreviewed global class resolution')
    paths = uid_paths((root/'.godot/uid_cache.bin').read_bytes())
    paths.update(resource_path(match.group(1)) for match in re.finditer(r'"path": "([^"\n]+)"',cache.read_text()))
    # Import metadata can redirect a source to a different artifact. Scan all files,
    # including remaps beside UID-indexed resources outside the usual asset trees.
    pending = list(paths)
    for directory in ('scripts','assets','localization','shaders','.godot/imported'):
        pending.extend(p.relative_to(root).as_posix() for p in (root/directory).rglob('*') if p.is_file())
    visited: set[str] = set()
    while pending:
        name = pending.pop()
        if name in visited:
            continue
        visited.add(name)
        for suffix in ('.import','.remap','.uid'):
            sidecar = name+suffix
            if (root/sidecar).is_file():
                paths.add(sidecar)
                pending.append(sidecar)
        if not name.endswith(('.import','.remap')):
            continue
        metadata = (root/name).read_text()
        if name.endswith('.import'):
            importers = [m.group(1) for m in re.finditer(r'^importer="([^"\n]+)"$',metadata,re.MULTILINE)]
            kinds = [m.group(1) for m in re.finditer(r'^type="([^"\n]+)"$',metadata,re.MULTILINE)]
            if (importers,kinds) not in ((['texture'],['CompressedTexture2D']),(['font_data_dynamic'],['FontFile'])):
                raise ValueError('unreviewed resource importer: '+name)
        # Closed syntax for path-bearing import keys: no escaped/relative/absolute
        # values, packed arrays or multiline path values admitted by approximation.
        for line in metadata.splitlines():
            if re.match(r'\s*(?:path(?:\.[\w]+)?|source_file|dest_files)\s*=',line):
                key,value = line.split('=',1)
                refs = [match.group(1) for match in re.finditer(r'"([^"\\]*)"',value)]
                compact = re.sub(r'"[^"\\]*"','""',value).replace(' ','')
                if not refs or compact not in ('""','['+','.join('""' for _ in refs)+']'):
                    raise ValueError('unreviewed import resolution syntax: '+name)
                for ref in refs:
                    target = resource_path(ref)
                    if key.strip() != 'source_file' and (not target.startswith('.godot/imported/')
                            or PurePosixPath(target).suffix not in {'.ctex','.fontdata'}):
                        raise ValueError('unreviewed imported artifact: '+target)
                    paths.add(target)
                    pending.append(target)
    return paths


def required_resources(root: Path) -> set[str]:
    required = set(PINS) | {'sim_bridge.gdextension', 'rust/target/debug/libsim_bridge.dylib', 'rust/target/release/libsim_bridge.dylib',
        'tools/v8/run_bounded.sh', 'tools/v8/fixtures/m6a_plant_art.json',
        '.godot/global_script_class_cache.cfg', '.godot/extension_list.cfg', '.godot/uid_cache.bin'}
    for name in ('scripts', 'assets', 'localization', 'shaders', '.godot/imported'):
        directory = root/name
        require(directory.is_dir(), '/resources', 'missing resource tree '+name)
        required.update(p.relative_to(root).as_posix() for p in directory.rglob('*') if p.is_file())
    required.update(resolution_paths(root))
    return required


def _check_tools(tools: Tooling) -> None:
    coordinator_sources(tools)
    require(file_hash(tools.python) == tools.python_sha256 and file_hash(tools.bash) == tools.bash_sha256,
            '/tools', 'tool executable changed')
    for name, expected in (('python3', tools.python), ('bash', tools.bash)):
        selected = shutil.which(name, path=tools.path)
        require(selected is not None and Path(selected).resolve() == expected, '/PATH', 'unbound '+name)
    real_path(tools.root, directory=True)
    require(all(dict(tools.files).get(name) == digest for name,digest in ORACLE_PINS.items()), '/oracle', 'unreviewed original oracle')
    require(len(dict(tools.files)) == len(tools.files), '/tools', 'duplicate source')
    require(dict(tools.files).get('generic_m6a_structural.py') == file_hash(Path(__file__).resolve()),
            '/adapter', 'tooling adapter differs from admitted coordinator')
    closure = import_closure(tools.root, 'generic_m6a_structural.py')
    require(set(closure) <= dict(tools.files).keys(), '/tools', 'incomplete transitive tooling closure')
    require(inventory(tools.root, list(dict(tools.files))) == dict(tools.files), '/tools', 'tooling changed')


def _check_product(product: ProbeProduct) -> None:
    real_path(product.input_root, directory=True)
    require(re.fullmatch('[0-9a-f]{40}', product.source_sha) is not None, '/source', 'full source SHA required')
    require(file_hash(product.executable) == product.executable_sha256, '/godot', 'Godot changed')
    inputs = dict(product.input_files)
    require(len(inputs) == len(product.input_files) and required_resources(product.input_root) <= inputs.keys(),
            '/inputs', 'incomplete effective resource inventory')
    require(all(inputs.get(name) == digest for name, digest in PINS.items()), '/classification', 'unreviewed no-world source')
    require(inventory(product.input_root, list(inputs)) == inputs, '/inputs', 'product changed')


def prepare(product: ProbeProduct, tools: Tooling, output: Path) -> StructuralPlan:
    _check_tools(tools)
    _check_product(product)
    require(not output.exists() and output == output.resolve(), '/output', 'fresh canonical output required')
    roots = (product.input_root, tools.root, output)
    require(all(not a.is_relative_to(b) and not b.is_relative_to(a) for i,a in enumerate(roots) for b in roots[i+1:]),
            '/roots', 'overlapping roots')
    real_path(output.parent, directory=True)
    plan_path = output.parent/(output.name+'-plan.json')
    raw: dict[str, Json] = {'schema':'m6a_structural_launch_v1', 'root':str(product.input_root),
        'command':[str(arg) for arg in structural_command(str(product.executable))],
        'output':str(output), 'source_sha':product.source_sha}
    atomic_json(plan_path, raw)
    digest = file_hash(plan_path)
    external = tuple((tools.root/name,sha) for name,sha in tools.files)
    external += ((tools.bash,tools.bash_sha256),(product.executable,product.executable_sha256),(plan_path,digest))
    spec = ProcessSpec((str(tools.python), str(tools.root/'generic_m6a_structural.py'), '--capture',str(plan_path),digest),
        product.input_root, tools.python, tools.python_sha256, product.input_root,product.input_files,output,
        (('HOME',str(Path.home())),('__CF_USER_TEXT_ENCODING',f'0x{os.getuid():X}:0x3:0x33'),('PATH',tools.path),('PYTHONNOUSERSITE','1'),('PYTHONDONTWRITEBYTECODE','1'),('PYTHONCOERCECLOCALE','0')),
        external_inputs=external, timeout_seconds=None)
    return StructuralPlan(product,tools,output,spec,plan_path,digest)


def capture_child(path: Path, digest: str) -> int:
    require(file_hash(path) == digest, '/plan', 'launch plan changed')
    plan = document(path.read_text())
    plan.field('schema').equals('m6a_structural_launch_v1')
    root = Path(text(plan.field('root')))
    output = Path(text(plan.field('output')))
    command = [text(arg) for arg in plan.field('command').array()]
    require(len(command) == 10 and command == [str(arg) for arg in structural_command(command[3])], '/argv', 'altered original command')
    require(Path.cwd() == root, '/cwd', 'wrong product root')
    code: int | None = None
    normal = False
    error: str | None = None
    atomic_json(output/'capture-start.json', {'schema':'m6a_structural_capture_v1','adapter_pid':os.getpid(),
        'plan_sha256':digest,'command':[arg for arg in command],'cwd':str(root),'environment':dict(os.environ)})
    try:
        original_run(root, [arg for arg in command], output/'plant-art-structural.log')
        code = 0
        normal = True
    except subprocess.CalledProcessError as failure:
        code = failure.returncode
        normal = code >= 0
        error = str(failure)
    except BaseException as failure:
        LOGGER.exception('Original wrapper completion is unknown')
        error = type(failure).__name__+': '+str(failure)
    atomic_json(output/'capture-completion.json', {'schema':'m6a_structural_capture_v1','adapter_pid':os.getpid(),
        'plan_sha256':digest,'command':[arg for arg in command],'cwd':str(root),'wrapper_returncode':code,
        'normal_wrapper_completion':normal,'godot_pid':None,'native_child_pid_evidence':'NOT_RECORDED',
        'error':error,'quiescence':'ORIGINAL_WRAPPER_COMPLETED' if normal else 'UNKNOWN'})
    return 0 if code == 0 and normal else 1


def capture_once(plan: StructuralPlan) -> ProcessResult:
    coordinator_sources(plan.tools)
    require(plan.spec.timeout_seconds is None, '/timeout', 'outer timeout must remain absent')
    return execute(plan.spec)


def _completed(plan: StructuralPlan, result: ProcessResult) -> bool:
    """Only an authenticated normal supervisor return admits final freezing."""
    if not (plan.output/'capture-completion.json').is_file():
        return False
    require(result.receipt == plan.output/'process.json' and type(result.pid) is int
            and result.pid > 0, '/completion', 'foreign adapter result')
    process = document(result.receipt.read_text())
    process.field('pid').equals(result.pid)
    process.field('native_exit').equals(result.native_exit)
    process.field('argv').equals(list(plan.spec.argv))
    process.field('cwd').equals(str(plan.spec.cwd))
    row = document((plan.output/'capture-completion.json').read_text())
    expected: dict[str, Json] = {'schema':'m6a_structural_capture_v1', 'adapter_pid':result.pid,
        'plan_sha256':plan.plan_sha256, 'cwd':str(plan.spec.cwd),
        'command':[str(arg) for arg in structural_command(str(plan.product.executable))]}
    for key,value in expected.items():
        row.field(key).equals(value)
    if row.field('normal_wrapper_completion').value is not True:
        return False
    code = integer(row.field('wrapper_returncode'))
    row.field('quiescence').equals('ORIGINAL_WRAPPER_COMPLETED')
    return code >= 0 and result.native_exit == (0 if code == 0 else 1)


def _renderer_oracle(raw: str) -> int:
    backends = re.findall(r'^M6A_PLANT_ART_BACKEND display=(\S+) method=(\S+)$',raw,re.MULTILINE)
    require(backends == [('macOS','gl_compatibility')], '/backend', 'missing/duplicate/unreviewed actual renderer')
    require(not re.search(r'(?:Vulkan|Metal|Dummy).*API',raw), '/backend', 'conflicting renderer banner')
    return validate(raw)


def inspect_output(plan: StructuralPlan, result: ProcessResult) -> int:
    coordinator_sources(plan.tools)
    require(_completed(plan,result), '/completion', 'original wrapper completion missing')
    require(result.receipt == plan.output/'process.json' and result.status == 'PASS' and result.native_exit == 0,
            '/process', 'adapter did not pass')
    require(type(result.pid) is int and result.pid > 0, '/pid', 'actual adapter PID missing')
    row = document(result.receipt.read_text())
    expected: dict[str, Json] = {'argv':list(plan.spec.argv),'cwd':str(plan.product.input_root),
        'pid':result.pid,'native_exit':0,'status':'PASS','errors':[],
        'executable':str(plan.tools.python),'expected_executable_sha256':plan.tools.python_sha256,
        'before_executable_sha256':plan.tools.python_sha256,'after_executable_sha256':plan.tools.python_sha256,
        'environment':dict(environment_for(plan.spec)),'environment_policy':'fixed_defaults_plus_explicit_only_no_inheritance',
        'expected_inputs':dict(plan.product.input_files),'before_inputs':dict(plan.product.input_files),
        'after_inputs':dict(plan.product.input_files),'expected_external_inputs':{str(p):h for p,h in plan.spec.external_inputs},
        'before_external_inputs':{str(p):h for p,h in plan.spec.external_inputs},
        'after_external_inputs':{str(p):h for p,h in plan.spec.external_inputs}}
    for key,value in expected.items():
        row.field(key).equals(value)
    require('timeout_seconds' not in row.mapping(), '/timeout', 'outer timeout introduced')
    for stream in ('stdout','stderr'):
        row.field(stream+'_sha256').equals(file_hash(plan.output/(stream+'.log')))
    command = [str(arg) for arg in structural_command(str(plan.product.executable))]
    for name in ('capture-start.json','capture-completion.json'):
        child = document((plan.output/name).read_text())
        metadata: dict[str, Json] = {'schema':'m6a_structural_capture_v1','adapter_pid':result.pid,'plan_sha256':plan.plan_sha256,
                          'command':[arg for arg in command],'cwd':str(plan.product.input_root)}
        for key,value in metadata.items():
            child.field(key).equals(value)
    document((plan.output/'capture-start.json').read_text()).field('environment').equals(dict(environment_for(plan.spec)))
    child.field('normal_wrapper_completion').equals(True)
    child.field('wrapper_returncode').equals(0)
    child.field('godot_pid').equals(None)
    child.field('native_child_pid_evidence').equals('NOT_RECORDED')
    child.field('error').equals(None)
    raw = (plan.output/'plant-art-structural.log').read_bytes().decode('utf-8')
    count = _renderer_oracle(raw)
    _check_product(plan.product)
    _check_tools(plan.tools)
    require(file_hash(plan.plan_path) == plan.plan_sha256, '/plan', 'plan changed')
    return count


def run_pair(products: Products, tools: Tooling, output: Path) -> ComponentResult:
    require(not output.exists() and output == output.resolve(), '/output', 'fresh component output required')
    roots = (products.base.input_root,products.candidate.input_root,tools.root,output)
    require(all(not a.is_relative_to(b) and not b.is_relative_to(a) for i,a in enumerate(roots) for b in roots[i+1:]),
            '/roots','overlapping roots')
    output.mkdir()
    plans: dict[str, StructuralPlan] = {}
    frozen: dict[str,dict[str,str]] = {}
    phases: dict[str,str] = dict.fromkeys(('base','candidate'),'NOT_RUN')
    errors: list[str] = []
    incomplete = False
    checks: dict[str,int] = {}
    try:
        for arm in phases:
            plans[arm] = prepare(products.arm(arm),tools,output/arm)
    except (OSError,ValueError) as failure:
        errors.append('preflight: '+str(failure))
    for arm,plan in (() if errors else plans.items()):
        phases[arm] = 'STARTED'
        try:
            result = capture_once(plan)
            if not _completed(plan,result):
                incomplete = True
                phases[arm] = 'INCOMPLETE_DESCENDANTS_UNKNOWN'
                errors.append(arm+': original wrapper completion not established')
                break
            frozen[arm] = inventory(plan.output,[p.name for p in plan.output.iterdir()])
            phases[arm] = 'COMPLETED'
            checks[arm] = inspect_output(plan,result)
        except BaseException as failure:
            LOGGER.exception('Structural arm did not qualify')
            errors.append(arm+': '+type(failure).__name__+': '+str(failure))
            if arm not in frozen:
                incomplete = True
                phases[arm] = 'INCOMPLETE_DESCENDANTS_UNKNOWN'
                break
    if not incomplete:
        try:
            for arm,digests in frozen.items():
                require(inventory(plans[arm].output,[p.name for p in plans[arm].output.iterdir()]) == digests,
                        '/artifacts','earlier arm changed')
            for plan in plans.values():
                _check_product(plan.product)
                _check_tools(plan.tools)
                require(file_hash(plan.plan_path)==plan.plan_sha256,'/plan','launch plan changed')
        except (OSError,ValueError) as failure:
            errors.append(str(failure))
    status = 'INCOMPLETE' if incomplete else ('FAIL' if errors else 'PASS')
    receipt = output/'structural-component.json'
    atomic_json(receipt,{'schema':'m6a_structural_component_v1','status':status,'errors':[e for e in errors],
        'obligation':'48:m6a.plant_art_structural','phases':{k:v for k,v in phases.items()},
        'checks':{k:v for k,v in checks.items()},'visual_status':'NOT_EVALUATED',
        'worlds':'NO_SIMULATION_WORLD','ordinary_safety':'NOT_APPLICABLE_SOURCE_BOUND',
        'godot_pid':None,'native_child_pid_evidence':'NOT_RECORDED','lane_release_claim':False,
        'terminal_inventory_complete':not incomplete,'artifacts':{k:{n:h for n,h in v.items()} for k,v in frozen.items()},
        'source_sha':{arm:products.arm(arm).source_sha for arm in phases},
        'plans':{arm:plan.plan_sha256 for arm,plan in plans.items()},
        'no_world_source_pins':dict(PINS),'execution_mode':'windowed_real_renderer',
        'coordinator_sources':{name:{'path':str(path),'sha256':digest} for name,(path,digest) in LOADED_SOURCES.items()
            if name+'.py' in dict(tools.files)},
        'resolution_policy':'sole_pinned_extension_class_map_and_bound_uid_import_paths_v2',
        'publication_authority':False})
    return ComponentResult(receipt,status)



@dataclass(frozen=True, slots=True)
class OwnedAdmission:
    products: Products
    tools: Tooling


@dataclass(frozen=True, slots=True)
class RecoveryRequired:
    attempt: str
    arm: str
    paths: tuple[Path,...]


@dataclass(frozen=True, slots=True)
class SettlementEvidence:
    attempt: str
    recovered: bool


@dataclass(frozen=True, slots=True)
class SettledStructural:
    evidence: SettlementEvidence


@dataclass(frozen=True, slots=True)
class StillUnknown:
    attempt: str
    reason: str


@dataclass(frozen=True, slots=True)
class OwnedPlan:
    product: ProbeProduct
    path: Path
    digest: str
    output: Path


def check_owned_helpers(product: ProbeProduct, tools: Tooling) -> None:
    """Bind actual loaded helper closure and each wrapper's materialized copy."""
    coordinator_sources(tools)
    require(dict(tools.files).get('gate_oracle_child.py')=='af1d16e2774710ad2800c3ebd6479454657a460067651cec1f0f5a11f43ad785','/helper','unreviewed owned helper')
    for name in import_closure(tools.root,'gate_oracle_child.py'):
        digest=dict(tools.files).get(name)
        require(digest is not None,'/helper','unsealed helper dependency')
        module=sys.modules.get(name.removesuffix('.py'))
        require(module is not None and module.__file__ is not None,'/helper','missing loaded dependency')
        assert module is not None and module.__file__ is not None
        require(file_hash(Path(module.__file__).resolve())==digest,'/helper','ambient helper differs')
        target=product.input_root/'tools/v8'/name
        require(dict(product.input_files).get('tools/v8/'+name)==digest and file_hash(target)==digest,
                '/helper','wrapper helper copy differs from authenticated loaded helper')


def _owned_plan(product: ProbeProduct, tools: Tooling, output: Path) -> OwnedPlan:
    _check_product(product);_check_tools(tools);check_owned_helpers(product,tools)
    output.mkdir();(output/'evidence').mkdir()
    path=output.parent/(output.name+'-owned-plan.json')
    atomic_json(path,_owned_plan_document(product,tools,output))
    return OwnedPlan(product,path,file_hash(path),output)


def _owned_plan_document(product: ProbeProduct, tools: Tooling, output: Path) -> dict[str, Json]:
    """One closed recipe shared by launch and historical admission."""
    inputs={str(product.input_root/name):sha for name,sha in product.input_files}
    inputs.update({str(tools.root/name):sha for name,sha in tools.files})
    inputs.update({str(tools.python):tools.python_sha256,str(tools.bash):tools.bash_sha256,
                   str(product.executable):product.executable_sha256})
    environment={'HOME':str(Path.home()),'__CF_USER_TEXT_ENCODING':f'0x{os.getuid():X}:0x3:0x33',
        'PATH':tools.path,'PYTHONNOUSERSITE':'1','PYTHONDONTWRITEBYTECODE':'1','PYTHONCOERCECLOCALE':'0'}
    return {'schema':oracle_child.SCHEMA,'mode':'structural','root':str(product.input_root),
        'evidence':str(output/'evidence'),'log':str(output/'plant-art-structural.log'),
        'argv':[str(arg) for arg in structural_command(str(product.executable))[3:]],
        'inputs':{k:v for k,v in inputs.items()},'environment':dict(environment)}


def _owned_join(plan: OwnedPlan, result: oracle_child.OwnedResult, tools: Tooling, lane_fd: int) -> oracle_child.OwnedResult:
    return _owned_chain(plan,result,tools,os.getpid(),lane_fd)


def _owned_chain(plan: OwnedPlan, result: oracle_child.OwnedResult, tools: Tooling,
                 owner_pid: int, lane_fd: int) -> oracle_child.OwnedResult:
    check_owned_helpers(plan.product,tools)
    require(owner_pid>0 and lane_fd>2,'/owner','invalid recorded owner identity')
    require(file_hash(plan.path)==plan.digest,'/plan','original plan changed')
    checked=oracle_child.inspect(plan.path,plan.digest,result.wrapper_pid,result.wrapper_exit)
    require(checked.settlement=='SETTLED','/settlement',checked.error)
    evidence=plan.output/'evidence'
    require({p.name for p in evidence.iterdir()}=={'wrapper-start.json','wrapper-terminal.json','native-start.json','native-terminal.json','log-closed.json'},'/receipts','incomplete/extra owned chain')
    start=document((evidence/'wrapper-start.json').read_text())
    start.field('parent_pid').equals(owner_pid)
    start.field('argv').equals([str(arg) for arg in structural_command(str(plan.product.executable))])
    launch=start.field('environment').mapping()
    environment=document(plan.path.read_text()).field('environment').mapping()
    control=start.field('environment').field(oracle_child.CONTROL).text()
    require(control.isdecimal(),'/control','invalid startup descriptor')
    expected={**environment,oracle_child.PLAN:str(plan.path),oracle_child.HASH:plan.digest,
              oracle_child.CONTROL:control,FD_ENV:str(lane_fd)}
    require(launch==expected,'/environment','wrong actual parent launch')
    document((evidence/'wrapper-terminal.json').read_text()).field('error').equals('')
    return checked


@dataclass(frozen=True, slots=True)
class OwnedComponentResult:
    receipt: Path
    status: str
    receipt_sha256: str
    files: tuple[tuple[str,str],...]


class StructuralSettledFailure(Exception):
    """Publication failed after this source owner confirmed descriptor closure."""
    def __init__(self, original: BaseException, diagnostic: BaseException | None, receipt: Path | None) -> None:
        super().__init__('structural settled finalization failed; original exceptions retained')
        self.original: BaseException=original
        self.diagnostic: BaseException | None=diagnostic
        self.receipt: Path | None=receipt


class StructuralReleaseUnknown(Exception):
    """Children settled; actual owner retained because descriptor closure is uncertain."""
    def __init__(self, owner: 'StructuralLaneOwner', original: BaseException,
                 diagnostic: BaseException | None) -> None:
        super().__init__('structural settled descriptor disposition uncertain; actual owner retained')
        self.owner: StructuralLaneOwner=owner
        self.original: BaseException=original
        self.diagnostic: BaseException | None=diagnostic


def _component_files(output: Path, *, terminal: bool) -> dict[str,str]:
    names=[path.name for path in output.iterdir() if terminal or path.name!='structural-owned-component.json']
    return inventory(output,names) if names else {}


class StructuralLaneOwner:
    """One live attempt owns its descriptor until authenticated settlement.

    Deliberately no destructor/context manager: UNKNOWN is a live recovery state.
    The standalone driver must retain this object; OS death is outside this guarantee.
    """
    def __init__(self, output: Path, admission: OwnedAdmission, descriptor: int) -> None:
        self.output: Path=output
        self.admission: OwnedAdmission=admission
        self.descriptor: int=descriptor
        self.attempt: str=uuid.uuid4().hex
        self.state: Literal['ACQUIRED','RUNNING','SETTLED','RECOVERY_REQUIRED','CLOSED']='ACQUIRED'
        self.plans: dict[str,OwnedPlan]={}
        self.results: dict[str,oracle_child.OwnedResult]={}
        self.frozen: dict[str,dict[str,str]]={}
        self.errors: list[str]=[]
        self.retained_errors: list[BaseException]=[]
        self.checks: dict[str,int]={}
        self.recovery: RecoveryRequired | None=None
        self.evidence: SettlementEvidence | None=None
        self.original_receipts: tuple[tuple[Path,str],...]=()
        self.dispatched: bool=False
        self.recovery_completion: oracle_child.OwnedResult | None=None
        self.event: int=0
        self.active_arm: str | None=None

    @classmethod
    def acquire(cls, output: Path, admitted_inputs: OwnedAdmission) -> 'StructuralLaneOwner':
        require(FD_ENV not in os.environ,'/lane','nested/inherited owner unsupported')
        require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh output required')
        real_path(output.parent,directory=True)
        roots=(output,admitted_inputs.tools.root,admitted_inputs.products.base.input_root,admitted_inputs.products.candidate.input_root)
        require(all(not a.is_relative_to(b) and not b.is_relative_to(a) for i,a in enumerate(roots) for b in roots[i+1:]),'/roots','overlapping ownership roots')
        descriptor=os.open(LOCK,os.O_RDWR|os.O_CREAT,0o600)
        try:
            fcntl.flock(descriptor,fcntl.LOCK_EX|fcntl.LOCK_NB)
            held=os.fstat(descriptor);canonical=LOCK.stat()
            require((held.st_dev,held.st_ino)==(canonical.st_dev,canonical.st_ino),'/lane','changed lock inode')
        except BaseException:
            os.close(descriptor)
            raise
        return cls(output,admitted_inputs,descriptor)

    def _settled(self, recovered: bool=False) -> SettledStructural:
        self.state='SETTLED';self.evidence=SettlementEvidence(self.attempt,recovered)
        return SettledStructural(self.evidence)

    def _unknown(self, arm: str, reason: str) -> RecoveryRequired:
        self.state='RECOVERY_REQUIRED';self.errors.append(reason)
        plan=self.plans[arm]
        self.recovery=RecoveryRequired(self.attempt,arm,(plan.path,plan.output/'evidence'))
        observed: list[tuple[Path,str]]=[]
        for path in (plan.output/'evidence').iterdir():
            try:observed.append((path,file_hash(path)))
            except (ValueError,OSError) as error:self.errors.append('partial observation: '+str(error))
        self.original_receipts=tuple(observed)
        self.record_status()
        return self.recovery

    def record_status(self) -> None:
        self.event+=1
        atomic_json(self.output/f'ownership-event-{self.event}.json',{'schema':'structural_live_owner_v1',
            'attempt':self.attempt,'owner_pid':os.getpid(),'state':self.state,'errors':[e for e in self.errors],
            'kernel_ownership':'live_descriptor_only_not_crash_resilient','final_inventory':False})

    def run(self, products: Products, tools: Tooling) -> SettledStructural | RecoveryRequired:
        require(self.state=='ACQUIRED' and OwnedAdmission(products,tools)==self.admission,'/owner','wrong admission/state')
        try:
            return self._run(products,tools)
        except BaseException as error:  # noqa: BLE001 - retain live ownership on unclassified faults
            self.retained_errors.append(error)
            if self.active_arm is not None:
                try:return self._unknown(self.active_arm,'unclassified dispatch fault; original exception retained in live owner')
                except BaseException as recording:  # noqa: BLE001 - diagnostics cannot release live ownership
                    self.retained_errors.append(recording)
                    self.state='RECOVERY_REQUIRED';self.errors.append('ownership recording failed; original exception retained in live owner')
                    plan=self.plans[self.active_arm]
                    self.recovery=RecoveryRequired(self.attempt,self.active_arm,(plan.path,plan.output/'evidence'))
                    return self.recovery
            self.errors.append('NOT_LAUNCHED: unclassified fault; original exception retained in live owner')
            return self._settled()

    def _run(self, products: Products, tools: Tooling) -> SettledStructural | RecoveryRequired:
        require(self.state=='ACQUIRED' and OwnedAdmission(products,tools)==self.admission,'/owner','wrong admission/state')
        self.state='RUNNING';self.output.mkdir()
        try:
            for arm in ('base','candidate'):
                self.plans[arm]=_owned_plan(products.arm(arm),tools,self.output/arm)
        except (OSError,ValueError,KeyboardInterrupt,SystemExit) as error:
            self.errors.append('NOT_LAUNCHED: '+str(error))
            return self._settled()
        for arm,plan in self.plans.items():
            dispatched=False
            try:
                _check_product(plan.product);_check_tools(tools);check_owned_helpers(plan.product,tools)
                self.active_arm=arm
                self.dispatched=True;dispatched=True
                result=oracle_child.capture(plan.path,plan.digest,self.descriptor)
                self.results[arm]=result
                if result.settlement!='SETTLED':return self._unknown(arm,result.error)
                checked=_owned_join(plan,result,tools,self.descriptor)
            except (OSError,ValueError,KeyboardInterrupt,SystemExit) as error:
                if not dispatched:
                    self.errors.append(arm+' NOT_LAUNCHED: '+str(error));return self._settled()
                return self._unknown(arm,str(error))
            self.frozen[arm]=inventory(plan.output,[p.name for p in plan.output.iterdir()])
            try:
                require(checked.native_exit==0 and checked.wrapper_exit==0,'/native','settled native failure')
                raw=(plan.output/'plant-art-structural.log').read_bytes().decode('utf-8')
                self.checks[arm]=_renderer_oracle(raw)
            except (OSError,ValueError) as error:self.errors.append(arm+': '+str(error))
            self.active_arm=None
        return self._settled()

    def inspect_recovery(self, token: RecoveryRequired) -> StillUnknown | SettlementEvidence:
        require(self.state=='RECOVERY_REQUIRED' and token is self.recovery,'/recovery','foreign attempt/token')
        plan=self.plans[token.arm];result=self.results.get(token.arm)
        if result is None:return StillUnknown(self.attempt,'no actual wrapper wait identity returned')
        try:
            require(len(self.original_receipts)==5,'/recovery','original completion files unavailable')
            require(all(file_hash(path)==sha for path,sha in self.original_receipts),'/recovery','original evidence changed')
            self.recovery_completion=_owned_join(plan,result,self.admission.tools,self.descriptor)
            self.frozen[token.arm]=inventory(plan.output,[p.name for p in plan.output.iterdir()])
        except (OSError,ValueError,KeyboardInterrupt,SystemExit) as error:
            self.errors.append('recovery inspection: '+str(error))
            return StillUnknown(self.attempt,str(error))
        self.errors.append('recovered original completion after UNKNOWN; workload not retried')
        return self._settled(True).evidence

    def finish(self, evidence: SettledStructural | SettlementEvidence) -> ComponentResult:
        result=self.finish_owned(evidence)
        return ComponentResult(result.receipt,result.status)

    def finish_owned(self, evidence: SettledStructural | SettlementEvidence) -> OwnedComponentResult:
        selected=evidence.evidence if isinstance(evidence,SettledStructural) else evidence
        require(self.state=='SETTLED' and selected is self.evidence,'/finish','not this owner authenticated settlement')
        try:
            result=self._publish_owned(selected)
        except BaseException as original:
            self.retained_errors.append(original)
            try:
                os.close(self.descriptor)
            except BaseException as diagnostic:
                self.retained_errors.append(diagnostic)
                raise StructuralReleaseUnknown(self,original,diagnostic) from diagnostic
            self.state='CLOSED'
            raise StructuralSettledFailure(original,None,None) from original
        try:
            os.close(self.descriptor)
        except BaseException as original:
            self.retained_errors.append(original)
            raise StructuralReleaseUnknown(self,original,None) from original
        self.state='CLOSED'
        return result

    def _publish_owned(self, selected: SettlementEvidence) -> OwnedComponentResult:
        try:
            _check_tools(self.admission.tools)
            for product in (self.admission.products.base,self.admission.products.candidate):_check_product(product)
            for arm,plan in self.plans.items():
                require(file_hash(plan.path)==plan.digest,'/plan','admitted plan drift')
                if arm in self.frozen:
                    require(inventory(plan.output,[p.name for p in plan.output.iterdir()])==self.frozen[arm],'/artifacts','settled arm drift')
        except (OSError,ValueError) as error:self.errors.append('postflight: '+str(error))
        status='FAIL' if self.errors else 'PASS'
        receipt=self.output/'structural-owned-component.json'
        complete=_component_files(self.output,terminal=False)
        atomic_json(receipt,{'schema':'structural_owned_component_v1','status':status,'attempt':self.attempt,
            'owner_pid':os.getpid(),'lane_fd':self.descriptor,'errors':[e for e in self.errors],'recovered':selected.recovered,
            'component_files':dict(complete),
            'source_sha':{arm:self.admission.products.arm(arm).source_sha for arm in ('base','candidate')},
            'plans':{arm:{'path':str(plan.path),'sha256':plan.digest} for arm,plan in self.plans.items()},
            'tooling':{name:sha for name,sha in self.admission.tools.files},
            'actual_helper_sources':{name:{'path':str(path),'sha256':sha} for name,(path,sha) in LOADED_SOURCES.items() if name+'.py' in dict(self.admission.tools.files)},
            'recovery_native_pid':self.recovery_completion.native_pid if self.recovery_completion is not None else None,
            'original_completions':{arm:{'settlement':row.settlement,'error':row.error,'wrapper_pid':row.wrapper_pid,'wrapper_exit':row.wrapper_exit} for arm,row in self.results.items()},
            'dispatched':self.dispatched,'checks':{k:v for k,v in self.checks.items()},
            'native_pids':{arm:row.native_pid for arm,row in self.results.items()},
            'worlds':'NO_SIMULATION_WORLD','visual_status':'NOT_EVALUATED','publication_authority':False,
            'execution_adapter':'owned_direct_wrapper_not_historical_original_run',
            'artifacts':{arm:{k:v for k,v in values.items()} for arm,values in self.frozen.items()}})
        digest=file_hash(receipt)
        require(_component_files(self.output,terminal=False)==complete,'/artifacts','component changed during publication')
        files=_component_files(self.output,terminal=True)
        require(files.get(receipt.name)==digest,'/receipt','terminal changed during publication')
        return OwnedComponentResult(receipt,status,digest,tuple(sorted(files.items())))


def authenticate_owned_result(products: Products, tools: Tooling, receipt: Path,
                              expected_digest: str) -> OwnedComponentResult:
    """Recheck captured historical endpoints; no live ownership or build admission."""
    real_path(receipt,directory=False)
    require(receipt.name=='structural-owned-component.json','/receipt','wrong owned endpoint')
    require(file_hash(receipt)==expected_digest,'/receipt','captured terminal changed')
    output=receipt.parent
    files=_component_files(output,terminal=True)
    require(files.get(receipt.name)==expected_digest,'/receipt','terminal changed during inventory')
    node=document(receipt.read_text())
    require(set(node.mapping())=={'schema','status','attempt','owner_pid','lane_fd','errors','recovered',
        'component_files','source_sha','plans','tooling','actual_helper_sources','recovery_native_pid',
        'original_completions','dispatched','checks','native_pids','worlds','visual_status',
        'publication_authority','execution_adapter','artifacts'},'/receipt','owned fields differ')
    for key,value in {'schema':'structural_owned_component_v1','worlds':'NO_SIMULATION_WORLD',
        'visual_status':'NOT_EVALUATED','publication_authority':False,
        'execution_adapter':'owned_direct_wrapper_not_historical_original_run'}.items():
        node.field(key).equals(value)
    status=text(node.field('status'));require(status in ('PASS','FAIL'),'/status','not a complete owned endpoint')
    attempt=text(node.field('attempt'));require(re.fullmatch('[0-9a-f]{32}',attempt) is not None,'/attempt','invalid attempt')
    owner_pid=integer(node.field('owner_pid'));lane_fd=integer(node.field('lane_fd'))
    require(owner_pid>0 and lane_fd>2,'/owner','invalid owner identity')
    errors=[text(row) for row in node.field('errors').array()]
    recovered=node.field('recovered').value;dispatched=node.field('dispatched').value
    require(type(recovered) is bool and type(dispatched) is bool,'/state','boolean state required')
    assert type(recovered) is bool and type(dispatched) is bool
    node.field('component_files').equals({name:sha for name,sha in files.items() if name!=receipt.name})
    node.field('source_sha').equals({arm:products.arm(arm).source_sha for arm in ('base','candidate')})
    node.field('tooling').equals(dict(tools.files))
    node.field('actual_helper_sources').equals({name:{'path':str(path),'sha256':sha}
        for name,(path,sha) in LOADED_SOURCES.items() if name+'.py' in dict(tools.files)})
    _check_tools(tools)
    for product in (products.base,products.candidate):
        _check_product(product);check_owned_helpers(product,tools)
    plans=node.field('plans').mapping();completions=node.field('original_completions').mapping()
    frozen=node.field('artifacts').mapping();checks=node.field('checks').mapping()
    pids=node.field('native_pids').mapping()
    require(list(plans) in ([],['base'],['base','candidate']),'/plans','not original paired plans')
    require(list(completions) in ([],['base'],['base','candidate']) and set(completions)<=set(plans),
            '/completions','not original launch prefix')
    require(set(frozen)==set(completions)==set(pids) and set(checks)<=set(completions),
            '/artifacts','completed arm accounting differs')
    require(dispatched==bool(completions),'/dispatch','dispatch/completion mismatch')
    require(status!='PASS' or (not errors and not recovered and set(checks)==set(plans)==set(completions)=={'base','candidate'}),
            '/status','incomplete or failed PASS')
    require(status!='FAIL' or bool(errors),'/errors','failure has no retained error')
    expected_names={'structural-owned-component.json'}
    completed: dict[str,oracle_child.OwnedResult]={}
    for arm in plans:
        product=products.arm(arm)
        path=output/(arm+'-owned-plan.json')
        plan_row=node.field('plans').field(arm)
        require(set(plan_row.mapping())=={'path','sha256'},'/plan','plan fields differ')
        plan_row.field('path').equals(str(path))
        digest=text(plan_row.field('sha256'))
        require(file_hash(path)==digest,'/plan','plan changed')
        document(path.read_text()).equals(_owned_plan_document(product,tools,output/arm))
        expected_names.update({arm,path.name})
        if arm not in completions:
            require(not any((output/arm/'evidence').iterdir()) and not (output/arm/'plant-art-structural.log').exists(),
                    '/plans','unlaunched arm has child evidence')
            continue
        row=node.field('original_completions').field(arm)
        require(set(row.mapping())=={'settlement','error','wrapper_pid','wrapper_exit'},'/completion','completion fields differ')
        recorded_settlement=text(row.field('settlement'));recorded_error=row.field('error').text()
        require(recorded_settlement=='SETTLED' and not recorded_error or
                recovered and arm==list(completions)[-1] and recorded_settlement=='UNKNOWN',
                '/completion','unresolved original completion')
        wrapper_pid=integer(row.field('wrapper_pid'));wrapper_exit=integer(row.field('wrapper_exit'))
        require(wrapper_pid>0 and wrapper_pid!=owner_pid,'/pid','invalid wrapper identity')
        plan=OwnedPlan(product,path,digest,output/arm)
        observed=oracle_child.OwnedResult('SETTLED',wrapper_pid,None,wrapper_exit,None,'')
        checked=_owned_chain(plan,observed,tools,owner_pid,lane_fd)
        completed[arm]=checked
        if recorded_settlement=='SETTLED':node.field('native_pids').field(arm).equals(checked.native_pid)
        else:require(node.field('native_pids').field(arm).value in (None,checked.native_pid),'/pid','recovered original pid differs')
        node.field('artifacts').field(arm).equals({name:sha for name,sha in inventory(plan.output,[p.name for p in plan.output.iterdir()]).items()})
        require({p.name for p in plan.output.iterdir()}=={'evidence','plant-art-structural.log'},'/arm','extra arm output')
        _historical_chain_fields(plan)
        if arm in checks:
            require(checked.native_exit==0 and checked.wrapper_exit==0,'/oracle','nonzero checked arm')
            node.field('checks').field(arm).equals(_renderer_oracle((plan.output/'plant-art-structural.log').read_bytes().decode('utf-8')))
        require(status!='PASS' or checked.native_exit==checked.wrapper_exit==0,'/native','failed PASS arm')
    if recovered:
        require(status=='FAIL' and bool(completed),'/recovery','invalid recovered endpoint')
        node.field('recovery_native_pid').equals(completed[list(completed)[-1]].native_pid)
    else:
        node.field('recovery_native_pid').equals(None)
    events=sorted((path for path in output.iterdir() if path.name.startswith('ownership-event-')),
                  key=lambda path: int(path.stem.removeprefix('ownership-event-')))
    for index,path in enumerate(events,1):
        require(path.name==f'ownership-event-{index}.json','/events','event sequence differs')
        event=document(path.read_text())
        require(set(event.mapping())=={'schema','attempt','owner_pid','state','errors','kernel_ownership','final_inventory'},
                '/events','event fields differ')
        for key,value in {'schema':'structural_live_owner_v1','attempt':attempt,'owner_pid':owner_pid,
            'kernel_ownership':'live_descriptor_only_not_crash_resilient','final_inventory':False}.items():
            event.field(key).equals(value)
        require(text(event.field('state')) in ('ACQUIRED','RUNNING','SETTLED','RECOVERY_REQUIRED'),'/events','invalid live state')
        for error in event.field('errors').array():_=text(error)
        expected_names.add(path.name)
    require({p.name for p in output.iterdir()}==expected_names,'/output','extra or missing component files')
    for product in (products.base,products.candidate):
        _check_product(product);check_owned_helpers(product,tools)
    _check_tools(tools)
    require(_component_files(output,terminal=True)==files,'/artifacts','historical bytes changed during authentication')
    return OwnedComponentResult(receipt,status,expected_digest,tuple(sorted(files.items())))


def _historical_chain_fields(plan: OwnedPlan) -> None:
    fields={
        'wrapper-start.json':{'schema','plan_sha256','parent_pid','wrapper_pid','argv','environment'},
        'wrapper-terminal.json':{'schema','plan_sha256','parent_pid','wrapper_pid','wrapper_exit','error'},
        'native-start.json':{'schema','plan_sha256','wrapper_pid','parent_pid','native_pid','argv','environment','stream_start'},
        'native-terminal.json':{'schema','plan_sha256','wrapper_pid','parent_pid','native_pid','native_exit',
            'timed_out','timeout_seconds','settlement','error','stream_start','stream_end','stream_sha256',
            'started_monotonic_ns','ended_monotonic_ns'},
        'log-closed.json':{'schema','plan_sha256','sha256'}}
    for name,keys in fields.items():
        row=document((plan.output/'evidence'/name).read_text())
        require(set(row.mapping())==keys,'/chain','child receipt fields differ')
        row.field('schema').equals(oracle_child.SCHEMA)


def run_owned_entrypoint(products: Products, tools: Tooling, output: Path, control_fd: int) -> ComponentResult:
    result=run_owned_result(products,tools,output,control_fd)
    return ComponentResult(result.receipt,result.status)


def _finish_entrypoint(owner: StructuralLaneOwner, evidence: SettledStructural | SettlementEvidence) -> OwnedComponentResult:
    try:
        return owner.finish_owned(evidence)
    except StructuralReleaseUnknown as failure:
        # Retain this actual owner. Never retry close against a potentially reused fd.
        while True:
            with suppress(BaseException):
                try:
                    signal.pause()
                except BaseException as interruption:
                    failure.owner.retained_errors.append(interruption)
                    raise


def run_owned_result(products: Products, tools: Tooling, output: Path, control_fd: int) -> OwnedComponentResult:
    """Retain a strong live owner; EOF does not authorize dropping its lane."""
    require(type(control_fd) is int and control_fd>2 and stat.S_ISFIFO(os.fstat(control_fd).st_mode),'/control','dedicated recovery pipe required')
    owner=StructuralLaneOwner.acquire(output,OwnedAdmission(products,tools))
    outcome=owner.run(products,tools)
    if isinstance(outcome,SettledStructural):return _finish_entrypoint(owner,outcome)
    pending: bytes=b''
    pause_after_error=False
    while True:
        try:
            if pause_after_error:
                signal.pause()
                pause_after_error=False
            data=os.read(control_fd,1024)
            if not data:
                try:owner.record_status()
                except BaseException as error:  # noqa: BLE001 - retain live ownership on unclassified faults
                    owner.retained_errors.append(error)
                    owner.errors.append('EOF status unavailable; original exception retained in live owner')
                while True:signal.pause()
            pending+=data
            require(len(pending)<=4096,'/control','oversized recovery request')
            while b'\n' in pending:
                pieces: list[bytes]=pending.split(b'\n',1)
                command: bytes=pieces[0];pending=pieces[1]
                if command==b'inspect':
                    inspected=owner.inspect_recovery(outcome)
                    if isinstance(inspected,SettlementEvidence):return _finish_entrypoint(owner,inspected)
                owner.record_status()
        except StructuralSettledFailure:
            raise
        except BaseException as error:  # noqa: BLE001 - retain live ownership on unclassified faults
            owner.retained_errors.append(error)
            owner.errors.append('recovery controller fault; original exception retained in live owner');pending=b''
            pause_after_error=isinstance(error,OSError)


if __name__ == '__main__':
    require(len(sys.argv)==4 and sys.argv[1]=='--capture','/argv','internal closed capture entry required')
    raise SystemExit(capture_child(Path(sys.argv[2]),sys.argv[3]))
