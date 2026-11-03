# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by whole-gate observation collector for original step21.
"""Bounded original play-owner join; native observer qualification remains explicit."""
from dataclasses import dataclass
from pathlib import Path

from generic_gate import boundary, document, require
from generic_process import file_hash
from mux2_gate_inputs import Json, Node

from generic_legacy_play_build import validate_play_build
from generic_play_owner import evidence_membership
from generic_play_projection import read_projection
from generic_whole_gate_play import QUALIFICATION_INPUTS
from generic_whole_gate_ui import bind_outer


@dataclass(frozen=True, slots=True)
class PlayCollection:
    arm: str
    original_case_exits: tuple[int,...]
    actual_native_children: int
    result_sha256: str
    qualification: str = 'INCOMPLETE'
    pending: tuple[str,...] = QUALIFICATION_INPUTS


def _anchored(stage: Node, path: Path) -> Node:
    stage.field('observer_artifacts').field(str(path)).equals(file_hash(path))
    return document(path.read_text())


@boundary
def collect_play(session: Node, stage: Node) -> PlayCollection:
    stage.field('step').equals(21);stage.field('name').equals('play_check')
    root,_=bind_outer(session,stage);arm=stage.field('arm').text()
    output=Path(session.field('output').text())/'steps'/arm/'21-play_check/play-owned'
    workspace=Path(session.field('output').text())/'play-workspaces'/arm
    argv=[entry.text() for entry in stage.field('argv').array()]
    require(len(argv) in {5,6} and argv[:2]==['python3','tools/v8/generic_whole_gate_play.py'] and argv[4]==arm,
            '/argv','foreign whole-play entrypoint')
    if len(argv)==6:require(argv[5]==str(output),'/argv','foreign play output')
    result=_anchored(stage,output/'play-result.json');plan=_anchored(stage,output/'plan.json')
    result.field('schema').equals('whole_gate_play_result.v1');result.field('settlement').equals('SETTLED')
    for node in (result,plan):
        node.field('arm').equals(arm);node.field('session').equals(argv[2]);node.field('session_sha256').equals(argv[3])
    require(file_hash(Path(argv[2]))==argv[3],'/session','original play session changed')
    result.field('qualification').equals('INCOMPLETE');result.field('pending_native_qualification').equals(list(QUALIFICATION_INPUTS))
    result.field('original_exit').equals(0);result.field('original_case_exits').equals([3,2,4,0,0,0])
    result.field('plan').equals(str(output/'plan.json'));result.field('plan_sha256').equals(file_hash(output/'plan.json'))
    terminal_path=workspace/'owner/terminal.json';result.field('owner_terminal').equals(str(terminal_path))
    require(file_hash(terminal_path)==result.field('owner_terminal_sha256').text(),'/owner','actual terminal changed')
    terminal=document(terminal_path.read_text());terminal.field('settlement').equals('SETTLED');terminal.field('outer_exit').equals(0)
    terminal.field('case_exits').equals([3,2,4,0,0,0]);terminal.field('native_waits').equals('OBSERVED')
    terminal.field('wrapper_wait_authority').equals('SOURCE_FOREGROUND_CASE_SHELL_WAIT')
    retained=terminal.field('retained_files').mapping()
    for name,value in retained.items():
        require(file_hash(Path(name))==Node(value).text(),name,'original owner retained artifact changed')
    for name,value in terminal.field('outer_log_hashes').mapping().items():
        require(file_hash(workspace/'owner'/name)==Node(value).text(),name,'original outer log changed')
    membership=terminal.field('native_output_membership').mapping()
    require(len(membership)==16,'/coverage','original native child count differs')
    for name,value in membership.items():
        path=Path(name);require(path.parent==workspace/'native','/output','foreign native output')
        require(evidence_membership(path)==Node(value).mapping(),name,'original native namespace changed')
        require(str(path/'receipt.json') in retained,'/receipt','native receipt lacks original owner anchor')
    intent_path=workspace/'owner/intent.json';require(str(intent_path) in retained,'/owner','original intent missing')
    intent=document(intent_path.read_text());intent.field('cwd').equals(str(root))
    intent.field('argv').equals(['/bin/bash','tools/v8/test_play.sh'])
    for name,value in intent.field('inputs').mapping().items():
        require(file_hash(Path(name))==Node(value).text(),name,'original plan input changed')
    for version in ('main','m1','v7'):
        path=workspace/'observer-exports'/(version+'.json')
        require(str(path) in retained,'/projection','projection lacks owner anchor')
        projection=read_projection(path);projection.recheck()
        result.field('projections').field(version).equals(projection.declaration())
        require(projection.original_root==workspace/'play-roots'/version,'/source','foreign original source')
        joined=document(projection.original_build_join.read_text())
        _=validate_play_build(joined,projection.commit,Path(joined.field('bridge').text()))
    return PlayCollection(arm,(3,2,4,0,0,0),16,file_hash(output/'play-result.json'))


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> tuple[Json,...]:
    base.field('arm').equals('base');candidate.field('arm').equals('candidate')
    values=(collect_play(session,base),collect_play(session,candidate))
    return tuple({'arm':item.arm,'original_case_exits':list(item.original_case_exits),
        'actual_native_children':item.actual_native_children,'result_sha256':item.result_sha256,
        'qualification':item.qualification,'pending_native_qualification':list(item.pending)} for item in values)
