"""Original no-world plant-art renderer; visual and native ancestry remain unqualified."""
import ast
import hashlib
import logging
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

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

LOGGER = logging.getLogger(__name__)
# Snapshot actual imported sources now, not an unrelated later copy of the tooling.
LOADED_SOURCES = {
    name: (Path(module.__file__).resolve(), hashlib.sha256(Path(module.__file__).read_bytes()).hexdigest())
    for name,module in tuple(sys.modules.items())
    if hasattr(module, '__file__') and module.__file__ is not None and module.__file__.endswith('.py')
}

# Source-reviewed scene/autoload/rebuild paths: layers never enter the scene tree.
PINS: dict[str, str] = {'scripts/world/art_hash.gd': 'e37c1a4553175450dbf598bfc3b7a0d86c51016bc2c781a6e4ef82a93e288afe', 'scripts/ui/person_names.gd': 'cd8afd961c3eb31fc6b7706ea1bdb4ea0c9334e229cff590974aa6e2de422791', 'sim_bridge.gdextension': '6c47616cc217caf4b9dfcecfba4c6157f98b8ee8954c18602081ab8a63eb1b5a', 'project.godot': 'cd16a69aa33fbde80fb64c2bf8fb584f6d7aaef0f4b135ed92c8a35cb6061c98', 'tools/v8/m6a_plant_art_test.gd': 'c8deea193b8ffc26436213fa8bece13a912d2870dc4276ec52c3f6da634c2006', 'tools/v8/run_bounded.sh': 'd2cec75c52279860395c3d76e2e886428e3957311eb082935bfb8331bbb0fee1', 'scripts/world/colony/plant_layer.gd': '84c7bb21b959d2511028b7df42d09b10cb16888b82c65aa59e849a66d95d0fc3', 'scripts/world/art/art_data.gd': 'fd3d27da528819c6ace5ad04824c1c7f8b8a5232b9f2535f40c3d7b1160fa919', 'scripts/ui/content_context.gd': 'e8cdac02c9d5241464a2d153e51c1c1db3a30696198d255d791213f0ac31ce10', 'scripts/core/locale.gd': 'b92c662d50966ce294be3596d91ef7530a3defd59010beee2c898e2e85ae50e3', 'scripts/core/sim_reads.gd': 'b50284a716ee296186fd007e78532a2c3c2569e7ce3e06f01b1b1428e6ba1128', 'scripts/world/colony/view.gd': 'f67192ce8caefbff8971a449f1b5e001c3b787cfb72807b8546c51f42f8e95ab'}
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


def coordinator_sources(tools: Tooling) -> None:
    """Authenticate loaded coordinator dependencies before trusting their helpers."""
    for filename,digest in tools.files:
        name = filename.removesuffix('.py').replace('/', '.')
        module = sys.modules.get(name)
        if module is None:
            continue  # Legacy function-local imports are never invoked by this adapter.
        admitted = LOADED_SOURCES.get(name)
        if admitted is None or module.__file__ is None or Path(module.__file__).resolve() != admitted[0]:
            raise ValueError('unrecorded coordinator module: '+name)
        if admitted[1] != digest or hashlib.sha256(admitted[0].read_bytes()).hexdigest() != digest:
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
    backends = re.findall(r'^M6A_PLANT_ART_BACKEND display=(\S+) method=(\S+)$',raw,re.MULTILINE)
    require(backends == [('macOS','gl_compatibility')], '/backend', 'missing/duplicate/unreviewed actual renderer')
    require(not re.search(r'(?:Vulkan|Metal|Dummy).*API',raw), '/backend', 'conflicting renderer banner')
    count = validate(raw)
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


if __name__ == '__main__':
    require(len(sys.argv)==4 and sys.argv[1]=='--capture','/argv','internal closed capture entry required')
    raise SystemExit(capture_child(Path(sys.argv[2]),sys.argv[3]))
