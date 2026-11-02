# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/generic_full_gate.py --plan /absolute/fresh/output
"""Closed obligation ledger and bounded adapter dispatch, never publication authority.

The Python API executes existing direct/UI components. Build authentication,
remaining native suites and original preparation chronology are explicitly incomplete.
"""
from __future__ import annotations

import fcntl
import json
import math
import os
import signal
import stat
import sys
import uuid
from collections.abc import Callable
from contextlib import nullcontext
from dataclasses import asdict, dataclass, replace
from datetime import datetime
from pathlib import Path
from typing import Final, Literal, NoReturn, final

import generic_bench_run as bench
import generic_direct_run as direct
import generic_perf_run as perf
import generic_renderer_run as renderer
import generic_static_run as static
import generic_test_run as native
import generic_ui_fixed_run as fixed
import generic_ui_pack_run as pack
import generic_ui_smoke_run as smoke
import generic_ui_visibility_run as visibility
import generic_ui_why_run as why
from generic_bench import AuditError, Product, validate_products
from generic_declarations import Declarations
from generic_fixture_registry import (
    DIRECT_IDS,
    ExpansionContext,
    Invocation,
    Registry,
    expand,
)
from generic_fixture_run import ComponentResult, Products
from generic_gate import GateContractError, document, integer, require
from generic_observations import keys
from generic_process import atomic_json, file_hash, timestamp
from generic_sources import inventory
from generic_test_invocation import invocations
from generic_test_observations import Binding, ExpectedOccurrence
from generic_test_safety import pair
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json, Node
from mux2_gate_lock import FD_ENV, LOCK, lane

STEP_NAMES: Final = ('fmt', 'bridge_build', 'godot_import', 'test_debug', 'test_release', 'clippy', 'forbidden_math', 'gdscript_strict', 'smoke', 'identity', 'determinism', 'perf', 'bridge_api', 'docs', 'inventory_closure', 'python_unittest', 'm1_tests', 'colony_determinism', 'replay', 'uid_check', 'play_check', 'schema_docs', 'renderer_agreement', 'save_roundtrip', 'bench', 'soak', 'nav_bench', 'knowledge', 'story', 'neighbours', 'art_manifest', 'resolver_parity', 'extensibility_grep', 'mart2_render', 'steward', 'data_only_ext', 'mux2_defence', 'mux2_oracles', 'mux2_visibility', 'mux2_why', 'mux2_stall_receipt', 'baseline_isolation', 'mind_data_only', 'core_shadow', 'core_evaluation', 'mux21_regressions', 'm6a_food_fixture', 'm6a_food_data_only', 'read_data_only', 'read_preservation', 'read_diagnosis_test')
ORDER: Final = (*range(1, 37), 42, 43, *range(37, 42), *range(44, 52))
COMPONENTS: Final = ('direct', 'renderer', 'render.ground', 'render.people', 'render.trees',
    'pack', 'visibility', 'why.base', 'why.en', 'why.ko', 'toast', 'marker.layout')


@dataclass(frozen=True, slots=True)
class Obligation:
    step: int
    identity: str
    component: str | None
    fixture: str | None = None


def obligations() -> tuple[Obligation, ...]:
    """Code-derived scope; callers cannot provide a smaller required set."""
    composite = {
        3: ('import.first', 'import.second'), 4: ('workspace.debug',), 5: ('workspace.release',),
        17: ('colony_commands.release', 'colony_jobs.release', 'colony_world.release',
             'engine_colony_ffi.release', 'sim-test.lib.probe::colony.release',
             'sim-systems.lib.jobs::.release', 'sim-core.lib.colony::.release', 'locale.trf'),
        21: ('unknown_version.exit3', 'no_godot.exit2', 'bad_argument.exit4', 'main.headless', 'm1.headless', 'v7.headless'),
        24: ('save_load.release',), 28: ('colony_knowledge.release', 'f10'),
        34: ('render.ground', 'render.people', 'render.trees'),
        36: ('data_only_ext.release', 'pack'),
        37: ('steward_defence.release', 'steward_defence_fingerprint.release', 'colony_flee_mux2.release',
             'colony_wildlife.release', 'mux2_firewood.release', 'mux2_fuel_sleep.release', 'colony_steward.release',
             'v10.fixture_manifest', 'steward_defence_migration.ignored.release',
             'legacy_migration_current.ignored.release', 'b5_flora.exact_ignored.release'),
        38: tuple('mux2_'+name for name in ('camp_framing', 'visibility_validator', 'marker_layout', 'coincident',
            'badge_index', 'badge_packing', 'overlap', 'badge_grid', 'badge_aggregate', 'bake_receipt', 'badge_extent',
            'wall_cache', 'badge_local.expect_lazy', 'person_cache.expect_cache', 'extent_owner.expect_cache', 'stall.self_test')),
        46: ('gdscript.four_files', 'toast', 'marker.layout', 'why.en', 'why.ko'),
    }
    direct_fixtures = direct.DIRECT_STEP_FIXTURES
    composite[28] = ('colony_knowledge.release', *direct_fixtures[28])
    composite[48] = (*direct_fixtures[48], 'm6a.plant_art_structural')
    mapped = {23:'renderer',39:'visibility',40:'why.base'}
    result: list[Obligation] = []
    for step in ORDER:
        for name in (composite[step] if step in (28,48) else direct_fixtures.get(step, composite.get(step, (STEP_NAMES[step-1],)))):
            fixture = name if name in direct_fixtures.get(step, ()) else None
            component = 'direct' if fixture else name if name in COMPONENTS else mapped.get(step)
            result.append(Obligation(step,f'{step}:{name}',component,'bench.timed_rss' if step==25 else fixture))
    return tuple(result)


@dataclass(frozen=True, slots=True)
class Outcome:
    component: str
    status: str
    receipt: Path | None
    sha256: str | None
    error: str
    files: tuple[tuple[str, str], ...] = ()
    root: Path | None = None
    returned_status: str | None = None
    reported_status: str | None = None
    snapshot_error: str = ''


def aggregate(rows: tuple[Outcome, ...]) -> str:
    """Even all supported components cannot discharge unsupported obligations."""
    names = [row.component for row in rows]
    require(len(set(names)) == len(names) and set(names) <= set(COMPONENTS), '/components', 'unknown or duplicate result')
    return 'FAIL' if any(row.status == 'FAIL' for row in rows) else 'INCOMPLETE'


def write_plan(output: Path, inputs: Inputs | None = None) -> Path:
    require(output.is_absolute() and output == output.resolve() and not output.exists(), '/output', 'fresh canonical output required')
    output.mkdir(parents=True)
    path = output/'plan.json'
    refs = references(inputs,output) if inputs else {}
    atomic_json(path, {'schema_version': 1, 'publication_authorized': False, 'required_steps': list(range(1, 52)),
        'original_step_order': list(ORDER), 'original_order_executed': False, 'component_sequence': list(COMPONENTS), 'required_core_fixture': 'core.steward_off',
        'limits': ['Source/build authentication belongs to caller; this slice cannot certify it.',
                  'Unsupported obligations remain required; no publication or full gate PASS is possible.'],
        'obligations': [{'step': row.step, 'identity': row.identity, 'component': row.component, 'fixture': row.fixture,
                         'invocation_references': refs.get(row.fixture or '', []),
                         'reference_status': 'EXPANDED' if row.fixture in refs else 'UNRESOLVED',
                         'initial_status': 'NOT_RUN' if row.component else 'NOT_IMPLEMENTED'} for row in obligations()]})
    return path


Action = Callable[[Path], ComponentResult | perf.PerfResult]


def endpoint(name: str) -> str:
    return ('result.json' if name in {'save_roundtrip','perf','nav'} else 'receipt.json' if name in {'direct','static'} else 'fixed-ui.json' if name in fixed.FIXTURES else
            'why.json' if name.startswith('why.') else 'pack-ui.json' if name == 'pack' else name+'.json')


def authenticate(name: str, node: Node, inputs: Inputs, *, step: int | None = None) -> str:
    """Bind the actual producer's terminal schema and identity, without invented fields."""
    state = node.field('status').text()
    require(state in ({'PASS','FAIL','INCOMPLETE'} if name == 'direct' and step is not None else {'PASS','FAIL','PENDING'}), '/status', 'unsupported component status')
    if name == 'direct':
        keys(node,{'schema','status','publication_authority','full_gate_status','registry_status','registry_sha256',
            'base','candidate','declaration_manifest_sha256','products','components','errors','unresolved_backlog','limit'} |
            ({'step','obligations','fixtures','whole_step_complete','original_order_executed'} if step is not None else set()))
        for key,value in (('schema','direct_components_v1' if step is None else 'direct_step_components_v1'),('registry_sha256',inputs.registry.sha256),
            ('base',inputs.probes.base.source_sha),('candidate',inputs.probes.candidate.source_sha),
            ('declaration_manifest_sha256',inputs.declarations.source_manifest_sha256),
            ('full_gate_status','NOT_QUALIFIED'),('registry_status','INCOMPLETE_ADAPTERS')):
            node.field(key).equals(value)
        if step is not None:
            node.field('step').equals(step)
            node.field('fixtures').equals(list(direct.step_fixtures(step)))
            node.field('obligations').equals([f'{step}:{fixture}' for fixture in direct.step_fixtures(step)])
            node.field('whole_step_complete').equals(False);node.field('original_order_executed').equals(False)
        node.field('publication_authority').equals(False)
        for arm in ('base','candidate'):
            product=inputs.probes.arm(arm); row=node.field('products').field(arm)
            row.field('root').equals(str(product.input_root));row.field('executable').equals(str(product.executable))
            row.field('executable_sha256').equals(product.executable_sha256)
        _=node.field('components').array();_=node.field('unresolved_backlog').array()
    elif name == 'smoke':
        keys(node,{'schema_version','status','errors','counts','matched_companions','originals','artifacts','build_provenance','full_gate_claim'})
        node.field('schema_version').equals(1);node.field('full_gate_claim').equals(False)
        node.field('build_provenance').equals('caller_owned_not_certified_here')
        for field in ('counts','artifacts'):_=node.field(field).mapping()
        for field in ('matched_companions','originals'):_=node.field(field).array()
    else:
        common={'schema_version','status','errors','build_provenance','full_gate_claim'}
        extra = ({'producer','processes','scope'} if name in fixed.FIXTURES else
                 {'variant','classification','mechanisms','artifacts'} if name.startswith('why.') else
                 {'classification','negative_DTO_checks','selected_pack_inputs','worlds','artifacts','rendered_capture','rust_mechanism_children'} if name=='pack' else
                 {'artifacts','history_chains'} if name=='visibility' else {'processes','safety','pending','outputs','sources'})
        keys(node,common|extra);node.field('schema_version').equals(1);node.field('full_gate_claim').equals(False)
        node.field('build_provenance').equals('caller_owned_not_certified_here')
        if name in fixed.FIXTURES:
            node.field('producer').equals(name);_=node.field('processes').array()
        elif name.startswith('why.'):
            node.field('variant').equals(name.removeprefix('why.'))
            node.field('classification').equals('ordinary_boot_prefix_then_direct_building_placed_event_mechanism')
            _=node.field('mechanisms').array();_=node.field('artifacts').mapping()
        elif name=='pack':
            node.field('classification').equals('ordinary_selected_pack_continuous_world')
            node.field('selected_pack_inputs').equals({key:value for key,value in pack.PINS.items() if key.startswith(str(Path(pack.PACK).parent)+'/')})
            node.field('rendered_capture').equals('NOT_RUN_HEADLESS');_=node.field('worlds').array();_=node.field('artifacts').mapping()
        elif name=='visibility':
            _=node.field('history_chains').array();_=node.field('artifacts').mapping()
        else:
            require(name=='renderer','/component','unknown identity')
            for arm in ('base','candidate'):node.field('sources').field(arm).equals(inputs.probes.arm(arm).source_sha)
            for field in ('processes','safety','pending'):_=node.field(field).array()
            _=node.field('outputs').mapping()
    _=node.field('errors').array()
    return state


def invoke(name: str, action: Action, output: Path, inputs: Inputs, *, step: int | None = None, static_product: static.StaticProduct | None = None, static_environment: static.ExecutionEnvironment | None = None, save_products: SaveRoundtripProducts | None = None, perf_context: PerfContext | None = None, existing_output: bool = False) -> Outcome:
    require((name in COMPONENTS or (step is not None and name in {'smoke','static'}) or (step == 24 and name == 'save_roundtrip') or ((name,step) in {('perf',12),('nav',27)})) and (not output.exists() or (existing_output and name=='static' and step in (8,14,46))), '/component', 'unknown component or stale output')
    error='';returned: str | None=None;reported: str | None=None;receipt: Path | None=None;digest: str | None=None
    expected=output/endpoint(name)
    try:
        result=action(output);returned=result.status
        require(result.receipt==expected and expected==expected.resolve(strict=True),'/receipt','foreign terminal path')
        receipt=expected;digest=file_hash(expected)
        reported=authenticate_perf(expected,name,step,perf_context) if name in {'perf','nav'} else authenticate_save_roundtrip(expected,save_products) if name == 'save_roundtrip' else authenticate_static(expected,step,static_product,static_environment) if name == 'static' else authenticate(name,document(expected.read_text()),inputs,step=step if name == 'direct' else None)
        require(returned==reported,'/status','returned and on-disk status differ')
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        error=str(caught)
        if expected.is_file():
            try:
                receipt=expected;digest=file_hash(expected)
                reported=authenticate_perf(expected,name,step,perf_context) if name in {'perf','nav'} else authenticate_save_roundtrip(expected,save_products) if name == 'save_roundtrip' else authenticate_static(expected,step,static_product,static_environment) if name == 'static' else authenticate(name,document(expected.read_text()),inputs,step=step if name == 'direct' else None)
            except (AuditError,GateContractError,OSError,ValueError) as terminal_error:
                error+='; terminal: '+str(terminal_error)
    files: tuple[tuple[str,str],...]=();snapshot_error=''
    if output.exists():
        try:files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:snapshot_error=str(caught)
    return Outcome(name,'FAIL' if error or snapshot_error else reported or 'FAIL',receipt,digest,error,files,output,
                   returned,reported,snapshot_error)


def recheck(row: Outcome) -> None:
    if row.root is not None and (row.files or row.root.exists()):
        require(not row.snapshot_error,'/outputs','initial inventory failed: '+row.snapshot_error)
        require(inventory(row.root,[p.name for p in row.root.iterdir()])==dict(row.files),'/outputs','completed artifacts changed')
    if row.receipt is not None:require(file_hash(row.receipt)==row.sha256,'/receipt','completed receipt changed')


@dataclass(frozen=True, slots=True)
class Inputs:
    registry: Registry
    probes: Products
    godot: Products
    declarations: Declarations


def references(inputs: Inputs, output: Path, *, validate: bool = False) -> dict[str,list[Json]]:
    """Expand the same immutable direct invocation IDs/paths as the coordinator."""
    result: dict[str,list[Json]]={};plans: list[Invocation]=[]
    for arm in ('base','candidate'):
        product=inputs.probes.arm(arm)
        context=ExpansionContext(product.input_root,product.input_root,output/'direct/runs',
            inputs.probes.base.executable,inputs.probes.candidate.executable,product.input_root/'tools/v8/gate_f10_no_kill.json')
        plans.extend(row for row in expand(inputs.registry,context) if row.arm==arm)
    if validate:
        require(len({row.id for row in plans})==len({row.out for row in plans})==len(plans),'/plans','duplicate identity/output')
        for fixture in inputs.registry.fixtures:
            expected=2*len(fixture.repeats)*(len(fixture.seeds) if '{seed}' in fixture.argv else 1)
            require(sum(row.fixture_id==fixture.id for row in plans)==expected,'/plans','fixture multiplicity differs')
    for row in plans:
        result.setdefault(row.fixture_id,[]).append({'id':row.id,'arm':row.arm,'repeat':row.repeat,
                'seeds':list(row.seeds),'roles':list(row.roles),'cwd':str(row.cwd),'out':str(row.out),
                'argv':list(row.argv) if row.argv else None,'blockers':list(row.blockers)})
    return result


def run(inputs: Inputs, output: Path) -> Path:
    """Execute the development component sequence; never claim original gate order."""
    products=(inputs.probes.base,inputs.probes.candidate,inputs.godot.base,inputs.godot.candidate)
    for product in products:
        require(not output.is_relative_to(product.input_root) and not product.input_root.is_relative_to(output),'/output','product overlap')
    require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh canonical output required')
    plan_hash: str | None=None;errors: list[str]=[]
    rows=[Outcome(name,'NOT_RUN',None,None,'') for name in COMPONENTS]
    try:
        plan=write_plan(output,inputs);plan_hash=file_hash(plan)
        actions: dict[str,Action]={
            'direct':lambda out:direct.run_all(inputs.registry,inputs.probes,inputs.declarations,out),
            'renderer':lambda out:renderer.run_pair(renderer.RendererProducts(inputs.probes,inputs.godot),out),
            'pack':lambda out:pack.run_pair(inputs.godot,out),
            'visibility':lambda out:visibility.run_pair(inputs.godot,out),
        }
        for producer in fixed.FIXTURES:actions[producer]=lambda out,producer=producer:fixed.run_pair(inputs.godot,producer,out)
        for variant in why.VARIANTS:actions['why.'+variant]=lambda out,variant=variant:why.run_pair(inputs.godot,variant,out)
        with lane():
            for index,name in enumerate(COMPONENTS):rows[index]=invoke(name,actions[name],output/name,inputs)
    except (GateContractError,OSError,ValueError,RuntimeError) as error:
        errors.append(str(error))
    for row in rows:
        try:recheck(row)
        except (GateContractError,OSError,ValueError) as error:errors.append(str(error))
    for product in products:
        try:
            require(file_hash(product.executable)==product.executable_sha256 and
                inventory(product.input_root,[name for name,_ in product.input_files])==dict(product.input_files),'/inputs','product drift')
        except (GateContractError,OSError,ValueError) as error:errors.append(str(error))
    try:require(plan_hash is not None and file_hash(output/'plan.json')==plan_hash,'/plan','expected scope changed')
    except (GateContractError,OSError,ValueError) as error:errors.append(str(error))
    output.mkdir(parents=True,exist_ok=True)
    terminal=output/'dispatch.json';outcomes={row.component:row for row in rows}
    atomic_json(terminal,{'schema_version':2,'status':'FAIL' if errors else aggregate(tuple(rows)),
        'publication_authorized':False,'full_gate_claim':False,'plan_sha256':plan_hash,'errors':[e for e in errors],
        'actual_component_sequence':[row.component for row in rows if row.status!='NOT_RUN'],'original_order_executed':False,
        'components':[{'component':row.component,'status':row.status,'receipt':str(row.receipt) if row.receipt else None,
            'sha256':row.sha256,'error':row.error,'files':dict(row.files),'returned_status':row.returned_status,
            'reported_status':row.reported_status,'snapshot_error':row.snapshot_error} for row in rows],
        'obligations':[{'identity':row.identity,'step':row.step,'fixture':row.fixture,
            'status':'NOT_IMPLEMENTED' if row.component is None else 'COMPONENT_ONLY',
            'component_status':None if row.component is None else outcomes[row.component].status} for row in obligations()]})
    return terminal



@dataclass(frozen=True, slots=True)
class SaveRoundtripProducts:
    base: Product
    candidate: Product


@dataclass(frozen=True, slots=True)
class PerfContext:
    base: Product
    candidate: Product
    deadline: float
    purpose: Literal['publication','self_proof','defect_proof']


def perf_fixture(name: str, step: int | None) -> str:
    require((name,step) in {('perf',12),('nav',27)}, '/perf', 'unknown perf/nav step mapping')
    return 'perf.natural_quick' if name == 'perf' else 'nav.soak'


def run_perf(context: PerfContext, fixture: str, output: Path) -> perf.PerfResult:
    path=perf.run_pair(fixture,context.base,context.candidate,output,deadline=context.deadline,purpose=context.purpose)
    require(path == output/'result.json' and path == path.resolve(strict=True), '/perf', 'foreign producer endpoint')
    return perf.authenticate_result(fixture,context.base,context.candidate,output,
        receipt_sha256=file_hash(path),deadline=context.deadline,purpose=context.purpose)


def authenticate_perf(path: Path, name: str, step: int | None, context: PerfContext | None) -> str:
    require(context is not None, '/perf', 'missing explicit perf context')
    assert context is not None
    result=perf.authenticate_result(perf_fixture(name,step),context.base,context.candidate,path.parent,
        receipt_sha256=file_hash(path),deadline=context.deadline,purpose=context.purpose)
    require(result.receipt == path, '/perf', 'foreign authenticated endpoint')
    return result.status


@dataclass(frozen=True, slots=True)
class OrderedInputs:
    components: Inputs
    static_product: static.StaticProduct
    environment: static.ExecutionEnvironment
    discovery_product: static.StaticProduct
    discovery_environment: static.ExecutionEnvironment
    diagnosis_product: static.StaticProduct
    diagnosis_environment: static.ExecutionEnvironment
    save_roundtrip: SaveRoundtripProducts
    perf: PerfContext


@dataclass(frozen=True, slots=True)
class StrictContext:
    product: static.StaticProduct
    environment: static.ExecutionEnvironment


@dataclass(frozen=True, slots=True)
class BenchContext:
    base: Product
    candidate: Product
    purpose: Literal['publication','self_proof','defect_proof']
    script: Path
    script_sha256: str
    justifications: tuple[dict[str,str], ...] = ()


@dataclass(frozen=True, slots=True)
class DocsContext:
    product: static.StaticProduct
    environment: static.ExecutionEnvironment


@dataclass(frozen=True, slots=True)
class OwnedOrderedInputs(OrderedInputs):
    strict: StrictContext
    control_fd: int
    bench: BenchContext | None = None
    docs: DocsContext | None = None


def strict_bindings(context: OwnedOrderedInputs) -> None:
    strict=context.strict;product=strict.product;environment=strict.environment
    candidate=context.components.godot.candidate
    require(product.root==candidate.input_root and product.source_sha==candidate.source_sha,
            '/strict','prepared candidate source differs')
    require(environment.held_lane_fd is None,'/strict','caller cannot supply transient lane ownership')
    require(dict(environment.values).get('GODOT_BIN')==str(candidate.executable),'/strict','Godot differs')
    require(file_hash(candidate.executable)==candidate.executable_sha256,'/strict','Godot changed')
    names=static.strict_input_names(product.root)
    require(len(dict(product.inputs))==len(product.inputs) and inventory(product.root,names)==dict(product.inputs),
            '/strict','complete strict input scope differs')
    for _,path,digest in environment.tools:
        require(file_hash(path)==digest,'/strict/tools','tool changed')
    for path,digest in environment.external_inputs:
        require(file_hash(path)==digest,'/strict/external','dependency changed')
    _=authenticate_prior_chain(context,environment.prior_receipts)


class _StrictOwner:
    """One actual descriptor; UNKNOWN keeps this owner alive, never a PID recovery."""
    ownership_label: str = 'strict'

    def __init__(self, control_fd: int, output: Path) -> None:
        require(FD_ENV not in os.environ,'/lane','inherited ownership unsupported')
        self.descriptor: int=os.open(LOCK,os.O_RDWR|os.O_CREAT,0o600)
        try:
            fcntl.flock(self.descriptor,fcntl.LOCK_EX|fcntl.LOCK_NB)
            held=os.fstat(self.descriptor);canonical=LOCK.stat()
            require(stat.S_ISREG(held.st_mode) and (held.st_dev,held.st_ino)==(canonical.st_dev,canonical.st_ino),
                    '/lane','canonical lane changed')
        except BaseException:
            os.close(self.descriptor)
            raise
        self.control_fd: int=control_fd
        self.output: Path=output
        self.attempt: str=uuid.uuid4().hex
        self.retained_error: BaseException | None=None
        self.event: int=0

    def status(self) -> None:
        self.event+=1
        atomic_json(self.output.parent/f'{self.ownership_label}-ownership-{self.event}.json',{
            'schema':f'{self.ownership_label}_ordered_live_owner_v1','attempt':self.attempt,'owner_pid':os.getpid(),
            'state':'INCOMPLETE_DESCENDANTS_UNKNOWN','final_inventory':False,
            'recovery_authority':'NONE_CURRENT_ADAPTER','publication_authorized':False})

    def hold(self, error: BaseException) -> NoReturn:
        self.retained_error=error
        try:self.status()
        except BaseException as diagnostic:  # noqa: BLE001 - diagnostics cannot release unknown descendants
            self.retained_error=diagnostic
        eof=False
        while True:
            try:
                if eof:
                    signal.pause()
                    continue
                data=os.read(self.control_fd,4096)
                eof=not data
                # Status/inspection are observations only; this adapter has no recovery witness.
                self.status()
            except BaseException as diagnostic:  # noqa: BLE001 - retain actual lane through controller faults
                self.retained_error=diagnostic
                if isinstance(diagnostic,OSError):eof=True


@final
class _BenchOwner(_StrictOwner):
    ownership_label: str = 'bench'


def bench_bindings(context: OwnedOrderedInputs) -> BenchContext:
    bound=context.bench
    require(bound is not None,'/bench','explicit owned bench context required')
    assert bound is not None
    require(bound.purpose==context.components.declarations.purpose,'/bench','purpose differs')
    validate_products(bound.base,bound.candidate,bound.purpose)
    for product,probe in ((bound.base,context.components.probes.base),(bound.candidate,context.components.probes.candidate)):
        require(product.root==probe.input_root and product.source_sha==probe.source_sha,'/bench','ordered source differs')
        require(product.binary.name=='worldsim_perf','/bench','release worldsim_perf required')
        require(bool(product.input_files) and len(dict(product.input_files))==len(product.input_files)
                and inventory(product.root,[*INPUT_DIRS,*INPUT_FILES])==dict(product.input_files),'/bench','complete effective inputs differ')
    name='tools/v8/bench_script.json'
    require(bound.script==bound.candidate.root/name and bound.script==bound.script.resolve(strict=True),'/bench','original candidate script required')
    require(bound.script_sha256=='cb655db4a4207c9c8fa07a3303d0db5b309e90a80221bf5aa2eb0972bd8aaa31'
            and file_hash(bound.script)==bound.script_sha256
            and all(dict(product.input_files).get(name)==bound.script_sha256 for product in (bound.base,bound.candidate)),
            '/bench','original paired script binding differs')
    rows=[document(row) for row in context.components.registry.backlog if document(row).field('id').text()=='bench.timed_rss']
    require(len(rows)==1,'/bench','missing/duplicate registry obligation')
    rows[0].field('consumers').equals([25])
    rows[0].field('scripted_suffix').equals(['--commands','{inputs}/tools/v8/bench_script.json'])
    return bound


def bench_dependencies(bound: BenchContext) -> dict[str,Json]:
    return {'purpose':bound.purpose,'script':str(bound.script),'script_sha256':bound.script_sha256,
        'justifications':[dict(row) for row in bound.justifications],
        'build_authority':'CALLER_OWNED_NOT_CERTIFIED_HERE',
        'products':{arm:{'root':str(product.root),'source_sha':product.source_sha,'binary':str(product.binary),
            'binary_sha256':product.binary_sha256,'build_receipt':str(product.build_receipt),
            'build_receipt_sha256':product.build_receipt_sha256,'inputs':dict(product.input_files)}
            for arm,product in (('base',bound.base),('candidate',bound.candidate))}}


def bench_outcome(bound: BenchContext, output: Path, *, failed: bool) -> Outcome:
    """Only entered after source-owned settlement; retained auth never settles children."""
    error='source-owned settled benchmark failure' if failed else ''
    receipt: Path | None=None;digest: str | None=None;reported: str | None=None
    try:
        terminal=output/'result.json' if (output/'result.json').exists() else output/'failure.json'
        digest=file_hash(terminal) if terminal.exists() else None
        result=bench.authenticate_result(bound.base,bound.candidate,bound.purpose,bound.script,
            bound.script_sha256,output,receipt_sha256=digest,justifications=bound.justifications)
        receipt=result.receipt;digest=result.sha256;reported=result.status
        require(receipt is None or receipt in (output/'result.json',output/'failure.json'),'/bench','foreign terminal')
        require(receipt is None or file_hash(receipt)==digest,'/bench','terminal changed')
        if result.errors:error+='; '+ '; '.join(result.errors)
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        error+='; '+str(caught)
    files: tuple[tuple[str,str],...]=();snapshot_error=''
    if output.exists():
        try:
            names=[p.name for p in output.iterdir()]
            files=tuple(inventory(output,names).items()) if names else ()
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:snapshot_error=str(caught)
    return Outcome('bench','FAIL' if error or snapshot_error else reported or 'FAIL',receipt,digest,error,
                   files,output,None,reported,snapshot_error)


def invoke_bench(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==OrderedCall(25,'bench',('25:bench',)),'/bench','unknown obligation')
    owner=_BenchOwner(context.control_fd,output)
    entered=False;settled=False
    try:
        bound=bench_bindings(context)
        entered=True
        try:
            _=bench.run_bench_owned(bound.base,bound.candidate,bound.purpose,bound.script,
                bound.script_sha256,output,bound.justifications)
        except bench.BenchSettledFailure:
            settled=True
            return bench_outcome(bound,output,failed=True)
        settled=True
        return bench_outcome(bound,output,failed=False)
    except BaseException as error:
        if entered and not settled:owner.hold(error)
        raise
    finally:
        if not entered or settled:os.close(owner.descriptor)


@final
class _DocsOwner(_StrictOwner):
    ownership_label: str = 'docs'


def docs_bindings(context: OwnedOrderedInputs) -> DocsContext:
    bound=context.docs
    require(bound is not None,'/docs','explicit owned docs context required')
    assert bound is not None
    product=bound.product;environment=bound.environment;candidate=context.components.probes.candidate
    require(product.root==candidate.input_root and product.source_sha==candidate.source_sha,'/docs','prepared source differs')
    require(environment.held_lane_fd is None,'/docs','caller cannot supply transient lane ownership')
    names=static.docs_input_names(product.root,environment)
    require(len(dict(product.inputs))==len(product.inputs) and inventory(product.root,names)==dict(product.inputs),
            '/docs','complete docs input scope differs')
    _=static.docs_bindings(product.root,environment)
    for _,path,digest in environment.tools:require(file_hash(path)==digest,'/docs/tools','tool changed')
    for path,digest in environment.external_inputs:require(file_hash(path)==digest,'/docs/external','dependency changed')
    _=authenticate_prior_chain(context,environment.prior_receipts)
    return bound


def docs_dependencies(bound: DocsContext) -> dict[str,Json]:
    return {'root':str(bound.product.root),'source_sha':bound.product.source_sha,
        'inputs':dict(bound.product.inputs),'environment':dict(bound.environment.values),
        'tools':{name:{'path':str(path),'sha256':digest} for name,path,digest in bound.environment.tools},
        'external_inputs':{str(path):digest for path,digest in bound.environment.external_inputs},
        'current_product_world_scope':'CALLER_REQUIRED','build_provenance':'CALLER_REQUIRED',
        'lookup_bindings':static.docs_bindings(bound.product.root,bound.environment)}


def safe_docs_exception(error: BaseException | None) -> str:
    """Inspect only exact trusted exception classes and exact primitive arguments."""
    if error is None:return 'none'
    known: tuple[tuple[type[BaseException],str],...]=(
        (ValueError,'ValueError'),(OSError,'OSError'),(RuntimeError,'RuntimeError'),
        (TypeError,'TypeError'),(KeyError,'KeyError'),(AssertionError,'AssertionError'),
        (FileNotFoundError,'FileNotFoundError'),(PermissionError,'PermissionError'),
        (KeyboardInterrupt,'KeyboardInterrupt'),(SystemExit,'SystemExit'),
        (GateContractError,'GateContractError'),(AuditError,'AuditError'))
    label=next((label for kind,label in known if type(error) is kind),None)
    if label is None:return 'uninspected exception subtype'
    arguments: tuple[object,...]=error.args
    values: list[Json]=[]
    for value in arguments:
        if type(value) in (str,int,bool) or value is None:
            assert isinstance(value,(str,int,bool)) or value is None
            values.append(value)
        else:values.append('<uninspected argument>')
    return label+' args='+json.dumps(values,ensure_ascii=True,allow_nan=False)


def docs_failure_message(failure: static.DocsSettledFailure) -> str:
    # Subclasses can override field access; do not invoke their properties.
    if type(failure) is not static.DocsSettledFailure:
        return 'source-owned docs settled failure; uninspected failure subtype'
    return ('source-owned docs settled failure; original='+safe_docs_exception(failure.original)
            +'; diagnostic='+safe_docs_exception(failure.diagnostic))


def invoke_docs(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==OrderedCall(14,'static',('14:docs',)),'/docs','unknown docs obligation')
    owner=_DocsOwner(context.control_fd,output);entered=False;settled=False
    try:
        bound=docs_bindings(context)
        environment=replace(bound.environment,held_lane_fd=owner.descriptor)
        _=static.validate_environment(environment,14)
        entered=True
        try:
            result=static.run_docs_owned(bound.product,output,environment)
        except static.DocsSettledFailure as failure:
            settled=True
            message=docs_failure_message(failure)
            def failed(_out: Path) -> ComponentResult:
                raise RuntimeError(message)
            return invoke('static',failed,output,context.components,step=14,
                          static_product=bound.product,static_environment=environment,existing_output=output.exists())
        require(result.status=='PASS','/docs','unexpected owned return')
        settled=True
        return invoke('static',lambda _out:result,output,context.components,step=14,
                      static_product=bound.product,static_environment=environment,existing_output=True)
    except BaseException as error:
        if entered and not settled:owner.hold(error)
        raise
    finally:
        if not entered or settled:os.close(owner.descriptor)


def invoke_strict(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    owner=_StrictOwner(context.control_fd,output)
    entered=False;settled=False
    try:
        strict_bindings(context)
        environment=replace(context.strict.environment,held_lane_fd=owner.descriptor)
        _=static.validate_environment(environment,call.step)
        entered=True
        result=static.run_step(context.strict.product,call.step,output,environment)
        # Only a source-owned normal return crosses into ordinary completed-output validation.
        settled=True
        return invoke('static',lambda _out:result,output,context.components,step=call.step,
                      static_product=context.strict.product,static_environment=environment, existing_output=True)
    except BaseException as error:
        if entered and not settled:owner.hold(error)
        raise
    finally:
        if not entered or settled:os.close(owner.descriptor)


def run_ordered_owned(context: OrderedInputs, strict: StrictContext, output: Path, control_fd: int, *, bench: BenchContext | None = None, docs: DocsContext | None = None) -> Path:
    require(type(control_fd) is int and control_fd>2 and stat.S_ISFIFO(os.fstat(control_fd).st_mode),
            '/control','dedicated persistent controller pipe required')
    owned=OwnedOrderedInputs(context.components,context.static_product,context.environment,
        context.discovery_product,context.discovery_environment,context.diagnosis_product,
        context.diagnosis_environment,context.save_roundtrip,context.perf,strict,control_fd,bench,docs)
    return _run_ordered(owned,output)


def perf_bindings(context: OrderedInputs) -> None:
    bound=context.perf
    require(type(bound.deadline) in (int,float) and math.isfinite(bound.deadline), '/perf/deadline', 'finite absolute deadline required')
    require(bound.purpose == context.components.declarations.purpose, '/perf/purpose', 'purpose differs from declarations')
    validate_products(bound.base,bound.candidate,bound.purpose)
    for product, probe in ((bound.base,context.components.probes.base),(bound.candidate,context.components.probes.candidate)):
        require(product.source_sha == probe.source_sha, '/perf/source', 'perf product differs from ordered arm')
        require(product.binary.name == 'worldsim_perf', '/perf/binary', 'release worldsim_perf required')
        require(bool(product.input_files) and len(dict(product.input_files)) == len(product.input_files)
            and inventory(product.root,[*INPUT_DIRS,*INPUT_FILES]) == dict(product.input_files), '/perf/inputs', 'complete effective inputs differ')
    script='tools/v8/soak_script.json'
    require(script in dict(bound.base.input_files) and script in dict(bound.candidate.input_files)
        and dict(bound.base.input_files)[script] == dict(bound.candidate.input_files)[script], '/perf/script', 'paired nav script binding differs')


def perf_dependencies(context: PerfContext) -> dict[str,Json]:
    return {'deadline':context.deadline,'purpose':context.purpose,'embedded_preparation':'NOT_QUALIFIED',
        'compile_root_authority':'CALLER_OWNED_NOT_CERTIFIED_HERE',
        'products':{arm:{'root':str(product.root),'source_sha':product.source_sha,'binary':str(product.binary),
            'binary_sha256':product.binary_sha256,'build_receipt':str(product.build_receipt),
            'build_receipt_sha256':product.build_receipt_sha256,'inputs':dict(product.input_files)}
            for arm,product in (('base',context.base),('candidate',context.candidate))}}


def save_roundtrip_bindings(context: OrderedInputs) -> None:
    products = context.save_roundtrip
    require(not products.base.root.is_relative_to(products.candidate.root) and
            not products.candidate.root.is_relative_to(products.base.root), '/save/products', 'independent roots required')
    require(not products.base.binary.samefile(products.candidate.binary), '/save/products', 'shared binary inode')
    for product, probe in ((products.base,context.components.probes.base),
                           (products.candidate,context.components.probes.candidate)):
        require(product.root == probe.input_root and product.source_sha == probe.source_sha,
                '/save/products', 'source identity differs from ordered arm')
        native.verify_test_product(product,'save_load','release')


def run_save_roundtrip(products: SaveRoundtripProducts, output: Path) -> ComponentResult:
    path = native.run_pair({'save_load':products.base},{'save_load':products.candidate},output,invocation='gate24')
    return ComponentResult(path,document(path.read_bytes().decode('utf-8')).field('status').text())


def separate_save_invocations(base: Binding, candidate: Binding) -> None:
    """A reusable OS PID is not an invocation identity; callers authenticate receipts."""
    left = (base.process.output_root/'process.json',base.receipt_sha256)
    right = (candidate.process.output_root/'process.json',candidate.receipt_sha256)
    require(base.process.output_root != candidate.process.output_root and left != right,
            '/save/process','reused authenticated invocation identity')


def authenticate_save_roundtrip(path: Path, products: SaveRoundtripProducts | None) -> str:
    """Rejoin real release invocations and all save histories; never shell authority."""
    if products is None:raise GateContractError('binding','/save','missing native products')
    require(path.name == 'result.json' and path.is_absolute() and path == path.resolve(strict=True), '/save/result', 'foreign terminal path')
    digest = file_hash(path)
    node = document(path.read_bytes().decode('utf-8'))
    keys(node,{'status','broader_gate17_37','publication_authorized','invocation_plan','invocations',
               'workspace_enumeration','step_coverage','cargo_working_directory_equivalence','products',
               'coverage','invocation_coverage','limits','families','errors','output_sha256','output_inventory_complete'})
    state = node.field('status').text()
    require(state in {'COMPONENT_PASS','FAIL'},'/save/status','unsupported save result')
    for key,value in (('invocation_plan','gate24'),('broader_gate17_37','INCOMPLETE'),
                      ('workspace_enumeration','INCOMPLETE'),('step_coverage','PARTIAL'),
                      ('cargo_working_directory_equivalence','PACKAGE_ROOT'),('publication_authorized',False),
                      ('coverage','Named invocation subset; see invocation_coverage.')):
        node.field(key).equals(value)
    node.field('invocations').equals([document(json.dumps(asdict(row))).value for row in invocations('gate24')])
    node.field('invocation_coverage').equals({'save_load':46})
    node.field('limits').equals(['Other world occurrences and native suite families remain unregistered.',
                                'Observer semantic qualification is a separate gate obligation.'])
    expected_products: dict[str,Json] = {}
    for arm,product in (('base',products.base),('candidate',products.candidate)):
        expected_products[arm] = {'save_load':{'source_sha':product.source_sha,'binary':str(product.binary),
            'binary_sha256':product.binary_sha256,'artifact_receipt':str(product.build_receipt),
            'artifact_receipt_sha256':product.build_receipt_sha256}}
    node.field('products').equals(expected_products)
    errors = node.field('errors').array()
    for error in errors:_ = error.text()
    _ = node.field('families').array()
    # Failed native processes can lack either invocation receipt. Never promote them.
    if state == 'FAIL':
        require(bool(errors),'/save/errors','failed report lacks failure reason')
        require(file_hash(path) == digest,'/save/result','terminal changed')
        return state
    require(not errors,'/save/errors','passing report contains errors')
    node.field('output_inventory_complete').equals(True)
    hashes = native.freeze(path.parent)
    require(hashes.pop('result.json',None) == digest,'/save/result','foreign terminal')
    node.field('output_sha256').equals(dict(hashes))
    bindings: list[Binding] = []
    plans: list[tuple[ExpectedOccurrence,...]] = []
    for arm,product in (('base',products.base),('candidate',products.candidate)):
        receipt = path.parent/f'{arm}-save_load-invocation.json'
        document(receipt.read_bytes().decode('utf-8')).field('arm').equals(arm)
        binding = native.authenticate_invocation(receipt,product,'gate24')
        planned = native.authenticate_family('save_load',binding)
        native.native_summary(binding,'save_load')
        require(len(planned) == 46,'/save/history','incomplete save scope')
        bindings.append(binding);plans.append(tuple(row.occurrence for row in planned))
    separate_save_invocations(bindings[0],bindings[1])
    paired = pair(bindings[0],plans[0],bindings[1],plans[1])
    expected_family = [document(json.dumps({'target':'save_load','status':'PAIRED_SAFETY_PASS','declared_setup':None,
        'occurrences':[{'base':asdict(row.base),'candidate':asdict(row.candidate),'safety':row.safety}
                       for row in paired.occurrences]})).value]
    node.field('families').equals(expected_family)
    for product in (products.base,products.candidate):native.verify_test_product(product,'save_load','release')
    require(native.freeze(path.parent) == {**hashes,'result.json':digest},'/save/output','evidence changed during authentication')
    return state


def static_context(context: OrderedInputs, step: int) -> tuple[static.StaticProduct, static.ExecutionEnvironment]:
    """Keep static eight, full discovery and selected diagnosis data scopes separate."""
    if step==14:
        require(isinstance(context,OwnedOrderedInputs) and context.docs is not None,'/docs','owned docs context required')
        if isinstance(context,OwnedOrderedInputs) and context.docs is not None:return context.docs.product,context.docs.environment
    if step in (8,46):
        require(isinstance(context,OwnedOrderedInputs),'/strict','owned entrypoint required')
        if isinstance(context,OwnedOrderedInputs):return context.strict.product,context.strict.environment
    require(step in (1,7,13,15,16,20,22,31,33,51), '/static', 'unknown static step')
    if step == 51:return context.diagnosis_product, context.diagnosis_environment
    return ((context.discovery_product, context.discovery_environment) if step == 16
            else (context.static_product, context.environment))


@dataclass(frozen=True, slots=True)
class OrderedCall:
    step: int
    component: str
    identities: tuple[str, ...]

    @property
    def identity(self) -> str:
        return f'{self.step}:{self.component}'


def ordered_calls() -> tuple[OrderedCall, ...]:
    """Closed supported subsequence; every omitted obligation stays in the ledger."""
    result: list[OrderedCall] = []
    for step in ORDER:
        rows = tuple(row for row in obligations() if row.step == step)
        if step in (1,7,13,15,20,22,31,33) or step in (16,51):
            result.append(OrderedCall(step,'static',tuple(row.identity for row in rows)))
        elif step in (12,27):
            result.append(OrderedCall(step,'perf' if step==12 else 'nav',tuple(row.identity for row in rows)))
        elif step == 24:
            result.append(OrderedCall(step,'save_roundtrip',('24:save_load.release',)))
        elif step == 9:
            result.append(OrderedCall(step,'smoke',tuple(row.identity for row in rows)))
        elif step in direct.DIRECT_STEP_FIXTURES:
            if step not in direct.UNSUPPORTED_DIRECT_STEPS:
                result.append(OrderedCall(step,'direct',tuple(row.identity for row in rows if row.fixture)))
        else:
            result.extend(OrderedCall(step,row.component,(row.identity,)) for row in rows if row.component is not None)
    return tuple(result)


def ordered_owned_calls(*, bench_enabled: bool = False, docs_enabled: bool = False) -> tuple[OrderedCall,...]:
    result=list(ordered_calls())
    for step,identity in ((8,'8:gdscript_strict'),(46,'46:gdscript.four_files')):
        insertion=next((index for index,call in enumerate(result) if ORDER.index(call.step)>=ORDER.index(step)),len(result))
        result.insert(insertion,OrderedCall(step,'static',(identity,)))
    if docs_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(14))
        result.insert(insertion,OrderedCall(14,'static',('14:docs',)))
    if bench_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(25))
        result.insert(insertion,OrderedCall(25,'bench',('25:bench',)))
    return tuple(result)


def authenticate_static(path: Path, step: int | None, product: static.StaticProduct | None, environment: static.ExecutionEnvironment | None = None) -> str:
    require(step is not None and product is not None,'/static','missing static binding')
    if step is None or product is None:raise GateContractError('binding','/static','missing static binding')
    initial_digest = file_hash(path)
    static.verify_receipt(path,initial_digest)
    node=document(path.read_text())
    required={'schema','step','name','scope','original_order_executed','full_gate_complete','source_admission_granted',
              'source_root','source_sha','source_invocation','started_at','status','process_status','artifacts',
              'tool_dependency_closure','errors','finished_at'}
    optional={'source_before','inputs_before','refs_before','tool_bindings','prior_receipts','native_exit','pid',
              'process_receipt','process_receipt_sha256','source_end','inputs_end','refs_end'}
    if step == 14:optional.update({'reviewed_world_scope','reviewed_initial_state_mutations','current_product_world_scope','build_provenance','docs_before','docs_end'})
    if step == 16:optional.add('unittest')
    require(required <= node.mapping().keys() <= required|optional,'/static','terminal schema differs')
    for field,value in (('step',step),('name',static.COMMANDS[step][0]),('source_root',str(product.root)),
                        ('source_sha',product.source_sha),('scope','COMPONENT_ONLY'),('original_order_executed',False),
                        ('full_gate_complete',False),('source_admission_granted',False)):
        node.field(field).equals(value)
    node.field('source_invocation').equals(list(static.COMMANDS[step][1]))
    state=node.field('status').text();require(state in {'PASS','FAIL'},'/status','invalid static status')
    _=node.field('errors').array()
    if step in (8,14,16,46,51):
        require(environment is not None, '/static', 'missing selected static environment')
        if environment is None:raise GateContractError('binding','/static','missing selected static environment')
        if step in (8,14,46):
            require('process_receipt' not in node.mapping() or 'prior_receipts' in node.mapping(),
                    '/strict','process has no invocation prior binding')
            if 'prior_receipts' in node.mapping():
                node.field('prior_receipts').equals({str(entry):digest for entry,digest in environment.prior_receipts})
        if step==14:
            if 'prior_receipts' not in node.mapping():require(not environment.prior_receipts,'/docs','unbound prior prefix')
            if state=='PASS' or 'process_receipt' in node.mapping():
                node.field('reviewed_world_scope').equals('construction_no_tick_advance')
                node.field('reviewed_initial_state_mutations').equals(True)
                node.field('current_product_world_scope').equals('CALLER_REQUIRED')
                node.field('build_provenance').equals('CALLER_REQUIRED')
            if state=='PASS':
                node.field('docs_before').equals(node.field('docs_end').value)
                node.field('docs_end').equals(static.docs_bindings(product.root,environment))
            else:require(bool(node.field('errors').array()),'/docs','failed receipt lacks errors')
        if step in (8,14,46) and state=='PASS':
            for field in ('inputs_before','inputs_end'):node.field(field).equals(dict(product.inputs))
            node.field('source_end').equals(node.field('source_before').value)
            node.field('source_before').field('source_sha').equals(product.source_sha)
            node.field('source_before').field('root').equals(str(product.root))
            node.field('source_before').field('source_clean').equals(True)
            node.field('refs_end').equals(node.field('refs_before').value)
            node.field('tool_bindings').equals({name:{'path':str(entry),'sha256':digest} for name,entry,digest in environment.tools})
        if step == 16:discovery_terminal(path, node, product, environment)
        else:
            static_process_terminal(path, node, product, environment, step)
            if state == 'PASS':
                stdout=(path.parent/'process/stdout.log').read_text()
                if step in (8,46):static.strict_terminal(stdout,(path.parent/'process/stderr.log').read_text(),len(static.strict_files(product.root,step)))
                elif step==14:static.docs_terminal(stdout)
                else:static.diagnosis_terminal(stdout)
    static.verify_receipt(path,initial_digest)
    return state


def static_process_terminal(path: Path, node: Node, product: static.StaticProduct,
                            environment: static.ExecutionEnvironment, step: int) -> None:
    """Closed8/14/16/46/51 process join: Python discovery or Bash diagnosis, never an inner PID."""
    require(type(step) is int and step in (8,14,16,46,51), '/step', 'unsupported process contract')
    fields = node.mapping()
    passing = node.field('status').text() == 'PASS'
    if 'process_receipt' in fields:
        process = path.parent / 'process/process.json'
        node.field('process_receipt').equals(str(process))
        node.field('process_receipt_sha256').equals(file_hash(process))
        native = document(process.read_text(encoding='utf-8'))
        tools = {name: (entry, digest) for name, entry, digest in environment.tools}
        tool = static.COMMANDS[step][1][0]
        require(tool in tools, '/tools', 'missing selected process entry')
        entry, digest = tools[tool]
        native.field('schema_version').equals(1)
        native.field('argv').equals([str(entry), *static.COMMANDS[step][1][1:]])
        for field, expected in (('cwd', str(product.root)), ('input_root', str(product.root)),
                                ('output_root', str(process.parent)), ('executable', str(entry)),
                                ('expected_executable_sha256', digest)):
            native.field(field).equals(expected)
        native.field('expected_inputs').equals(dict(product.inputs))
        native.field('environment').equals(dict(environment.values))
        externals: dict[str,Json] = {str(external): value for external,value in environment.external_inputs}
        for _,external,value in environment.tools:
            require(str(external) not in externals or externals[str(external)] == value, '/external', 'conflicting tool binding')
            externals[str(external)] = value
        native.field('expected_external_inputs').equals(externals)
        require(not {'timeout_seconds','timed_out'} & native.mapping().keys(), '/process', 'unexpected static timeout')
        native.field('status').equals(node.field('process_status').value)
        native.field('native_exit').equals(node.field('native_exit').value)
        native.field('pid').equals(node.field('pid').value)
        if step in (8,46):
            if native.field('pid').value is None:
                native.field('native_exit').equals(None)
                require(not (process.parent/'process-spawn.json').exists(),'/strict','unjoined spawn receipt')
            else:
                require(integer(native.field('native_exit'))>=0,'/strict','unknown shell completion is not a terminal prior')
                static_completion(process,native)
        for stream in ('stdout', 'stderr'):
            native.field(stream + '_sha256').equals(file_hash(process.parent / (stream + '.log')))
        if passing or step==14:
            native.field('status').equals('PASS');native.field('native_exit').equals(0)
            static_completion(process, native)
            require(not native.field('errors').array(), '/process', 'PASS process has errors')
            for field in ('before_inputs','after_inputs'):native.field(field).equals(dict(product.inputs))
            for field in ('before_executable_sha256','after_executable_sha256'):native.field(field).equals(digest)
            for field in ('before_external_inputs','after_external_inputs'):native.field(field).equals(externals)
    else:
        require(not passing and 'unittest' not in fields, '/process', 'missing static process receipt')
        if step==14:
            node.field('process_status').equals('NOT_RUN')
            require(not {'pid','native_exit','process_receipt_sha256'} & fields.keys(),'/docs','unjoined native fields in no-launch receipt')
        if step in (8,14,46):
            require(not (path.parent/'process').exists(),'/strict','unjoined process artifacts are not a no-launch failure')
            node.field('artifacts').equals({})
    if passing:require(not node.field('errors').array(), '/static', 'PASS receipt has errors')


def discovery_terminal(path: Path, node: Node, product: static.StaticProduct,
                       environment: static.ExecutionEnvironment) -> None:
    static_process_terminal(path, node, product, environment, 16)
    fields = node.mapping()
    passing = node.field('status').text() == 'PASS'
    if 'unittest' in fields:
        summary = static.unittest_terminal((path.parent / 'process/stderr.log').read_bytes().decode('utf-8'))
        node.field('unittest').equals(summary)
        if passing:require(summary['outcome'] == 'OK', '/unittest', 'discovery did not pass')
    else:
        require(not passing, '/unittest', 'PASS discovery lacks actual terminal')
    if passing:require(not node.field('errors').array(), '/static', 'PASS receipt has errors')


def static_completion(path: Path, native: Node) -> None:
    """Bind producer start/spawn/finish records, without inferring OS attestation."""
    require(integer(native.field('pid')) > 0, '/pid', 'PASS needs a positive process PID')
    require(integer(native.field('duration_ns')) > 0, '/duration', 'PASS needs positive elapsed time')
    native.field('environment_policy').equals('fixed_defaults_plus_explicit_only_no_inheritance')
    native.field('fresh_directories').equals([])
    native.field('build_provenance_verified').equals(False)
    start = document((path.parent / 'process-start.json').read_text(encoding='utf-8'))
    spawn = document((path.parent / 'process-spawn.json').read_text(encoding='utf-8'))
    keys(spawn, {'pid','spawned_at'})
    spawn.field('pid').equals(native.field('pid').value)
    for field in ('schema_version','argv','cwd','executable','expected_executable_sha256','input_root',
                  'expected_inputs','output_root','environment','expected_external_inputs','fresh_directories',
                  'environment_policy','started_at','before_executable_sha256','before_inputs','before_external_inputs',
                  'build_provenance_verified'):
        start.field(field).equals(native.field(field).value)
    for field in ('pid','native_exit'):start.field(field).equals(None)
    start.field('status').equals('FAIL');start.field('errors').equals([])
    start.field('after_external_inputs').equals({})
    require(not {'timeout_seconds','timed_out','ended_at','duration_ns'} & start.mapping().keys(), '/process', 'invalid start record')
    moments = tuple(datetime.fromisoformat(value) for value in
                    (native.field('started_at').text(), spawn.field('spawned_at').text(), native.field('ended_at').text()))
    require(all(value.utcoffset() is not None for value in moments), '/process', 'timestamps must have offsets')
    require(moments[0] <= moments[1] <= moments[2], '/process', 'start/spawn/end chronology differs')


def ordered_action(call: OrderedCall, context: OrderedInputs) -> Action:
    inputs=context.components;name=call.component
    if name=='static':
        product, environment = static_context(context, call.step)
        return lambda out:static.run_step(product,call.step,out,environment)
    if name in {'perf','nav'}:
        fixture=perf_fixture(name,call.step)
        return lambda out:run_perf(context.perf,fixture,out)
    if name=='save_roundtrip':
        require(call.step == 24, '/step', 'save adapter only supports step24')
        return lambda out:run_save_roundtrip(context.save_roundtrip,out)
    if name=='smoke':return lambda out:smoke.run_pair(inputs.godot,out)
    if name=='direct':return lambda out:direct.run_step(inputs.registry,inputs.probes,inputs.declarations,call.step,out)
    if name=='renderer':return lambda out:renderer.run_pair(renderer.RendererProducts(inputs.probes,inputs.godot),out)
    if name=='pack':return lambda out:pack.run_pair(inputs.godot,out)
    if name=='visibility':return lambda out:visibility.run_pair(inputs.godot,out)
    if name in fixed.FIXTURES:return lambda out:fixed.run_pair(inputs.godot,name,out)
    require(name.startswith('why.') and name.removeprefix('why.') in why.VARIANTS,'/component','unknown ordered adapter')
    return lambda out:why.run_pair(inputs.godot,name.removeprefix('why.'),out)


def ordered_bindings(context: OrderedInputs) -> None:
    if isinstance(context,OwnedOrderedInputs):
        strict_bindings(context)
        if context.bench is not None:_=bench_bindings(context)
        if context.docs is not None:_=docs_bindings(context)
    save_roundtrip_bindings(context)
    perf_bindings(context)
    inputs=context.components
    for product in (inputs.probes.base,inputs.probes.candidate,inputs.godot.base,inputs.godot.candidate):
        require(file_hash(product.executable)==product.executable_sha256 and
                inventory(product.input_root,[name for name,_ in product.input_files])==dict(product.input_files),'/inputs','product drift')
    for step in (1,7,13,15,16,20,22,31,33,51):
        _, environment = static_context(context, step)
        _ = static.validate_environment(environment, step)
    for step in (1,16,51):
        product, environment = static_context(context, step)
        require(product.root==inputs.probes.candidate.input_root and product.source_sha==inputs.probes.candidate.source_sha,
                '/static','static source differs from candidate')
        names = [*INPUT_DIRS, *INPUT_FILES, *(['docs'] if step == 16 else [static.DIAGNOSIS_RESULT] if step == 51 else [])]
        require(len(dict(product.inputs)) == len(product.inputs) and inventory(product.root,names)==dict(product.inputs),
                '/static','complete static input scope differs')
    _=authenticate_prior_chain(context,configured_priors(context))


def authenticate_prior(context: OrderedInputs, path: Path, digest: str, prefix: tuple[tuple[Path,str],...] | None = None) -> None:
    require(path.is_absolute() and path == path.resolve(strict=True), '/prior', 'noncanonical receipt path')
    static.verify_receipt(path,digest)
    step = integer(document(path.read_text(encoding='utf-8')).field('step'))
    product, environment = static_context(context,step)
    if step in (8,14,46) and prefix is not None:environment=replace(environment,prior_receipts=prefix)
    _ = authenticate_static(path,step,product,environment)


def configured_priors(context: OrderedInputs) -> tuple[tuple[Path,str],...]:
    entries=(*context.environment.prior_receipts,*context.discovery_environment.prior_receipts,
             *context.diagnosis_environment.prior_receipts)
    if isinstance(context,OwnedOrderedInputs):
        entries+=context.strict.environment.prior_receipts
        if context.docs is not None:entries+=context.docs.environment.prior_receipts
    return entries


def authenticate_prior_chain(context: OrderedInputs, entries: tuple[tuple[Path,str],...]) -> tuple[tuple[Path,str],...]:
    """Join each invocation to the already authenticated preceding receipt prefix."""
    priors: dict[Path,str]={}
    for path,digest in entries:
        require(path not in priors or priors[path]==digest,'/prior','conflicting receipt binding')
        if path in priors:continue
        authenticate_prior(context,path,digest,tuple(priors.items()))
        priors[path]=digest
    return tuple(priors.items())


def with_static_priors(context: OrderedInputs, rows: tuple[Outcome, ...]) -> OrderedInputs:
    """Propagate authenticated FAIL as well as PASS with historical invocation prefixes."""
    candidates=list(configured_priors(context))
    candidates.extend((row.receipt,row.sha256) for row in rows if row.component == 'static'
                      and row.reported_status in {'PASS','FAIL'} and row.receipt is not None and row.sha256 is not None)
    frozen=authenticate_prior_chain(context,tuple(candidates))
    if isinstance(context,OwnedOrderedInputs):
        context=replace(context,strict=replace(context.strict,environment=replace(context.strict.environment,prior_receipts=frozen)))
        if context.docs is not None:
            context=replace(context,docs=replace(context.docs,environment=replace(context.docs.environment,prior_receipts=frozen)))
    return replace(context,environment=replace(context.environment,prior_receipts=frozen),
                   discovery_environment=replace(context.discovery_environment,prior_receipts=frozen),
                   diagnosis_environment=replace(context.diagnosis_environment,prior_receipts=frozen))


def run_ordered(context: OrderedInputs, output: Path) -> Path:
    require(not isinstance(context,OwnedOrderedInputs),'/entrypoint','use explicit owned entrypoint')
    return _run_ordered(context,output)


def _run_ordered(context: OrderedInputs, output: Path) -> Path:
    """Execute only supported calls at their original positions; never full-gate PASS.

    Missing preparation/native obligations prevent original-order qualification.
    Static tools are explicit caller bindings, not inferred build provenance.
    Exhaustive runtime trees, links and absent configurations remain caller admission;
    this coordinator checks supplied regular files, not hermetic runtime closure.
    """
    inputs=context.components
    require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh canonical output required')
    for product in (inputs.probes.base,inputs.probes.candidate,inputs.godot.base,inputs.godot.candidate):
        require(not output.is_relative_to(product.input_root) and not product.input_root.is_relative_to(output),'/output','product overlap')
    calls=ordered_owned_calls(bench_enabled=context.bench is not None,docs_enabled=context.docs is not None) if isinstance(context,OwnedOrderedInputs) else ordered_calls();rows: dict[str,Outcome]={};errors: list[str]=[];sequence: list[str]=[]
    tail_stop: str | None=None;bench_inputs: dict[str,Json] | None=None;docs_inputs: dict[str,Json] | None=None
    plan_hash: str | None=None;blocked: dict[int,str]={};events: list[Json]=[];perf_inputs: dict[str,Json] | None=None
    try:
        output.mkdir(parents=True)
        _=direct.partition(inputs.registry)
        perf_bindings(context);perf_inputs=perf_dependencies(context.perf)
        if isinstance(context,OwnedOrderedInputs) and context.bench is not None:
            bench_inputs=bench_dependencies(bench_bindings(context))
        if isinstance(context,OwnedOrderedInputs) and context.docs is not None:
            docs_inputs=docs_dependencies(docs_bindings(context))
        for product in (context.perf.base,context.perf.candidate):
            require(not output.is_relative_to(product.root) and not product.root.is_relative_to(output),'/output','perf product overlap')
        require(inputs.declarations.base==inputs.probes.base.source_sha and inputs.declarations.candidate==inputs.probes.candidate.source_sha,
                '/declarations','source identity differs')
        require(DIRECT_IDS <= set(inputs.declarations.known_fixtures),'/declarations','incomplete coverage')
        for step in direct.DIRECT_STEP_FIXTURES:_=direct.step_fixtures(step)
        refs: dict[int,dict[str,list[Json]]]={step:references(inputs,output/f'step-{step:02d}',validate=True) for step in direct.DIRECT_STEP_FIXTURES}
        atomic_json(output/'plan.json',{'schema_version':3,'original_step_order':list(ORDER),
            'original_order_executed':False,'publication_authorized':False,
            'supported_call_sequence':[call.identity for call in calls],'perf_dependencies':perf_inputs,'bench_dependencies':bench_inputs,'docs_dependencies':docs_inputs,
            'strict_owned_entrypoint':isinstance(context,OwnedOrderedInputs),
            'obligations':[{'identity':row.identity,'step':row.step,'fixture':row.fixture,
                'invocation_references':refs.get(row.step,{}).get(row.fixture or '',[])} for row in obligations()]})
        plan_hash=file_hash(output/'plan.json')
        ordered_bindings(context)
        _ = with_static_priors(context,())
        for call in calls:
            if tail_stop is not None:
                rows[call.identity]=Outcome(call.component,'NOT_RUN',None,None,'blocked by '+tail_stop);continue
            if call.step in blocked:
                rows[call.identity]=Outcome(call.component,'NOT_RUN',None,None,'blocked by '+blocked[call.step]);continue
            for row in rows.values():recheck(row)
            ordered_bindings(context)
            if isinstance(context,OwnedOrderedInputs) and context.bench is not None:
                require(bench_dependencies(context.bench)==bench_inputs,'/bench','context binding changed')
            if isinstance(context,OwnedOrderedInputs) and context.docs is not None:
                require(docs_dependencies(context.docs)==docs_inputs,'/docs','context binding changed')
            require(file_hash(output/'plan.json')==plan_hash,'/plan','scope changed')
            parent=output/f'step-{call.step:02d}';parent.mkdir(exist_ok=True)
            destination=parent/call.component
            try:
                with nullcontext() if call.component in {'static','save_roundtrip','perf','nav','bench'} else lane():
                    selected = with_static_priors(context,tuple(rows.values())) if call.component == 'static' else context
                    action=ordered_action(call,selected) if call.component!='bench' else None
                    static_product, static_environment = static_context(selected,call.step) if call.component == 'static' else (None,None)
                    sequence.append(call.identity);events.append({'call':call.identity,'event':'start','at':timestamp()})
                    try:
                        if call.step==14 and call.component=='static' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_docs(call,selected,destination)
                        elif call.component=='bench' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_bench(call,selected,destination)
                        elif call.step in (8,46) and call.component=='static' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_strict(call,selected,destination)
                        else:
                            assert action is not None
                            rows[call.identity]=invoke(call.component,action,destination,inputs,
                                                       step=call.step,static_product=static_product,static_environment=static_environment,
                                                       save_products=selected.save_roundtrip,perf_context=selected.perf)
                    finally:events.append({'call':call.identity,'event':'end','at':timestamp()})
            except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
                rows[call.identity]=replace(rows[call.identity],status='FAIL',error=rows[call.identity].error+'; lane: '+str(error)) if call.identity in rows else Outcome(call.component,'FAIL',None,None,str(error))
            if call.component=='bench' and rows[call.identity].status!='COMPONENT_PASS':tail_stop=call.identity
            if rows[call.identity].status=='FAIL' and call.step in (34,46):blocked[call.step]=call.identity
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:errors.append(str(error))
    for row in rows.values():
        try:recheck(row)
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:errors.append(str(error))
    try:
        ordered_bindings(context)
        if isinstance(context,OwnedOrderedInputs) and context.bench is not None:
            require(bench_dependencies(context.bench)==bench_inputs,'/bench','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.docs is not None:
            require(docs_dependencies(context.docs)==docs_inputs,'/docs','context binding changed')
        require(plan_hash is not None and file_hash(output/'plan.json')==plan_hash,'/plan','scope changed or plan failed')
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:errors.append(str(error))
    by_obligation={identity:call.identity for call in calls for identity in call.identities}
    output.mkdir(parents=True,exist_ok=True);terminal=output/'ordered-dispatch.json'
    atomic_json(terminal,{'schema_version':3,'status':'FAIL' if errors or any(row.status=='FAIL' for row in rows.values()) else 'INCOMPLETE',
        'publication_authorized':False,'full_gate_claim':False,'original_order_executed':False,
        'original_step_order':list(ORDER),'perf_dependencies':perf_inputs,'bench_dependencies':bench_inputs,'docs_dependencies':docs_inputs,'dispatch_events':events,'actual_component_sequence':list(sequence),'plan_sha256':plan_hash,'errors':list(errors),
        'components':[{'identity':call.identity,'step':call.step,'component':call.component,
            'status':rows[call.identity].status if call.identity in rows else 'NOT_RUN',
            'receipt':str(rows[call.identity].receipt) if call.identity in rows and rows[call.identity].receipt else None,
            'sha256':rows[call.identity].sha256 if call.identity in rows else None,
            'files':dict(rows[call.identity].files) if call.identity in rows else {},
            'error':rows[call.identity].error if call.identity in rows else '',
            'returned_status':rows[call.identity].returned_status if call.identity in rows else None,
            'reported_status':rows[call.identity].reported_status if call.identity in rows else None,
            'snapshot_error':rows[call.identity].snapshot_error if call.identity in rows else ''} for call in calls],
        'obligations':[{'identity':row.identity,'step':row.step,'fixture':row.fixture,
            'status':'NOT_RUN' if tail_stop is not None and ORDER.index(row.step)>ORDER.index(25) else 'COMPONENT_ONLY' if row.identity in by_obligation else 'NOT_IMPLEMENTED',
            'blocked_by':tail_stop if tail_stop is not None and ORDER.index(row.step)>ORDER.index(25) else None,
            'call':by_obligation.get(row.identity),
            'component_status':rows[by_obligation[row.identity]].status if row.identity in by_obligation and by_obligation[row.identity] in rows else None}
            for row in obligations()]})
    return terminal


def main(argv: tuple[str,...]) -> int:
    require(len(argv)==2 and argv[0]=='--plan','/argv','only --plan ABSOLUTE_FRESH_OUTPUT supported; no full execution CLI yet')
    _=write_plan(Path(argv[1]))
    return 2


if __name__=='__main__':
    sys.exit(main(tuple(sys.argv[1:])))
