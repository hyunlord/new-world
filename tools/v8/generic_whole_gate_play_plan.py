# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by generic_whole_gate_play.py before original test_play.sh.
"""Source-owned exact six-case plan from an authenticated whole session."""
import os
import sys
from dataclasses import replace
from pathlib import Path

from gate_play_child import CASES
from generic_gate import require
from generic_play_owner import PlayPlan
from generic_process import atomic_json, file_hash
from generic_whole_gate_contract import load_session
from generic_whole_gate_environment import process_spec
from generic_whole_gate_play_sources import bindings
from mux2_gate_inputs import Json, Node

ROLES=('version','build','import-1','import-2','headless')


def explicit_home(session: Node, environment: dict[str,str]) -> Path:
    require('HOME' in environment,'/environment','original launcher requires explicit HOME')
    session.field('environment').field('HOME').equals(environment['HOME'])
    home=Path(environment['HOME'])
    require(home.is_absolute() and home==home.resolve() and home.is_dir(),
            '/environment/HOME','explicit canonical existing HOME required')
    return home


def build_plan(session_path: Path, session_sha256: str, arm: str, output: Path) -> PlayPlan:
    session=load_session(session_path,session_sha256)
    require(arm in {'base','candidate'},'/arm','unknown play arm')
    root=Path(session.field('arms').field(arm).field('root').text())
    allowed=Path(session.field('output').text())/'steps'/arm/'21-play_check'
    require(output.is_absolute() and output==output.resolve() and output.is_relative_to(allowed) and not output.exists(),
            '/output','play output must be fresh inside the original owned stage21')
    workspace=Path(session.field('output').text())/'play-workspaces'/arm
    require(not workspace.exists() and workspace==workspace.resolve(),'/workspace','fresh owned play workspace required')
    # A's explicit source/environment contract is reused without executing its Bash wrapper.
    template=process_spec(session,arm,('bash','tools/v8/test_play.sh'),output/'unused',{},None)
    environment=dict(template.environment)
    # macOS Python injects this CoreFoundation value before user code. Freeze the
    # actual owner startup value explicitly instead of waiving child drift.
    startup={key:os.environ[key] for key in ('__CF_USER_TEXT_ENCODING',) if key in os.environ}
    require(all(key not in environment or environment[key]==value for key,value in startup.items()),
            '/environment','owner startup locale differs from frozen environment')
    environment.update(startup)
    home=explicit_home(session,environment)
    inherited_target=environment.get('CARGO_TARGET_DIR')
    # Cargo resolves environment paths against the unchanged build cwd. This
    # overrides inherited arm/config targets with each original worktree target.
    environment['CARGO_TARGET_DIR']='target'
    environment.update(PLAY_ROOT=str(workspace/'play-roots'),PLAY_DATA=str(workspace/'play-data'),
        WORLDSIM_GATE_OBSERVER='1',WORLDSIM_GATE_UI_BOOT='1',WORLDSIM_GATE_UI_OUT=str(workspace/'ui-worlds'),
        WORLDSIM_GATE_INVOCATION_ID='whole-play:'+arm+':'+str(output),WORLDSIM_TEST_SETTINGS_DIR=str(workspace/'settings'))
    for key in tuple(environment):
        require(not key.startswith('WORLDSIM_GATE_PLAY_') and key!='WORLDSIM_MUX2_LANE_FD','/environment','caller play protocol override')
    history,builds=bindings(session,arm,environment,workspace/'play-roots')
    godot=environment['GODOT_BIN'];session.field('external_inputs').field(godot).equals(file_hash(Path(godot)))
    python=Path(sys.executable).resolve(strict=True)
    session.field('python').field('path').equals(str(python))
    # Freeze all helper/source/tool bytes; native original worktrees are separately granted at actual build boundaries.
    inputs={root/name: Node(digest).text() for name,digest in session.field('arms').field(arm).field('source_inputs').mapping().items()}
    inputs.update({Path(name):Node(digest).text() for name,digest in session.field('external_inputs').mapping().items()})
    inputs.update({session_path:session_sha256,history.git[0]:history.git[1],Path('/bin/bash').resolve():file_hash(Path('/bin/bash').resolve()),python:file_hash(python)})
    output.mkdir(parents=True,exist_ok=False)
    workspace.mkdir(parents=True,exist_ok=False)
    (output/'plans').mkdir();(workspace/'ui-worlds').mkdir();(workspace/'settings').mkdir()
    cases: dict[str,Json]={}
    for index,case in enumerate(CASES):
        if index in (1,2):cases[case]=None;continue
        version=('main','m1','v7')[index-3] if index>=3 else None
        roles: dict[str,Json]={}
        for role in ROLES if version is not None else ('version',):
            cwd=root if role=='version' else workspace/'play-roots'/str(version)
            argv=(godot,'--version');executable=Path(godot);expected=file_hash(executable);artifacts: list[str]=[]
            if role=='build':
                assert version is not None
                argv=builds[version];cwd/='rust';executable,expected=history.cargo
                profile='release' if '--release' in argv else 'debug'
                artifacts=[str(cwd/'target'/profile/'libsim_bridge.dylib')]
            elif role.startswith('import'):
                argv=(godot,'--headless','--path','.', '--import','--editor','--quit')
            elif role=='headless':argv=(godot,'--headless','--path','.', '--quit-after','300')
            row: dict[str,Json]={'argv':list(argv),'cwd':str(cwd),'executable':str(executable),'executable_sha256':expected,
                'inputs':{str(path):digest for path,digest in inputs.items()},'artifacts':[value for value in artifacts],
                'output':str(workspace/'native'/(str(index)+'-'+role)),
                'timeout_seconds':600 if role in {'import-1','import-2','headless'} else None,'merged':role!='version'}
            if role in {'import-1','import-2','headless'}:
                row['projection_manifest']=str(workspace/'observer-exports'/(str(version)+'.json'))
            roles[role]=row
        plan_path=output/'plans'/('case-'+str(index)+'.json')
        atomic_json(plan_path,{'schema_version':1,'roles':roles});inputs[plan_path]=file_hash(plan_path)
        cases[case]={'path':str(plan_path),'sha256':inputs[plan_path]}
    suite=output/'plans/suite.json';atomic_json(suite,{'schema_version':1,'cases':cases});inputs[suite]=file_hash(suite)
    child_environment={**environment,'PATH':environment['PATH']+':'+environment['HOME']+'/.cargo/bin:/opt/homebrew/bin:/usr/local/bin',
        'WORLDSIM_GATE_PLAY_RECEIPTS':'1','WORLDSIM_GATE_PLAY_SUITE_PLAN':str(suite),'WORLDSIM_GATE_PLAY_SUITE_PLAN_SHA256':file_hash(suite)}
    projections=tuple((case,role,tuple(sorted({**child_environment,'WORLDSIM_GATE_PLAY_CASE':case}.items())))
        for index,case in enumerate(CASES) if index not in (1,2) for role in (ROLES if index>=3 else ('version',)))
    observer=root/'tools/v8/gate_legacy_observer.gd'
    require(observer in inputs,'/observer','legacy observer is not in admitted source inputs')
    plan=PlayPlan(root,workspace/'owner',suite,file_hash(suite),tuple(sorted(inputs.items())),tuple(sorted(environment.items())),
        projections,history,(observer,inputs[observer]),workspace/'observer-exports',(session_path,session_sha256))
    atomic_json(output/'plan.json',{'schema':'whole_gate_play_plan.v1','arm':arm,'session':str(session_path),'session_sha256':session_sha256,
        'suite':str(suite),'suite_sha256':file_hash(suite),'owner_python_startup_environment':{key:value for key,value in startup.items()},'cases':list(CASES),'expected_native_children':16,
        'original_argv':['/bin/bash','tools/v8/test_play.sh'],'owner_output':str(plan.output),
        'cargo_target_projection':{'inherited':inherited_target,'effective':'target','relative_to':'each_original_build_cwd'},
        'explicit_home':str(home),
        'isolated_roots':{'PLAY_ROOT':environment['PLAY_ROOT'],'PLAY_DATA':environment['PLAY_DATA'],'observer_exports':str(plan.projection_root)},
        'historical_sources':[{'version':s.version,'commit':s.commit,'tracked_manifest_sha256':s.tracked_manifest_sha256,
             'lock_sha256':s.lock_sha256,'tag_oid':s.tag_oid} for s in history.sources],
        'config_presence':{str(path):digest for path,digest in history.configs},'git':{'path':str(history.git[0]),'sha256':history.git[1]},
        'qualification':'SOURCE_BOUND_PLAN_NATIVE_QUALIFICATION_PENDING'})
    plan=replace(plan,inputs=(*plan.inputs,(output/'plan.json',file_hash(output/'plan.json'))))
    _=load_session(session_path,session_sha256)
    return plan
