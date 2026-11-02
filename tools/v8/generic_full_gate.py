# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/generic_full_gate.py --plan /absolute/fresh/output
"""Closed obligation ledger and bounded adapter dispatch, never publication authority.

The Python API executes existing direct/UI components. Build authentication,
remaining native suites and performance dispatch are explicitly incomplete.
"""
from __future__ import annotations

import sys
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import generic_direct_run as direct
import generic_renderer_run as renderer
import generic_ui_fixed_run as fixed
import generic_ui_pack_run as pack
import generic_ui_visibility_run as visibility
import generic_ui_why_run as why
from generic_declarations import Declarations
from generic_fixture_registry import ExpansionContext, Registry, expand
from generic_fixture_run import ComponentResult, Products
from generic_gate import GateContractError, document, require
from generic_observations import keys
from generic_process import atomic_json, file_hash
from generic_sources import inventory
from mux2_gate_inputs import Json, Node
from mux2_gate_lock import lane

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
    direct_fixtures = {
        10: ('identity.baseline_samples','identity.baseline_twins'), 11: ('determinism.baseline',),
        18: ('colony.u10',), 19: ('replay.generate','replay.recorded','replay.scripted'),
        26: ('economy.soak_no_beasts',), 29: ('story.script48',), 30: ('neighbours.script48',),
        35: ('steward.on24_twins',), 44: ('core.baseline10','core.colony_u10','core.colony_soak20',
            'core.steward_off','core.steward_on','m6a.lifecycle_off','m6a.lifecycle_on'),
        47: ('m6a.food72',), 48: ('m6a.food_data_only',),
        50: ('read.determinism','read.soak20','read.story48','read.neighbours48','read.steward_on24','read.m6a_steward_on72'),
    }
    composite[28] = ('colony_knowledge.release','knowledge.f10_injected','knowledge.f10_no_kill')
    mapped = {23:'renderer',39:'visibility',40:'why.base'}
    result: list[Obligation] = []
    for step in ORDER:
        for name in direct_fixtures.get(step, composite.get(step, (STEP_NAMES[step-1],))):
            fixture = name if step in direct_fixtures or name.startswith('knowledge.f10_') else None
            component = 'direct' if fixture else name if name in COMPONENTS else mapped.get(step)
            result.append(Obligation(step,f'{step}:{name}',component,fixture))
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


Action = Callable[[Path], ComponentResult]


def endpoint(name: str) -> str:
    return ('receipt.json' if name == 'direct' else 'fixed-ui.json' if name in fixed.FIXTURES else
            'why.json' if name.startswith('why.') else 'pack-ui.json' if name == 'pack' else name+'.json')


def authenticate(name: str, node: Node, inputs: Inputs) -> str:
    """Bind the actual producer's terminal schema and identity, without invented fields."""
    state = node.field('status').text()
    require(state in {'PASS','FAIL','PENDING'}, '/status', 'unsupported component status')
    if name == 'direct':
        keys(node,{'schema','status','publication_authority','full_gate_status','registry_status','registry_sha256',
            'base','candidate','declaration_manifest_sha256','products','components','errors','unresolved_backlog','limit'})
        for key,value in (('schema','direct_components_v1'),('registry_sha256',inputs.registry.sha256),
            ('base',inputs.probes.base.source_sha),('candidate',inputs.probes.candidate.source_sha),
            ('declaration_manifest_sha256',inputs.declarations.source_manifest_sha256),
            ('full_gate_status','NOT_QUALIFIED'),('registry_status','INCOMPLETE_ADAPTERS')):
            node.field(key).equals(value)
        node.field('publication_authority').equals(False)
        for arm in ('base','candidate'):
            product=inputs.probes.arm(arm); row=node.field('products').field(arm)
            row.field('root').equals(str(product.input_root));row.field('executable').equals(str(product.executable))
            row.field('executable_sha256').equals(product.executable_sha256)
        _=node.field('components').array();_=node.field('unresolved_backlog').array()
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


def invoke(name: str, action: Action, output: Path, inputs: Inputs) -> Outcome:
    require(name in COMPONENTS and not output.exists(), '/component', 'unknown component or stale output')
    error='';returned: str | None=None;reported: str | None=None;receipt: Path | None=None;digest: str | None=None
    expected=output/endpoint(name)
    try:
        result=action(output);returned=result.status
        require(result.receipt==expected and expected==expected.resolve(strict=True),'/receipt','foreign terminal path')
        receipt=expected;digest=file_hash(expected)
        reported=authenticate(name,document(expected.read_text()),inputs)
        require(returned==reported,'/status','returned and on-disk status differ')
    except (GateContractError,OSError,ValueError,RuntimeError) as caught:
        error=str(caught)
        if expected.is_file():
            try:
                receipt=expected;digest=file_hash(expected)
                reported=authenticate(name,document(expected.read_text()),inputs)
            except (GateContractError,OSError,ValueError) as terminal_error:
                error+='; terminal: '+str(terminal_error)
    files: tuple[tuple[str,str],...]=();snapshot_error=''
    if output.exists():
        try:files=tuple(inventory(output,[p.name for p in output.iterdir()]).items())
        except (GateContractError,OSError,ValueError,RuntimeError) as caught:snapshot_error=str(caught)
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


def references(inputs: Inputs, output: Path) -> dict[str,list[Json]]:
    """Expand the same immutable direct invocation IDs/paths as the coordinator."""
    result: dict[str,list[Json]]={}
    for arm in ('base','candidate'):
        product=inputs.probes.arm(arm)
        context=ExpansionContext(product.input_root,product.input_root,output/'direct/runs',
            inputs.probes.base.executable,inputs.probes.candidate.executable,product.input_root/'tools/v8/gate_f10_no_kill.json')
        for row in expand(inputs.registry,context):
            if row.arm==arm:result.setdefault(row.fixture_id,[]).append({'id':row.id,'arm':row.arm,'repeat':row.repeat,
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


def main(argv: tuple[str,...]) -> int:
    require(len(argv)==2 and argv[0]=='--plan','/argv','only --plan ABSOLUTE_FRESH_OUTPUT supported; no full execution CLI yet')
    _=write_plan(Path(argv[1]))
    return 2


if __name__=='__main__':
    sys.exit(main(tuple(sys.argv[1:])))
