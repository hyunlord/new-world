"""Test-only recipes for the actual-image C owner; no subprocesses or native claims."""
import json
import os
import shlex
import struct
import sys
import tempfile
from pathlib import Path

from generic_gate import document
from mux2_gate_inputs import Json, Node

CARGO_COMMIT = '083ac5135f967fd9dc906ab057a2315861c7a80d'
MACROS = {'godot_macros': ('godot-macros', '0.4.5'), 'nanoserde_derive': ('nanoserde-derive', '0.2.1'),
          'serde_derive': ('serde_derive', '1.0.229'), 'thiserror_impl': ('thiserror-impl', '1.0.69'),
          'zerocopy_derive': ('zerocopy-derive', '0.8.63')}
BINS = ('worldsim_perf', 'worldsim_probe', 'worldsim_systems')
LIBRARIES = ('sim-bridge', 'sim-core', 'sim-engine', 'sim-systems', 'sim-test')
NATIVE = '''#!{python}
import sys
if '--list' in sys.argv:
 print('0 tests, 0 benchmarks' if '--ignored' in sys.argv else 'fixture_case: test\\n\\n1 test, 0 benchmarks')
else:
 print('\\nrunning 1 test\\ntest fixture_case ... ok\\n\\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0s\\n')
'''


def field(value: str) -> bytes:
    raw=value.encode();return struct.pack('!I',len(raw))+raw


def output(value: str, *, stderr: bool=False) -> bytes:
    return (b'E' if stderr else b'O')+field(value)


def launch(cwd: Path, argv: tuple[str,...]) -> bytes:
    return b'X'+field(str(cwd))+struct.pack('!I',len(argv))+b''.join(field(arg) for arg in argv)


def finish() -> bytes:
    return b'Q'+struct.pack('!I',0)


def rustdoc(argv: tuple[str,...]) -> bytes:
    if '--version' in argv:
        return output('rustdoc 1.93.1 synthetic fixture\nrelease: 1.93.1\ncommit-hash: 01f6ddf7588f42ae2d7eb0a2f21d44e8e96674cf\nhost: aarch64-apple-darwin\n')+finish()
    if '--list' in argv:
        return output('0 tests, 0 benchmarks\n' if '--ignored' in argv else 'fixture_case: test\n\n1 test, 0 benchmarks\n')+finish()
    temporary=os.confstr(65537)
    if not temporary:raise RuntimeError('fixture requires Darwin temporary root')
    owned=Path(tempfile.mkdtemp(prefix='rustdoctest-fixture-',dir=temporary)).resolve()
    binary=owned/'rust_out';_=binary.write_text(NATIVE.format(python=sys.executable));binary.chmod(0o755)
    runner=argv[argv.index('--test-runtool')+1]
    helper=argv[argv.index('--test-runtool-arg')+1]
    cwd=Path(argv[argv.index('--test-run-directory')+1])
    # The C rustdoc image launches this exact immediate child and performs waitpid.
    # The test cleans only transient paths retained in actual capture receipts.
    return launch(cwd,(runner,helper,str(binary)))+finish()


def main(argv: tuple[str,...]) -> bytes:
    if Path(argv[0]).name=='rustdoc':return rustdoc(argv)
    if '--version' in argv:
        return output('cargo 1.93.1 synthetic fixture\nrelease: 1.93.1\ncommit-hash: '+CARGO_COMMIT+'\nhost: aarch64-apple-darwin\n')+finish()
    workspace=Path.cwd();target=Path(os.environ.get('CARGO_TARGET_DIR',str(workspace/'target')))
    profile='release' if '--release' in argv else 'debug';deps=target/profile/'deps'
    # JSON is fixture-local and source-bound by the test's actual BuildSpec closure.
    packages: list[dict[str,Json]]=[]
    for name in LIBRARIES:
        package=workspace/'crates'/name;crate=name.replace('-','_')
        types=['cdylib','rlib'] if name=='sim-bridge' else ['lib']
        targets: list[Json]=[{'name':crate,'src_path':str(package/'src/lib.rs'),'kind':[*types],'crate_types':[*types],
                  'edition':'2021','doctest':True,'test':True}]
        if name=='sim-test':
            targets.extend({'name':name,'src_path':str(package/'src/bin'/f'{name}.rs'),'kind':['bin'],
                            'crate_types':['bin'],'edition':'2021','doctest':False,'test':False} for name in BINS)
        packages.append({'id':'path+file://'+str(package)+'#0.7.0','name':name,'manifest_path':str(package/'Cargo.toml'),
                         'features':{},'dependencies':[],'targets':targets})
    metadata: dict[str,Json]={'version':1,'workspace_root':str(workspace),'target_directory':str(target),
              'workspace_members':[package['id'] for package in packages],'packages':[row for row in packages]}
    if argv[1]=='metadata':return output(json.dumps(metadata)+'\n')+finish()
    if argv[1] in ('fmt','clippy'):return finish()
    if argv[1]=='test' and '--no-run' not in argv:
        manifest=document(Path(os.environ['CARGO_CAPTURE_MANIFEST']).read_text())
        runner=tuple(shlex.split(os.environ['CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER']))
        result=b''
        for artifact in manifest.field('artifacts').array():
            binary=Path(artifact.field('path').text());name=binary.name.split('-')[0].replace('_','-')
            args=(*runner,str(binary),'--test-threads=8')
            result+=output('     Running `'+shlex.join(args)+'`\n',stderr=True)
            result+=launch(workspace/'crates'/name,args)
        for invocation in manifest.field('doctest_policy').field('invocations').array():
            policy=invocation.field('argument_policy')
            middle=tuple(token for name,path in policy.field('externs').mapping().items() for token in ('--extern',name+'='+Node(path).text()))
            args=(os.environ['RUSTDOC'],*(node.text() for node in policy.field('prefix').array()),*middle,*(node.text() for node in policy.field('suffix').array()))
            result+=output('   Doc-tests '+invocation.field('crate_name').text()+'\n',stderr=True)
            result+=output('     Running `'+shlex.join(args)+'`\n',stderr=True)
            result+=launch(Path(invocation.field('compiler_cwd').text()),args)
        return result+finish()
    deps.mkdir(parents=True,exist_ok=True)
    records: list[dict[str,Json]]=[]
    for package in packages:
        for selected in Node(package).field('targets').array():
            name=selected.field('name').text();normal=selected.field('kind').value==['bin']
            compiled=target/profile/name if normal else deps/('lib'+name+'.rlib')
            _=compiled.write_bytes(b'inert normal '+name.encode());compiled.chmod(0o755 if normal else 0o644)
            files=[str(compiled)]
            if name=='sim_bridge':
                bridge=deps/'libsim_bridge.dylib';_=bridge.write_bytes(b'fixture bridge');bridge.chmod(0o755)
                top=target/profile/'libsim_bridge.dylib';_=top.write_bytes(bridge.read_bytes());top.chmod(0o755);files.append(str(bridge))
            if normal:
                alias=deps/(name+'-1234567812345678');_=alias.write_bytes(compiled.read_bytes());alias.chmod(0o755)
            row: dict[str,Json]={'reason':'compiler-artifact','package_id':package['id'],'manifest_path':package['manifest_path'],
                 'target':selected.value,'profile':{'test':False},'features':[],'fresh':False,
                 'executable':str(compiled) if normal else None,'filenames':[name for name in files]}
            records.append(row)
            if not normal:
                binary=deps/(name+'-testfixture');_=binary.write_text(NATIVE.format(python=sys.executable));binary.chmod(0o755)
                records.append({**row,'profile':{'test':True},'executable':str(binary),'filenames':[str(binary)]})
    home=Path(os.environ['CARGO_HOME'])
    for name,(package,version) in MACROS.items():
        source=home/'registry/src/index.crates.io-1949cf8c6b5b557f'/f'{package}-{version}'/'src/lib.rs'
        source.parent.mkdir(parents=True,exist_ok=True);_=source.write_text('// fixture proc macro')
        binary=deps/('lib'+name+'-1234567812345678.dylib');_=binary.write_bytes(name.encode());binary.chmod(0o755)
        records.append({'reason':'compiler-artifact','package_id':f'registry+https://github.com/rust-lang/crates.io-index#{package}@{version}',
            'target':{'name':name,'kind':['proc-macro'],'crate_types':['proc-macro'],'src_path':str(source)},
            'profile':{'test':False},'fresh':False,'executable':None,'filenames':[str(binary)]})
    if '--no-run' not in argv:return finish()
    return b''.join(output(json.dumps(row)+'\n') for row in records)+output('{"reason":"build-finished","success":true}\n')+finish()


if __name__=='__main__':
    _=sys.stdout.buffer.write(main(tuple(sys.argv[1:])))
