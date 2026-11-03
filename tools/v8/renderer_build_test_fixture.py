"""Synthetic parser fixtures, never executed as product/build evidence."""
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from generic_process import file_hash
from generic_sources import inventory
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json, Node


@dataclass(frozen=True, slots=True)
class BuildFixture:
    session: Node
    build: Node
    process: Node
    messages: list[Json]
    output: Path
    binary: Path
    cargo_home: Path

    def logs(self) -> None:
        _ = (self.output/'stdout.log').write_text(''.join(json.dumps(row)+'\n' for row in self.messages))


def make_fixture(top: Path) -> BuildFixture:
    root=top/'source'; root.mkdir()
    for name in INPUT_DIRS: (root/name).mkdir(parents=True,exist_ok=True)
    for name in INPUT_FILES:
        p=root/name; p.parent.mkdir(parents=True,exist_ok=True); _ = p.write_text('fixture')
    _ = (root/'rust/Cargo.toml').write_text('[workspace.package]\nversion="0.1.0"\n')
    for args in [('init',),('add','.'),('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','-m','Synthetic reader fixture')]:
        _ = subprocess.run(['git','-C',str(root),*args],capture_output=True,check=True)
    sha=subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip()
    tool=Path(sys.executable).resolve(); digest=file_hash(tool)
    selector=top/'selector.json'; _ = selector.write_text(json.dumps({'selectors':{name:{'path':str(tool),'sha256':digest} for name in ['cargo','rustc']}}))
    session=Node({'selector_path':str(selector),'selector_sha256':file_hash(selector),'external_inputs':{str(tool):digest}})
    target=root/'rust/target'; binary=target/'debug/libsim_bridge.dylib'; binary.parent.mkdir(parents=True); _ = binary.write_text('synthetic text, never executed'); binary.chmod(0o755)
    output=top/'cargo-build'; output.mkdir(); cargo_home=top/'cargo-home'; cargo_home.mkdir()
    inputs=inventory(root,[*INPUT_DIRS,*INPUT_FILES])
    env: dict[str, Json]={'PATH':str(tool.parent)+':/usr/bin:/bin:/usr/sbin:/sbin','CARGO_HOME':str(cargo_home),'CARGO_TARGET_DIR':str(target),'RUSTC':str(tool),'RUSTC_WRAPPER':'','RUSTC_WORKSPACE_WRAPPER':''}
    argv: list[Json]=[str(tool),'build','--locked','--message-format=json-render-diagnostics','-p','sim-bridge']
    configs={str(parent/'.cargo'/name):file_hash(parent/'.cargo'/name) for parent in (root/'rust',*(root/'rust').parents) for name in ['config','config.toml'] if (parent/'.cargo'/name).exists()}
    build: dict[str, Json]={'cargo_sha256':digest,'rustc_sha256':digest,'argv':argv,'profile':'debug','packages':['sim-bridge'],'features':[],'source_kind':'DERIVED_OBSERVER','root':str(root),'source_sha':sha,'effective_inputs_before':{name: value for name, value in inputs.items()},'environment':env,'cargo_configs':{name: value for name, value in configs.items()},'target_dir':str(target),'artifacts':{'sim_bridge':{'path':str(binary),'sha256':file_hash(binary)}},'process_receipt':str(output/'process.json')}
    process: dict[str, Json]={'argv':argv,'errors':[],'cwd':str(root/'rust'),'input_root':str(root),'executable':str(tool),'expected_executable_sha256':digest,'expected_inputs':{name: value for name, value in inputs.items()},'environment':env|{'LANG':'C','LC_ALL':'C'},'output_root':str(output)}
    artifact: dict[str, Json]={'reason':'compiler-artifact','fresh':False,'target':{'name':'sim_bridge','kind':['cdylib','rlib'],'src_path':str(root/'rust/crates/sim-bridge/src/lib.rs')},'package_id':'path+'+(root/'rust/crates/sim-bridge').as_uri()+'#0.1.0','profile':{'debug_assertions':True,'test':False},'executable':None,'filenames':[str(binary)]}
    messages: list[Json]=[artifact,{'reason':'build-finished','success':True}]
    fixture = BuildFixture(session, Node(build), Node(process), messages, output, binary, cargo_home)
    fixture.logs()
    return fixture


def complete_receipt(fixture: BuildFixture, product: str) -> Node:
    """Supply full synthetic envelope for integration parsing, never runtime proof."""
    from copy import deepcopy
    build = Node(deepcopy(fixture.build.value))
    process = Node(deepcopy(fixture.process.value))
    messages = deepcopy(fixture.messages)
    output = fixture.output.parent/product/'cargo-build'
    output.mkdir(parents=True)
    root = Path(build.field('root').text())
    artifact = Node(messages[0])
    if product == 'worldsim_probe':
        binary = root/'rust/target/release/worldsim_probe'
        binary.parent.mkdir(parents=True)
        _ = binary.write_text('synthetic parser control only')
        binary.chmod(0o755)
        build.mapping().update({'profile': 'release', 'packages': ['sim-test'],
            'argv': [process.field('executable').text(), 'build', '--locked',
                     '--message-format=json-render-diagnostics', '--release', '-p', 'sim-test'],
            'artifacts': {product: {'path': str(binary), 'sha256': file_hash(binary)}}})
        process.mapping()['argv'] = build.field('argv').value
        artifact.mapping().update({'target': {'name': product, 'kind': ['bin'],
            'src_path': str(root/'rust/crates/sim-test/src/bin/worldsim_probe.rs')},
            'package_id': 'path+'+(root/'rust/crates/sim-test').as_uri()+'#0.1.0',
            'profile': {'debug_assertions': False, 'test': False},
            'executable': str(binary), 'filenames': [str(binary)]})
    process.mapping().update({'output_root': str(output), 'status': 'PASS', 'native_exit': 0,
                              'pid': 123, 'expected_external_inputs': {}})
    for phase in ('before', 'after'):
        process.mapping()[phase+'_inputs'] = process.field('expected_inputs').value
        process.mapping()[phase+'_external_inputs'] = {}
        process.mapping()[phase+'_executable_sha256'] = process.field('expected_executable_sha256').value
    for name, content in (('stdout', ''.join(json.dumps(row)+'\n' for row in messages)), ('stderr', '')):
        _ = (output/(name+'.log')).write_text(content)
        process.mapping()[name+'_sha256'] = file_hash(output/(name+'.log'))
    receipt = output/'process.json'
    _ = receipt.write_text(json.dumps(process.value))
    inputs = build.field('effective_inputs_before').mapping()
    tracked: dict[str, Json] = {name: ['100644', digest, digest] for name, digest in inputs.items()}
    effective: dict[str, Json] = {'manifest': {name: ['100644', digest] for name, digest in inputs.items()}}
    build.mapping().update({'status': 'PASS', 'exit_code': 0, 'source_before': {'tracked_manifest': tracked},
        'effective_source_before': effective, 'effective_source_after': effective,
        'process_receipt': str(receipt), 'process_receipt_sha256': file_hash(receipt)})
    path = output.parent/'build.json'
    _ = path.write_text(json.dumps(build.value))
    return Node({'build': {'path': str(path), 'sha256': file_hash(path), 'mode': path.stat().st_mode}})
