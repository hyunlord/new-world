"""Closed staged setup-prefix selectors; runtime construction is source-owned."""
import hashlib
import json
import os
import re
import shlex
import stat
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import generic_cli_prepare as cli_prepare
import generic_cli_products as cli_products
import generic_cli_setup_handoff as cli_handoff
import generic_setup_bridge as setup_bridge
import generic_setup_clippy as setup_clippy
import generic_setup_prepare as setup_prepare
import generic_static_run as static
import generic_workspace_arguments as workspace_arguments
import generic_workspace_policy as workspace_policy
import generic_workspace_run as workspace_run
from generic_build import Overlay, WorkspaceSourceProof, recheck_workspace_source
from generic_cli_prepare import ArmSelector, CliSelectors
from generic_gate import document, integer, require
from generic_process import (
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
    real_path,
)
from generic_setup_prepare import TOOLS, Tool
from generic_sources import freeze_source
from generic_static_run import ExecutionEnvironment
from generic_workspace_cases import ListingProcess
from generic_workspace_endpoints import (
    EvidenceAnchor,
    WorkspaceChain,
    WorkspaceRequest,
    anchor,
    encoded,
    generated,
)
from generic_workspace_tests import Inventory, inspect_inventory
from mux2_gate_inputs import Json, Node


@dataclass(frozen=True, slots=True)
class SetupSelectors:
    tools: tuple[Tool, ...]
    environment: tuple[tuple[str,str], ...]
    lock_origin: Path
    lock_origin_sha256: str
    cargo_wait_sources: tuple[Path, ...]
    clippy_wait_sources: tuple[Path, ...]


@dataclass(frozen=True, slots=True)
class PrefixSpec:
    output: Path
    control_fd: int
    selectors: CliSelectors
    setup: SetupSelectors
    static: ExecutionEnvironment
    strict: ExecutionEnvironment
    rustdoc: Tool


def fields(node: Node, expected: set[str]) -> None:
    require(set(node.mapping())==expected,node.label,'closed schema keys differ')


def path(node: Node) -> Path:
    value=Path(node.text())
    require(value.is_absolute() and '..' not in value.parts,node.label,'absolute normalized path required')
    return value


def digest(node: Node, size: int=64) -> str:
    value=node.text()
    require(len(value)==size and all(c in '0123456789abcdef' for c in value),node.label,'invalid digest')
    return value


def pairs(node: Node) -> tuple[tuple[str,str], ...]:
    result: list[tuple[str,str]]=[]
    for row in node.array():
        values=row.array();require(len(values)==2,row.label,'exact pair required')
        result.append((values[0].text(),values[1].text()))
    require(len(dict(result))==len(result),node.label,'duplicate key')
    return tuple(result)


def tool(node: Node) -> Tool:
    fields(node,{'name','path','sha256'})
    return Tool(node.field('name').text(),path(node.field('path')),digest(node.field('sha256')))


def external(node: Node) -> tuple[tuple[Path,str], ...]:
    result: list[tuple[Path,str]]=[]
    for row in node.array():
        fields(row,{'path','sha256'})
        result.append((path(row.field('path')),digest(row.field('sha256'))))
    require(len(dict(result))==len(result),node.label,'duplicate external path')
    return tuple(result)


def arm(node: Node) -> ArmSelector:
    fields(node,{'root','source_sha','cargo_home','configs','environment','overlay'})
    patch=node.field('overlay');overlay: Overlay | None=None
    if patch.value is not None:
        fields(patch,{'patch','patch_sha256','manifest','manifest_sha256'})
        overlay=Overlay(path(patch.field('patch')),digest(patch.field('patch_sha256')),
                        path(patch.field('manifest')),digest(patch.field('manifest_sha256')))
    return ArmSelector(path(node.field('root')),digest(node.field('source_sha'),40),
                       path(node.field('cargo_home')),tuple((str(p),d) for p,d in external(node.field('configs'))),
                       pairs(node.field('environment')),overlay)


def environment(node: Node) -> ExecutionEnvironment:
    fields(node,{'values','tools','external_inputs'})
    tools=tuple(tool(row) for row in node.field('tools').array())
    require(len({row.name for row in tools})==len(tools),node.label,'duplicate tool')
    return ExecutionEnvironment(pairs(node.field('values')),tuple((t.name,t.path,t.sha256) for t in tools),
                                external(node.field('external_inputs')))


def parse_spec(node: Node) -> PrefixSpec:
    fields(node,{'schema','output','control_fd','selectors','setup','static','strict','rustdoc'})
    node.field('schema').equals('cli_setup_prefix.v1')
    output=path(node.field('output'));control=integer(node.field('control_fd'))
    require(control>=0,'/control_fd','actual controller descriptor required')
    selected=node.field('selectors')
    fields(selected,{'harness_root','harness_sha','published_sha','purpose','base','candidate','cargo','rustc'})
    purpose=selected.field('purpose').text()
    require(purpose in {'publication','self_proof','defect_proof'},'/purpose','unsupported purpose')
    closed_purpose: Literal['publication','self_proof','defect_proof']
    match purpose:
        case 'publication':closed_purpose='publication'
        case 'self_proof':closed_purpose='self_proof'
        case 'defect_proof':closed_purpose='defect_proof'
        case _:raise AssertionError('validated purpose differs')
    cargo=tool(selected.field('cargo'));rustc=tool(selected.field('rustc'))
    require(cargo.name=='cargo' and rustc.name=='rustc','/tools','wrong selected compiler tool')
    selectors=CliSelectors(path(selected.field('harness_root')),digest(selected.field('harness_sha'),40),
        digest(selected.field('published_sha'),40),closed_purpose,arm(selected.field('base')),arm(selected.field('candidate')),
        cargo.path,cargo.sha256,rustc.path,rustc.sha256,output/'antecedents')
    setup=node.field('setup')
    fields(setup,{'tools','environment','lock_origin','lock_origin_sha256','cargo_wait_sources','clippy_wait_sources'})
    tools=tuple(tool(row) for row in setup.field('tools').array())
    require(len(tools)==len(TOOLS) and frozenset(t.name for t in tools)==TOOLS,'/setup/tools','exact setup tools required')
    require(next(t for t in tools if t.name=='cargo')==cargo and next(t for t in tools if t.name=='rustc')==rustc,
            '/setup/tools','compiler selection differs')
    prepared=SetupSelectors(tools,pairs(setup.field('environment')),path(setup.field('lock_origin')),
        digest(setup.field('lock_origin_sha256')),tuple(path(row) for row in setup.field('cargo_wait_sources').array()),
        tuple(path(row) for row in setup.field('clippy_wait_sources').array()))
    rustdoc=tool(node.field('rustdoc'));require(rustdoc.name=='rustdoc','/rustdoc','wrong tool name')
    return PrefixSpec(output,control,selectors,prepared,environment(node.field('static')),environment(node.field('strict')),rustdoc)


@dataclass(frozen=True, slots=True)
class PrefixPrepared:
    spec: PrefixSpec
    products: cli_products.CliProducts
    handoff: cli_handoff.SetupProductHandoff
    wait: setup_bridge.CargoWaitEvidence
    clippy_wait: setup_clippy.ClippyWaitEvidence
    static_product: static.StaticProduct
    policies: tuple[workspace_policy.PolicyTemplate,workspace_policy.PolicyTemplate]
    rustdoc_version: ListingProcess
    temporary_root: Path
    version_anchor: EvidenceAnchor
    temporary_identity: tuple[int,int,int,int]


@dataclass(slots=True)
class InitializationOwner:
    """Retain actual build/spec and launch boundary until each owned call returns."""
    in_flight: bool=False
    prepared: cli_prepare.PreparedCli | None=None
    builds: cli_prepare.CliBuilds | None=None
    process: ProcessSpec | None=None


def setup_input_files(manifest: Node, build_inputs: tuple[tuple[str,str], ...]) -> tuple[tuple[str,str], ...]:
    """Project already-authenticated source hashes; this confers no source authority."""
    result: dict[str,str]={}
    for name,row in manifest.mapping().items():
        values=Node(row).array()
        require(len(values)==2,'/setup/inputs','effective manifest entry shape differs')
        result[name]=digest(values[1])
    require(len(dict(build_inputs))==len(build_inputs),'/setup/inputs','duplicate build input')
    for name,value in build_inputs:
        require(name not in result or result[name]==value,'/setup/inputs','source/build input hash conflict')
        result[name]=value
    return tuple(sorted(result.items()))


def setup_source_inputs(proof: WorkspaceSourceProof) -> tuple[tuple[str,str], ...]:
    recheck_workspace_source(proof)
    result=setup_input_files(document(proof.effective_json).field('manifest'),proof.effective_inputs)
    recheck_workspace_source(proof)
    return result


def initialize(spec: PrefixSpec, owner: InitializationOwner) -> PrefixPrepared:
    """Execute real build antecedents and retain the pre-step2 warm handoff."""
    require(spec.output.is_absolute() and spec.output==spec.output.resolve()
            and not spec.output.exists() and not spec.output.is_symlink(),'/output','fresh canonical output required')
    require(spec.control_fd>2 and stat.S_ISFIFO(os.fstat(spec.control_fd).st_mode),'/control','live dedicated controller pipe required')
    for root in (spec.selectors.harness_root,spec.selectors.base.root,spec.selectors.candidate.root,
                 spec.selectors.base.cargo_home,spec.selectors.candidate.cargo_home):
        require(not spec.output.is_relative_to(root) and not root.is_relative_to(spec.output),'/output','overlapping selector root')
    spec.output.mkdir()
    owner.prepared=cli_prepare.prepare_cli(spec.selectors)
    owner.in_flight=True
    builds=cli_prepare.build_cli(owner.prepared)
    owner.builds=builds;owner.in_flight=False
    require(builds.status=='BUILD_ANTECEDENTS_ONLY' and builds.candidate is not None,'/build','actual two-arm build antecedents required')
    assert builds.candidate is not None
    products=cli_products.construct_products(builds,spec.output/'products')
    python=next(t.path for t in spec.setup.tools if t.name=='python3')
    helper=spec.selectors.harness_root/'tools/v8/generic_cargo_child.py'
    policies=(workspace_policy.build_policy(builds.candidate,'debug',(python,helper)),
              workspace_policy.build_policy(builds.candidate,'release',(python,helper)))
    setup=spec.setup;candidate=spec.selectors.candidate
    inputs=setup_prepare.SetupInputs(candidate.root,candidate.source_sha,candidate.cargo_home,
        setup.tools,setup.environment,setup_source_inputs(products.sources[1]),candidate.configs,
        setup.lock_origin,setup.lock_origin_sha256,spec.output/'settings',spec.output/'bridge')
    prepared=setup_prepare.prepare_warm(inputs,builds.candidate)
    handoff=cli_handoff.begin_setup_handoff(products,prepared)
    versions: list[ListingProcess]=[]
    for name in ('cargo','rustc'):
        selected=next(t for t in setup.tools if t.name==name)
        process=ProcessSpec((str(selected.path),'--version','--verbose'),candidate.root,
            selected.path,selected.sha256,candidate.root,prepared.materialized,spec.output/(name+'-version'),setup.environment,
            external_inputs=prepared.external)
        owner.process=process;owner.in_flight=True
        result=execute(process)
        owner.in_flight=False
        require(result.status=='PASS' and result.native_exit==0,'/version','actual tool version invocation failed')
        versions.append(ListingProcess(process,result,file_hash(result.receipt)))
    wait=setup_bridge.CargoWaitEvidence(setup.cargo_wait_sources,versions[0],versions[1])
    clippy=setup_clippy.ClippyWaitEvidence(wait,setup.clippy_wait_sources)
    _=setup_bridge.qualify_wait(prepared,wait)
    _=setup_clippy.qualify_clippy(prepared,clippy)
    product=static.StaticProduct(candidate.root,candidate.source_sha,products.probes.candidate.input_files)
    version_spec=ProcessSpec((str(spec.rustdoc.path),'--version','--verbose'),candidate.root,
        spec.rustdoc.path,spec.rustdoc.sha256,candidate.root,prepared.materialized,spec.output/'rustdoc-version',setup.environment,
        external_inputs=prepared.external)
    owner.process=version_spec;owner.in_flight=True
    version_result=execute(version_spec)
    owner.in_flight=False
    require(version_result.status=='PASS' and version_result.native_exit==0,'/rustdoc','version process failed')
    version_anchor=anchor(version_spec.output_root)
    version=ListingProcess(version_spec,version_result,file_hash(version_result.receipt))
    raw=(version_result.receipt.parent/'stdout.log').read_text().splitlines()
    require(raw.count('commit-hash: '+setup_bridge.COMMITS['rustc'])==1
            and raw.count('release: 1.93.1')==1 and raw.count('host: aarch64-apple-darwin')==1,
            '/rustdoc','unsupported actual Rustdoc version')
    temporary=_temporary_root(setup.environment)
    temp_stat=temporary.stat();temporary_identity=(temp_stat.st_dev,temp_stat.st_ino,temp_stat.st_mode,temp_stat.st_uid)
    cli_products.recheck_products(products)
    for policy in policies:_=workspace_policy.recheck_policy(policy)
    version_anchor.recheck()
    return PrefixPrepared(spec,products,handoff,wait,clippy,product,policies,version,temporary,version_anchor,temporary_identity)


@dataclass(frozen=True, slots=True)
class PrefixDocument:
    path: Path
    sha256: str
    spec: PrefixSpec


def read_spec(source: Path) -> PrefixDocument:
    real_path(source,directory=False)
    digest=file_hash(source)
    selected=parse_spec(document(source.read_text(encoding='utf-8')))
    require(file_hash(source)==digest,'/spec','spec changed during parsing')
    require(not source.is_relative_to(selected.output),'/spec','spec must remain outside fresh output')
    return PrefixDocument(source,digest,selected)


def check_spec(bound: PrefixDocument) -> None:
    real_path(bound.path,directory=False)
    require(file_hash(bound.path)==bound.sha256,'/spec','immutable selector document changed')


def _temporary_root(environment: tuple[tuple[str,str], ...]) -> Path:
    """Pinned Darwin success branch only; no authority over existing temp members."""
    require(sys.platform=='darwin' and 'TMPDIR' not in dict(environment),'/temporary','unsupported temporary-root branch')
    raw=os.confstr(65537)
    require(raw is not None and bool(raw),'/temporary','Darwin query failed; fallback unsupported')
    assert raw is not None
    lexical=Path(raw)
    require(lexical.is_absolute() and '..' not in lexical.parts,'/temporary','invalid OS root')
    for parent in (*reversed(lexical.parents),lexical):
        if parent.is_symlink():
            require(parent==Path('/var') and os.readlink(parent)=='private/var','/temporary','unsupported root alias')
    canonical=lexical.resolve(strict=True);real_path(canonical,directory=True)
    require(canonical.stat().st_uid==os.getuid(),'/temporary','OS root owner differs')
    return canonical


def _harness_files(bound: PrefixPrepared) -> tuple[tuple[Path,str], ...]:
    selected=bound.spec.selectors
    frozen=freeze_source(selected.harness_root,selected.harness_sha)
    require(json.dumps(frozen,sort_keys=True)==bound.products.builds.prepared.harness_json,
            '/harness','original source closure changed')
    tracked=document(json.dumps(frozen)).field('tracked_manifest')
    files=tuple((selected.harness_root/name,tracked.field(name).array()[2].text())
                for name in sorted(tracked.mapping()) if name.startswith('tools/v8/') and name.endswith('.py'))
    for module in (workspace_policy,workspace_arguments):
        module_path=module.__file__
        require(module_path is not None,'/harness','loaded helper has no source path')
        assert module_path is not None
        source=Path(module_path).resolve()
        require(dict(files).get(source)==file_hash(source),
                '/harness','loaded policy/argument module outside authenticated helper closure')
    return files


def _profile_evidence(bound: PrefixPrepared, step: Literal[4,5]) -> tuple[Inventory,tuple[Node,...],Path]:
    pair=bound.products.builds.candidate
    assert pair is not None
    result=pair.debug if step==4 else pair.release
    require(result is not None and result.status=='PASS','/inventory','actual profile build required')
    assert result is not None
    metadata=result.receipt.parent/'cargo-metadata/stdout.log'
    stream=result.receipt.parent/'cargo-build/stdout.log'
    policy=bound.policies[0 if step==4 else 1]
    inputs=dict(policy.inputs)
    require(inputs.get(metadata)==file_hash(metadata) and inputs.get(stream)==file_hash(stream),
            '/inventory','compiler evidence outside retained policy')
    messages=tuple(document(line) for line in stream.read_text().splitlines())
    return inspect_inventory(document(metadata.read_text()),messages,step),messages,result.receipt


def process_bindings(root: Path, values: dict[str,str]) -> tuple[tuple[tuple[str,str], ...],tuple[tuple[Path,str], ...]]:
    require(root.is_absolute() and root==root.resolve() and '..' not in root.parts,
            '/inputs','noncanonical process input root')
    materialized: list[tuple[str,str]]=[];external: list[tuple[Path,str]]=[]
    for name,checksum in values.items():
        path=Path(name)
        require(path.is_absolute() and str(path)==name and path==path.resolve() and '..' not in path.parts,
                '/inputs','aliased process input path')
        if path.is_relative_to(root):materialized.append((str(path.relative_to(root)),checksum))
        else:external.append((path,checksum))
    return tuple(sorted(materialized)),tuple(sorted(external))


def retained_normal_output(product: Path, profile: Literal['debug','release'], path: Path, target: Node) -> str:
    require(path in {product/'rust/target'/profile/name for name in BINS},
            '/compiled','unbound classified output is not a closed normal target')
    relative=str(path.relative_to(product/'rust/target'))
    value=target.field(relative).text().split(':')
    require(len(value)==3 and value[0]=='file' and value[1].isdigit(),
            '/compiled','retained target entry is not a regular file')
    mode=int(value[1]);checksum=digest(Node(value[2]))
    require(stat.S_ISREG(mode) and path.lstat().st_mode==mode and file_hash(path)==checksum,
            '/compiled','original target mode/hash differs')
    return checksum


def workspace_request(bound: PrefixPrepared, step: Literal[4,5], chain: WorkspaceChain,
                      output: Path, preparation: Path, descriptor: int) -> WorkspaceRequest:
    """Construct exact closed workspace request from actual pair and predecessor."""
    require(chain.prepared is bound.handoff.prepared,'/factory','foreign prepared chain')
    prepared=chain.prepared;before=generated(prepared)
    template=bound.policies[0 if step==4 else 1]
    policy=workspace_policy.bind_policy(template,chain,step).mapping()
    helpers=_harness_files(bound)
    require(preparation.is_dir() and not tuple(preparation.iterdir()) and preparation==preparation.resolve(),
            '/factory','empty canonical preparation root required')
    require(not preparation.is_relative_to(prepared.inputs.root) and not output.is_relative_to(prepared.inputs.root),
            '/factory','factory output overlaps product')
    python=next(t for t in prepared.inputs.tools if t.name=='python3')
    cargo=next(t for t in prepared.inputs.tools if t.name=='cargo')
    helper=bound.spec.selectors.harness_root/'tools/v8/generic_cargo_child.py'
    rustdoc_helper=helper.with_name('gate_rustdoc_exec.py')
    require(dict(helpers).get(helper)==file_hash(helper) and dict(helpers).get(rustdoc_helper)==file_hash(rustdoc_helper),
            '/helper','unbound child helpers')
    shell=next((row for row in bound.spec.static.tools if row[0]=='sh'),None)
    require(shell is not None and shell[1]==Path('/bin/sh').resolve() and file_hash(shell[1])==shell[2],
            '/shell','fixed wrapper interpreter must be an explicit tool binding')
    assert shell is not None
    entrypoint=preparation/'rustdoc-wrapper'
    wrapper='#!/bin/sh\nexec '+shlex.quote(str(python.path))+' '+shlex.quote(str(rustdoc_helper))+' "$@"\n'
    initial=anchor(preparation)
    expected_wrapper: dict[str,Json]={**document(initial.state).mapping(),entrypoint.name:{'mode':stat.S_IFREG|0o755,
        'sha256':hashlib.sha256(wrapper.encode()).hexdigest()}}
    with entrypoint.open('x',encoding='utf-8') as handle:_=handle.write(wrapper)
    entrypoint.chmod(0o755)
    require(entrypoint.read_text()==wrapper,'/wrapper','generated wrapper differs')
    prepared_anchor=EvidenceAnchor(preparation,encoded(expected_wrapper));prepared_anchor.recheck()
    native,_messages,build_receipt=_profile_evidence(bound,step)
    pair=bound.products.builds.candidate
    assert pair is not None
    result=pair.debug if step==4 else pair.release
    assert result is not None
    selected={item.binary for item in native.executables}
    artifacts=[{'id':name,'path':str(path),'sha256':checksum} for name,path,checksum in result.artifacts if path in selected]
    require({Path(row['path']) for row in artifacts}==selected,'/artifacts','missing selected actual libtest artifacts')
    artifact_map={Path(row['path']):(row['id'],row['sha256']) for row in artifacts}
    bindings=(*template.inputs,*helpers,*prepared.external,
        *((prepared.inputs.root/name,checksum) for name,checksum in prepared.materialized))
    inputs: dict[str,str]={}
    for path,checksum in bindings:
        require(str(path) not in inputs or inputs[str(path)]==checksum,'/inputs','conflicting bound source/input hash')
        inputs[str(path)]=checksum
    inputs.update({str(prepared.inputs.root/name):checksum for name,checksum in prepared.materialized})
    inputs.update({str(shell[1]):shell[2],str(entrypoint):file_hash(entrypoint),str(bound.spec.rustdoc.path):bound.spec.rustdoc.sha256})
    version=bound.rustdoc_version
    require(file_hash(version.result.receipt)==version.receipt_sha256,'/version','version evidence changed')
    bound.version_anchor.recheck()
    for path in version.spec.output_root.iterdir():
        real_path(path,directory=False);inputs[str(path)]=file_hash(path)
    for path in selected:_=inputs.pop(str(path),None)
    compiled: list[Json]=[];searches: list[Json]=[];references: set[str]=set()
    deps=prepared.inputs.root/'rust/target'/template.profile/'deps'
    states=[document(state) for path,state in template.trees if path==deps]
    require(len(states)==1,'/dependencies','missing original dependency tree')
    state=states[0];members=state.field('members').array()
    directories=[row.array()[0].text() for row in members if stat.S_ISDIR(integer(row.array()[1]))]
    modes={row.array()[0].text():integer(row.array()[1]) for row in members}
    members={name:state.field('files').field(name).text() for name in state.field('files').mapping()}
    compiled_bound=classify_compiled(prepared.inputs.root,prepared.inputs.cargo_home,template.profile,
        build_receipt.parent/'cargo-build/stdout.log',
        {str(path):value for path,value in artifact_map.items()},members)
    compiled=list(compiled_bound.table)
    require(dict(template.inputs).get(pair.receipt)==file_hash(pair.receipt),
            '/compiled','original pair receipt is outside sealed policy inputs')
    original_target=document(pair.receipt.read_text()).field('target_after')
    for path,checksum in compiled_bound.inputs:
        if str(path) not in inputs:
            inputs[str(path)]=retained_normal_output(prepared.inputs.root,template.profile,path,original_target)
        require(inputs.get(str(path))==checksum,'/compiled','classified output outside retained build closure')
    compiled_by_path={Node(row).field('path').text():Node(row).field('id').text() for row in compiled_bound.table}
    files: list[Json]=[]
    for name in sorted(state.field('files').mapping()):
        path=deps/name;checksum=state.field('files').field(name).text()
        if path in artifact_map:
            ident,actual=artifact_map[path];require(actual==checksum,'/artifact','compiler bytes differ')
            kind='artifact';references.add(ident)
        elif modes[name]&0o111:
            require(str(path) in compiled_by_path,'/compiled','executable dependency lacks closed compiler provenance')
            ident=compiled_by_path[str(path)];kind='compiled_dependency'
        else:
            ident=str(path);kind='input'
            require(inputs.get(ident)==checksum,'/dependency','missing retained source/input member')
        files.append({'relative_path':name,'binding_kind':kind,'binding_id':ident,'sha256':checksum})
    search: dict[str,Json]={'path':str(deps),'directories':[name for name in sorted(directories)],'files':files}
    searches.append(search)
    compiled.sort(key=lambda row: Node(row).field('id').text())
    provenance=preparation/'compiled-provenance.json'
    prepared_anchor.recheck()
    prepared_anchor=publish(provenance,{'schema':'cli_compiler_dependencies.v1','build_receipt':str(build_receipt),
        'build_receipt_sha256':file_hash(build_receipt),'profile':template.profile,'compiled_dependencies':compiled,
        'compiler_provenance':compiled_bound.provenance})
    inputs[str(provenance)]=file_hash(provenance)
    manifest=preparation/'capture-manifest.json'
    env=dict(prepared.inputs.environment)
    env.update(RUSTDOC=str(entrypoint),CARGO_CAPTURE_MANIFEST=str(manifest),
               CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER=shlex.join((str(python.path),str(helper))))
    process_inputs={**inputs,**{str(path):checksum for path,(_,checksum) in artifact_map.items()}}
    source_inputs,external_bindings=process_bindings(prepared.inputs.root,process_inputs)
    externals=dict(external_bindings)
    argv=workspace_run.workspace_argv(str(cargo.path),step)
    spec=ProcessSpec(argv,prepared.inputs.root/'rust',cargo.path,cargo.sha256,prepared.inputs.root,source_inputs,
                     output/'parent',tuple(sorted(env.items())),tuple(sorted(externals.items())))
    parent: dict[str,Json]={'executable':str(spec.executable),'executable_sha256':spec.executable_sha256,
        'argv':list(spec.argv),'cwd':str(spec.cwd),'input_root':str(spec.input_root),
        'environment':dict(environment_for(spec)),'inputs':dict(spec.inputs),
        'external_inputs':{str(p):d for p,d in spec.external_inputs},'fresh_directories':list(spec.fresh_directories)}
    policy.update(parent_spec=parent,parent_receipt=str(spec.output_root/'process.json'),
        rustdoc=str(bound.spec.rustdoc.path),rustdoc_sha256=bound.spec.rustdoc.sha256,
        version_receipt=str(version.spec.output_root/'stdout.log'),version_sha256=file_hash(version.spec.output_root/'stdout.log'),
        entrypoint=str(entrypoint),entrypoint_sha256=file_hash(entrypoint),wrapper_sha256=file_hash(rustdoc_helper),
        exec_root=str(output/'rustdoc-execs'),temporary_root=str(bound.temporary_root),
        artifact_dependencies=[name for name in sorted(references)],dependency_searches=searches,compiled_dependencies=compiled,
        compiled_dependency_provenance={'path':str(provenance),'sha256':file_hash(provenance)})
    prepared_anchor.recheck()
    prepared_anchor=publish(manifest,{'schema_version':1,'invocation_id':f'cli-candidate-step{step}','step':step,'arm':'candidate',
        'profile':template.profile,'output_root':str(output/'capture'),'launcher_sha256':file_hash(helper),
        'python_sha256':python.sha256,'inputs':{key:value for key,value in inputs.items()},'artifacts':[{key:value for key,value in row.items()} for row in artifacts],'observer_variable':None,'doctest_policy':policy})
    frozen=prepared_anchor
    checksum=file_hash(manifest);env['CARGO_CAPTURE_MANIFEST_SHA256']=checksum;externals[manifest]=checksum
    spec=ProcessSpec(argv,spec.cwd,spec.executable,spec.executable_sha256,spec.input_root,spec.inputs,
                     spec.output_root,tuple(sorted(env.items())),tuple(sorted(externals.items())))
    request=WorkspaceRequest(step,'candidate',f'cli-candidate-step{step}',spec,native,manifest,checksum,output,descriptor,chain)
    _=workspace_run.checked(request)
    _=workspace_policy.bind_policy(template,chain,step)
    require(_temporary_root(prepared.inputs.environment)==bound.temporary_root,'/temporary','OS identity changed')
    current=bound.temporary_root.stat()
    require((current.st_dev,current.st_ino,current.st_mode,current.st_uid)==bound.temporary_identity,
            '/temporary','temporary root inode/mode/owner changed')
    require(generated(prepared)==before,'/factory','factory changed generated inputs')
    bound.version_anchor.recheck()
    frozen.recheck()
    return request


def publish(path: Path, payload: dict[str,Json]) -> EvidenceAnchor:
    """Bind intended receipt bytes and exact old membership before publication."""
    before=document(anchor(path.parent).state).mapping()
    require(path.name not in before,'/publication','fresh factory output required')
    raw=json.dumps(payload,indent=2,sort_keys=True,allow_nan=False)+'\n'
    expected: dict[str,Json]={**before,path.name:{'mode':stat.S_IFREG|(stat.S_IMODE(path.parent.stat().st_mode)&0o666),
                                'sha256':hashlib.sha256(raw.encode()).hexdigest()}}
    atomic_json(path,payload)
    retained=EvidenceAnchor(path.parent,encoded(expected));retained.recheck()
    return retained


# Source: /Users/rexxa/work/m6b-data/stage0/workspace-runtime/workspace-debug-qualification-v9-draft/compiled_producer.py
# SHA256: a369c664d6746df393b495c9867823d62401db38e783bef1ac0317c2459245e1
REGISTRY = {
    'godot_macros': ('godot-macros', '0.4.5'),
    'nanoserde_derive': ('nanoserde-derive', '0.2.1'),
    'serde_derive': ('serde_derive', '1.0.229'),
    'thiserror_impl': ('thiserror-impl', '1.0.69'),
    'zerocopy_derive': ('zerocopy-derive', '0.8.63'),
}
BINS = ('worldsim_perf', 'worldsim_probe', 'worldsim_systems')


@dataclass(frozen=True, slots=True)
class Compiled:
    table: tuple[dict[str, Json], ...]
    inputs: tuple[tuple[Path, str], ...]
    provenance: dict[str, Json]


def regular(path: Path, root: Path) -> None:
    require(path.is_absolute() and path.resolve(strict=True) == path and path.is_relative_to(root)
            and path.is_file() and not path.is_symlink(), str(path), 'foreign/noncanonical compiler output')


def classify_compiled(product: Path, cargo_home: Path, profile: Literal['debug','release'], stream: Path,
                      native: dict[str, tuple[str, str]], members: dict[str, str]) -> Compiled:
    """Only the reviewed five proc macros, bridge and three normal bins are admitted."""
    deps = product/'rust/target'/profile/'deps'
    require(deps.is_dir() and deps.resolve() == deps, '/compiled', 'canonical dependency root required')
    raw = stream.read_bytes(); stream_hash = hashlib.sha256(raw).hexdigest()
    selected: dict[str, tuple[int, bytes, Node, Path]] = {}
    compiler_outputs: dict[Path, str] = {}
    names = {*REGISTRY, 'sim_bridge', *BINS}
    for index, line in enumerate(raw.splitlines()):
        row = document(line.decode())
        if row.field('reason').value != 'compiler-artifact':
            continue
        name = row.field('target').field('name').text()
        if name not in names or row.field('profile').field('test').value is not False:
            continue
        row.field('fresh').equals(False)
        require(name not in selected, '/compiled', 'ambiguous non-test compiler record')
        target = row.field('target'); source = Path(target.field('src_path').text())
        filenames = tuple(Path(item.text()) for item in row.field('filenames').array())
        require(len(filenames) == len(set(filenames)), '/compiled', 'duplicate compiler filenames')
        if name in REGISTRY:
            package, version = REGISTRY[name]
            row.field('package_id').equals(f'registry+https://github.com/rust-lang/crates.io-index#{package}@{version}')
            target.field('kind').equals(['proc-macro']); target.field('crate_types').equals(['proc-macro'])
            require(source == cargo_home/'registry/src/index.crates.io-1949cf8c6b5b557f'/f'{package}-{version}'/'src/lib.rs', '/compiled', 'registry source differs')
            row.field('executable').equals(None)
            candidates = [path for path in filenames if path.parent == deps and re.fullmatch(r'lib'+name+r'-[0-9a-f]{16}\.dylib', path.name)]
        elif name == 'sim_bridge':
            row.field('package_id').equals(f'path+file://{product}/rust/crates/sim-bridge#0.7.0')
            target.field('kind').equals(['cdylib', 'rlib']); target.field('crate_types').equals(['cdylib', 'rlib'])
            require(source == product/'rust/crates/sim-bridge/src/lib.rs', '/compiled', 'bridge source differs')
            row.field('executable').equals(None)
            candidates = [path for path in filenames if path == deps/'libsim_bridge.dylib']
        else:
            row.field('package_id').equals(f'path+file://{product}/rust/crates/sim-test#0.7.0')
            target.field('kind').equals(['bin']); target.field('crate_types').equals(['bin'])
            require(source == product/'rust/crates/sim-test/src/bin'/f'{name}.rs', '/compiled', 'normal bin source differs')
            expected = product/'rust/target'/profile/name
            row.field('executable').equals(str(expected))
            require(filenames == (expected,), '/compiled', 'normal executable filename differs')
            candidates = [expected]
        require(len(candidates) == 1, '/compiled', 'missing/ambiguous selected compiler filename')
        regular(source, cargo_home if name in REGISTRY else product)
        for output in filenames:
            regular(output, product/'rust/target'/profile)
            compiler_outputs[output] = file_hash(output)
        regular(candidates[0], product/'rust/target'/profile)
        selected[name] = (index, line, row, candidates[0])
    require(set(selected) == names, '/compiled', 'incomplete non-test compiler inventory')
    inputs: dict[Path, str] = {stream: stream_hash, **compiler_outputs}
    table: list[dict[str, Json]] = []; proofs: list[Json] = []; used: set[str] = set()
    for relative, digest in sorted(members.items()):
        if digest == 'directory':
            continue
        path = deps/relative
        regular(path, deps)
        if str(path) in native:
            require(native[str(path)][1] == digest, '/compiled', 'native digest differs')
            continue
        if not path.stat().st_mode & 0o111:
            continue
        matches: list[str] = []
        for name, (_, _, _, output) in selected.items():
            if path == output or (name in BINS and path.parent == deps and re.fullmatch(name+r'-[0-9a-f]{16}', path.name)):
                matches.append(name)
        require(len(matches) == 1, '/compiled', 'unknown/ambiguous executable search member')
        name = matches[0]
        require(name not in used, '/compiled', 'duplicate compiled target member')
        used.add(name); index, line, row, output = selected[name]
        actual = file_hash(path); output_hash = file_hash(output)
        require(actual == digest == output_hash, '/compiled', 'member/output/predispatch bytes differ')
        identity = 'compiled-'+name
        require(identity not in {value[0] for value in native.values()}, '/compiled', 'native identity collision')
        inputs[path] = actual; inputs[output] = output_hash
        table.append({'id': identity, 'path': str(path), 'sha256': actual})
        proofs.append({'id': identity, 'member_path': str(path), 'member_sha256': actual,
                       'relation': 'exact_filename' if path == output else 'byte_equal_alias',
                       'compiler_stream': str(stream), 'compiler_stream_sha256': stream_hash,
                       'compiler_line_index': index, 'compiler_line_sha256': hashlib.sha256(line).hexdigest(),
                       'compiler_record': row.value, 'compiler_output': str(output), 'compiler_output_sha256': output_hash})
    require(used == names, '/compiled', 'missing executable compiled member')
    require(file_hash(stream) == stream_hash, '/compiled', 'compiler stream changed')
    return Compiled(tuple(sorted(table, key=lambda row: str(row['id']))), tuple(sorted(inputs.items())),
                    {'schema_version': 1, 'authority': 'DRIVER_AUTHENTICATED_BUILD_REQUIRED', 'members': proofs})
