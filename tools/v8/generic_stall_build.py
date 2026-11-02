# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Closed stall evidence joins; scratch transformation is not historical execution."""
import json
import re
import subprocess
import tomllib
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path, PurePosixPath
from typing import Literal

from generic_build import (
    OBSERVER_EXACT_PATHS,
    Artifact,
    BuildSpec,
    Overlay,
    config_inventory,
)
from generic_gate import GateContractError, boundary, document, integer, require
from generic_process import (
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
    real_path,
)
from generic_sources import freeze_source, inventory, manifest_hash, safe_path
from generic_stall_source import (
    DerivedSourceSpec,
    authenticate_derived_source,
    recheck_derived_source,
)
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json, Node


@dataclass(frozen=True, slots=True)
class TransformSpec:
    root: Path
    origin_sha: str
    overlay: Overlay
    git: Path
    git_sha256: str
    scratch: Path
    output: Path


@dataclass(frozen=True, slots=True)
class BuildEvidenceSpec:
    build: BuildSpec
    transform: TransformSpec
    transformation_files: tuple[tuple[str, str], ...]
    build_files: tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class BuildEvidence:
    status: Literal['CONSISTENT_SUCCESS', 'BUILD_FAILED']
    errors: tuple[str, ...]
    artifact: tuple[Path, str] | None = None
    effective_source_sha256: str | None = None
    execution_provenance: Literal['CALLER_DRIVER_EVIDENCE_REQUIRED'] = field(default='CALLER_DRIVER_EVIDENCE_REQUIRED', init=False)
    runtime_admission: Literal['PENDING'] = field(default='PENDING', init=False)
    observer_semantics: Literal['NOT_RUN'] = field(default='NOT_RUN', init=False)
    publication_authority: Literal[False] = field(default=False, init=False)


def _files(root: Path) -> dict[str, str]:
    return inventory(root, [p.name for p in root.iterdir()]) if any(root.iterdir()) else {}


def _frozen(root: Path, files: tuple[tuple[str, str], ...]) -> None:
    require(bool(files) and len(dict(files)) == len(files) and _files(root) == dict(files), '/inventory', 'evidence changed')


def _scratch(root: Path) -> dict[str, Json]:
    rows: dict[str, Json] = {}
    for path in sorted(root.rglob('*')):
        require(path.name.casefold() != '.git' and not path.is_symlink(), str(path), 'Git entry or symlink in scratch')
        real_path(path, directory=path.is_dir())
        if path.is_file():rows[path.relative_to(root).as_posix()] = ['100755' if path.stat().st_mode & 0o111 else '100644', file_hash(path)]
    return rows


def _git_environment(spec: TransformSpec) -> tuple[tuple[str,str], ...]:
    home=spec.output/'anchors'
    return (('PATH','/usr/bin:/bin'),('LANG','C'),('LC_ALL','C'),('HOME',str(home/'home')),('XDG_CONFIG_HOME',str(home/'xdg')),
            ('GIT_CONFIG_NOSYSTEM','1'),('GIT_CONFIG_GLOBAL','/dev/null'),('GIT_CONFIG_SYSTEM','/dev/null'),
            ('GIT_ATTR_NOSYSTEM','1'),('GIT_CEILING_DIRECTORIES',str(spec.scratch.parent)),('GIT_NO_REPLACE_OBJECTS','1'))


@boundary
def prove_transformation(spec: TransformSpec) -> Path:
    """Apply only in new non-repository scratch; preserve failed attempts and product anchors."""
    for path in (spec.root, spec.git, spec.output.parent, spec.scratch.parent):real_path(path, directory=path != spec.git)
    roots = (spec.root, spec.output, spec.scratch)
    require(all(not a.is_relative_to(b) and not b.is_relative_to(a) for i,a in enumerate(roots) for b in roots[i+1:]), '/roots', 'overlapping scratch/product/evidence')
    require(not spec.output.exists() and not spec.scratch.exists(), '/output', 'fresh scratch/evidence required')
    spec.output.mkdir();spec.scratch.mkdir()
    receipt: dict[str, Json] = {'schema':'stall_patch_transform.v1','status':'FAIL','origin_sha':spec.origin_sha,
        'root':str(spec.root),'scratch':str(spec.scratch),'git':str(spec.git),'git_sha256':spec.git_sha256,
        'patch_application_provenance':'NOT_VERIFIED','historical_product_application':'NOT_VERIFIED'}
    errors: list[str] = []; source: dict[str,Json] | None = None; anchors: dict[str,str] = {}; processes: dict[str,str] = {}
    try:
        require(file_hash(spec.git) == spec.git_sha256, '/git', 'Git changed')
        source = freeze_source(spec.root,spec.origin_sha);receipt['source_before'] = source
        overlay = spec.overlay
        require(file_hash(overlay.patch) == overlay.patch_sha256 and file_hash(overlay.manifest) == overlay.manifest_sha256, '/overlay','evidence changed')
        home=spec.output/'anchors';home.mkdir();(home/'home').mkdir();(home/'xdg').mkdir()
        for name,path in (('overlay.patch',overlay.patch),('overlay.json',overlay.manifest)):_=(home/name).write_bytes(path.read_bytes())
        env=_git_environment(spec)
        atomic_json(home/'origin.json',source)
        node=document((home/'overlay.json').read_text());node.field('schema_version').equals(1)
        node.field('product_base_sha').equals(spec.origin_sha);node.field('observer_patch_sha256').equals(overlay.patch_sha256)
        expected: dict[str,Json]={}; original=Node(source).field('tracked_manifest')
        for row in node.field('files').array():
            name=row.field('path').text();relative=PurePosixPath(name)
            require(bool(name) and not relative.is_absolute() and '..' not in relative.parts and all(part.casefold() != '.git' for part in relative.parts)
                    and relative.as_posix()==name and name not in expected, '/overlay','unsafe or duplicate path')
            require(name in OBSERVER_EXACT_PATHS or name.startswith(('rust/crates/sim-test/src/','rust/crates/sim-bridge/src/','tools/v8/','docs/v8/')),name,'outside observer scope')
            (spec.scratch/name).parent.mkdir(parents=True,exist_ok=True);target=safe_path(spec.scratch,name)
            before=row.field('base0_sha256').value;after=row.field('observer_sha256').text();mode='100644'
            require(re.fullmatch('[0-9a-f]{64}',after) is not None, '/overlay','bad postimage')
            if before is not None:
                entry=original.field(name).array();mode=entry[0].text();entry[2].equals(before)
                require(mode in {'100644','100755'},name,'not regular original');row.field('operation').equals('modify')
                # Explicit pinned Git/env. generic_sources.git intentionally is NOT used here.
                result=subprocess.run((str(spec.git),'-C',str(spec.root),'cat-file','blob',entry[1].text()),
                    env=dict(env),capture_output=True,check=False)
                require(result.returncode == 0 and not result.stderr, name,'origin blob read failed')
                _=target.write_bytes(result.stdout);target.chmod(0o755 if mode=='100755' else 0o644)
                require(file_hash(target) == before,name,'blob preimage differs')
                anchor=home/'preimages'/name;anchor.parent.mkdir(parents=True,exist_ok=True);_=anchor.write_bytes(result.stdout)
            else:
                require(name not in original.mapping(),name,'addition exists in origin');row.field('operation').equals('add')
            expected[name]=[mode,after]
        allowed=tuple(row.text() for row in node.field('allowed_paths').array())
        require(bool(expected) and len(set(allowed))==len(allowed) and set(expected)==set(allowed),'/overlay','path coverage differs')
        patch=(home/'overlay.patch').read_text()
        headers=tuple((match[1],match[2]) for match in re.finditer(r'^diff --git a/([^\n]+) b/([^\n]+)$',patch,re.MULTILINE))
        require(len(headers)==len(expected) and all(a==b and a in expected for a,b in headers) and {a for a,_ in headers}==set(expected),'/patch','patch paths differ')
        require(not re.search(r'^(?:GIT binary patch|Binary files |(?:old|new) mode |deleted file mode |(?:rename|copy) (?:from|to) |similarity index |dissimilarity index )',patch,re.MULTILINE),'/patch','unsupported patch form')
        receipt['expected']=expected;receipt['scratch_before']=_scratch(spec.scratch)
        anchors=_files(home);receipt['anchors']=dict(anchors);receipt['environment']=dict(env)
        for label,args,code in (('discovery',('rev-parse','--is-inside-work-tree'),128),
                                ('check',('apply','--check',str(home/'overlay.patch')),0),
                                ('apply',('apply',str(home/'overlay.patch')),0)):
            before=_scratch(spec.scratch)
            process=ProcessSpec((str(spec.git),*args),spec.scratch,spec.git,spec.git_sha256,home,
                                tuple(anchors.items()),spec.output/label,env)
            result=execute(process);processes.update(inventory(spec.output,[label]))
            require(result.native_exit == code,label,'unexpected Git exit')
            if label=='discovery':
                _process(process,processes[label+'/process.json'],code)
                require((result.receipt.parent/'stdout.log').read_bytes()==b'' and
                        'not a git repository' in (result.receipt.parent/'stderr.log').read_text(),label,'repository discovery not rejected')
            else:require(result.status=='PASS',label,'Git process failed integrity checks')
            if label!='apply':require(_scratch(spec.scratch)==before,label,'read-only phase changed scratch')
        require(_scratch(spec.scratch)==expected,'/transform','patch does not produce exact postimages/modes')
        receipt['status']='PASS';receipt['transformation']='PATCH_TRANSFORMATION_MATCH'
    except (OSError,ValueError) as error:errors.append(str(error))
    finally:
        for label,check in (('source',lambda: require(source is not None and freeze_source(spec.root,spec.origin_sha)==source,'/source','product changed')),
                            ('patch',lambda: require(file_hash(spec.overlay.patch)==spec.overlay.patch_sha256 and file_hash(spec.overlay.manifest)==spec.overlay.manifest_sha256,'/overlay','evidence changed')),
                            ('git',lambda: require(file_hash(spec.git)==spec.git_sha256,'/git','Git changed')),
                            ('anchors',lambda: require(not anchors or _files(spec.output/'anchors')==anchors,'/anchors','immutable anchors changed')),
                            ('processes',lambda: require(all(file_hash(spec.output/name)==value for name,value in processes.items()),'/processes','earlier process changed'))):
            try:check()
            except (OSError,ValueError) as error:errors.append(label+': '+str(error))
        try:receipt['scratch_end']=_scratch(spec.scratch)
        except (OSError,ValueError) as error:errors.append('scratch: '+str(error))
        receipt['errors']=list(errors);receipt['process_files']=dict(processes)
        if errors:receipt['status']='FAIL'
        terminal=spec.output/'transform.json';atomic_json(terminal,receipt)
    return terminal


def _process(spec: ProcessSpec, digest: str, exit_code: int = 0) -> None:
    path=spec.output_root/'process.json';require(file_hash(path)==digest,'/process','receipt digest changed')
    node=document(path.read_text());start=document((spec.output_root/'process-start.json').read_text())
    spawn=document((spec.output_root/'process-spawn.json').read_text())
    expected: dict[str,Json]={'schema_version':1,'status':'PASS' if exit_code==0 else 'FAIL','native_exit':exit_code,'argv':list(spec.argv),'cwd':str(spec.cwd),
        'executable':str(spec.executable),'expected_executable_sha256':spec.executable_sha256,'input_root':str(spec.input_root),
        'output_root':str(spec.output_root),'environment':dict(environment_for(spec)),'expected_inputs':dict(spec.inputs),
        'before_inputs':dict(spec.inputs),'after_inputs':dict(spec.inputs),'before_executable_sha256':spec.executable_sha256,
        'after_executable_sha256':spec.executable_sha256,'errors':[] if exit_code==0 else ['native exit '+str(exit_code)],'fresh_directories':[],
        'environment_policy':'fixed_defaults_plus_explicit_only_no_inheritance','build_provenance_verified':False}
    for key,value in expected.items():node.field(key).equals(value)
    require(not {'timeout_seconds','timed_out'} & node.mapping().keys(),'/process','unexpected timeout')
    for key in ('expected_external_inputs','before_external_inputs','after_external_inputs'):node.field(key).equals({})
    require(integer(node.field('pid'))>0,'/pid','missing actual PID');spawn.field('pid').equals(node.field('pid').value)
    require(integer(node.field('duration_ns'))>0,'/duration','completed process needs positive elapsed time')
    moments=tuple(datetime.fromisoformat(x) for x in (node.field('started_at').text(),spawn.field('spawned_at').text(),node.field('ended_at').text()))
    require(all(x.utcoffset() is not None for x in moments) and moments[0]<=moments[1]<=moments[2],'/time','bad process chronology')
    for key in ('argv','cwd','executable','expected_executable_sha256','input_root','output_root','environment',
                'expected_inputs','before_inputs','before_executable_sha256','started_at'):
        start.field(key).equals(node.field(key).value)
    start.field('pid').equals(None);start.field('native_exit').equals(None);start.field('status').equals('FAIL')
    start.field('environment_policy').equals('fixed_defaults_plus_explicit_only_no_inheritance')
    for stream in ('stdout','stderr'):node.field(stream+'_sha256').equals(file_hash(spec.output_root/(stream+'.log')))
    require(file_hash(spec.executable)==spec.executable_sha256,'/tool','executable changed')


@boundary
def authenticate_build(spec: BuildEvidenceSpec) -> BuildEvidence:
    """Validate driver-pinned evidence consistency; no receipt grants execution authority."""
    build=spec.build;_frozen(build.output,spec.build_files)
    node=document((build.output/'build.json').read_text())
    require(build.profile=='debug' and build.packages==('sim-bridge',) and not build.features and
            build.artifacts==(Artifact('sim_bridge','debug/libsim_bridge.dylib'),) and not build.test_targets and not build.workspace_tests,
            '/build','not closed debug bridge recipe')
    for key,value in (('root',str(build.root)),('source_sha',build.source_sha),('profile','debug'),('source_kind','DERIVED_OBSERVER')):node.field(key).equals(value)
    node.field('packages').equals(['sim-bridge']);node.field('features').equals([])
    if node.field('status').text()=='FAIL':
        errors=tuple(node.field(name).text() for name in ('error','postflight_error') if name in node.mapping())
        _frozen(build.output,spec.build_files)
        return BuildEvidence('BUILD_FAILED',errors or ('failed build lacks primary diagnostic',))
    node.field('status').equals('PASS');node.field('exit_code').equals(0)
    node.field('schema_version').equals(1);node.field('publication_authority').equals(False)
    node.field('observer_semantic_qualification').equals('NOT_RUN')
    require(build.overlay is not None,'/overlay','missing overlay')
    if build.overlay is None:raise GateContractError('MISSING_OVERLAY','/overlay','missing overlay')
    transform=spec.transform
    require((transform.root,transform.origin_sha,transform.overlay)==(build.root,build.source_sha,build.overlay),'/transform','foreign source')
    _frozen(transform.output,spec.transformation_files)
    proof=document((transform.output/'transform.json').read_text());proof.field('status').equals('PASS')
    proof.field('transformation').equals('PATCH_TRANSFORMATION_MATCH');proof.field('historical_product_application').equals('NOT_VERIFIED')
    proof.field('root').equals(str(build.root));proof.field('origin_sha').equals(build.source_sha)
    proof.field('git_sha256').equals(transform.git_sha256);require(file_hash(transform.git)==transform.git_sha256,'/git','Git changed')
    proof.field('scratch_end').equals(proof.field('expected').value)
    require(file_hash(transform.output/'anchors/overlay.patch')==build.overlay.patch_sha256 and
            file_hash(transform.output/'anchors/overlay.json')==build.overlay.manifest_sha256,'/transform','different overlay')
    source=authenticate_derived_source(DerivedSourceSpec(build.root,build.source_sha,build.overlay))
    before=node.field('source_before');before.field('source_sha').equals(build.source_sha);before.field('root').equals(str(build.root))
    require(manifest_hash(before.field('tracked_manifest').mapping())==source.origin_manifest_sha256,'/origin','origin manifest differs')
    proof.field('source_before').equals(before.value)
    expected: dict[str,Json]={}
    for name,old,new in source.changed:
        mode=before.field('tracked_manifest').field(name).array()[0].text() if old is not None else '100644'
        expected[name]=[mode,new]
    proof.field('expected').equals(expected)
    anchors=tuple((name,Node(value).text()) for name,value in proof.field('anchors').mapping().items())
    _frozen(transform.output/'anchors',anchors)
    proof.field('environment').equals(dict(_git_environment(transform)))
    for label,args,code in (('discovery',('rev-parse','--is-inside-work-tree'),128),
                            ('check',('apply','--check',str(transform.output/'anchors/overlay.patch')),0),
                            ('apply',('apply',str(transform.output/'anchors/overlay.patch')),0)):
        path=transform.output/label
        recorded=proof.field('process_files').field(label+'/process.json').text()
        _process(ProcessSpec((str(transform.git),*args),transform.scratch,transform.git,transform.git_sha256,
                            transform.output/'anchors',anchors,path,_git_environment(transform)),recorded,code)
    require((transform.output/'discovery/stdout.log').read_bytes()==b'' and
            'not a git repository' in (transform.output/'discovery/stderr.log').read_text(),'/transform','discovery check differs')
    node.field('effective_source_before').equals(node.field('effective_source_after').value)
    node.field('effective_source_before').field('sha256').equals(source.effective_source_sha256)
    require(manifest_hash(node.field('effective_source_before').field('manifest').mapping())==source.effective_source_sha256,'/source','effective manifest differs')
    node.field('overlay').equals({'patch':str(build.overlay.patch),'patch_sha256':build.overlay.patch_sha256,
        'manifest':str(build.overlay.manifest),'manifest_sha256':build.overlay.manifest_sha256})
    inputs=inventory(build.root,[*INPUT_DIRS,*INPUT_FILES]);node.field('effective_inputs_before').equals(dict(inputs))
    node.field('cargo_configs').equals(dict(config_inventory(build)))
    node.field('cargo_sha256').equals(build.cargo_sha256);node.field('rustc_sha256').equals(build.rustc_sha256)
    target=build.root/'rust/target';node.field('target_dir').equals(str(target))
    env=(('PATH',f'{build.rustc.parent}:/usr/bin:/bin:/usr/sbin:/sbin'),('CARGO_HOME',str(build.cargo_home)),
         ('CARGO_TARGET_DIR',str(target)),('RUSTC',str(build.rustc)),('RUSTC_WRAPPER',''),('RUSTC_WORKSPACE_WRAPPER',''))
    node.field('environment').equals(dict(env))
    for name,tool,digest in (('cargo',build.cargo,build.cargo_sha256),('rustc',build.rustc,build.rustc_sha256)):
        row=node.field('toolchain_versions').field(name);path=build.output/(name+'-version')
        row.field('process_receipt').equals(str(path/'process.json'))
        _process(ProcessSpec((str(tool),'--version','--verbose'),build.root/'rust',tool,digest,build.root,tuple(inputs.items()),path,env),row.field('process_receipt_sha256').text())
        text=(path/'stdout.log').read_text();row.field('text').equals(text)
        channel=document(json.dumps(tomllib.loads((build.root/'rust/rust-toolchain.toml').read_text()))).field('toolchain').field('channel').text()
        releases=tuple(match[1] for match in re.finditer(r'^release: ([0-9]+\.[0-9]+\.[0-9]+)$',text,re.MULTILINE))
        require(re.fullmatch(r'[0-9]+\.[0-9]+(?:\.[0-9]+)?',channel) is not None and len(releases)==1
                and (releases[0]==channel or releases[0].startswith(channel+'.')),'/version','toolchain version differs')
    argv=(str(build.cargo),'build','--locked','--message-format=json-render-diagnostics','-p','sim-bridge')
    node.field('argv').equals(list(argv));node.field('process_receipt').equals(str(build.output/'cargo-build/process.json'))
    _process(ProcessSpec(argv,build.root/'rust',build.cargo,build.cargo_sha256,build.root,tuple(inputs.items()),build.output/'cargo-build',env),node.field('process_receipt_sha256').text())
    rows=[document(line) for line in (build.output/'cargo-build/stdout.log').read_text().splitlines()]
    terminal=[row for row in rows if row.field('reason').value=='build-finished']
    require(len(terminal)==1 and rows[-1] is terminal[0],'/compiler','missing/duplicate/nonfinal terminal');terminal[0].field('success').equals(True)
    artifact=target/'debug/libsim_bridge.dylib';digest=file_hash(artifact)
    matches=[row for row in rows if row.field('reason').value=='compiler-artifact' and row.field('target').field('name').value=='sim_bridge']
    require(len(matches)==1,'/compiler','missing/duplicate selected target');matches[0].field('fresh').equals(False)
    crate=build.root/'rust/crates/sim-bridge';manifest=crate/'Cargo.toml'
    package=document(json.dumps(tomllib.loads(manifest.read_text())))
    package.field('package').field('name').equals('sim-bridge')
    package.field('package').field('version').field('workspace').equals(True)
    workspace=document(json.dumps(tomllib.loads((build.root/'rust/Cargo.toml').read_text())))
    version=workspace.field('workspace').field('package').field('version').text()
    library=package.field('lib');kinds=tuple(item.text() for item in library.field('crate-type').array())
    require('cdylib' in kinds and len(set(kinds))==len(kinds) and set(kinds)<={'cdylib','rlib'},'/target','not admitted library crate types')
    library_path=Node(library.mapping().get('path','src/lib.rs')).text()
    src=safe_path(crate,library_path)
    Node(library.mapping().get('name','sim_bridge')).equals('sim_bridge')
    matches[0].field('package_id').equals('path+'+crate.as_uri()+'#'+version)
    matches[0].field('manifest_path').equals(str(manifest))
    matches[0].field('target').field('kind').equals(list(kinds))
    matches[0].field('target').field('crate_types').equals(list(kinds))
    matches[0].field('target').field('src_path').equals(str(src))
    require(str(artifact) in [value.text() for value in matches[0].field('filenames').array()],'/compiler','artifact path differs')
    node.field('artifacts').equals({'sim_bridge':{'path':str(artifact),'sha256':digest}});node.field('binary_sha256').equals(digest)
    side=document((build.output/'artifact-sim_bridge.json').read_text())
    for key,value in {'source_sha':build.source_sha,'binary_sha256':digest,'binary':str(artifact),'profile':'debug','exit_code':0,
                      'source_kind':'DERIVED_OBSERVER','build_receipt':str(build.output/'build.json'),'build_receipt_sha256':file_hash(build.output/'build.json')}.items():side.field(key).equals(value)
    descriptor=(build.root/'sim_bridge.gdextension').read_text()
    sections=tuple(match[1] for match in re.finditer(r'^\[libraries\]\s*\n([^\[]*)',descriptor,re.MULTILINE))
    require(len(sections)==1,'/descriptor','missing or duplicate libraries section')
    debug=tuple((match[1],match[2]) for match in re.finditer(r'^(macos\.debug[^=\s]*)\s*=\s*"([^"]+)"\s*$',sections[0],re.MULTILINE))
    require(debug==(('macos.debug','res://rust/target/debug/libsim_bridge.dylib'),),'/descriptor','debug bridge resolution differs')
    require(config_inventory(build)==dict(build.config_inputs) and file_hash(build.cargo)==build.cargo_sha256
            and file_hash(build.rustc)==build.rustc_sha256 and file_hash(transform.git)==transform.git_sha256,'/tools','late tool/config drift')
    _=recheck_derived_source(source);_frozen(transform.output,spec.transformation_files);_frozen(build.output,spec.build_files)
    require(inventory(build.root,[*INPUT_DIRS,*INPUT_FILES])==inputs and file_hash(artifact)==digest,'/inputs','late source/artifact drift')
    return BuildEvidence('CONSISTENT_SUCCESS',(),(artifact,digest),source.effective_source_sha256)
