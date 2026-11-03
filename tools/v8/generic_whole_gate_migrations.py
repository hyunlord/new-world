"""Authentic legacy loaded windows; proposed safety never grants ordinary admission."""
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from gate_test_summary import parse_log, validate_cases
from generic_gate import boundary, document, integer, require, text
from generic_process import file_hash
from generic_test_endpoint import Endpoint, authenticate_endpoint
from generic_test_endpoint_types import ExpectedOccurrence, Loss, bound_bytes
from generic_test_history_graph import ResolvedHistory, identity, resolve_graph, extends
from generic_test_plans import Planned
from generic_whole_gate_aggregate import aggregate_output
from generic_whole_gate_endpoints import bind_observer
from mux2_gate_fixtures import validate_fixtures
from mux2_gate_inputs import Json, Node

V10: Final = 'genuine_v10_saves_gain_defence_defaults_and_continue_identically_after_resave'
LEGACY: Final = 'genuine_legacy_inventory_preserves_fields_and_current_policy_continuation'
B5: Final = 'actual_h_format9_fixture_loads_as_oak_then_resaves_as_current_format'
TARGETS: Final = {'steward_defence_migration':V10,'legacy_migration_current':LEGACY,'b5_flora':B5}
ENV: Final = ('MUX2_V10_FIXTURES','MUX2_V10_MANIFEST','WORLDSIM_MUX_FIXTURES','WORLDSIM_B5_LEGACY9')
SOURCE_PINS: Final = {'rust/crates/sim-test/tests/steward_defence_migration.rs': '00f5b80f9e692ebbc9c2e9771c930072ea51bff4fb90d6b6a62dc8853efbe2ce', 'rust/crates/sim-test/tests/legacy_migration_current.rs': 'b7664df1c3a3bc6c4e00523706f323aaa9901ef2d0eaaf93c8778ceefef0751f', 'rust/crates/sim-test/tests/b5_flora.rs': '78e45dfed2005065d20b3363fcc9ad209c26c85dd62efda47a0b7183ed60cc7d', 'rust/crates/sim-test/tests/legacy_migration_support/inventory.rs': '087990f9a5a75f1c9179badd8b3e2c7f26219315073e464d8c4f58c4f519f5b5', 'rust/crates/sim-test/tests/legacy_migration_support/wire.rs': '0d2294417cad814c5c5e546d1b8195cff157479484d456ed2357718019cbd24e', 'tools/v8/mux2_gate_checks.sh': '527313c36d44e0f462a1986d181718c370ca819e282fbbf7e6f029982222471c', 'tools/v8/mux2_gate_fixtures.py': '5f44a642c68e63daf907d55316415312e300a2e5d3e69a4eaad8c0630b793525'}
LEGACY_ROWS: Final = (('historical/m55/tick-2000.save', '5043fca3c5262693ea6040c9746fe8ea249ff9b6db6676d121879ebbd19c7efa', 6, 2000, 5), ('historical/m55/tick-3000.save', '94616c6ded2e86989300d8b15b75159cd925242eb123c83bfd77fdfa414607b3', 6, 3000, 5), ('historical/mart2/tick-2000.save', '5043fca3c5262693ea6040c9746fe8ea249ff9b6db6676d121879ebbd19c7efa', 6, 2000, 5), ('historical/mart2/tick-3000.save', '94616c6ded2e86989300d8b15b75159cd925242eb123c83bfd77fdfa414607b3', 6, 3000, 5), ('base/on-tick-5000.save', 'b24605f9f8086a06f6de42597b4eb84a8d8dd3d535dd43afd41832efdfd46397', 7, 5000, 5), ('base/on-tick-6000.save', '362a377d37a759e42c3b6c9d0e20f0c49b9dae793ec1a3653406a8e256d8ee53', 7, 6000, 5), ('core8/on-tick-5000.save', 'fed3fe2616fb94001858493aedb523421d3e93273e96e39c9db971e82130dadb', 8, 5000, 5), ('core8/on-tick-6000.save', 'c6ada78c90ca7420791fabc2245033645f950771b6d693cfa2e0ab8b5878ce7f', 8, 6000, 5), ('b4/h-format9-fixture/tick4000.save', 'e315e1c952b7e0504fd7cbe3b46c867b8c16d0735787e27e11ddab65f6894141', 9, 4000, 0))
LEGACY_PROVENANCE: Final = {'historical/acquisition-receipt.json': 'd75171a4d51a9867e37c5bb670412511b16e05c394dbf47ae55f2ec05640508e', 'historical/m55/source-commit.txt': '898b1e64ef4bf3de5c94d1552e4430c7b17e6a377b0bf3492d29add961780709', 'historical/mart2/source-commit.txt': '0ece54b3e9ffc01873d2bea9885e48a03c77aced9516417f730274e85e672b16', 'historical/m55/continuation-2000-3000.json': '4a44834c6720e5e02619c0db6e9e08bf74b2bd9ac131f2929203be8af47357f6', 'historical/mart2/continuation-2000-3000.json': '4a44834c6720e5e02619c0db6e9e08bf74b2bd9ac131f2929203be8af47357f6', 'base/provenance.json': '109c3dda2e1b2d5583d0fab115d4df31677716b3d29f2bed566abb4213c17db4', 'core8/acquisition-receipt.json': 'dac021d09b22ad8792b721ef84c54e094216366b3240e3f09b4c293a4d2b7aae', 'b4/h-format9-fixture/receipt.json': '2fdae7bcf9481347afaeda967f49144febadff727258f3f6eba8e38d2cf15bbc'}


@dataclass(frozen=True, slots=True)
class LoadedFamily:
    plans: tuple[Planned,...]
    endpoints: tuple[Endpoint,...]
    histories: tuple[ResolvedHistory,...]
    pid: int
    external: tuple[tuple[str,str],...]


def selected_output(raw:str,target:str) -> None:
    headers=list(re.finditer(r'^\s*Running (?:unittests |tests/)?([^\s(]+)\s+\([^\n]+\)\s*$',raw,re.MULTILINE))
    segments=[raw[match.end():headers[index+1].start() if index+1<len(headers) else len(raw)]
              for index,match in enumerate(headers) if Path(match.group(1)).stem==target]
    require(len(segments)==1,target,'missing or duplicate migration invocation')
    summary=parse_log(segments[0]);validate_cases(summary,(TARGETS[target],),())
    require(summary.passed==1 and summary.failed==summary.ignored==summary.measured==0,target,'migration oracle not successful')
    require([line.strip() for line in segments[0].splitlines() if line.strip().startswith('running ')]==['running 1 test'],target,'zero or extra selected migrations')


@boundary
def collect_migration(session:Node,stage:Node,target:str) -> LoadedFamily:
    require(integer(stage.field('step'))==37 and target in TARGETS,'/route','unregistered migration route')
    raw,_=aggregate_output(session,stage);selected_output(raw,target)
    arm=text(stage.field('arm'));product=session.field('arms').field(arm);root=Path(text(product.field('root')))
    inputs={name:text(Node(value)) for name,value in product.field('source_inputs').mapping().items()}
    for name,digest in SOURCE_PINS.items():
        require(inputs.get(name)==digest and file_hash(root/name)==digest,name,'unregistered migration source scope')
    environment={name:text(Node(value)) for name,value in stage.field('environment').mapping().items()}
    anchors={name:text(Node(value)) for name,value in session.field('external_inputs').mapping().items()}
    retained:dict[str,str]={}
    def external(path:Path,expected:str|None=None) -> bytes:
        require(str(path) in anchors,str(path),'external input absent from original session anchors')
        digest=anchors[str(path)];require(expected is None or digest==expected,str(path),'source-pinned original input differs')
        data=bound_bytes(path,digest);retained[str(path)]=digest;return data
    evidence,graph,folder=bind_observer(stage,root,inputs,target,(TARGETS[target],))
    hashes=dict(evidence.output_hashes)
    def read(path:Path) -> Node:
        name=path.relative_to(evidence.root).as_posix();require(name in hashes,name,'unbound migration observation')
        return document(bound_bytes(path,hashes[name]).decode())
    def occurrence(branch:str) -> tuple[Path,int,int]:
        candidates=list(folder.glob(TARGETS[target]+'_s*_'+branch))
        require(len(candidates)==1,branch,'missing or duplicate source-planned migration branch')
        path=candidates[0];initial=read(path/'initial.json')
        seed=integer(initial.field('seed'));start=integer(initial.field('initial').field('tick'))
        require(path.name==f'{TARGETS[target]}_s{seed}_{branch}',branch,'emitted seed/branch mismatch')
        return path,seed,start
    def saved_input(path:Path,original:bytes,version:int) -> None:
        name=(path/'save_external_input.wsave').relative_to(evidence.root).as_posix()
        require(name in hashes,name,'external save bytes not retained')
        data=bound_bytes(evidence.root/name,hashes[name]);require(data==original and data[:12]==b'WORLDSIM'+version.to_bytes(4,'little'),name,'wrong actual migration source bytes/version')
    rows:list[Planned]=[]
    def add(branch:str,duration:int,stop:str,seed:int|None=None,start:int|None=None) -> Path:
        path,actual_seed,actual_start=occurrence(branch)
        require((seed is None or seed==actual_seed) and (start is None or start==actual_start),branch,'source-planned loaded seed/tick differs')
        rows.append(Planned(ExpectedOccurrence(TARGETS[target],TARGETS[target],actual_seed,branch,actual_start,actual_start+duration,False),stop))
        return path
    if target=='steward_defence_migration':
        directory=Path(environment['MUX2_V10_FIXTURES']);manifest=Path(environment['MUX2_V10_MANIFEST']);_=external(manifest)
        validate_fixtures(directory,manifest)
        for index,path in enumerate(sorted(p for p in directory.iterdir() if p.suffix in {'.save','.wsave'})):
            original=external(path);parent=add(f'fixture_{index}_migrated',60,'60_ticks')
            source=read(parent/'checkpoint_source.json');source.field('detail').equals({'path':str(path),'version':10})
            source.field('state').equals(read(parent/'initial.json').field('initial').value)
            saved_input(parent,original,10)
            _,seed,start=occurrence(f'fixture_{index}_migrated')
            _=add(f'fixture_{index}_reloaded',60,'60_ticks',seed,start)
    elif target=='legacy_migration_current':
        directory=Path(environment['WORLDSIM_MUX_FIXTURES'])
        for name,digest in LEGACY_PROVENANCE.items():_=external(directory/name,digest)
        for index,(relative,digest,version,start,seed) in enumerate(LEGACY_ROWS):
            path=directory/relative;original=external(path,digest)
            parent=add(f'fixture_{index}_initial',0,'load_and_resave_only',seed,start)
            read(parent/'checkpoint_source.json').field('detail').equals({'path':str(path),'sha256':digest,'version':version})
            saved_input(parent,original,version)
            duration=24 if version==9 else 1000
            fresh=add(f'fixture_{index}_fresh',duration,'current_policy_duration_reached',seed,start);saved_input(fresh,original,version)
            _=add(f'fixture_{index}_twin',duration,'current_policy_duration_reached',seed,start)
            if version==6 and start==2000:
                expected=external(path.parent/'continuation-2000-3000.json')
                name=(fresh/'input_captured_bills.bin').relative_to(evidence.root).as_posix()
                require(name in hashes and bound_bytes(evidence.root/name,hashes[name])==expected,name,'actual continuation commands changed or absent')
    else:
        path=Path(environment['WORLDSIM_B5_LEGACY9']);original=external(path,LEGACY_ROWS[-1][1])
        parent=add('migrated',24,'24_ticks_before_chop_mechanism',0,4000);saved_input(parent,original,9)
        _=add('twin',24,'24_ticks',0,4000)
        post=add('post_chop_mechanism_load',0,'post_chop_load_only',0)
        _,_,start=occurrence('post_chop_mechanism_load');require(4024<=start<=5464,'/chop','mechanism finished outside original bounded loop')
        read(post/'checkpoint_classification.json').field('detail').equals({'classification':'mechanism_load_only','source':'same test migrated branch after explicit chop setup; not ordinary continuation','interventions':'cancel active jobs; M3 rules; move interval 1; positioned workers; stabilized needs; chop command'})
    require({path.name for path in folder.iterdir()}=={identity(row.occurrence) for row in rows},target,'extra or missing loaded occurrence')
    endpoints=tuple(authenticate_endpoint(evidence,row.occurrence) for row in rows)
    histories=resolve_graph(graph,tuple(row.occurrence for row in rows),{identity(row.occurrence):endpoint for row,endpoint in zip(rows,endpoints,strict=True)})
    for row,endpoint in zip(rows,endpoints,strict=True):
        require(endpoint.stop_reason==row.stop,target,'migration stop differs')
        extends(endpoint.initial,endpoint.terminal,target)
        require(endpoint.founding is None,target,'legacy root acquired invented founding census')
    return LoadedFamily(tuple(rows),endpoints,histories,evidence.pid,tuple(sorted(retained.items())))


def proposed_window(base:Endpoint,candidate:Endpoint) -> dict[str,Json]:
    """Diagnostic proposal only: cannot replace strict ordinary/full-history admission."""
    same=(base.initial==candidate.initial and base.terminal.tick==candidate.terminal.tick)
    losses=candidate.terminal.losses or ();prefix=candidate.initial.losses or ()
    new=losses[len(prefix):]
    no_new=losses[:len(prefix)]==prefix and not new
    alive=len(candidate.terminal.alive_ids)>=len(base.terminal.alive_ids)
    def encoded(rows:tuple[Loss,...]) -> list[Json]:
        return [[row.tick,row.node,row.holder] for row in rows]
    return {'policy':'PROPOSED_LOADED_WINDOW_NOT_ADOPTED','status':'PASS' if same and no_new and alive else 'FAIL',
            'same_actual_initial_prefix':same,'no_new_candidate_loss':no_new,'candidate_alive_at_least_base':alive,
            'initial_tick':candidate.initial.tick,'terminal_tick':candidate.terminal.tick,
            'initial_loaded_alive':len(candidate.initial.alive_ids),'base_alive':len(base.terminal.alive_ids),
            'candidate_alive':len(candidate.terminal.alive_ids),'candidate_inherited_losses':encoded(prefix),
            'candidate_full_recorded_losses':encoded(losses),'candidate_new_losses':encoded(new),
            'base_full_recorded_losses':encoded(base.terminal.losses or ()),
            'strict_recorded_zero_loss':not losses,
            'founding_census':'UNKNOWN_EXTERNAL_ORIGIN','full_history_admission':'UNRESOLVED',
            'strict_ordinary_admission':False}


@boundary
def evaluate_migrations(session:Node,base:Node,candidate:Node) -> dict[str,Json]:
    missing=[name for name in ENV if name not in base.field('environment').mapping() or name not in candidate.field('environment').mapping()]
    if missing:return {'status':'INCOMPLETE','coverage':[],'proposed_windows':[],'unsupported':['migration external environment missing: '+','.join(missing)]}
    coverage:list[Json]=[];windows:list[Json]=[];unresolved:list[Json]=[]
    for target in TARGETS:
        left,right=collect_migration(session,base,target),collect_migration(session,candidate,target)
        require(left.plans==right.plans and left.external==right.external,target,'loaded source/horizon contract differs between arms')
        for plan,a,b,history in zip(left.plans,left.endpoints,right.endpoints,right.histories,strict=True):
            mechanism=plan.occurrence.branch=='post_chop_mechanism_load'
            load_only=plan.occurrence.start_tick==plan.occurrence.end_tick
            coverage.append({'target':target,'test':plan.occurrence.test,'branch':plan.occurrence.branch,'seed':plan.occurrence.seed,
                'initial_tick':plan.occurrence.start_tick,'terminal_tick':plan.occurrence.end_tick,'base_emitted_pid':left.pid,'candidate_emitted_pid':right.pid,
                'scope':'mechanism_load_only' if mechanism else ('legacy_load_only' if load_only else 'legacy_loaded_continuation'),
                'founding_census':None,'prefix_pending':[str(value) for value in history.pending],
                'full_recorded_losses':[[loss.tick,loss.node,loss.holder] for loss in b.terminal.losses or ()]})
            if load_only:
                if not mechanism:
                    require(a.initial==b.initial and a.terminal==b.terminal,target,'BASE/candidate load-only preservation differs')
                continue
            windows.append(proposed_window(a,b))
            unresolved.append({'test':plan.occurrence.test,'branch':plan.occurrence.branch,'reason':'external bytes and actual loaded history authenticated; original founding census unavailable; loaded-window policy requires owner review'})
    return {'status':'INCOMPLETE','coverage':coverage,'proposed_windows':windows,'unsupported':unresolved}
