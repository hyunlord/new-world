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
import shutil
import signal
import stat
import sys
import uuid
from collections.abc import Callable
from contextlib import nullcontext, suppress
from dataclasses import asdict, dataclass, replace
from datetime import datetime
from pathlib import Path
from typing import Final, Literal, NoReturn, final

import generic_bench_run as bench
import generic_cli_execute as cli_execute
import generic_cli_setup_handoff as cli_handoff
import generic_core_evaluation as core_evaluation
import generic_direct_run as direct
import generic_integration_prefix_run as integration_prefix
import generic_library_run as library49
import generic_m1_run as m1_runtime
import generic_m6a_structural as structural
import generic_mux2_run as mux2_runtime
import generic_oracles_run as oracles
import generic_perf_run as perf
import generic_play_owner as play_owner
import generic_play_sources as play_sources
import generic_renderer_run as renderer
import generic_resolver_run as resolver
import generic_setup_bridge as setup_bridge
import generic_setup_clippy as setup_clippy
import generic_setup_import as setup_import
import generic_setup_prepare as setup_prepare
import generic_static_run as static
import generic_test_run as native
import generic_ui_fixed_run as fixed
import generic_ui_pack_run as pack
import generic_ui_smoke_run as smoke
import generic_ui_visibility_run as visibility
import generic_ui_why_run as why
import generic_workspace_endpoints as workspace_endpoints
import generic_workspace_run as workspace_runtime
import mux2_gate_receipts as stall_capture
from gate_oracle_child import SUITE_NAMES
from gate_stall_child import ChildPlan, projected_environment
from generic_bench import AuditError, Product, process_inputs, validate_products
from generic_declarations import Declarations
from generic_fixture_registry import (
    DIRECT_IDS,
    ExpansionContext,
    Invocation,
    Registry,
    expand,
)
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document, integer, require
from generic_library_siblings import LibraryStep
from generic_library_siblings import recipe as library_recipe
from generic_observations import keys
from generic_process import (
    ProcessSpec,
    atomic_json,
    environment_for,
    file_hash,
    timestamp,
)
from generic_sources import freeze_source, inventory
from generic_stall_build import BuildEvidenceReference, derived_source_binding
from generic_stall_outcome import StallOutcomePlan, validate_derived_bounded_outcome
from generic_test_invocation import invocations
from generic_test_observations import Binding, ExpectedOccurrence
from generic_test_safety import pair
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json, Node, snapshot
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
        36: ('data_only_ext.debug', 'pack'),
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
    library49_result: library49.Library49Result | None = None
    library49_members: tuple[str,...] = ()
    core_prior: core_evaluation.PriorArtifacts | None = None
    core_result: core_evaluation.EvaluationResult | None = None
    play_evidence: PlayEvidence | None = None
    prefix_result: integration_prefix.NativePrefixResult | None = None
    prefix_state: tuple[tuple[str,tuple[str,str]], ...] = ()
    m1_result: m1_runtime.M1Result | None = None
    m1_state: tuple[tuple[str,tuple[str,str]], ...] = ()
    mux2_result: mux2_runtime.DefenceResult | None = None
    mux2_state: tuple[tuple[str,tuple[str,str]], ...] = ()
    setup_evidence: SetupEvidence | None = None
    setup_state: str = ''


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
    if row.component=='setup':recheck_setup_anchor(row)
    elif row.root is not None and (row.files or row.root.exists()):
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
class OracleContext:
    inputs: oracles.OracleInputs


@dataclass(frozen=True, slots=True)
class ResolverContext:
    product: ProbeProduct
    environment: static.ExecutionEnvironment


@dataclass(frozen=True, slots=True)
class Structural48Context:
    products: Products
    tools: structural.Tooling


@dataclass(frozen=True, slots=True)
class StallContext:
    root: Path
    source_sha: str
    godot: Path
    godot_sha256: str
    reference: BuildEvidenceReference
    environment: tuple[tuple[str,str], ...]
    tools: tuple[tuple[Path,str], ...]


@dataclass(frozen=True, slots=True)
class Library49Context:
    source_sha: str
    request: library49.Library49Request


@dataclass(frozen=True, slots=True)
class M1Context:
    source_sha: str
    request: m1_runtime.M1Request


@dataclass(frozen=True, slots=True)
class Mux2Context:
    source_sha: str
    request: mux2_runtime.DefenceRequest


@dataclass(frozen=True, slots=True)
class PlayContext:
    plan: play_owner.PlayPlan


@dataclass(frozen=True, slots=True)
class PlayEvidence:
    owner: play_owner.PlayOwner
    returned: play_owner.PlayOutcome
    retained: tuple[tuple[Path,str], ...]
    native_members: tuple[tuple[Path,tuple[tuple[str,str], ...]], ...]
    members: tuple[tuple[str,str], ...]
    failure: BaseException | None = None
    rejected_external: tuple[tuple[Path,str], ...] = ()
    rejected_members: tuple[tuple[Path,tuple[tuple[str,str], ...]], ...] = ()
    rejected_native_files: tuple[tuple[Path,tuple[tuple[str,str], ...]], ...] = ()


@dataclass(frozen=True, slots=True)
class OwnedOrderedInputs(OrderedInputs):
    strict: StrictContext
    control_fd: int
    bench: BenchContext | None = None
    docs: DocsContext | None = None
    oracles: OracleContext | None = None
    resolver: ResolverContext | None = None
    structural: Structural48Context | None = None
    stall: StallContext | None = None
    library49: Library49Context | None = None
    library42: Library49Context | None = None
    library43: Library49Context | None = None
    play: PlayContext | None = None
    prefix28: Library49Context | None = None
    prefix36: Library49Context | None = None
    m1: M1Context | None = None
    mux2: Mux2Context | None = None
    setup: SetupContext | None = None


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


@dataclass(frozen=True, slots=True)
class SetupContext:
    prepared: setup_prepare.PreparedSetup
    wait: setup_bridge.CargoWaitEvidence
    clippy_wait: setup_clippy.ClippyWaitEvidence
    # Construction only: no producer execution. New manifests belong below preparation.
    workspace_factory: Callable[[Literal[4,5], workspace_endpoints.WorkspaceChain, Path, Path, int], workspace_endpoints.WorkspaceRequest]


@dataclass(frozen=True, slots=True)
class SetupPrefixContext:
    """Actual setup selectors without fabricated future runtime products."""
    setup: SetupContext
    candidate_root: Path
    candidate_sha: str
    candidate_inputs: tuple[tuple[str,str], ...]
    godot: Path
    godot_sha256: str


@dataclass(frozen=True, slots=True)
class SetupEvidence:
    step: int
    result: setup_bridge.BridgeResult | setup_import.ImportResult | workspace_endpoints.WorkspaceResult | setup_clippy.ClippyResult
    anchor: workspace_endpoints.EvidenceAnchor
    receipt_sha256: str
    endpoint: workspace_endpoints.WorkspaceEndpoint | setup_clippy.ClippyEndpoint | None
    import_owner: setup_import.ImportOwner | None = None


def setup_calls() -> tuple[OrderedCall,...]:
    return tuple(OrderedCall(step,'setup',tuple(row.identity for row in obligations() if row.step==step))
                 for step in range(2,7))


def setup_bindings(context: OwnedOrderedInputs | SetupPrefixContext) -> SetupContext:
    if isinstance(context,SetupPrefixContext):
        bound=context.setup;prepared=bound.prepared
        tool=next(t for t in prepared.inputs.tools if t.name=='godot')
        require((prepared.inputs.root,prepared.inputs.source_sha)==(context.candidate_root,context.candidate_sha),
                '/setup','candidate source differs')
        require((tool.path,tool.sha256)==(context.godot,context.godot_sha256),'/setup','Godot binding differs')
        setup_bridge.immutable_inputs(prepared)
        require(all(dict(prepared.materialized).get(name)==sha for name,sha in context.candidate_inputs),
                '/setup','candidate input binding differs')
        return bound
    bound=context.setup
    require(bound is not None,'/setup','explicit prepared context required')
    assert bound is not None
    prepared=bound.prepared;candidate=context.components.probes.candidate
    require(prepared.inputs.root==candidate.input_root and prepared.inputs.source_sha==candidate.source_sha,
            '/setup','candidate source differs')
    godot=context.components.godot.candidate
    tool=next(t for t in prepared.inputs.tools if t.name=='godot')
    require(godot.input_root==prepared.inputs.root and godot.source_sha==prepared.inputs.source_sha
            and godot.executable==tool.path and godot.executable_sha256==tool.sha256,
            '/setup','candidate Godot binding differs')
    setup_bridge.immutable_inputs(prepared)
    require(all(dict(prepared.materialized).get(name)==sha for name,sha in candidate.input_files),
            '/setup','candidate input binding differs')
    return bound


def setup_dependencies(bound: SetupContext) -> dict[str,Json]:
    prepared=bound.prepared
    return {'root':str(prepared.inputs.root),'source_sha':prepared.inputs.source_sha,
        'source_digest':prepared.source_digest,'output':str(prepared.inputs.output),
        'inputs':dict(prepared.materialized),'external':{str(path):sha for path,sha in prepared.external},
        'environment':dict(prepared.inputs.environment),
        'wait_inputs':{str(path):sha for path,sha in setup_bridge.qualify_wait(prepared,bound.wait)},
        'clippy_wait_inputs':{str(path):sha for path,sha in setup_clippy.qualify_clippy(prepared,bound.clippy_wait)},
        'factory_scope':'CONSTRUCTION_ONLY_NO_NATIVE_PRODUCER',
        'source_build_qualification':'PENDING','observer_world_qualification':'PENDING'}


@final
class SetupOwner(_StrictOwner):
    ownership_label: str = 'setup'

    def __init__(self, control: int, output: Path, bound: SetupContext) -> None:
        super().__init__(control,output)
        self.bound=bound
        self.bridge: setup_bridge.BridgeResult | None=None
        self.import_owner: setup_import.ImportOwner | None=None
        self.imported: setup_import.ImportResult | None=None
        self.workspace: workspace_endpoints.WorkspaceEndpoint | None=None
        self.request: workspace_endpoints.WorkspaceRequest | setup_clippy.ClippyRequest | None=None
        self.retained_failure: BaseException | None=None
        self.in_flight=False
        self.closed=False
        self.next_step=2
        self.rows: list[Outcome]=[]
        self.generated: workspace_endpoints.GeneratedState | None=None
        self.evidence_root: Path=output
        self.factory_anchor: workspace_endpoints.EvidenceAnchor | None=None

    def retain_unknown(self, error: BaseException) -> NoReturn:
        self.retained_failure=error
        self.hold(error)

    def close(self) -> None:
        require(not self.in_flight,'/setup','cannot close unknown workload')
        if not self.closed:
            os.close(self.descriptor);self.closed=True


def setup_files(state: str) -> tuple[tuple[str,str], ...]:
    node=document(state)
    return tuple((name,node.field(name).field('sha256').text()) for name in node.mapping()
                 if node.field(name).field('sha256').value is not None)


def recheck_setup_anchor(row: Outcome) -> None:
    if row.component!='setup' or row.root is None:return
    require(workspace_endpoints.anchor(row.root).state==row.setup_state,
            '/setup','historical output bytes/kinds/membership changed')


def recheck_setup(context: OrderedInputs | SetupPrefixContext, row: Outcome) -> None:
    if row.component!='setup':return
    recheck_setup_anchor(row)
    evidence=row.setup_evidence
    if evidence is None:
        require(row.status in {'FAIL','NOT_RUN'},'/setup','missing actual evidence')
        return
    require(isinstance(context,(OwnedOrderedInputs,SetupPrefixContext)),'/setup','owned entrypoint required')
    assert isinstance(context,(OwnedOrderedInputs,SetupPrefixContext))
    _=setup_bindings(context)
    recheck(row);evidence.anchor.recheck()
    require(file_hash(evidence.result.receipt)==evidence.receipt_sha256,'/setup','historical receipt changed')
    if isinstance(evidence.endpoint,workspace_endpoints.WorkspaceEndpoint):
        workspace_endpoints.check_endpoint(evidence.endpoint)
    elif isinstance(evidence.endpoint,setup_clippy.ClippyEndpoint):
        setup_clippy.check_clippy_endpoint(evidence.endpoint)
    evidence.anchor.recheck()


def setup_workspace_request(owner: SetupOwner, step: Literal[4,5], output: Path) -> workspace_endpoints.WorkspaceRequest:
    prepared=owner.bound.prepared
    if step==4:
        require(owner.import_owner is not None and owner.imported is not None,'/setup','missing actual import return')
        assert owner.import_owner is not None and owner.imported is not None
        prior: workspace_endpoints.ImportPredecessor | workspace_endpoints.WorkspaceEndpoint = workspace_endpoints.ImportPredecessor(
            owner.import_owner,owner.imported,file_hash(owner.imported.receipt))
        _=workspace_endpoints.admit_import(prepared,prior)
    else:
        require(owner.workspace is not None and owner.workspace.step==4,'/setup','missing actual debug endpoint')
        assert owner.workspace is not None
        prior=owner.workspace;workspace_endpoints.check_endpoint(prior)
    chain=workspace_endpoints.WorkspaceChain(prepared,prior)
    before=workspace_endpoints.generated(prepared)
    history=tuple(row.setup_evidence.anchor for row in owner.rows if row.setup_evidence is not None)
    preparation=output.parent/'workspace-preparation'
    require(not preparation.exists() and not preparation.is_symlink(),'/setup/factory','fresh preparation directory required')
    preparation.mkdir()
    request=owner.bound.workspace_factory(step,chain,output,preparation,owner.descriptor)
    owner.factory_anchor=workspace_endpoints.anchor(preparation)
    require(workspace_endpoints.generated(prepared)==before,'/setup/factory','factory changed generated producer state')
    setup_bridge.immutable_inputs(prepared)
    for item in history:item.recheck()
    require(request.step==step and request.arm=='candidate' and request.chain is chain
            and request.spec.input_root==prepared.inputs.root and request.output==output
            and request.held_lane_fd==owner.descriptor and request.manifest.is_relative_to(preparation)
            and request.manifest==request.manifest.resolve(),'/setup/factory','foreign request/manifest/chain')
    require(request.spec.argv==workspace_runtime.workspace_argv(str(request.spec.executable),step),
            '/setup/factory','original workspace argv required')
    owner.factory_anchor.recheck()
    return request


def invoke_setup(call: OrderedCall, context: OwnedOrderedInputs | SetupPrefixContext, output: Path, owner: SetupOwner) -> Outcome:
    require(call in setup_calls() and call.step==owner.next_step and not owner.closed,'/setup','setup chronology differs')
    bound=setup_bindings(context)
    require(bound is owner.bound,'/setup','context replaced')
    for row in owner.rows:recheck_setup(context,row)
    if owner.generated is not None:
        require(workspace_endpoints.generated(bound.prepared)==owner.generated,'/setup','generated drift between original steps')
    if call.step==2:output=bound.prepared.inputs.output
    owner.evidence_root=output.parent if call.step in (4,5) else output
    result: setup_bridge.BridgeResult | setup_import.ImportResult | workspace_endpoints.WorkspaceResult | setup_clippy.ClippyResult
    endpoint: workspace_endpoints.WorkspaceEndpoint | setup_clippy.ClippyEndpoint | None=None
    try:
        if call.step==2:
            owner.in_flight=True
            result=setup_bridge.run_bridge_owned(bound.prepared,bound.wait,owner.descriptor)
            owner.in_flight=False;owner.bridge=result
        elif call.step==3:
            require(owner.bridge is not None,'/setup','missing actual bridge return')
            assert owner.bridge is not None
            prior=setup_import.PriorBridge(owner.bridge,file_hash(owner.bridge.receipt),bound.wait)
            owner.in_flight=True
            owner.import_owner=setup_import.start_import_pair(bound.prepared,prior,output,owner.descriptor)
            result=owner.import_owner.finish()
            owner.in_flight=False;owner.imported=result
        elif call.step in (4,5):
            request=setup_workspace_request(owner,4 if call.step==4 else 5,output);owner.request=request
            _=workspace_runtime.checked(request)
            owner.in_flight=True
            result=workspace_runtime.run_workspace_owned(request)
            owner.in_flight=False
            require(result.endpoint is not None,'/setup','actual workspace endpoint missing')
            owner.workspace=result.endpoint;endpoint=result.endpoint
        else:
            require(owner.workspace is not None and owner.workspace.step==5,'/setup','actual release endpoint missing')
            assert owner.workspace is not None
            clippy_request=setup_clippy.ClippyRequest(bound.prepared,bound.clippy_wait,output,owner.descriptor,owner.workspace)
            owner.request=clippy_request;owner.in_flight=True
            result=setup_clippy.run_clippy_request(clippy_request)
            owner.in_flight=False;endpoint=result.endpoint
            require(endpoint is not None,'/setup','actual Clippy endpoint missing')
        # Freeze the actual return before any coordinator validation callback.
        if owner.factory_anchor is not None:owner.factory_anchor.recheck()
        evidence=SetupEvidence(call.step,result,workspace_endpoints.anchor(owner.evidence_root),file_hash(result.receipt),endpoint,owner.import_owner if call.step==3 else None)
        owner.generated=workspace_endpoints.generated(bound.prepared)
        failed=result.status in {'FAIL','FAIL_STEP3'}
        state=evidence.anchor.state
        row=Outcome('setup','FAIL' if failed else 'PENDING',result.receipt,evidence.receipt_sha256,
            'source/build/observer/world qualification pending',
            setup_files(state),owner.evidence_root,
            result.status,result.status,setup_evidence=evidence,setup_state=state)
        recheck_setup(context,row)
        owner.rows.append(row);owner.next_step+=1
        if call.step==6:owner.close()
        return row
    except setup_bridge.BridgeOwnershipUnknown as unknown:owner.retain_unknown(unknown)
    except setup_clippy.ClippyOwnershipUnknown as unknown:owner.retain_unknown(unknown)
    except workspace_runtime.WorkspaceOwnershipUnknown as unknown:
        owner.request=unknown.request;owner.retain_unknown(unknown)
    except setup_import.ImportFailure as unknown:
        owner.import_owner=unknown.owner;owner.retain_unknown(unknown)
    except (setup_bridge.BridgeFailure,setup_clippy.ClippyFailure,workspace_runtime.WorkspaceSettledFailure):
        owner.in_flight=False
        raise
    except BaseException as error:
        if owner.in_flight:owner.retain_unknown(error)
        raise




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
    for product in (bound.base,bound.candidate):
        if product.runtime_inputs:
            _=process_inputs(product)
    return {'purpose':bound.purpose,'script':str(bound.script),'script_sha256':bound.script_sha256,
        'justifications':[dict(row) for row in bound.justifications],
        'build_authority':'CALLER_OWNED_NOT_CERTIFIED_HERE',
        'products':{arm:{'root':str(product.root),'source_sha':product.source_sha,'binary':str(product.binary),
            'binary_sha256':product.binary_sha256,'build_receipt':str(product.build_receipt),
            'build_receipt_sha256':product.build_receipt_sha256,'inputs':dict(product.input_files),
            **({'runtime_inputs':dict(product.runtime_inputs)} if product.runtime_inputs else {})}
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


@final
class _OraclesOwner(_StrictOwner):
    ownership_label: str = 'oracles'
    # Diagnostic faults may overwrite retained_error; actual handles never live there alone.
    retained_live: oracles.LiveInvocation | None = None


def oracle_bindings(context: OwnedOrderedInputs) -> OracleContext:
    bound=context.oracles
    require(bound is not None,'/oracles','explicit oracle context required')
    assert bound is not None
    require(bound.inputs.product==context.components.godot.candidate,'/oracles','prepared candidate differs')
    _=oracles.inputs_current(bound.inputs)
    return bound


def oracle_dependencies(bound: OracleContext) -> dict[str,Json]:
    product=bound.inputs.product
    return {'source_sha':product.source_sha,'input_root':str(product.input_root),
        'executable':str(product.executable),'executable_sha256':product.executable_sha256,
        'inputs':dict(product.input_files),'tool_root':str(bound.inputs.tools.root),
        'tools':dict(bound.inputs.tools.files),'python':str(bound.inputs.tools.python),
        'python_sha256':bound.inputs.tools.python_sha256,'bash':str(bound.inputs.tools.bash),
        'bash_sha256':bound.inputs.tools.bash_sha256,'path':bound.inputs.tools.path,
        'effective_dependencies':{str(path):digest for path,digest in oracles.inputs_current(bound.inputs)},
        'build_authority':'CALLER_OWNED_NOT_CERTIFIED_HERE'}


def oracle_outcome(bound: OracleContext, output: Path, result: oracles.OracleResult | None) -> Outcome:
    # This function is reached only after source-owned settlement, never to infer it.
    error='source-owned settled oracle failure' if result is None else ''
    receipt: Path | None=None;digest: str | None=None;reported: str | None=None
    try:
        terminal=output/'result.json'
        if result is not None or terminal.exists():
            require(result is None or result.receipt==terminal,'/oracles','foreign terminal')
            digest=file_hash(terminal) if result is None else result.sha256
            authenticated=oracles.authenticate_result(bound.inputs,terminal,digest)
            receipt=authenticated.receipt;reported=authenticated.status
            require(result is None or result==authenticated,'/oracles','returned result differs')
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        error+='; oracle terminal authentication failed: '+safe_docs_exception(caught)
    files: tuple[tuple[str,str],...]=();snapshot_error=''
    if output.exists():
        try:files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
            snapshot_error='oracle artifact snapshot failed: '+safe_docs_exception(caught)
    return Outcome('oracles','FAIL' if error or snapshot_error else reported or 'FAIL',receipt,digest,error,
                   files,output,result.status if result is not None else None,reported,snapshot_error)


def recheck_oracle(context: OrderedInputs, row: Outcome) -> None:
    if row.component!='oracles' or row.receipt is None:return
    require(isinstance(context,OwnedOrderedInputs) and context.oracles is not None,'/oracles','missing historical context')
    assert isinstance(context,OwnedOrderedInputs) and context.oracles is not None
    require(row.sha256 is not None,'/oracles','missing historical digest')
    assert row.sha256 is not None
    result=oracles.authenticate_result(context.oracles.inputs,row.receipt,row.sha256)
    require(result.status==row.reported_status,'/oracles','historical status changed')


def invoke_oracles(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==oracle_call(),'/oracles','unknown oracle obligation')
    owner=_OraclesOwner(context.control_fd,output);entered=False;settled=False
    try:
        bound=oracle_bindings(context)
        entered=True
        try:
            result=oracles.run_owned_component(bound.inputs,output,owner.descriptor)
        except oracles.OracleSuiteOwnershipUnknown as unknown:
            owner.retained_live=unknown.live
            owner.hold(unknown)
        except oracles.OracleSettledFailure as failure:
            settled=True
            row=oracle_outcome(bound,output,None)
            primary=safe_docs_exception(failure.primary) if type(failure) is oracles.OracleSettledFailure else 'uninspected failure subtype'
            return replace(row,error=row.error+'; primary='+primary)
        settled=True
        return oracle_outcome(bound,output,result)
    except BaseException as error:
        if entered and not settled:owner.hold(error)
        raise
    finally:
        if not entered or settled:os.close(owner.descriptor)


@final
class _ResolverOwner(_StrictOwner):
    ownership_label: str = 'resolver'
    # Preserve the original failure/traceback independently of fallible status reporting.
    retained_failure: BaseException | None = None


def resolver_bindings(context: OwnedOrderedInputs) -> ResolverContext:
    bound=context.resolver
    require(bound is not None,'/resolver','explicit resolver context required')
    assert bound is not None
    require(bound.product==context.components.godot.candidate,'/resolver','prepared candidate differs')
    require(bound.environment.held_lane_fd is None,'/resolver','caller cannot supply transient ownership')
    _=resolver.bindings(bound.product,bound.environment)
    _=authenticate_prior_chain(context,bound.environment.prior_receipts)
    return bound


def resolver_dependencies(bound: ResolverContext) -> dict[str,Json]:
    product=bound.product;environment=bound.environment
    return {'source_sha':product.source_sha,'input_root':str(product.input_root),
        'executable':str(product.executable),'executable_sha256':product.executable_sha256,
        'inputs':dict(product.input_files),'environment':dict(environment.values),
        'tools':{name:{'path':str(path),'sha256':digest} for name,path,digest in environment.tools},
        'external_inputs':{str(path):digest for path,digest in environment.external_inputs},
        'prior_receipts':{str(path):digest for path,digest in environment.prior_receipts},
        'effective_dependencies':{str(path):digest for path,digest in resolver.bindings(product,environment)},
        'source_build_admission':'PENDING','historical_settlement_authority':False}


def resolver_outcome(bound: ResolverContext, output: Path, result: ComponentResult | None) -> Outcome:
    # No snapshot or historical parser may be used to settle an in-flight invocation.
    error='source-owned settled resolver failure' if result is None else ''
    receipt: Path | None=None;digest: str | None=None;reported: str | None=None
    try:
        terminal=output/'receipt.json'
        if result is not None or terminal.exists():
            require(result is None or result.receipt==terminal,'/resolver','foreign terminal')
            digest=file_hash(terminal)
            authenticated=resolver.authenticate_result(bound.product,bound.environment,output,digest)
            receipt=authenticated.receipt;reported=authenticated.status
            require(result is None or result==authenticated,'/resolver','returned result differs')
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        error+='; resolver terminal authentication failed: '+safe_docs_exception(caught)
    files: tuple[tuple[str,str],...]=();snapshot_error=''
    if output.exists():
        try:files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
            snapshot_error='resolver artifact snapshot failed: '+safe_docs_exception(caught)
    return Outcome('resolver','FAIL' if error or snapshot_error else reported or 'FAIL',receipt,digest,error,
                   files,output,result.status if result is not None else None,reported,snapshot_error)


def recheck_resolver(context: OrderedInputs, row: Outcome) -> None:
    if row.component!='resolver' or row.receipt is None:return
    require(isinstance(context,OwnedOrderedInputs) and context.resolver is not None,'/resolver','missing historical context')
    assert isinstance(context,OwnedOrderedInputs) and context.resolver is not None
    require(row.sha256 is not None and row.root is not None and row.receipt==row.root/'receipt.json',
            '/resolver','missing historical root/digest or foreign receipt')
    assert row.sha256 is not None and row.root is not None
    bound=context.resolver
    result=resolver.authenticate_result(bound.product,bound.environment,row.root,row.sha256)
    require(result.receipt==row.receipt and result.status==row.reported_status,'/resolver','historical result changed')


def invoke_resolver(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==OrderedCall(32,'resolver',('32:resolver_parity',)),'/resolver','unknown obligation')
    owner=_ResolverOwner(context.control_fd,output);entered=False;settled=False
    try:
        bound=resolver_bindings(context)
        environment=replace(bound.environment,held_lane_fd=owner.descriptor)
        entered=True
        try:
            result=resolver.run_owned(bound.product,environment,output)
        except resolver.ResolverSettledFailure as failure:
            settled=True
            row=resolver_outcome(bound,output,None)
            diagnostic='uninspected failure subtype'
            if type(failure) is resolver.ResolverSettledFailure:
                diagnostic=safe_docs_exception(failure.original)+'; secondary='+','.join(safe_docs_exception(error) for error in failure.secondary)
            return replace(row,error=row.error+'; primary='+diagnostic)
        settled=True
        return resolver_outcome(bound,output,result)
    except BaseException as error:
        if entered and not settled:
            owner.retained_failure=error
            owner.hold(error)
        raise
    finally:
        if not entered or settled:os.close(owner.descriptor)


def structural_bindings(context: OwnedOrderedInputs) -> Structural48Context:
    bound=context.structural
    require(bound is not None,'/structural','explicit structural context required')
    assert bound is not None
    require(bound.products==context.components.godot,'/structural','paired prepared Godot products differ')
    structural.coordinator_sources(bound.tools)
    require(len(dict(bound.tools.files))==len(bound.tools.files) and
        set(structural.import_closure(bound.tools.root,'generic_m6a_structural.py'))<=dict(bound.tools.files).keys(),
        '/structural/tools','incomplete or duplicate helper inventory')
    for product in (bound.products.base,bound.products.candidate):
        require(len(dict(product.input_files))==len(product.input_files) and
            structural.required_resources(product.input_root)<=dict(product.input_files).keys(),
            '/structural/inputs','incomplete or duplicate effective input inventory')
        structural.check_owned_helpers(product,bound.tools)
    _=structural_dependencies(bound)
    return bound


def structural_dependencies(bound: Structural48Context) -> dict[str,Json]:
    tools=bound.tools
    require(file_hash(tools.python)==tools.python_sha256 and file_hash(tools.bash)==tools.bash_sha256 and
        inventory(tools.root,[name for name,_ in tools.files])==dict(tools.files),'/structural/tools','tool drift')
    for product in (bound.products.base,bound.products.candidate):
        require(file_hash(product.executable)==product.executable_sha256 and
            inventory(product.input_root,[name for name,_ in product.input_files])==dict(product.input_files),
            '/structural/inputs','product drift')
    return {'products':{arm:{'source_sha':product.source_sha,'input_root':str(product.input_root),
        'executable':str(product.executable),'executable_sha256':product.executable_sha256,
        'inputs':dict(product.input_files)} for arm,product in (('base',bound.products.base),('candidate',bound.products.candidate))},
        'tool_root':str(tools.root),'tools':dict(tools.files),'python':str(tools.python),'python_sha256':tools.python_sha256,
        'bash':str(tools.bash),'bash_sha256':tools.bash_sha256,'path':tools.path,
        'adaptation':'paired_BASE_then_candidate_original_structural_recipe',
        'build_import_backend_neutrality':'CALLER_OWNED_NOT_CERTIFIED_HERE'}


def structural_outcome(bound: Structural48Context, output: Path, result: structural.OwnedComponentResult | None,
                       failure_message: str = '') -> Outcome:
    # Reached only after a returned source-owned result or typed settled finalization failure.
    error=failure_message or ('source-owned structural settled failure' if result is None else '')
    receipt: Path | None=None;digest: str | None=None;reported: str | None=None
    try:
        if result is not None:
            require(result.receipt==output/'structural-owned-component.json','/structural','foreign terminal')
            receipt=result.receipt;digest=result.receipt_sha256
            checked=structural.authenticate_owned_result(bound.products,bound.tools,receipt,digest)
            require(checked==result,'/structural','source-captured result or inventory differs')
            reported=checked.status
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        error+='; structural endpoint authentication failed: '+safe_docs_exception(caught)
    files: tuple[tuple[str,str],...]=();snapshot_error=''
    if output.exists():
        try:
            names=[path.name for path in output.iterdir()]
            files=tuple(sorted(inventory(output,names).items())) if names else ()
            require(result is None or files==result.files,'/structural','source-captured inventory changed')
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
            snapshot_error='structural artifact snapshot failed: '+safe_docs_exception(caught)
    return Outcome('structural','FAIL' if error or snapshot_error else reported or 'FAIL',receipt,digest,error,
        files,output,result.status if result is not None else None,reported,snapshot_error)


def recheck_structural(context: OrderedInputs, row: Outcome) -> None:
    if row.component!='structural' or row.receipt is None:return
    require(isinstance(context,OwnedOrderedInputs) and context.structural is not None,
            '/structural','missing historical context')
    assert isinstance(context,OwnedOrderedInputs) and context.structural is not None
    require(row.sha256 is not None,'/structural','missing source-captured digest')
    assert row.sha256 is not None
    checked=structural.authenticate_owned_result(context.structural.products,context.structural.tools,row.receipt,row.sha256)
    require(checked.status==row.reported_status and checked.files==row.files,'/structural','historical result changed')


def retain_structural_exception(original: BaseException) -> NoReturn:
    """No disposition token: preserve the exception, without claiming or guessing a lease."""
    retained=[original]
    while True:
        with suppress(BaseException):
            try:signal.pause()
            except BaseException as interruption:
                retained.append(interruption)
                raise


def invoke_structural(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==OrderedCall(48,'structural',('48:m6a.plant_art_structural',)),
            '/structural','unknown structural obligation')
    bound=structural_bindings(context)
    require(type(context.control_fd) is int and context.control_fd>2 and stat.S_ISFIFO(os.fstat(context.control_fd).st_mode),
            '/structural/control','dedicated controller pipe required')
    roots=(output,bound.tools.root,bound.products.base.input_root,bound.products.candidate.input_root)
    require(output.is_absolute() and output==output.resolve() and not output.exists() and
        all(not a.is_relative_to(b) and not b.is_relative_to(a) for i,a in enumerate(roots) for b in roots[i+1:]),
        '/structural/output','fresh disjoint canonical output required')
    require(FD_ENV not in os.environ,'/structural/lane','inherited owner unsupported')
    unclassified: BaseException | None=None
    try:
        result=structural.run_owned_result(bound.products,bound.tools,output,context.control_fd)
    except structural.StructuralSettledFailure as failure:
        message='source-owned structural settled failure'
        if type(failure) is structural.StructuralSettledFailure:
            message+='; primary='+safe_docs_exception(failure.original)+'; diagnostic='+safe_docs_exception(failure.diagnostic)
        return structural_outcome(bound,output,None,message)
    except BaseException as original:
        unclassified=original
        raise
    finally:
        if unclassified is not None:retain_structural_exception(unclassified)
    return structural_outcome(bound,output,result)


def structural_predecessor(inputs: Inputs, rows: dict[str,Outcome]) -> None:
    prior=rows.get('48:direct')
    require(prior is not None and prior.status==prior.returned_status==prior.reported_status=='PASS',
            '/structural/predecessor','step48 direct predecessor has not admitted PASS')
    assert prior is not None
    require(prior.receipt is not None and prior.sha256 is not None and prior.root is not None and
            prior.receipt==prior.root/'receipt.json','/structural/predecessor','missing direct endpoint')
    assert prior.receipt is not None
    recheck(prior)
    require(authenticate('direct',document(prior.receipt.read_text()),inputs,step=48)=='PASS',
            '/structural/predecessor','direct predecessor no longer admits PASS')


@final
class _StallOwner(_StrictOwner):
    ownership_label: str = 'stall'


def stall_bindings(context: OwnedOrderedInputs) -> StallContext:
    bound=context.stall
    require(bound is not None,'/stall','explicit stall context required')
    assert bound is not None
    require(bound.source_sha==context.components.probes.candidate.source_sha,'/stall','candidate identity differs')
    source=derived_source_binding(bound.root,bound.reference)
    require(source['origin_sha']==bound.source_sha,'/stall','derived origin differs')
    require(bound.godot==context.components.godot.candidate.executable and
            bound.godot_sha256==context.components.godot.candidate.executable_sha256 and
            file_hash(bound.godot)==bound.godot_sha256,'/stall','Godot differs')
    require(len(dict(bound.environment))==len(bound.environment) and
            len(dict(bound.tools))==len(bound.tools),'/stall','duplicate environment/tool binding')
    reserved={FD_ENV,'WORLDSIM_STALL_CHILD_PLAN','WORLDSIM_STALL_CHILD_PLAN_SHA256',
        'WORLDSIM_STALL_CHILD_CONTROL_FD','WORLDSIM_TEST_SETTINGS_DIR','WORLDSIM_MUX2_PROFILE'}
    require(not any(key in reserved or key.startswith('CORE_P7_') for key,_ in bound.environment),
            '/stall','caller cannot override capture or protocol environment')
    for tool,sha in bound.tools:require(file_hash(tool)==sha,'/stall/tools','tool changed')
    for name in ('bash','python3'):
        selected=shutil.which(name,path=dict(bound.environment).get('PATH','/usr/bin:/bin:/usr/sbin:/sbin'))
        require(selected is not None and Path(selected).resolve() in dict(bound.tools),'/stall/tools','unbound shell/interpreter')
    require(Path(sys.executable).resolve() in dict(bound.tools),'/stall/tools','current interpreter unbound')
    return bound


def stall_dependencies(bound: StallContext) -> dict[str,Json]:
    return {'source':derived_source_binding(bound.root,bound.reference),'root':str(bound.root),
        'source_sha':bound.source_sha,'godot':str(bound.godot),'godot_sha256':bound.godot_sha256,
        'environment':dict(bound.environment),'tools':{str(p):sha for p,sha in bound.tools},
        'inputs':snapshot(bound.root),'source_build_runtime_authority':'CALLER_OWNED_NOT_CERTIFIED_HERE',
        'ordinary_safety':'PENDING','final_conjunction':'PENDING'}


def stall_plan(bound: StallContext, output: Path, *, create: bool = False) -> ChildPlan:
    inputs={(bound.root/name,Node(sha).text()) for name,sha in snapshot(bound.root).items()}
    inputs.update(bound.tools);inputs.update({(bound.godot,bound.godot_sha256),(bound.reference.path,bound.reference.sha256)})
    require(len({p for p,_ in inputs})==len(inputs),'/stall','conflicting input hashes')
    capture=output/'capture';evidence=output/'chain';path=output/'child-plan.json'
    argv=(str(bound.godot),'--path',str(bound.root),'--resolution','1920x1080','--always-on-top',
        '--windowed','--script','res://tools/v8/mux2_stall_test.gd','--',str(capture/'run'))
    node: dict[str,Json]={'schema':'stall_bounded_child_v1','protocol':'NONE','root':str(bound.root),
        'capture':str(capture),'evidence':str(evidence),'argv':list(argv),'inputs':{str(p):sha for p,sha in sorted(inputs)}}
    if create:
        output.mkdir();evidence.mkdir();atomic_json(path,node)
    require(document(path.read_text()).value==node,'/stall','child plan differs')
    return ChildPlan(path,file_hash(path),bound.root,capture,evidence,argv,tuple(sorted(inputs)))


def stall_environment(bound: StallContext, child: ChildPlan, descriptor: int) -> dict[str,str]:
    require(type(descriptor) is int and descriptor>2,'/stall','invalid historical/actual descriptor')
    spec=ProcessSpec(child.argv,bound.root,bound.godot,bound.godot_sha256,bound.root,(),child.evidence,
        environment=bound.environment)
    return {**environment_for(spec),FD_ENV:str(descriptor),'WORLDSIM_STALL_CHILD_PLAN':str(child.path),
        'WORLDSIM_STALL_CHILD_PLAN_SHA256':child.digest,
        'WORLDSIM_TEST_SETTINGS_DIR':str(child.capture/'settings'),'WORLDSIM_MUX2_PROFILE':'1'}


def stall_evidence(bound: StallContext, output: Path, completion: stall_capture.DerivedCaptureCompletion,
                   receipt_sha256: str, files: tuple[tuple[str,str],...]) -> str:
    child=stall_plan(bound,output)
    require(child==completion.child,'/stall','foreign completion plan')
    env=dict(completion.environment)
    descriptor=env.get(FD_ENV,'')
    require(descriptor.isdecimal() and env==stall_environment(bound,child,int(descriptor)),
            '/stall','capture environment differs')
    require(dict(inventory(child.capture,[p.name for p in child.capture.iterdir()]).items())==dict(files),
            '/stall','capture changed since producer')
    inputs=tuple((name,Node(sha).text()) for name,sha in snapshot(bound.root).items())
    external=tuple((p,sha) for p,sha in child.inputs if not p.is_relative_to(bound.root))
    spec=ProcessSpec(child.argv,bound.root,bound.godot,bound.godot_sha256,bound.root,inputs,child.evidence,
        environment=tuple(projected_environment(bound.root,env).items()),external_inputs=external)
    native_sha=dict(completion.chain_files).get('native-terminal.json')
    require(native_sha is not None,'/stall','missing native terminal')
    assert native_sha is not None
    plan=StallOutcomePlan('step41','NONE',bound.source_sha,child.capture,spec,receipt_sha256,native_sha,files)
    evidence=validate_derived_bounded_outcome(plan,child,env,completion.chain_files,bound.reference)
    require(evidence.outcome.native_exit==completion.native_exit,'/stall','native exit differs')
    return evidence.outcome.verdict.status


def stall_outcome(bound: StallContext, output: Path, result: stall_capture.OwnedDerivedCapture | None,
                  failure: str = '') -> Outcome:
    error=failure;reported: str | None=None;receipt: Path | None=None;sha: str | None=None
    try:
        if result is not None:
            require(result.receipt==output/'capture/receipt.json','/stall','foreign capture receipt')
            reported=stall_evidence(bound,output,result.completion,result.sha256,result.files)
            receipt=result.receipt;sha=result.sha256
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        error+='; '+safe_docs_exception(caught)
    files: tuple[tuple[str,str],...]=();snapshot_error=''
    try:
        files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:snapshot_error=safe_docs_exception(caught)
    return Outcome('stall','FAIL' if error or snapshot_error or reported is None else 'PENDING',receipt,sha,error,
        files,output,None,reported,snapshot_error)


def recheck_stall(context: OrderedInputs, row: Outcome) -> None:
    if row.component!='stall' or row.receipt is None:return
    require(isinstance(context,OwnedOrderedInputs) and context.stall is not None and row.root is not None and
            row.sha256 is not None,'/stall','missing historical bindings')
    assert isinstance(context,OwnedOrderedInputs) and context.stall is not None and row.root is not None and row.sha256 is not None
    require(tuple(inventory(row.root,[p.name for p in row.root.iterdir()]).items())==row.files,'/stall','retained inventory changed')
    child=stall_plan(context.stall,row.root)
    wrapper=document((child.evidence/'wrapper-start.json').read_text())
    env=tuple((k,Node(v).text()) for k,v in wrapper.field('environment').mapping().items())
    chain=tuple(inventory(child.evidence,[p.name for p in child.evidence.iterdir()]).items())
    code=integer(document((child.evidence/'native-terminal.json').read_text()).field('native_exit'))
    completion=stall_capture.DerivedCaptureCompletion(child,env,chain,code)
    files=tuple(sorted((name.removeprefix('capture/'),sha) for name,sha in row.files if name.startswith('capture/')))
    require(stall_evidence(context.stall,row.root,completion,row.sha256,files)==row.reported_status,
            '/stall','retained result differs')
    require(tuple(inventory(row.root,[p.name for p in row.root.iterdir()]).items())==row.files,'/stall','retained inventory changed during validation')


def invoke_stall(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==OrderedCall(41,'stall',('41:mux2_stall_receipt',)),'/stall','unknown obligation')
    owner=_StallOwner(context.control_fd,output)
    entered=False;settled=False;previous=dict(os.environ)
    try:
        bound=stall_bindings(context)
        require(output.is_absolute() and output==output.resolve() and not output.exists() and
            not output.is_relative_to(bound.root) and not bound.root.is_relative_to(output),'/stall','fresh disjoint output required')
        child=stall_plan(bound,output,create=True)
        env=stall_environment(bound,child,owner.descriptor)
        os.environ.clear();os.environ.update(env)
        entered=True
        try:
            result=stall_capture.capture_derived_stall_owned(bound.root,child.capture,bound.godot,bound.reference)
        except stall_capture.DerivedCaptureSettledFailure as failure:
            settled=True
            error='source-owned settled stall failure; primary='+safe_docs_exception(failure.original)
            error+='; diagnostics='+','.join(safe_docs_exception(item) for item in failure.diagnostics)
            return stall_outcome(bound,output,None,error)
        settled=True
        return stall_outcome(bound,output,result)
    except BaseException as error:
        if entered and not settled:owner.hold(error)
        raise
    finally:
        if not entered or settled:
            os.environ.clear();os.environ.update(previous)
            os.close(owner.descriptor)


@final
class _Library49Owner(_StrictOwner):
    ownership_label: str = 'library49'
    retained_request: library49.Library49Request | None = None
    retained_failure: BaseException | None = None


def library49_bindings(context: OwnedOrderedInputs, step: Literal[42,43,49] = 49) -> Library49Context:
    bound={42:context.library42,43:context.library43,49:context.library49}[step]
    require(bound is not None,'/library49','explicit context required')
    assert bound is not None
    candidate=context.components.probes.candidate;request=bound.request
    require(bound.source_sha==candidate.source_sha and request.root==candidate.input_root,
            '/library49','prepared candidate source differs')
    require(request.arm=='candidate' and request.held_lane_fd==-1,'/library49','candidate template without transient FD required')
    require(bool(request.invocation) and request.executable.step==step and request.executable.profile==library_recipe(LibraryStep(step)).profile,
            '/library49','wrong invocation or selected executable')
    supplied=dict(request.inputs)
    require(len(supplied)==len(request.inputs) and all(supplied.get(name)==digest for name,digest in candidate.input_files),
            '/library49','candidate effective inputs omitted or changed')
    require(len(dict(request.tools))==len(request.tools),'/library49','duplicate tool binding')
    for name,digest in request.inputs:require(file_hash(request.root/name)==digest,'/library49','source input changed')
    for path,digest in request.tools:require(file_hash(path)==digest,'/library49','tool or executable changed')
    return bound


def library49_dependencies(bound: Library49Context) -> dict[str,Json]:
    request=bound.request;selected=request.executable;target=selected.target
    return {'source_sha':bound.source_sha,'root':str(request.root),'invocation':request.invocation,'arm':request.arm,
        'binary':str(selected.binary),'target':{'package_id':target.package_id,'package':target.package,'name':target.name,
        'kinds':list(target.kinds),'source':str(target.source),'manifest':str(target.manifest),'doctest':target.doctest,'test':target.test},
        'step':selected.step,'profile':selected.profile,'filter':selected.filter,'ignored':selected.ignored,
        'template_output':str(request.output),'cargo':str(request.cargo),'bash':str(request.bash),
        'python':str(request.python),'helper':str(request.helper),'host':request.host,
        'inputs':dict(request.inputs),'tools':{str(path):digest for path,digest in request.tools},
        'environment':dict(request.environment),'source_build_admission':'PENDING_CALLER_HANDOFF',
        'observer_qualification':'PENDING_ORIGINAL_DIRECT_CONTROL','historical_settlement_authority':False}


def authenticate_library49(bound: Library49Context, output: Path, result: library49.Library49Result) -> str:
    # Only an actual in-process source-owned return supplies result; JSON is never completion authority.
    require(result.receipt==output/'result.json','/library49','foreign source-owned terminal')
    require(result.source_build_admission=='PENDING_CALLER_HANDOFF'
            and result.observer_qualification=='PENDING_ORIGINAL_DIRECT_CONTROL' and not result.publication_authority,
            '/library49','unsupported admission upgrade')
    node=document(result.receipt.read_text());request=bound.request
    recipe=library_recipe(LibraryStep(request.executable.step))
    values: dict[str,Json]={'schema_version':1,'component':f'LIBRARY{int(recipe.step)}_RUNTIME','status':result.status,'step':int(recipe.step),
        'invocation_id':request.invocation,'arm':request.arm,'source_build_admission':'PENDING_CALLER_HANDOFF',
        'observer_qualification':'PENDING_ORIGINAL_DIRECT_CONTROL','observer_transformation':'PATH_CARGO_SHIM_AND_HOST_RUNNER',
        'original_environment_byte_identity':False,'recipe_scope':recipe.source_classification,
        'world_admission':'PENDING_SOURCE_BUILD_HANDOFF','publication_authority':False}
    for key,value in values.items():node.field(key).equals(value)
    node.field('native_exit').equals(result.accounting.native_exit)
    require(result.accounting.recipe==recipe and result.status in {'PASS','FAIL'},'/library49','wrong returned accounting')
    require(result.status!='PASS' or result.accounting.accounting_status=='PASS','/library49','successful return contradicts accounting')
    for key in ('outer_pid','cargo_pid','native_pid'):
        value=node.field(key).value;require(type(value) is int and value>0,'/library49','missing actual PID')
    node.field('output_inventory').equals({name:digest for name,digest in inventory(output,[p.name for p in output.iterdir()]).items()
                                         if name!='result.json'})
    return result.status


def library49_outcome(bound: Library49Context, output: Path, result: library49.Library49Result | None) -> Outcome:
    error='source-owned settled library49 failure' if result is None else ''
    receipt: Path | None=None;digest: str | None=None;reported: str | None=None
    try:
        if result is not None:
            reported=authenticate_library49(bound,output,result);receipt=result.receipt;digest=file_hash(receipt)
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        error+='; library49 terminal authentication failed: '+safe_docs_exception(caught)
    files: tuple[tuple[str,str],...]=();snapshot_error=''
    if output.exists():
        try:files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
            snapshot_error='library49 artifact snapshot failed: '+safe_docs_exception(caught)
    members=tuple(sorted(str(path.relative_to(output)) for path in output.rglob('*'))) if output.exists() else ()
    return Outcome(f'library{bound.request.executable.step}','FAIL' if error or snapshot_error or reported!='PASS' else 'PENDING',receipt,digest,error,
        files,output,None if result is None else result.status,reported,snapshot_error,result,members)


def recheck_library49(context: OrderedInputs, row: Outcome) -> None:
    if row.component not in {'library42','library43','library49'}:return
    if row.root is None:
        require(row.status in {'FAIL','NOT_RUN'} and row.library49_result is None and row.receipt is None and not row.files,
                '/library49','unanchored completed result')
        return
    require(isinstance(context,OwnedOrderedInputs),'/library49','missing historical binding')
    assert isinstance(context,OwnedOrderedInputs)
    bound={'library42':context.library42,'library43':context.library43,'library49':context.library49}[row.component]
    require(bound is not None,'/library49','missing historical binding')
    assert bound is not None
    recheck(row)
    require(tuple(sorted(str(path.relative_to(row.root)) for path in row.root.rglob('*')))==row.library49_members,
            '/library49','completed directory membership changed')
    if row.library49_result is not None:
        require(authenticate_library49(bound,row.root,row.library49_result)==row.reported_status,
                '/library49','source-owned returned result changed')
    recheck(row)


def invoke_library49(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==OrderedCall(49,'library49',('49:read_data_only',)),'/library49','unknown obligation')
    return _invoke_library(call,context,output,49)


def invoke_library_sibling(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    if call==OrderedCall(42,'library42',('42:baseline_isolation',)):
        return _invoke_library(call,context,output,42)
    require(call==OrderedCall(43,'library43',('43:mind_data_only',)),'/library','unknown sibling obligation')
    return _invoke_library(call,context,output,43)


def _invoke_library(call: OrderedCall, context: OwnedOrderedInputs, output: Path, step: Literal[42,43,49]) -> Outcome:
    owner=_Library49Owner(context.control_fd,output);owner.ownership_label=call.component;entered=False;settled=False
    try:
        bound=library49_bindings(context,step)
        request=replace(bound.request,output=output,held_lane_fd=owner.descriptor)
        validate={42:library49.checked42_request,43:library49.checked43_request,49:library49.checked_request}[step]
        launch={42:library49.run42_owned,43:library49.run43_owned,49:library49.run49_owned}[step]
        _=validate(request)
        owner.retained_request=request
        entered=True
        try:
            result=launch(request)
        except library49.LibraryOwnershipUnknown as unknown:
            owner.retained_request=unknown.request
            owner.retained_failure=unknown
            owner.hold(unknown)
        except library49.LibrarySettledFailure as failure:
            settled=True
            row=library49_outcome(bound,output,None)
            return replace(row,error=row.error+'; primary='+safe_docs_exception(failure.original))
        settled=True
        return library49_outcome(bound,output,result)
    except BaseException as error:
        if entered and not settled:
            owner.retained_failure=error
            owner.hold(error)
        raise
    finally:
        if not entered or settled:os.close(owner.descriptor)


@final
class _M1Owner(_StrictOwner):
    ownership_label: str = 'm1'
    retained_request: m1_runtime.M1Request | None = None
    retained_live: m1_runtime.Live | None = None
    retained_failure: BaseException | None = None


def m1_call() -> OrderedCall:
    identities=tuple(row.identity for row in obligations() if row.step==17)
    require(len(identities)==8,'/m1','original eight obligations required')
    return OrderedCall(17,'m1',identities)


def m1_bindings(context: OwnedOrderedInputs) -> M1Context:
    bound=context.m1
    require(bound is not None,'/m1','explicit context required')
    assert bound is not None
    request=bound.request;r=request.template;candidate=context.components.probes.candidate
    require(bound.source_sha==candidate.source_sha and r.root==candidate.input_root,
            '/m1','candidate source differs')
    require(r.arm=='candidate' and r.held_lane_fd==-1 and bool(r.invocation),'/m1','candidate request template required')
    require(request.godot==context.components.godot.candidate.executable and
            context.components.godot.candidate.source_sha==bound.source_sha and
            file_hash(request.godot)==context.components.godot.candidate.executable_sha256,'/m1','Godot product differs')
    inputs=dict(r.inputs)
    require(len(inputs)==len(r.inputs) and all(inputs.get(name)==sha for name,sha in candidate.input_files),
            '/m1','candidate input scope differs')
    require(len(dict(r.tools))==len(r.tools),'/m1','duplicate tool binding')
    for name,sha in r.inputs:require(file_hash(r.root/name)==sha,'/m1','source input drift')
    for path,sha in r.tools:require(file_hash(path)==sha,'/m1','tool drift')
    return bound


def m1_dependencies(bound: M1Context) -> dict[str,Json]:
    request=bound.request
    return {'template':library49_dependencies(Library49Context(bound.source_sha,request.template)),
        'godot':str(request.godot),'executables':[{'binary':str(e.binary),'cwd':str(e.cwd),
        'step':e.step,'profile':e.profile,'filter':e.filter,'ignored':e.ignored,'threads':e.threads,
        'package_id':e.target.package_id,'package':e.target.package,'target':e.target.name,
        'kinds':list(e.target.kinds),'manifest':str(e.target.manifest),'source':str(e.target.source),
        'test':e.target.test,'doctest':e.target.doctest} for e in request.executables]}


def authenticate_m1(bound: M1Context, output: Path, result: m1_runtime.M1Result) -> str:
    require(result.receipt==output/'result.json' and result.status in {'PASS','FAIL'} and result.qualification=='PENDING',
            '/m1','foreign return or qualification upgrade')
    node=document(result.receipt.read_bytes().decode())
    for key,value in {'schema':1,'component':'M1_RUNTIME','step':17,'status':result.status,
                      'invocation':bound.request.template.invocation,'source_build_qualification':'PENDING',
                      'observer_qualification':'PENDING','world_qualification':'PENDING','publication_authority':False}.items():
        node.field(key).equals(value)
    pid=node.field('outer_pid').value;code=node.field('outer_exit').value
    require(type(pid) is int and pid>0 and type(code) is int and code>=0,'/m1','missing actual outer wait')
    completed=tuple(row.text() for row in node.field('completed_producers').array())
    skipped=tuple(row.text() for row in node.field('skipped_producers').array())
    require(len(completed)==len(set(completed)) and set(completed)<=set(m1_runtime.PRODUCERS)
            and skipped==tuple(name for name in m1_runtime.PRODUCERS if name not in completed),'/m1','producer scope differs')
    require(result.status!='PASS' or code==0 and not skipped and len(node.field('cases').array())==7,
            '/m1','successful body incomplete')
    state=integration_prefix.output_state(output)
    node.field('output_inventory').equals({name:sha for name,(kind,sha) in state.items() if kind=='file' and name!='result.json'})
    members: list[Json]=[name for name in sorted(state) if name!='result.json']
    node.field('output_members').equals(members)
    return result.status


def recheck_m1_anchor(row: Outcome) -> None:
    if row.component!='m1' or row.root is None:return
    recheck(row)
    require(tuple(sorted(integration_prefix.output_state(row.root).items()))==row.m1_state,
            '/m1','completed output membership/kind/hash changed')


def recheck_m1(context: OrderedInputs, row: Outcome) -> None:
    if row.component!='m1':return
    if row.root is None:
        require(row.status in {'FAIL','NOT_RUN'} and row.m1_result is None and not row.m1_state,'/m1','missing anchor')
        return
    require(isinstance(context,OwnedOrderedInputs) and row.m1_result is not None,'/m1','missing actual returned result')
    assert isinstance(context,OwnedOrderedInputs) and row.m1_result is not None
    recheck_m1_anchor(row)
    require(authenticate_m1(m1_bindings(context),row.root,row.m1_result)==row.reported_status,'/m1','result changed')
    recheck_m1_anchor(row)


def m1_outcome(bound: M1Context, output: Path, result: m1_runtime.M1Result) -> Outcome:
    error='';reported: str | None=None;sha: str | None=None
    state: tuple[tuple[str,tuple[str,str]], ...]=();snapshot_error=''
    try:reported=authenticate_m1(bound,output,result);sha=file_hash(result.receipt)
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:error=safe_docs_exception(caught)
    try:state=tuple(sorted(integration_prefix.output_state(output).items()))
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:snapshot_error=safe_docs_exception(caught)
    return Outcome('m1','PENDING' if not error and not snapshot_error and reported=='PASS' else 'FAIL',
        result.receipt if sha is not None else None,sha,error,
        tuple((name,digest) for name,(kind,digest) in state if kind=='file'),output,result.status,reported,snapshot_error,
        m1_result=result,m1_state=state)


def invoke_m1(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==m1_call(),'/m1','wrong original-body obligation')
    owner=_M1Owner(context.control_fd,output);entered=False;settled=False
    try:
        bound=m1_bindings(context)
        request=replace(bound.request,template=replace(bound.request.template,output=output,held_lane_fd=owner.descriptor))
        _=m1_runtime.checked(request);owner.retained_request=request;entered=True
        try:result=m1_runtime.run_owned(request)
        except m1_runtime.M1OwnershipUnknown as unknown:
            owner.retained_live=unknown.live;owner.retained_request=unknown.live.request
            owner.retained_failure=unknown;owner.hold(unknown)
        settled=True
        return m1_outcome(bound,output,result)
    except BaseException as error:
        if entered and not settled:owner.retained_failure=error;owner.hold(error)
        raise
    finally:
        if not entered or settled:os.close(owner.descriptor)


@final
class _Mux2Owner(_StrictOwner):
    ownership_label: str = 'mux2'
    retained_request: mux2_runtime.DefenceRequest | None = None
    retained_live: mux2_runtime.Live | None = None
    retained_failure: BaseException | None = None


def mux2_call() -> OrderedCall:
    identities=tuple(row.identity for row in obligations() if row.step==37)
    require(len(identities)==11,'/mux2','original eleven obligations required')
    return OrderedCall(37,'mux2',identities)


def mux2_bindings(context: OwnedOrderedInputs) -> Mux2Context:
    bound=context.mux2
    require(bound is not None,'/mux2','explicit context required')
    assert bound is not None
    request=bound.request;r=request.template;candidate=context.components.probes.candidate
    require(bound.source_sha==candidate.source_sha and r.root==candidate.input_root,
            '/mux2','candidate source differs')
    require(r.arm=='candidate' and r.held_lane_fd==-1 and bool(r.invocation),'/mux2','candidate request template required')
    inputs=dict(r.inputs)
    require(len(inputs)==len(r.inputs) and all(inputs.get(name)==sha for name,sha in candidate.input_files),
            '/mux2','candidate input scope differs')
    require(len(dict(r.tools))==len(r.tools),'/mux2','duplicate tool binding')
    for name,sha in r.inputs:require(file_hash(r.root/name)==sha,'/mux2','source input drift')
    for path,sha in r.tools:require(file_hash(path)==sha,'/mux2','tool drift')
    return bound


def mux2_dependencies(bound: Mux2Context) -> dict[str,Json]:
    request=bound.request
    return {'template':library49_dependencies(Library49Context(bound.source_sha,request.template)),
        'legacy':str(request.legacy),'h9':str(request.h9),'v10':str(request.v10),'manifest':str(request.manifest),
        'fixture_members':{str(path):{name:[kind,sha] for name,(kind,sha) in integration_prefix.output_state(path).items()}
                           for path in (request.legacy,request.v10)},
        'h9_sha256':file_hash(request.h9),'manifest_sha256':file_hash(request.manifest),'executables':[{'binary':str(e.binary),'cwd':str(e.cwd),
        'step':e.step,'profile':e.profile,'filter':e.filter,'ignored':e.ignored,'threads':e.threads,
        'package_id':e.target.package_id,'package':e.target.package,'target':e.target.name,
        'kinds':list(e.target.kinds),'manifest':str(e.target.manifest),'source':str(e.target.source),
        'test':e.target.test,'doctest':e.target.doctest} for e in request.executables]}


def authenticate_mux2(bound: Mux2Context, output: Path, result: mux2_runtime.DefenceResult) -> str:
    require(result.receipt==output/'result.json' and result.status in {'PASS','FAIL'} and result.qualification=='PENDING',
            '/mux2','foreign return or qualification upgrade')
    node=document(result.receipt.read_bytes().decode())
    for key,value in {'schema':1,'component':'MUX2_DEFENCE_RUNTIME','step':37,'status':result.status,
                      'invocation':bound.request.template.invocation,'source_build_qualification':'PENDING',
                      'observer_qualification':'PENDING','fixture_provenance':'PENDING','world_safety':'PENDING','publication_authority':False}.items():
        node.field(key).equals(value)
    caller=node.field('caller');caller.field('wait_observed').equals(True)
    caller.field('argv').equals([str(bound.request.template.bash),'tools/v8/mux2_gate_checks.sh','defence',str(output/'logs')])
    caller.field('cwd').equals(str(bound.request.template.root))
    pid=caller.field('pid').value;code=caller.field('native_exit').value
    require(type(pid) is int and pid>0 and type(code) is int and code>=0,'/mux2','missing actual outer wait')
    completed=tuple(row.text() for row in node.field('completed_producers').array())
    skipped=tuple(row.text() for row in node.field('skipped_producers').array())
    require(len(completed)==len(set(completed)) and set(completed)<=set(mux2_runtime.PRODUCERS)
            and skipped==tuple(name for name in mux2_runtime.PRODUCERS if name not in completed),'/mux2','producer scope differs')
    require(result.status!='PASS' or code==0 and not skipped and len(node.field('cases').array())==10,
            '/mux2','successful body incomplete')
    state=integration_prefix.output_state(output)
    node.field('output_inventory').equals({name:sha for name,(kind,sha) in state.items() if kind=='file' and name!='result.json'})
    members: list[Json]=[name for name in sorted(state) if name!='result.json']
    node.field('output_members').equals(members)
    return result.status


def recheck_mux2_anchor(row: Outcome) -> None:
    if row.component!='mux2' or row.root is None:return
    recheck(row)
    require(tuple(sorted(integration_prefix.output_state(row.root).items()))==row.mux2_state,
            '/mux2','completed output membership/kind/hash changed')


def recheck_mux2(context: OrderedInputs, row: Outcome) -> None:
    if row.component!='mux2':return
    if row.root is None:
        require(row.status in {'FAIL','NOT_RUN'} and row.mux2_result is None and not row.mux2_state,'/mux2','missing anchor')
        return
    require(isinstance(context,OwnedOrderedInputs) and row.mux2_result is not None,'/mux2','missing actual returned result')
    assert isinstance(context,OwnedOrderedInputs) and row.mux2_result is not None
    recheck_mux2_anchor(row)
    require(authenticate_mux2(mux2_bindings(context),row.root,row.mux2_result)==row.reported_status,'/mux2','result changed')
    recheck_mux2_anchor(row)


def mux2_outcome(bound: Mux2Context, output: Path, result: mux2_runtime.DefenceResult) -> Outcome:
    error='';reported: str | None=None;sha: str | None=None
    state: tuple[tuple[str,tuple[str,str]], ...]=();snapshot_error=''
    try:state=tuple(sorted(integration_prefix.output_state(output).items()))
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:snapshot_error=safe_docs_exception(caught)
    try:
        reported=authenticate_mux2(bound,output,result);sha=file_hash(result.receipt)
        require(tuple(sorted(integration_prefix.output_state(output).items()))==state,
                '/mux2','output changed during first authentication')
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:error=safe_docs_exception(caught)
    return Outcome('mux2','PENDING' if not error and not snapshot_error and reported=='PASS' else 'FAIL',
        result.receipt if sha is not None else None,sha,error,
        tuple((name,digest) for name,(kind,digest) in state if kind=='file'),output,result.status,reported,snapshot_error,
        mux2_result=result,mux2_state=state)


def invoke_mux2(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==mux2_call(),'/mux2','wrong original-body obligation')
    owner=_Mux2Owner(context.control_fd,output);entered=False;settled=False
    try:
        bound=mux2_bindings(context)
        request=replace(bound.request,template=replace(bound.request.template,output=output,held_lane_fd=owner.descriptor))
        _=mux2_runtime.checked(request);owner.retained_request=request;entered=True
        try:result=mux2_runtime.run_defence_owned(request)
        except mux2_runtime.DefenceOwnershipUnknown as unknown:
            owner.retained_live=unknown.live;owner.retained_request=unknown.live.request
            owner.retained_failure=unknown;owner.hold(unknown)
        settled=True
        return mux2_outcome(bound,output,result)
    except BaseException as error:
        if entered and not settled:owner.retained_failure=error;owner.hold(error)
        raise
    finally:
        if not entered or settled:os.close(owner.descriptor)


def prefix_call(step: int) -> OrderedCall:
    name={28:'colony_knowledge.release',36:'data_only_ext.debug'}[step]
    return OrderedCall(step,'native_prefix',(f'{step}:{name}',))


def prefix_bindings(context: OwnedOrderedInputs, step: int) -> Library49Context:
    bound={28:context.prefix28,36:context.prefix36}[step]
    require(bound is not None,'/prefix','explicit context required')
    assert bound is not None
    candidate=context.components.probes.candidate;request=bound.request
    require(bound.source_sha==candidate.source_sha and request.root==candidate.input_root,
            '/prefix','prepared candidate source differs')
    require(request.arm=='candidate' and request.held_lane_fd==-1 and bool(request.invocation),
            '/prefix','candidate request template required')
    name,profile,threads,_=integration_prefix.recipe(step)
    selected=request.executable
    require(selected.step==step and selected.target.name==name and selected.profile==profile
            and selected.threads==threads and selected.filter is None and selected.ignored=='exclude',
            '/prefix','wrong native invocation')
    supplied=dict(request.inputs)
    require(len(supplied)==len(request.inputs) and all(supplied.get(name)==sha for name,sha in candidate.input_files),
            '/prefix','candidate inputs omitted or changed')
    require(len(dict(request.tools))==len(request.tools),'/prefix','duplicate tool binding')
    for name,sha in request.inputs:require(file_hash(request.root/name)==sha,'/prefix','source drift')
    for path,sha in request.tools:require(file_hash(path)==sha,'/prefix','tool drift')
    return bound


def authenticate_prefix(bound: Library49Context, output: Path, result: integration_prefix.NativePrefixResult) -> str:
    require(result.receipt==output/'result.json' and result.status in {'PASS_PREFIX_ONLY','FAIL'},
            '/prefix','foreign source-owned return')
    require(result.compound_tail=='NOT_RUN' and result.source_build_admission=='PENDING_CALLER_HANDOFF'
            and result.ordinary_safety=='PENDING_SEPARATE_COMPONENT' and not result.publication_authority,
            '/prefix','unsupported authority upgrade')
    node=document(result.receipt.read_bytes().decode());step=bound.request.executable.step
    for key,value in {'schema_version':1,'component':'INTEGRATION_PREFIX','step':step,'status':result.status,
                      'compound_tail':'NOT_RUN','source_build_admission':'PENDING_CALLER_HANDOFF',
                      'ordinary_safety':'PENDING_SEPARATE_COMPONENT','publication_authority':False}.items():
        node.field(key).equals(value)
    for key in ('outer_pid','cargo_pid','native_pid'):
        value=node.field(key).value;require(type(value) is int and value>0,'/prefix','missing process identity')
    exits=[node.field(key).value for key in ('outer_exit','native_exit')]
    require(all(type(code) is int and code>=0 for code in exits),'/prefix','missing native outcomes')
    expected='PASS_PREFIX_ONLY' if exits==[0,0] and result.accounting.section.failed==0 else 'FAIL'
    require(result.status==expected,'/prefix','return contradicts native outcomes')
    node.field('nested_ranges').equals([[start,end,sha] for start,end,sha in result.accounting.nested_ranges])
    state=integration_prefix.output_state(output)
    node.field('output_inventory').equals({name:sha for name,(kind,sha) in state.items() if kind=='file' and name!='result.json'})
    node.field('output_membership').equals({name:kind for name,(kind,_sha) in state.items() if name!='result.json'})
    return result.status


def prefix_outcome(bound: Library49Context, output: Path, result: integration_prefix.NativePrefixResult | None) -> Outcome:
    error='source-owned settled native prefix failure' if result is None else ''
    receipt: Path | None=None;sha: str | None=None;reported: str | None=None
    state: tuple[tuple[str,tuple[str,str]], ...]=();snapshot_error=''
    try:
        if result is not None:
            reported=authenticate_prefix(bound,output,result);receipt=result.receipt;sha=file_hash(receipt)
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        error+='; prefix authentication failed: '+safe_docs_exception(caught)
    try:
        if output.exists():state=tuple(sorted(integration_prefix.output_state(output).items()))
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        snapshot_error=safe_docs_exception(caught)
    return Outcome('native_prefix','PENDING' if not error and not snapshot_error and reported=='PASS_PREFIX_ONLY' else 'FAIL',
        receipt,sha,error,tuple((name,digest) for name,(kind,digest) in state if kind=='file'),output,
        None if result is None else result.status,reported,snapshot_error,prefix_result=result,prefix_state=state)


def recheck_prefix_anchor(row: Outcome) -> None:
    if row.component!='native_prefix' or row.root is None:return
    recheck(row)
    require(tuple(sorted(integration_prefix.output_state(row.root).items()))==row.prefix_state,
            '/prefix','completed output membership/kind/hash changed')


def recheck_prefix(context: OrderedInputs, row: Outcome) -> None:
    if row.component!='native_prefix':return
    if row.root is None:
        require(row.status in {'FAIL','NOT_RUN'} and row.prefix_result is None and not row.prefix_state,
                '/prefix','missing evidence anchor')
        return
    recheck_prefix_anchor(row)
    if row.prefix_result is not None:
        require(isinstance(context,OwnedOrderedInputs),'/prefix','missing owned binding')
        assert isinstance(context,OwnedOrderedInputs)
        step=document(row.prefix_result.receipt.read_bytes().decode()).field('step').value
        require(type(step) is int and step in (28,36),'/prefix','wrong historical step')
        assert isinstance(step,int)
        require(authenticate_prefix(prefix_bindings(context,step),row.root,row.prefix_result)==row.reported_status,
                '/prefix','returned result changed')
    recheck_prefix_anchor(row)


def invoke_prefix(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call.step in (28,36) and call==prefix_call(call.step),'/prefix','wrong obligation')
    owner=_Library49Owner(context.control_fd,output);owner.ownership_label=f'prefix{call.step}'
    entered=False;settled=False
    try:
        bound=prefix_bindings(context,call.step)
        request=replace(bound.request,output=output,held_lane_fd=owner.descriptor)
        _=integration_prefix.checked_request(request,call.step)
        owner.retained_request=request;entered=True
        try:result=integration_prefix.run_owned(request,call.step)
        except library49.LibraryOwnershipUnknown as unknown:
            owner.retained_request=unknown.request;owner.retained_failure=unknown;owner.hold(unknown)
        except library49.LibrarySettledFailure as failure:
            settled=True
            row=prefix_outcome(bound,output,None)
            return replace(row,error=row.error+'; primary='+safe_docs_exception(failure.original))
        settled=True
        return prefix_outcome(bound,output,result)
    except BaseException as error:
        if entered and not settled:owner.retained_failure=error;owner.hold(error)
        raise
    finally:
        if not entered or settled:os.close(owner.descriptor)


def play_call() -> OrderedCall:
    return OrderedCall(21,'play',tuple(row.identity for row in obligations() if row.step==21))


def play_bindings(context: OwnedOrderedInputs) -> PlayContext:
    bound=context.play
    require(bound is not None,'/play','explicit context required')
    assert bound is not None
    plan=bound.plan;candidate=context.components.probes.candidate
    require(plan.historical is not None,'/play','historical source grants required')
    assert plan.historical is not None
    play_sources.check_bindings(plan.historical)
    require(plan.historical.sources[0].commit==context.components.probes.base.source_sha
            ==context.components.declarations.base,'/play','historical main differs from published baseline')
    _=freeze_source(plan.root,candidate.source_sha)
    supplied=dict(plan.inputs)
    require(len(supplied)==len(plan.inputs) and all(supplied.get(plan.root/name)==digest
            for name,digest in candidate.input_files),'/play','candidate launcher input closure differs')
    play_owner.current(plan)
    return bound


def play_dependencies(bound: PlayContext) -> dict[str,Json]:
    plan=bound.plan
    require(plan.historical is not None,'/play','missing historical source grants')
    assert plan.historical is not None
    declaration=play_sources.SourceGrants(plan.historical,plan.root,Path(dict(plan.environment)['PLAY_ROOT'])).declaration()
    return {'root':str(plan.root),'template_output':str(plan.output),'suite':str(plan.suite),
        'suite_sha256':plan.suite_sha256,'inputs':{str(p):h for p,h in plan.inputs},
        'environment':dict(plan.environment),'child_environments':[
            {'case':case,'role':role,'environment':dict(env)} for case,role,env in plan.child_environments],
        'historical':declaration,'source_build_qualification':'PENDING','observer_qualification':'PENDING',
        'world_admission':'PENDING','publication_authority':False}


@final
class _PlayOwner(_StrictOwner):
    ownership_label: str = 'play'
    retained_owner: play_owner.PlayOwner | None = None
    retained_failure: BaseException | None = None


def authenticate_play(evidence: PlayEvidence, output: Path) -> str:
    live=evidence.owner;result=evidence.returned
    require(live.closed and not live.unknown and live.shell is not None,'/play','no source-owned settlement')
    assert live.shell is not None
    require(live.plan.output==output and live.shell.returncode==result.outer_exit
            and tuple(live.case_exits)==result.case_exits and result.settlement=='SETTLED'
            and result.qualification=='INCOMPLETE','/play','returned outcome differs from actual owner')
    node=document((output/'terminal.json').read_text())
    values: dict[str,Json]={'settlement':'SETTLED','outer_pid':live.shell.pid,'outer_exit':result.outer_exit,
                      'case_exits':list(result.case_exits),'qualification':'INCOMPLETE',
                      'wrapper_wait_authority':'SOURCE_FOREGROUND_CASE_SHELL_WAIT','native_waits':'OBSERVED'}
    for key,value in values.items():node.field(key).equals(value)
    node.field('retained_files').equals({str(p):h for p,h in evidence.retained})
    node.field('native_output_membership').equals({str(p):dict(m) for p,m in evidence.native_members})
    require(dict(evidence.retained)==live.anchors and {p:dict(m) for p,m in evidence.native_members}==live.native_memberships,
            '/play','source-owned frozen evidence differs')
    require(len(evidence.native_members)==16,'/play','original child coverage missing')
    for path,digest in evidence.retained:require(file_hash(path)==digest,'/play','retained input/artifact changed')
    for path,members in evidence.native_members:
        require(play_owner.evidence_membership(path)==dict(members),'/play','external native membership changed')
    require(play_owner.evidence_membership(output)==dict(evidence.members),'/play','owner output membership changed')
    expected_members={'.':'directory'}
    expected_files={output/name for name in ('terminal.json','stdout.log','stderr.log')} | {p for p,_ in evidence.retained if p.is_relative_to(output)}
    for path in expected_files:
        expected_members[str(path.relative_to(output))]='file'
        for parent in path.parents:
            if parent==output:break
            expected_members[str(parent.relative_to(output))]='directory'
    require(dict(evidence.members)==expected_members,'/play','unaccounted owner evidence membership')
    for name in ('stdout.log','stderr.log'):
        node.field('outer_log_hashes').field(name).equals(file_hash(output/name))
    require(live.source_grants is not None,'/play','source grants absent')
    assert live.source_grants is not None
    live.source_grants.check()
    require(set(live.source_grants.built)=={'main','m1','v7'},'/play','original build joins absent')
    return 'PENDING' if result.outer_exit==0 and result.case_exits==(3,2,4,0,0,0) else 'FAIL'


def play_outcome(live: play_owner.PlayOwner, result: play_owner.PlayOutcome) -> Outcome:
    output=live.plan.output
    evidence=PlayEvidence(live,result,tuple(live.anchors.items()),
        tuple((p,tuple(m.items())) for p,m in live.native_memberships.items()),
        tuple(play_owner.evidence_membership(output).items()))
    receipt=output/'terminal.json';digest=file_hash(receipt)
    files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
    error=''
    try:status=authenticate_play(evidence,output)
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as caught:
        status='FAIL';error='settled play evidence rejected: '+safe_docs_exception(caught)
        # This is a retained rejected observation, never refreshed successful evidence.
        evidence=replace(evidence,failure=caught,
            rejected_external=tuple((p,file_hash(p)) for p,_ in evidence.retained if not p.is_relative_to(output)),
            rejected_members=tuple((p,tuple(play_owner.evidence_membership(p).items())) for p,_ in evidence.native_members),
            rejected_native_files=tuple((p,tuple(inventory(p,[child.name for child in p.iterdir()]).items()))
                                        for p,_ in evidence.native_members))
    return Outcome('play',status,receipt,digest,error,files,output,result.settlement,
                   result.qualification,play_evidence=evidence)


def recheck_play(context: OrderedInputs, row: Outcome) -> None:
    if row.component!='play':return
    if row.play_evidence is None:
        require(row.status in {'FAIL','NOT_RUN'},'/play','missing actual source-owned result')
        return
    require(isinstance(context,OwnedOrderedInputs) and context.play is not None,'/play','lost historical context')
    assert row.root is not None
    assert isinstance(context,OwnedOrderedInputs) and context.play is not None
    require(row.play_evidence.owner.plan==replace(context.play.plan,output=row.root),
            '/play','historical request changed')
    recheck(row)
    evidence=row.play_evidence
    if evidence.failure is not None:
        require(row.status=='FAIL','/play','rejected evidence promoted')
        require(play_owner.evidence_membership(row.root)==dict(evidence.members),'/play','rejected owner membership changed')
        for path,digest in evidence.rejected_external:
            require(file_hash(path)==digest,'/play','rejected external evidence changed')
        for path,members in evidence.rejected_members:
            require(play_owner.evidence_membership(path)==dict(members),'/play','rejected native membership changed')
        for path,files in evidence.rejected_native_files:
            require(inventory(path,[child.name for child in path.iterdir()])==dict(files),
                    '/play','rejected native file bytes changed')
    else:
        status=authenticate_play(evidence,row.root)
        require(row.status=='FAIL' or row.status==status,'/play','historical outcome changed')
    recheck(row)


def invoke_play(call: OrderedCall, context: OwnedOrderedInputs, output: Path) -> Outcome:
    require(call==play_call(),'/play','not original whole six-case obligation')
    owner=_PlayOwner(context.control_fd,output);entered=False;settled=False
    try:
        bound=play_bindings(context);plan=replace(bound.plan,output=output)
        live=play_owner.PlayOwner(plan,owner.descriptor)
        owner.retained_owner=live
        entered=True
        live.launch();result=live.wait()
        settled=True
        return play_outcome(live,result)
    except BaseException as error:
        if entered and not settled:
            owner.retained_failure=error
            if isinstance(error,play_owner.PlayOwnershipUnknown):owner.retained_owner=error.owner
            owner.hold(error)
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


def run_ordered_owned(context: OrderedInputs, strict: StrictContext, output: Path, control_fd: int, *, bench: BenchContext | None = None, docs: DocsContext | None = None, oracles: OracleContext | None = None, resolver: ResolverContext | None = None, structural: Structural48Context | None = None, stall: StallContext | None = None, library49: Library49Context | None = None, library42: Library49Context | None = None, library43: Library49Context | None = None, play: PlayContext | None = None, prefix28: Library49Context | None = None, prefix36: Library49Context | None = None, m1: M1Context | None = None, mux2: Mux2Context | None = None, setup: SetupContext | None = None) -> Path:
    require(type(control_fd) is int and control_fd>2 and stat.S_ISFIFO(os.fstat(control_fd).st_mode),
            '/control','dedicated persistent controller pipe required')
    owned=OwnedOrderedInputs(context.components,context.static_product,context.environment,
        context.discovery_product,context.discovery_environment,context.diagnosis_product,
        context.diagnosis_environment,context.save_roundtrip,context.perf,strict,control_fd,bench,docs,oracles,resolver,structural,stall,library49,library42,library43,play,prefix28,prefix36,m1,mux2,setup)
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
    for product in (context.base,context.candidate):
        if product.runtime_inputs:
            _=process_inputs(product)
    return {'deadline':context.deadline,'purpose':context.purpose,'embedded_preparation':'NOT_QUALIFIED',
        'compile_root_authority':'CALLER_OWNED_NOT_CERTIFIED_HERE',
        'products':{arm:{'root':str(product.root),'source_sha':product.source_sha,'binary':str(product.binary),
            'binary_sha256':product.binary_sha256,'build_receipt':str(product.build_receipt),
            'build_receipt_sha256':product.build_receipt_sha256,'inputs':dict(product.input_files),
            **({'runtime_inputs':dict(product.runtime_inputs)} if product.runtime_inputs else {})}
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


def oracle_call() -> OrderedCall:
    identities=tuple(row.identity for row in obligations() if row.step==38)
    require(len(identities)==16 and tuple(identity.split(':',1)[1].split('.',1)[0]+'_test' for identity in identities)==SUITE_NAMES,
            '/oracles','original oracle coverage differs')
    return OrderedCall(38,'oracles',identities)


def core_predecessor(inputs: Inputs, row: Outcome | None) -> core_evaluation.PriorArtifacts:
    """Use only the actual saved step44 return, never rebind a later disk snapshot."""
    require(row is not None, '/core45', 'missing actual step44 return')
    assert row is not None
    require(row.component == 'direct' and row.root is not None and row.receipt == row.root/'receipt.json'
            and row.sha256 is not None and not row.error and not row.snapshot_error
            and row.returned_status == row.reported_status == row.status
            and row.status in {'PASS','FAIL','INCOMPLETE'}, '/core45', 'unconfirmed step44 producer return')
    assert row.receipt is not None and row.sha256 is not None
    recheck(row)
    require(authenticate('direct', document(row.receipt.read_text()), inputs, step=44) == row.returned_status,
            '/core45', 'original step44 status changed')
    prior = core_evaluation.PriorArtifacts(row.receipt, row.sha256, row.files)
    core_evaluation.recheck(prior)
    return prior


def recheck_core_evaluation(inputs: Inputs, row: Outcome) -> None:
    """Bind source-returned evaluation and its unchanged original step44 artifacts."""
    if row.component != 'core_evaluation':return
    recheck(row)
    if row.core_result is None:
        require(row.status == 'FAIL', '/core45', 'missing source-returned evaluation')
        return
    result = row.core_result; prior = row.core_prior
    require(prior is not None and row.root is not None, '/core45', 'missing prior/output binding')
    assert prior is not None and row.root is not None
    core_evaluation.recheck(prior)
    require(result.receipt == row.root/'evaluation.json' and result.receipt == row.receipt
            and result.sha256 == row.sha256 == file_hash(result.receipt)
            and result.status == row.returned_status == row.reported_status
            and result.status in {'FAIL','PENDING'}, '/core45', 'source-returned evaluation differs')
    node = document(result.receipt.read_text())
    expected: dict[str,Json] = {'schema':'generic_core_evaluation_v1','step':45,'status':result.status,
        'fixtures':list(direct.step_fixtures(44)), 'base':inputs.probes.base.source_sha,
        'candidate':inputs.probes.candidate.source_sha,'registry_sha256':inputs.registry.sha256,
        'declaration_manifest_sha256':inputs.declarations.source_manifest_sha256,
        'prior':{'path':str(prior.receipt),'sha256':prior.sha256,'files':dict(prior.files)},
        'ordinary_safety':'PENDING_SOURCE_BUILD_QUALIFICATION','source_build_authority':'CALLER_OWNED_PENDING',
        'global_coverage':'PENDING','final_conjunction':'PENDING','generic_performance':'PENDING_SEPARATE_BENCH_AUTHENTICATION',
        'legacy_core':{name:'HISTORICAL_NOT_FRESH_ACCEPTANCE' for name in ('heldout_outcomes','heldout_why','story','performance')},
        'publication_authority':False,'full_gate_claim':False}
    keys(node,set(expected)|{'evaluated_scope','pairs','errors','original_results'})
    for name,value in expected.items():node.field(name).equals(value)
    errors=node.field('errors').array()
    node.field('evaluated_scope').equals('FAIL' if errors else 'PASS')
    require(bool(errors) == (result.status == 'FAIL'), '/core45', 'failure status differs')
    _=node.field('pairs').array();_=node.field('original_results').array()
    if result.status == 'PENDING':
        plans=core_evaluation.plans_for(inputs.registry,inputs.probes,prior.receipt.parent)
        core_evaluation.authenticate_prior(inputs.registry,inputs.probes,inputs.declarations,prior,plans)
    core_evaluation.recheck(prior);recheck(row)


def invoke_core_evaluation(inputs: Inputs, predecessor: Outcome | None, output: Path) -> Outcome:
    """Read-only step45 under the caller lane; no child lifetime or wait is invented."""
    prior: core_evaluation.PriorArtifacts | None=None
    result: core_evaluation.EvaluationResult | None=None
    try:
        prior=core_predecessor(inputs,predecessor)
        result=core_evaluation.evaluate(inputs.registry,inputs.probes,inputs.declarations,prior,output)
        require(result.receipt == output/'evaluation.json', '/core45', 'foreign evaluation return')
        files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
        row=Outcome('core_evaluation',result.status,result.receipt,result.sha256,'',files,output,
                    result.status,result.status,core_prior=prior,core_result=result)
        recheck_core_evaluation(inputs,row)
        return row
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
        files=();snapshot_error=''
        if output.exists():
            try:files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
            except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as failure:snapshot_error=safe_docs_exception(failure)
        return Outcome('core_evaluation','FAIL',None if result is None else result.receipt,
            None if result is None else result.sha256,safe_docs_exception(error),files,output,
            None if result is None else result.status,None if result is None else result.status,
            snapshot_error,core_prior=prior,core_result=result)


def ordered_owned_calls(*, bench_enabled: bool = False, docs_enabled: bool = False, oracles_enabled: bool = False, resolver_enabled: bool = False, structural_enabled: bool = False, stall_enabled: bool = False, library49_enabled: bool = False, library42_enabled: bool = False, library43_enabled: bool = False, play_enabled: bool = False, prefix28_enabled: bool = False, prefix36_enabled: bool = False, m1_enabled: bool = False, mux2_enabled: bool = False, setup_enabled: bool = False) -> tuple[OrderedCall,...]:
    result=[call for call in ordered_calls() if not m1_enabled or call.step!=17]
    if mux2_enabled:
        result=[call for call in result if call.step!=37]
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(37))
        result.insert(insertion,mux2_call())
    if setup_enabled:
        insertion=next(index for index,call in enumerate(result) if call.step>1)
        result[insertion:insertion]=setup_calls()
    if m1_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(17))
        result.insert(insertion,m1_call())
    for enabled,step in ((prefix28_enabled,28),(prefix36_enabled,36)):
        if enabled:
            insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>=ORDER.index(step))
            result.insert(insertion,prefix_call(step))
    insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(45))
    result.insert(insertion,OrderedCall(45,'core_evaluation',('45:core_evaluation',)))
    for step,identity in ((8,'8:gdscript_strict'),(46,'46:gdscript.four_files')):
        insertion=next((index for index,call in enumerate(result) if ORDER.index(call.step)>=ORDER.index(step)),len(result))
        result.insert(insertion,OrderedCall(step,'static',(identity,)))
    if play_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(21))
        result.insert(insertion,play_call())
    if docs_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(14))
        result.insert(insertion,OrderedCall(14,'static',('14:docs',)))
    if bench_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(25))
        result.insert(insertion,OrderedCall(25,'bench',('25:bench',)))
    if resolver_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(32))
        result.insert(insertion,OrderedCall(32,'resolver',('32:resolver_parity',)))
    if oracles_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(38))
        result.insert(insertion,oracle_call())
    if stall_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(41))
        result.insert(insertion,OrderedCall(41,'stall',('41:mux2_stall_receipt',)))
    if structural_enabled:
        insertion=next(index for index,call in enumerate(result) if call.identity=='48:direct')+1
        result.insert(insertion,OrderedCall(48,'structural',('48:m6a.plant_art_structural',)))
    for enabled,step,name in ((library42_enabled,42,'baseline_isolation'),(library43_enabled,43,'mind_data_only')):
        if enabled:
            insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(step))
            result.insert(insertion,OrderedCall(step,f'library{step}',(f'{step}:{name}',)))
    if library49_enabled:
        insertion=next(index for index,call in enumerate(result) if ORDER.index(call.step)>ORDER.index(49))
        result.insert(insertion,OrderedCall(49,'library49',('49:read_data_only',)))
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
    dispatch_context: Final[OrderedInputs]=context
    inputs=context.components
    require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh canonical output required')
    for product in (inputs.probes.base,inputs.probes.candidate,inputs.godot.base,inputs.godot.candidate):
        require(not output.is_relative_to(product.input_root) and not product.input_root.is_relative_to(output),'/output','product overlap')
    calls=ordered_owned_calls(bench_enabled=context.bench is not None,docs_enabled=context.docs is not None,oracles_enabled=context.oracles is not None,resolver_enabled=context.resolver is not None,structural_enabled=context.structural is not None,stall_enabled=context.stall is not None,library49_enabled=context.library49 is not None,library42_enabled=context.library42 is not None,library43_enabled=context.library43 is not None,play_enabled=context.play is not None,prefix28_enabled=context.prefix28 is not None,prefix36_enabled=context.prefix36 is not None,m1_enabled=context.m1 is not None,mux2_enabled=context.mux2 is not None,setup_enabled=context.setup is not None) if isinstance(context,OwnedOrderedInputs) else ordered_calls();rows: dict[str,Outcome]={};errors: list[str]=[];sequence: list[str]=[]
    tail_stop: str | None=None;bench_inputs: dict[str,Json] | None=None;docs_inputs: dict[str,Json] | None=None;oracle_inputs: dict[str,Json] | None=None;resolver_inputs: dict[str,Json] | None=None;structural_inputs: dict[str,Json] | None=None;stall_inputs: dict[str,Json] | None=None;library49_inputs: dict[str,Json] | None=None;library42_inputs: dict[str,Json] | None=None;library43_inputs: dict[str,Json] | None=None;play_inputs: dict[str,Json] | None=None;prefix_inputs: dict[str,Json]={};m1_inputs: dict[str,Json] | None=None;mux2_inputs: dict[str,Json] | None=None
    setup_owner: SetupOwner | None=None;setup_inputs: dict[str,Json] | None=None
    terminal_generated: workspace_endpoints.GeneratedState | None=None
    plan_hash: str | None=None;blocked: dict[int,str]={};events: list[Json]=[];perf_inputs: dict[str,Json] | None=None
    try:
        output.mkdir(parents=True)
        if isinstance(context,OwnedOrderedInputs) and context.setup is not None:
            setup_inputs=setup_dependencies(setup_bindings(context))
            bridge_output=context.setup.prepared.inputs.output
            require(not bridge_output.exists() and not bridge_output.is_relative_to(output) and not output.is_relative_to(bridge_output),
                    '/setup','fresh external prepared bridge output must not overlap dispatch')
        _=direct.partition(inputs.registry)
        perf_bindings(context);perf_inputs=perf_dependencies(context.perf)
        if isinstance(context,OwnedOrderedInputs) and context.bench is not None:
            bench_inputs=bench_dependencies(bench_bindings(context))
        if isinstance(context,OwnedOrderedInputs) and context.docs is not None:
            docs_inputs=docs_dependencies(docs_bindings(context))
        if isinstance(context,OwnedOrderedInputs) and context.oracles is not None:
            oracle_inputs=oracle_dependencies(oracle_bindings(context))
        if isinstance(context,OwnedOrderedInputs) and context.resolver is not None:
            resolver_inputs=resolver_dependencies(resolver_bindings(context))
        if isinstance(context,OwnedOrderedInputs) and context.structural is not None:
            structural_inputs=structural_dependencies(structural_bindings(context))
        if isinstance(context,OwnedOrderedInputs) and context.stall is not None:
            stall_inputs=stall_dependencies(stall_bindings(context))
        if isinstance(context,OwnedOrderedInputs) and context.library49 is not None:
            library49_inputs=library49_dependencies(library49_bindings(context))
        if isinstance(context,OwnedOrderedInputs) and context.library42 is not None:
            library42_inputs=library49_dependencies(library49_bindings(context,42))
        if isinstance(context,OwnedOrderedInputs) and context.library43 is not None:
            library43_inputs=library49_dependencies(library49_bindings(context,43))
        if isinstance(context,OwnedOrderedInputs) and context.play is not None:
            play_inputs=play_dependencies(play_bindings(context))
        if isinstance(context,OwnedOrderedInputs):
            for step,bound in ((28,context.prefix28),(36,context.prefix36)):
                if bound is not None:prefix_inputs[str(step)]=library49_dependencies(prefix_bindings(context,step))
        if isinstance(context,OwnedOrderedInputs) and context.m1 is not None:m1_inputs=m1_dependencies(m1_bindings(context))
        if isinstance(context,OwnedOrderedInputs) and context.mux2 is not None:mux2_inputs=mux2_dependencies(mux2_bindings(context))
        for product in (context.perf.base,context.perf.candidate):
            require(not output.is_relative_to(product.root) and not product.root.is_relative_to(output),'/output','perf product overlap')
        require(inputs.declarations.base==inputs.probes.base.source_sha and inputs.declarations.candidate==inputs.probes.candidate.source_sha,
                '/declarations','source identity differs')
        require(DIRECT_IDS <= set(inputs.declarations.known_fixtures),'/declarations','incomplete coverage')
        for step in direct.DIRECT_STEP_FIXTURES:_=direct.step_fixtures(step)
        refs: dict[int,dict[str,list[Json]]]={step:references(inputs,output/f'step-{step:02d}',validate=True) for step in direct.DIRECT_STEP_FIXTURES}
        atomic_json(output/'plan.json',{'schema_version':3,'original_step_order':list(ORDER),
            'original_order_executed':False,'publication_authorized':False,
            'supported_call_sequence':[call.identity for call in calls],'perf_dependencies':perf_inputs,'bench_dependencies':bench_inputs,'docs_dependencies':docs_inputs,'oracle_dependencies':oracle_inputs,'resolver_dependencies':resolver_inputs,'structural_dependencies':structural_inputs,'stall_dependencies':stall_inputs,'library49_dependencies':library49_inputs,'library42_dependencies':library42_inputs,'library43_dependencies':library43_inputs,'play_dependencies':play_inputs,'native_prefix_dependencies':prefix_inputs,'m1_dependencies':m1_inputs,'mux2_dependencies':mux2_inputs,'setup_dependencies':setup_inputs,
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
            if call.component=='setup' and setup_owner is not None and setup_owner.closed:
                rows[call.identity]=Outcome('setup','NOT_RUN',None,None,'previous setup endpoint unavailable');continue
            if call.component=='structural':
                try:structural_predecessor(inputs,rows)
                except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as failure:
                    blocked[48]='48:direct'
                    rows[call.identity]=Outcome('structural','NOT_RUN',None,None,'blocked by 48:direct; '+safe_docs_exception(failure))
                    continue
            for row in rows.values():
                recheck(row);recheck_oracle(context,row);recheck_resolver(context,row);recheck_structural(context,row);recheck_stall(context,row);recheck_library49(context,row);recheck_play(context,row);recheck_prefix(context,row);recheck_m1(context,row);recheck_mux2(context,row);recheck_setup(context,row);recheck_core_evaluation(context.components,row)
            ordered_bindings(context)
            if isinstance(context,OwnedOrderedInputs) and context.setup is not None:
                require(setup_dependencies(setup_bindings(context))==setup_inputs,'/setup','context drift')
            if isinstance(context,OwnedOrderedInputs) and context.bench is not None:
                require(bench_dependencies(context.bench)==bench_inputs,'/bench','context binding changed')
            if isinstance(context,OwnedOrderedInputs) and context.docs is not None:
                require(docs_dependencies(context.docs)==docs_inputs,'/docs','context binding changed')
            if isinstance(context,OwnedOrderedInputs) and context.oracles is not None:
                require(oracle_dependencies(oracle_bindings(context))==oracle_inputs,'/oracles','context binding changed')
            if isinstance(context,OwnedOrderedInputs) and context.resolver is not None:
                require(resolver_dependencies(resolver_bindings(context))==resolver_inputs,'/resolver','context binding changed')
            if isinstance(context,OwnedOrderedInputs) and context.structural is not None:
                require(structural_dependencies(structural_bindings(context))==structural_inputs,'/structural','context binding changed')
            if isinstance(context,OwnedOrderedInputs) and context.stall is not None:
                require(stall_dependencies(stall_bindings(context))==stall_inputs,'/stall','context binding changed')
            if isinstance(context,OwnedOrderedInputs) and context.library49 is not None:
                require(library49_dependencies(library49_bindings(context))==library49_inputs,'/library49','context binding changed')
            if isinstance(context,OwnedOrderedInputs) and context.library42 is not None:
                require(library49_dependencies(library49_bindings(context,42))==library42_inputs,'/library42','context binding changed')
            if isinstance(context,OwnedOrderedInputs) and context.library43 is not None:
                require(library49_dependencies(library49_bindings(context,43))==library43_inputs,'/library43','context binding changed')
            if isinstance(context,OwnedOrderedInputs):
                for step in (28,36):
                    if str(step) in prefix_inputs:require(library49_dependencies(prefix_bindings(context,step))==prefix_inputs[str(step)],'/prefix','context drift')
            if isinstance(context,OwnedOrderedInputs) and context.m1 is not None:
                require(m1_dependencies(m1_bindings(context))==m1_inputs,'/m1','context drift')
            if isinstance(context,OwnedOrderedInputs) and context.mux2 is not None:
                require(mux2_dependencies(mux2_bindings(context))==mux2_inputs,'/mux2','context drift')
            require(file_hash(output/'plan.json')==plan_hash,'/plan','scope changed')
            parent=output/f'step-{call.step:02d}';parent.mkdir(exist_ok=True)
            if isinstance(context,OwnedOrderedInputs) and context.play is not None:
                require(play_dependencies(play_bindings(context))==play_inputs,'/play','context binding changed')
            destination=parent/call.component
            try:
                with nullcontext() if call.component in {'static','save_roundtrip','perf','nav','bench','oracles','resolver','structural','stall','library49','library42','library43','play','native_prefix','m1','mux2','setup'} else lane():
                    selected = with_static_priors(dispatch_context,tuple(rows.values())) if call.component == 'static' else dispatch_context
                    action=ordered_action(call,selected) if call.component not in {'bench','oracles','resolver','structural','stall','library49','library42','library43','core_evaluation','play','native_prefix','m1','mux2','setup'} else None
                    static_product, static_environment = static_context(selected,call.step) if call.component == 'static' else (None,None)
                    sequence.append(call.identity);events.append({'call':call.identity,'event':'start','at':timestamp()})
                    try:
                        if call.component=='setup' and isinstance(selected,OwnedOrderedInputs):
                            if setup_owner is None:
                                require(call.step==2,'/setup','bridge must start chain')
                                setup_owner=SetupOwner(selected.control_fd,destination,setup_bindings(selected))
                            rows[call.identity]=invoke_setup(call,selected,destination,setup_owner)
                        elif call.step==14 and call.component=='static' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_docs(call,selected,destination)
                        elif call.component=='mux2' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_mux2(call,selected,destination)
                        elif call.component=='m1' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_m1(call,selected,destination)
                        elif call.component=='native_prefix' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_prefix(call,selected,destination)
                        elif call.component=='play' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_play(call,selected,destination)
                        elif call.component=='core_evaluation':
                            rows[call.identity]=invoke_core_evaluation(inputs,rows.get('44:direct'),destination)
                        elif call.component=='resolver' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_resolver(call,selected,destination)
                        elif call.component in {'library42','library43'} and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_library_sibling(call,selected,destination)
                        elif call.component=='library49' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_library49(call,selected,destination)
                        elif call.component=='stall' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_stall(call,selected,destination)
                        elif call.component=='structural' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_structural(call,selected,destination)
                        elif call.component=='oracles' and isinstance(selected,OwnedOrderedInputs):
                            rows[call.identity]=invoke_oracles(call,selected,destination)
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
                if call.component=='setup' and setup_owner is not None:
                    if setup_owner.in_flight:setup_owner.retain_unknown(error)
                    setup_owner.retained_error=error;setup_owner.close()
                    root=setup_owner.evidence_root
                    state=workspace_endpoints.anchor(root).state if root.exists() else ''
                    rows[call.identity]=Outcome('setup','FAIL',None,None,safe_docs_exception(error),
                        setup_files(state) if state else (),root if root.exists() else None,setup_state=state)
                else:rows[call.identity]=replace(rows[call.identity],status='FAIL',error=rows[call.identity].error+'; lane: '+str(error)) if call.identity in rows else Outcome(call.component,'FAIL',None,None,str(error))
            if call.component=='native_prefix' and rows[call.identity].reported_status!='PASS_PREFIX_ONLY':blocked[call.step]=call.identity
            if call.component=='native_prefix' and rows[call.identity].status=='FAIL':blocked[call.step]=call.identity
            if call.component=='bench' and rows[call.identity].status!='COMPONENT_PASS':tail_stop=call.identity
            if rows[call.identity].status=='FAIL' and call.step in (34,46):blocked[call.step]=call.identity
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:errors.append(str(error))
    finally:
        if setup_owner is not None and setup_owner.in_flight:
            setup_owner.retain_unknown(RuntimeError('setup call did not establish settlement'))
        if isinstance(context,OwnedOrderedInputs) and context.setup is not None:
            try:terminal_generated=workspace_endpoints.generated(context.setup.prepared)
            except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
                errors.append('terminal generated snapshot: '+safe_docs_exception(error))
        if setup_owner is not None:setup_owner.close()
    for row in rows.values():
        try:
            recheck(row);recheck_oracle(context,row);recheck_resolver(context,row);recheck_structural(context,row);recheck_stall(context,row);recheck_library49(context,row);recheck_play(context,row);recheck_prefix(context,row);recheck_m1(context,row);recheck_mux2(context,row);recheck_setup(context,row);recheck_core_evaluation(context.components,row)
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:errors.append(str(error))
    try:
        ordered_bindings(context)
        if isinstance(context,OwnedOrderedInputs) and context.bench is not None:
            require(bench_dependencies(context.bench)==bench_inputs,'/bench','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.docs is not None:
            require(docs_dependencies(context.docs)==docs_inputs,'/docs','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.oracles is not None:
            require(oracle_dependencies(oracle_bindings(context))==oracle_inputs,'/oracles','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.resolver is not None:
            require(resolver_dependencies(resolver_bindings(context))==resolver_inputs,'/resolver','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.structural is not None:
            require(structural_dependencies(structural_bindings(context))==structural_inputs,'/structural','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.stall is not None:
            require(stall_dependencies(stall_bindings(context))==stall_inputs,'/stall','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.play is not None:
            require(play_dependencies(play_bindings(context))==play_inputs,'/play','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.library49 is not None:
            require(library49_dependencies(library49_bindings(context))==library49_inputs,'/library49','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.library42 is not None:
            require(library49_dependencies(library49_bindings(context,42))==library42_inputs,'/library42','context binding changed')
        if isinstance(context,OwnedOrderedInputs) and context.library43 is not None:
            require(library49_dependencies(library49_bindings(context,43))==library43_inputs,'/library43','context binding changed')
        if isinstance(context,OwnedOrderedInputs):
            for step in (28,36):
                if str(step) in prefix_inputs:require(library49_dependencies(prefix_bindings(context,step))==prefix_inputs[str(step)],'/prefix','context drift')
        if isinstance(context,OwnedOrderedInputs) and context.m1 is not None:
            require(m1_dependencies(m1_bindings(context))==m1_inputs,'/m1','context drift')
        if isinstance(context,OwnedOrderedInputs) and context.mux2 is not None:
            require(mux2_dependencies(mux2_bindings(context))==mux2_inputs,'/mux2','context drift')
        require(plan_hash is not None and file_hash(output/'plan.json')==plan_hash,'/plan','scope changed or plan failed')
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:errors.append(str(error))
    for row in rows.values():
        try:recheck_prefix_anchor(row);recheck_m1_anchor(row);recheck_mux2_anchor(row);recheck_setup_anchor(row)
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
            errors.append('final native evidence: '+safe_docs_exception(error))
    if isinstance(context,OwnedOrderedInputs) and context.setup is not None:
        try:
            require(terminal_generated is not None and workspace_endpoints.generated(context.setup.prepared)==terminal_generated,
                    '/setup/terminal_generated','current generated state changed after producers settled')
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
            errors.append('terminal generated evidence: '+safe_docs_exception(error))
    by_obligation={identity:call.identity for call in calls for identity in call.identities}
    output.mkdir(parents=True,exist_ok=True);terminal=output/'ordered-dispatch.json'
    payload: dict[str,Json]={'schema_version':3,'status':'FAIL' if errors or any(row.status=='FAIL' for row in rows.values()) else 'INCOMPLETE',
        'publication_authorized':False,'full_gate_claim':False,'original_order_executed':False,
        'original_step_order':list(ORDER),'perf_dependencies':perf_inputs,'bench_dependencies':bench_inputs,'docs_dependencies':docs_inputs,'oracle_dependencies':oracle_inputs,'resolver_dependencies':resolver_inputs,'structural_dependencies':structural_inputs,'stall_dependencies':stall_inputs,'library49_dependencies':library49_inputs,'library42_dependencies':library42_inputs,'library43_dependencies':library43_inputs,'play_dependencies':play_inputs,'native_prefix_dependencies':prefix_inputs,'m1_dependencies':m1_inputs,'mux2_dependencies':mux2_inputs,'setup_dependencies':setup_inputs,'dispatch_events':events,'actual_component_sequence':list(sequence),'plan_sha256':plan_hash,'errors':list(errors),
        'components':[{'identity':call.identity,'step':call.step,'component':call.component,
            'status':rows[call.identity].status if call.identity in rows else 'NOT_RUN',
            'receipt':str(rows[call.identity].receipt) if call.identity in rows and rows[call.identity].receipt else None,
            'sha256':rows[call.identity].sha256 if call.identity in rows else None,
            'files':dict(rows[call.identity].files) if call.identity in rows else {},
            'error':rows[call.identity].error if call.identity in rows else '',
            'returned_status':rows[call.identity].returned_status if call.identity in rows else None,
            'reported_status':rows[call.identity].reported_status if call.identity in rows else None,
            'snapshot_error':rows[call.identity].snapshot_error if call.identity in rows else '',
            'mux2_state':{name:[kind,sha] for name,(kind,sha) in rows[call.identity].mux2_state} if call.identity in rows else {},
            'setup_state':document(rows[call.identity].setup_state).mapping() if call.identity in rows and rows[call.identity].setup_state else {},
            'm1_state':{name:[kind,sha] for name,(kind,sha) in rows[call.identity].m1_state} if call.identity in rows else {},
            'native_prefix_state':{name:[kind,sha] for name,(kind,sha) in rows[call.identity].prefix_state} if call.identity in rows else {},
            'library49_members':list(rows[call.identity].library49_members) if call.identity in rows else []} for call in calls],
        'obligations':[{'identity':row.identity,'step':row.step,'fixture':row.fixture,
            'status':'NOT_RUN' if tail_stop is not None and ORDER.index(row.step)>ORDER.index(25) or row.identity=='48:m6a.plant_art_structural' and blocked.get(48)=='48:direct' else 'COMPONENT_ONLY' if row.identity in by_obligation else 'NOT_IMPLEMENTED',
            'blocked_by':tail_stop if tail_stop is not None and ORDER.index(row.step)>ORDER.index(25) else '48:direct' if row.identity=='48:m6a.plant_art_structural' and blocked.get(48)=='48:direct' else None,
            'call':by_obligation.get(row.identity),
            'component_status':rows[by_obligation[row.identity]].status if row.identity in by_obligation and by_obligation[row.identity] in rows else None}
            for row in obligations()]}
    if terminal_generated is not None:payload['setup_terminal_generated']=workspace_endpoints.state_json(terminal_generated)
    atomic_json(terminal,payload)
    before_publication_check=len(errors)
    if isinstance(context,OwnedOrderedInputs) and context.setup is not None:
        try:
            require(setup_dependencies(setup_bindings(context))==setup_inputs,'/setup','published inputs drift')
            for row in rows.values():recheck_setup(context,row)
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
            errors.append('published setup evidence: '+safe_docs_exception(error))
    if isinstance(context,OwnedOrderedInputs) and context.m1 is not None:
        try:require(m1_dependencies(m1_bindings(context))==m1_inputs,'/m1','published source/context drift')
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
            errors.append('published M1 inputs: '+safe_docs_exception(error))
    if isinstance(context,OwnedOrderedInputs) and context.mux2 is not None:
        try:require(mux2_dependencies(mux2_bindings(context))==mux2_inputs,'/mux2','published source/context drift')
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
            errors.append('published MUX2 inputs: '+safe_docs_exception(error))
    for row in rows.values():
        try:recheck_prefix_anchor(row);recheck_m1_anchor(row);recheck_mux2_anchor(row);recheck_setup_anchor(row)
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
            errors.append('published native evidence: '+safe_docs_exception(error))
    if isinstance(context,OwnedOrderedInputs) and context.setup is not None:
        try:
            require(terminal_generated is not None and workspace_endpoints.generated(context.setup.prepared)==terminal_generated,
                    '/setup/terminal_generated','current generated state changed after producers settled')
        except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as error:
            errors.append('terminal generated evidence: '+safe_docs_exception(error))
    if len(errors)!=before_publication_check:
        payload['status']='FAIL';payload['errors']=list(errors)
        payload['supersedes']={'path':str(terminal),'sha256':file_hash(terminal)}
        terminal=output/'ordered-dispatch-prefix-failure.json'
        # Preserve rejected evidence too: this observation never replaces the original anchors.
        mux2_failed_states={row.root:integration_prefix.output_state(row.root) for row in rows.values()
                            if row.component=='mux2' and row.root is not None}
        mux2_failed_inputs: dict[Path,str]={}
        if isinstance(context,OwnedOrderedInputs) and context.mux2 is not None:
            request=context.mux2.request.template
            mux2_failed_inputs={path:file_hash(path) for path in
                                (*(request.root/name for name,_ in request.inputs),*(path for path,_ in request.tools))}
        atomic_json(terminal,payload)
        for root,state in mux2_failed_states.items():
            require(integration_prefix.output_state(root)==state,'/mux2','rejected evidence changed during failure publication')
        for path,digest in mux2_failed_inputs.items():
            require(file_hash(path)==digest,'/mux2','inputs changed during failure publication')
    return terminal


def prefix_container(step: int, root: Path) -> workspace_endpoints.EvidenceAnchor:
    """Close a completed original step's container before admitting its history."""
    children: set[str] = {'static'} if step in (1,7,8) else set() if step==2 else {'workspace-preparation','setup'} if step in (4,5) else {'setup'}
    require(step in range(1,9) and {path.name for path in root.iterdir()}==children,
            '/prefix/container','unexpected completed step membership')
    require(all(stat.S_ISDIR((root/name).lstat().st_mode) for name in children),
            '/prefix/container','component root kind changed')
    return workspace_endpoints.anchor(root)


def prefix_membership(root: Path, steps: tuple[int,...], stage: bool, terminal: bool, mode: int) -> None:
    """Allow only source-scheduled prefix paths, including empty directories."""
    files={'plan.json'}
    if stage:files.add('stage-06.json')
    if terminal:files.add('setup-prefix.json')
    directories={f'step-{step:02d}' for step in steps}
    require(root.lstat().st_mode==mode and {path.name for path in root.iterdir()}==files|directories,
            '/prefix/membership','unscheduled prefix path or root mode changed')
    require(all(stat.S_ISREG((root/name).lstat().st_mode) for name in files)
            and all(stat.S_ISDIR((root/name).lstat().st_mode) for name in directories),
            '/prefix/membership','scheduled prefix path kind changed')


def _prefix_static(step: Literal[1,7,8], product: static.StaticProduct,
                   environment: static.ExecutionEnvironment, output: Path, control_fd: int) -> tuple[Outcome,workspace_endpoints.EvidenceAnchor]:
    owner=_StrictOwner(control_fd,output) if step==8 else None
    entered=False;settled=False
    try:
        selected=replace(environment,held_lane_fd=owner.descriptor) if owner is not None else environment
        _=static.validate_environment(selected,step)
        entered=True
        result=static.run_step(product,step,output,selected)
        settled=True
        retained=workspace_endpoints.anchor(output)
        container=prefix_container(step,output.parent)
        status=authenticate_static(result.receipt,step,product,selected)
        retained.recheck();container.recheck()
        return Outcome('static','COMPONENT_PASS' if status=='PASS' else 'FAIL',result.receipt,
            file_hash(result.receipt),'',setup_files(retained.state),output,result.status,status),retained
    except BaseException as error:
        if owner is not None and entered and not settled:owner.hold(error)
        raise
    finally:
        if owner is not None and (not entered or settled):os.close(owner.descriptor)


def _run_setup_prefix(context: SetupPrefixContext, handoff: cli_handoff.SetupProductHandoff,
                      product: static.StaticProduct, environment: static.ExecutionEnvironment,
                      strict: static.ExecutionEnvironment, output: Path, control_fd: int, selectors: cli_execute.PrefixDocument) -> Path:
    """Private closed CLI continuation; no eager future runtime product placeholders."""
    require(type(control_fd) is int and control_fd>2 and stat.S_ISFIFO(os.fstat(control_fd).st_mode),
            '/control','dedicated persistent controller pipe required')
    require(handoff.prepared is context.setup.prepared and product.root==context.candidate_root
            and product.source_sha==context.candidate_sha,'/prefix','source-owned construction differs')
    require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh prefix output required')
    require(not environment.prior_receipts and not strict.prior_receipts,'/prior','prefix starts without imported result authority')
    bound=setup_bindings(context)
    require(context.candidate_inputs==handoff.products.probes.candidate.input_files
            and product.inputs==context.candidate_inputs,'/prefix','source projection differs')
    handoff=cli_handoff.begin_setup_handoff(handoff.products,bound.prepared)
    dependencies=setup_dependencies(bound)
    output.mkdir(parents=True)
    root_mode=output.lstat().st_mode
    plan=output/'plan.json'
    _=cli_execute.publish(plan,{'schema':'closed_setup_prefix.v1','original_step_order':list(ORDER),
        'supported_prefix':list(range(1,9)),'source_sha':context.candidate_sha,
        'selectors':str(selectors.path),'selectors_sha256':selectors.sha256,
        'setup_dependencies':dependencies,'publication_authorized':False,
        'obligations':[{'identity':row.identity,'step':row.step,'fixture':row.fixture} for row in obligations()]})
    plan_sha=file_hash(plan)
    rows: dict[int,Outcome]={};anchors: list[workspace_endpoints.EvidenceAnchor]=[]
    static_rows: list[tuple[int,Outcome,static.StaticProduct,static.ExecutionEnvironment]]=[]
    owner: SetupOwner | None=None;runtime: cli_handoff.RuntimeProducts | None=None
    errors: list[str]=[];stage_path: Path | None=None;stage_hash: str | None=None
    scheduled: list[int]=[];step_modes: dict[int,int]={};terminal_written=False
    def check_history() -> None:
        prefix_membership(output,tuple(scheduled),stage_path is not None,terminal_written,root_mode)
        cli_execute.check_spec(selectors)
        require(file_hash(plan)==plan_sha,'/plan','immutable plan changed')
        if stage_path is not None:require(file_hash(stage_path)==stage_hash,'/stage','retained runtime stage changed')
        require(setup_dependencies(setup_bindings(context))==dependencies,'/setup','immutable selectors changed')
        for step,mode in step_modes.items():
            require((output/f'step-{step:02d}').lstat().st_mode==mode,'/prefix/container','step container mode changed')
        for retained in anchors:retained.recheck()
        for row in rows.values():recheck_setup(context,row)
        for step,row,source,selected in static_rows:
            assert row.receipt is not None
            require(authenticate_static(row.receipt,step,source,selected)==row.reported_status,'/static','historical outcome differs')
        if runtime is not None:cli_handoff.check_runtime_current(runtime)
        # Recheck after all semantic callbacks, before the next producer or publication.
        for step,mode in step_modes.items():
            require((output/f'step-{step:02d}').lstat().st_mode==mode,'/prefix/container','step container mode changed')
        for retained in anchors:retained.recheck()
        require(file_hash(plan)==plan_sha,'/plan','immutable plan changed')
        if stage_path is not None:require(file_hash(stage_path)==stage_hash,'/stage','retained runtime stage changed')
        prefix_membership(output,tuple(scheduled),stage_path is not None,terminal_written,root_mode)
    try:
        for step in range(1,9):
            check_history()
            destination=output/f'step-{step:02d}';destination.mkdir();scheduled.append(step);step_modes[step]=destination.lstat().st_mode
            if 2<=step<=6:
                if owner is None:owner=SetupOwner(control_fd,destination/'setup',bound)
                call=setup_calls()[step-2]
                rows[step]=invoke_setup(call,context,destination/'setup',owner)
                if step==6:
                    evidence=rows[step].setup_evidence
                    require(evidence is not None and isinstance(evidence.endpoint,setup_clippy.ClippyEndpoint),
                            '/handoff','actual completed step6 endpoint required')
                    assert evidence is not None and isinstance(evidence.endpoint,setup_clippy.ClippyEndpoint)
                    runtime=cli_handoff.finish_setup_handoff(handoff,evidence.endpoint)
                    stage_path=output/'stage-06.json'
                    _= cli_execute.publish(stage_path,{'schema':'closed_setup_runtime.v1','endpoint_sha256':runtime.endpoint.receipt_sha256,
                        'runtime_inputs':{'base':dict(runtime.runtime_inputs[0]),'candidate':dict(runtime.runtime_inputs[1])},
                        'runtime_qualification':'NOT_RUN','publication_authorized':False})
                    stage_hash=file_hash(stage_path)
                    cli_handoff.check_runtime_current(runtime)
                    require(file_hash(stage_path)==stage_hash,'/stage','runtime stage changed')
            else:
                require(step in (1,7,8),'/step','unsupported prefix call')
                phase: Literal[1,7,8]=1 if step==1 else 7 if step==7 else 8
                priors=tuple((row.receipt,row.sha256) for _,row,_,_ in static_rows
                             if row.receipt is not None and row.sha256 is not None)
                selected=replace(strict if step==8 else environment,prior_receipts=priors)
                source=product
                if step==8:
                    require(runtime is not None,'/strict','actual step6 handoff required')
                    assert runtime is not None
                    cli_handoff.check_runtime_current(runtime)
                    source=static.StaticProduct(product.root,product.source_sha,
                        tuple(sorted(inventory(product.root,static.strict_input_names(product.root)).items())))
                    cli_handoff.check_runtime_current(runtime)
                row,retained=_prefix_static(phase,source,selected,destination/'static',control_fd)
                rows[step]=row;anchors.append(retained);static_rows.append((step,row,source,selected))
            require(destination.lstat().st_mode==step_modes[step],'/prefix/container','step container mode changed')
            anchors.append(prefix_container(step,destination))
            check_history()
        check_history()
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as failure:
        if owner is not None and owner.in_flight:owner.retain_unknown(failure)
        errors.append(safe_docs_exception(failure))
    finally:
        if owner is not None:
            if owner.in_flight:owner.retain_unknown(RuntimeError('prefix setup did not settle'))
            owner.close()
    try:check_history()
    except (AuditError,GateContractError,OSError,ValueError,RuntimeError) as failure:errors.append(safe_docs_exception(failure))
    terminal=output/'setup-prefix.json'
    payload: dict[str,Json]={'schema':'closed_setup_prefix_result.v1',
        'status':'FAIL' if errors or any(row.status=='FAIL' for row in rows.values()) else 'INCOMPLETE',
        'publication_authorized':False,'full_gate_claim':False,'original_order_executed':False,
        'plan_sha256':plan_sha,'stage06':str(stage_path) if stage_path else None,'errors':list(errors),
        'actual_steps':list(rows),'obligations':[{'identity':ob.identity,'step':ob.step,
            'status':rows[ob.step].status if ob.step in rows else 'NOT_RUN'} for ob in obligations()]}
    terminal_anchor=cli_execute.publish(terminal,payload)
    terminal_written=True
    check_history()
    terminal_anchor.recheck()
    return terminal


def execute_setup_prefix(spec_path: Path) -> Path:
    selected=cli_execute.read_spec(spec_path)
    spec=selected.spec
    require(spec.control_fd>2 and stat.S_ISFIFO(os.fstat(spec.control_fd).st_mode),'/control','persistent controller pipe required')
    initializer=cli_execute.InitializationOwner()
    owner=_StrictOwner(spec.control_fd,spec.output/'antecedent')
    try:
        try:
            prepared=cli_execute.initialize(spec,initializer)
        except BaseException as error:
            if initializer.in_flight:owner.hold(error)
            if not spec.output.exists():raise
            terminal=spec.output/'setup-prefix-failure.json'
            retained=cli_execute.publish(terminal,{'schema':'closed_setup_prefix_result.v1','status':'FAIL',
                'publication_authorized':False,'full_gate_claim':False,'original_order_executed':False,
                'spec_sha256':selected.sha256,'errors':[safe_docs_exception(error)],'actual_steps':[],
                'original_step_order':list(ORDER),
                'obligations':[{'identity':ob.identity,'step':ob.step,'status':'NOT_RUN'} for ob in obligations()]})
            cli_execute.check_spec(selected);retained.recheck()
            return terminal
    finally:
        if not initializer.in_flight:os.close(owner.descriptor)
    cli_execute.check_spec(selected)
    def factory(step: Literal[4,5], chain: workspace_endpoints.WorkspaceChain,
                output: Path, preparation: Path, descriptor: int) -> workspace_endpoints.WorkspaceRequest:
        cli_execute.check_spec(selected)
        request=cli_execute.workspace_request(prepared,step,chain,output,preparation,descriptor)
        cli_execute.check_spec(selected)
        return request
    setup=SetupContext(prepared.handoff.prepared,prepared.wait,prepared.clippy_wait,factory)
    godot=next(tool for tool in spec.setup.tools if tool.name=='godot')
    context=SetupPrefixContext(setup,spec.selectors.candidate.root,spec.selectors.candidate.source_sha,
                               prepared.products.probes.candidate.input_files,godot.path,godot.sha256)
    result=_run_setup_prefix(context,prepared.handoff,prepared.static_product,spec.static,
        spec.strict,spec.output/'ordered',spec.control_fd,selected)
    returned=workspace_endpoints.anchor(spec.output/'ordered')
    cli_execute.check_spec(selected)
    returned.recheck()
    return result


def main(argv: tuple[str,...]) -> int:
    require(len(argv)==2,'/argv','expected one closed mode and absolute path')
    if argv[0]=='--execute-setup-prefix':
        result=execute_setup_prefix(Path(argv[1]))
        return 1 if document(result.read_text()).field('status').text()=='FAIL' else 2
    require(argv[0]=='--plan','/argv','only --plan or --execute-setup-prefix supported')
    _=write_plan(Path(argv[1]))
    return 2


if __name__=='__main__':
    sys.exit(main(tuple(sys.argv[1:])))
