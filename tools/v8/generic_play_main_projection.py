# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: source-owned play owner after original main build returns.
"""Current main observer export from existing whole-session BASE+O antecedents."""
import shutil
from pathlib import Path

from generic_gate import document, require
from generic_process import atomic_json, file_hash
from generic_sources import freeze_source, safe_path
from generic_whole_gate_contract import current_product_inputs, load_session
from mux2_gate_inputs import BRIDGE, Node

from generic_legacy_play_build import validate_play_build
from generic_play_projection import PlayProjection
from generic_play_sources import SourceGrants


def prepare_main_projection(grants: SourceGrants, build_join: Path, join_sha256: str,
                            session_path: Path, session_sha256: str, destination: Path) -> PlayProjection:
    """Original main build and actual observer bridge are separately retained, never conflated."""
    session=load_session(session_path,session_sha256)
    base=session.field('arms').field('base');source=grants.source('main')
    base.field('source_sha').equals(source.commit)
    grants.check();grants.check_built('main');built=grants.built['main']
    require(file_hash(build_join)==join_sha256,'/join','original main build join changed')
    joined=document(build_join.read_text())
    require(validate_play_build(joined,source.commit,built.path)==built.sha256,'/build','original main build differs')
    joined.field('build_receipt').equals(str(built.receipt));joined.field('build_receipt_sha256').equals(built.receipt_sha256)
    joined.field('build_pid').equals(built.pid)
    original=grants.play_root/'main';before=freeze_source(original,source.commit)
    require(before==grants.started['main'],'/source','original main source grant differs')
    runtime=current_product_inputs(session,'base','bridge')
    root=Path(base.field('root').text())
    require(destination.is_absolute() and destination==destination.resolve() and not destination.exists()
            and not destination.is_relative_to(root) and not destination.is_relative_to(original),
            '/projection','fresh separate main export required')
    destination.mkdir(parents=True,exist_ok=False)
    omitted_links: dict[str,str]={}
    for name,value in base.field('effective_source').field('manifest').mapping().items():
        entry=Node(value).array()
        if entry[0].text()=='120000':
            require(name.startswith('.agents/skills/'),name,'runtime symlink projection needs explicit qualification')
            omitted_links[name]=entry[1].text()
    immutable: dict[Path,str]={}
    for name,value in base.field('source_inputs').mapping().items():
        require(not name.startswith('.godot/'),name,'generated cache cannot be source input')
        path=safe_path(root,name);expected=Node(value).text()
        require(file_hash(path)==expected,name,'BASE+O source changed before copy')
        target=destination/name;target.parent.mkdir(parents=True,exist_ok=True)
        _=safe_path(destination,name)
        _=shutil.copyfile(path,target);require(file_hash(target)==expected,name,'copied BASE+O source differs')
        immutable[target]=expected
    binary=Path(runtime.field('executable').text());runtime_hash=runtime.field('executable_sha256').text()
    target=destination/BRIDGE;target.parent.mkdir(parents=True,exist_ok=True)
    _=shutil.copyfile(binary,target);require(file_hash(target)==runtime_hash,'/bridge','observer bridge copy changed')
    immutable[target]=runtime_hash
    provenance=destination/'gate_observer/main-provenance.json';provenance.parent.mkdir(exist_ok=True)
    atomic_json(provenance,{'schema':'play_main_observer_projection.v1','product_commit':source.commit,
        'original_build_join':str(build_join),'original_build_join_sha256':join_sha256,
        'original_bridge_sha256':built.sha256,'observer_bridge_sha256':runtime_hash,
        'observer_build_receipt':runtime.field('build_receipt').value,
        'observer_build_receipt_sha256':runtime.field('build_receipt_sha256').value,
        'whole_session':str(session_path),'whole_session_sha256':session_sha256,
        'observer_source_inputs':base.field('source_inputs').value,'omitted_agent_skill_symlinks':{name:digest for name,digest in omitted_links.items()},
        'qualification':'EXISTING_SESSION_BASE_PLUS_O_NATIVE_QUALIFICATION_PENDING'})
    immutable[provenance]=file_hash(provenance)
    helper=destination/'tools/v8/gate_ui_observer.gd'
    require(helper in immutable,'/observer','current observer source absent from materialized BASE')
    projection=PlayProjection('main',original,destination,source.commit,build_join,join_sha256,file_hash(helper),tuple(sorted(immutable.items())))
    require(freeze_source(original,source.commit)==before,'/source','main export modified original source')
    _=load_session(session_path,session_sha256);grants.check();projection.recheck()
    return projection
