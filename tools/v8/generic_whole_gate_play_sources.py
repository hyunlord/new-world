# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: source-owned whole-gate play planner before original six cases launch.
"""Freeze local Git source and tool/config inputs for the original launcher cases."""
import hashlib
import os
import re
import shutil
import subprocess
from pathlib import Path

from generic_gate import require
from generic_process import file_hash
from generic_sources import git, manifest_hash
from mux2_gate_inputs import Json, Node

from generic_play_sources import HistoricalBindings, HistoricalSource


def object_manifest(root: Path, commit: str) -> tuple[dict[str,Json],dict[str,bytes]]:
    """Read committed Git blobs in one batch; no checkout, fetch or source rewrite."""
    rows: list[tuple[str,str,str]]=[]
    for record in git(root,'ls-tree','-rz',commit).split(b'\0'):
        if not record:continue
        header,encoded=record.split(b'\t',1);mode,kind,oid=header.decode().split()
        require(kind=='blob' and mode in {'100644','100755','120000'},'/tree','unsupported original Git object')
        rows.append((os.fsdecode(encoded),mode,oid))
    require(bool(rows),'/tree','empty historical source')
    response=subprocess.run(['git','-C',str(root),'cat-file','--batch'],
        input=('\n'.join(oid for _,_,oid in rows)+'\n').encode(),check=True,capture_output=True).stdout
    cursor=0;manifest: dict[str,Json]={};selected: dict[str,bytes]={}
    for name,mode,expected in rows:
        end=response.index(b'\n',cursor);oid,kind,size=response[cursor:end].decode().split()
        require(oid==expected and kind=='blob','/tree','Git batch identity mismatch')
        start=end+1;cursor=start+int(size);blob=response[start:cursor]
        require(response[cursor:cursor+1]==b'\n','/tree','truncated Git blob');cursor+=1
        manifest[name]=[mode,oid,hashlib.sha256(blob).hexdigest()]
        if name in {'sim_bridge.gdextension','rust/Cargo.lock'}:selected[name]=blob
    require(cursor==len(response),'/tree','unexpected Git batch suffix')
    return manifest,selected


def selected_tool(environment: dict[str,str], name: str, session: Node, *, prebound: bool = True) -> tuple[Path,str]:
    found=shutil.which(name,path=environment['PATH']);require(found is not None,'/tools','missing '+name)
    assert found is not None
    path=Path(found).resolve(strict=True);digest=file_hash(path)
    if prebound:session.field('external_inputs').field(str(path)).equals(digest)
    return path,digest


def next_admin(common: Path, name: str) -> Path:
    """Freeze the first unused Git worktree administration name before launch."""
    path=common/'worktrees'/name;index=0
    while path.exists():
        index+=1;require(index<=10000,'/worktree','unbounded administrative name collisions')
        path=common/'worktrees'/(name+str(index))
    return path


def bindings(session: Node, arm: str, environment: dict[str,str], play_root: Path) -> tuple[HistoricalBindings,dict[str,tuple[str,...]]]:
    root=Path(session.field('arms').field(arm).field('root').text())
    published=session.field('arms').field('base').field('source_sha').text()
    require(git(root,'rev-parse','origin/v8/main^{commit}').decode().strip()==published,'/main','published ref differs before original fetch')
    versions=(('main',published,None),('m1','892edca1d31b26e535a3e9b62b37d62b99bb1130',None),
              ('v7','b1ca032f8fa84cee87d68a185e471b8e2c2bddce','7db2907651b069f2db9dd843599abe1872e581b2'))
    sources: list[HistoricalSource]=[];builds: dict[str,tuple[str,...]]={}
    common=Path(os.fsdecode(git(root,'rev-parse','--path-format=absolute','--git-common-dir')).strip())
    configs={common/'config',Path(os.fsdecode(git(root,'rev-parse','--absolute-git-dir')).strip())/'config.worktree'}
    home=Path(environment['HOME']);cargo_home=Path(environment.get('CARGO_HOME',str(home/'.cargo')))
    configs|={home/'.gitconfig',home/'.config/git/config',Path('/etc/gitconfig').resolve(),cargo_home/'config',cargo_home/'config.toml'}
    for version,commit,tag in versions:
        manifest,selected=object_manifest(root,commit)
        lock=hashlib.sha256(selected['rust/Cargo.lock']).hexdigest() if 'rust/Cargo.lock' in selected else file_hash(root/'rust/Cargo.lock')
        sources.append(HistoricalSource(version,commit,manifest_hash(manifest),lock,tag))
        extension=selected['sim_bridge.gdextension'].decode()
        paths=re.findall(r'^macos\.debug *= *"res://([^"\n]+)"$',extension,re.MULTILINE)
        require(len(paths)==1 and paths[0] in {'rust/target/debug/libsim_bridge.dylib','rust/target/release/libsim_bridge.dylib'},
                '/bridge','unsupported original bridge selection')
        builds[version]=('cargo','build','-p','sim-bridge',*(('--release',) if '/release/' in paths[0] else ()))
        for parent in (play_root/version/'rust',*(play_root/version/'rust').parents):
            configs|={parent/'.cargo/config',parent/'.cargo/config.toml'}
        configs.add(next_admin(common,version)/'config.worktree')
    require(git(root,'rev-parse','v7-final-art').decode().strip()==versions[2][2],'/tag','historical tag object changed')
    frozen=tuple((path,file_hash(path) if path.exists() else None) for path in sorted(configs))
    return HistoricalBindings(tuple(sources),selected_tool(environment,'git',session,prebound=False),
        selected_tool(environment,'cargo',session),selected_tool(environment,'rustc',session),frozen),builds
