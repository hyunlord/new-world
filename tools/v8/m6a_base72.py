"""Distinct, authenticated BASE observer adapter; never relabel the original binary."""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import subprocess
import time
from pathlib import Path

from m6a_protocol import protocol, recorded, check_freeze
import core_p7_publication as inventory

from m6a_evidence import (INPUT_ROOTS, EvidenceError, Node, Reference, digest, git,
                         reference_json, source_hash, write_json)

BASE = '5b0c88812374b87e7854cc9203f144ad0ccd9009'
POLICY = '7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8'
DEADLINE = 1793335103
MANIFEST = frozenset(('schema_version', 'kind', 'base', 'base_library_delta', 'entry_delta',
    'library_source_sha256', 'source_before', 'source_after', 'lock', 'original_entry',
    'entry', 'original_probe', 'binary', 'build', 'qualification48', 'qualification72', 'horizon_checks'))
BUILD = frozenset(('profile', 'features', 'cargo_argv', 'rustc_argv', 'compiler',
    'compiler_version', 'target', 'sysroot', 'linker', 'inputs', 'cargo_messages', 'log',
    'started_epoch', 'ended_epoch', 'exit_code', 'wrapper_records', 'input_inventory', 'source_inventory',
    'library_terminal', 'entry_terminal', 'source_before_record', 'source_after_record'))
QUALIFICATION = frozenset(('schema_version', 'days', 'seed', 'original_binary', 'adapter_binary',
    'original_summary', 'adapter_summary', 'original_log', 'adapter_log', 'original_exit', 'adapter_exit'))
HORIZON = ('tick69120_contiguous', 'tick103679_contiguous', 'late_predator_counted',
           'duplicate_tick_rejected', 'missing_tick_rejected')


def need(ok: bool, message: str) -> None:
    if not ok:
        raise EvidenceError(message)


def keys(node: Node, expected: frozenset[str] | set[str]) -> None:
    need({key for key, _ in node.fields()} == expected, 'unexpected/missing adapter fields')


def ref(root: Path, node: Node) -> Path:
    path = Path(Reference.parse(node).path)
    need(not path.is_relative_to(root.resolve()), 'adapter evidence must be external')
    return path


def blob(root: Path, relative: str) -> bytes:
    return subprocess.check_output(['git', '-C', str(root), 'show', f'{BASE}:{relative}'])


def validate_world(summary: Node, seed: int, days: int, policy: str = POLICY,
                   base: str = BASE) -> None:
    need(summary.get('scene').text() == 'colony' and summary.get('days').integer() == days,
         'wrong BASE summary horizon')
    worlds = summary.get('seeds').rows()
    need(len(worlds) == 1, 'BASE adapter requires one world')
    world = worlds[0]
    need(world.get('seed').integer() == seed and world.get('days').integer() == days,
         'wrong BASE world seed/horizon')
    for counter in ('ledger_violation_ticks', 'reservation_violation_ticks'):
        world.get(counter).integer()  # Observed nonzero violations are scorer failures, never erased.
    need(world.get('mux').get('ticks_per_day').integer() == 1440, 'BASE calendar changed')
    mux = world.get('mux2')
    need(mux.get('start_tick').integer() == 0
         and mux.get('end_tick').integer() == mux.get('observed_ticks').integer() == days * 1440,
         'incomplete BASE observer coverage')
    need(not mux.get('observer_integrity_errors').rows(), 'BASE observer integrity error')
    inputs = mux.get('inputs')
    need(inputs.get('steward').boolean() and inputs.get('defence').boolean()
         and inputs.get('story').text() == inputs.get('neighbours').text() == 'on'
         and not inputs.get('player_script').boolean()
         and inputs.get('policy_sha256').text() == policy
         and inputs.get('source_sha').text() == base, 'BASE observer inputs differ')
    events = mux.get('wolf_death_events').rows()
    need(mux.get('wolf_deaths').integer() == len(events), 'fatal event count differs')
    victims: set[int] = set()
    for event in events:
        victim = event.get('victim').integer()
        need(victim not in victims and event.get('tick').integer() < days * 1440
             and event.get('mechanic').text() == 'predator', 'invalid fatal event')
        event.get('attacker').integer(); event.get('kind').integer()
        victims.add(victim)


def compare_qualification(original: Node, adapter: Node, days: int) -> None:
    left, right = copy.deepcopy(original.value), copy.deepcopy(adapter.value)
    if days == 72:
        need(isinstance(left, dict) and isinstance(right, dict), 'qualification summary type')
        old_rows, new_rows = Node(left).get('seeds').rows(), Node(right).get('seeds').rows()
        need(len(old_rows) == len(new_rows) == 1, 'qualification seed coverage')
        need('mux2' not in dict(old_rows[0].fields()), 'ordinary72 unexpectedly has observer')
        row = new_rows[0].value
        need(isinstance(row, dict) and 'mux2' in row, 'adapter72 observer missing')
        del row['mux2']
    else:
        need(days == 48, 'unsupported qualification horizon')
    need(json.dumps(left, sort_keys=True, separators=(',', ':'), allow_nan=False)
         == json.dumps(right, sort_keys=True, separators=(',', ':'), allow_nan=False),
         'BASE qualification changed deterministic fields')


def validate_invocation(invocation: Node, compiler_sha: str, sources: dict[str, str],
                        inputs: set[tuple[str, int]],
                        cache: dict[Path, tuple[str, int]] | None = None,
                        builtin_externs: dict[str, Path] | None = None) -> None:
    """Bind recorded rustc inputs to actual source/library bytes, never just exit status."""
    need(invocation.get('exit_code').integer() == 0, 'failed compiler invocation')
    argv = [arg.text() for arg in invocation.get('argv').rows()]
    need(bool(argv), 'empty compiler argv')
    cwd = Path(invocation.get('cwd').text()).resolve(strict=True)
    captured = {} if cache is None else cache
    def resolved(value: str) -> Path:
        path = Path(value)
        return (path if path.is_absolute() else cwd / path).resolve(strict=True)
    def fingerprint(path: Path) -> tuple[str, int]:
        if path not in captured:
            value = Reference.capture(path)
            captured[path] = (value.sha256, value.size)
        return captured[path]
    def consumed(value: str, directory: bool = False) -> None:
        path = resolved(value)
        if directory:
            need(path.is_dir(), 'missing compiler input directory')
            for child in path.rglob('*'):
                if child.is_file():
                    need(fingerprint(child.resolve()) in inputs, 'unbound compiler directory input: ' + str(child))
        else:
            need(fingerprint(path) in inputs, 'unbound compiler input: ' + str(path))
    need(fingerprint(resolved(argv[0]))[0] == compiler_sha, 'Cargo compiler identity differs')
    source_args = [arg for arg in argv[1:] if arg.endswith('.rs') and not arg.startswith('-')]
    if not source_args:
        # Cargo's no-source discovery requests have no compilation/output side effects.
        probe = ['-', '--crate-name', '___', '--print=file-names', '--crate-type', 'bin',
                 '--crate-type', 'rlib', '--crate-type', 'dylib', '--crate-type', 'cdylib',
                 '--crate-type', 'staticlib', '--crate-type', 'proc-macro', '--print=sysroot',
                 '--print=split-debuginfo', '--print=crate-name', '--print=cfg', '-Wwarnings']
        need(argv[1:] in (['-vV'], ['--version'], probe), 'unrecognized source-free rustc command')
        return
    need(len(source_args) == 1, 'ambiguous rustc source')
    source = resolved(source_args[0]); identity = fingerprint(source)
    if str(source) in sources:
        need(identity[0] == sources[str(source)], 'compiled BASE/entry source differs')
    else:
        need(identity in inputs, 'registry compiler source absent from frozen inputs')
    cursor = 1
    while cursor < len(argv):
        arg = argv[cursor]
        if arg in ('--extern', '-L'):
            need(cursor + 1 < len(argv), 'missing compiler link argument')
            value = argv[cursor + 1]; cursor += 1
        elif arg.startswith('--extern='):
            value = arg.removeprefix('--extern='); arg = '--extern'
        elif arg.startswith('-L') and len(arg) > 2:
            value = arg[2:]; arg = '-L'
        else:
            cursor += 1; continue
        if arg == '--extern':
            if '=' in value:
                consumed(value.split('=', 1)[1])
            else:
                need(value == 'proc_macro' and builtin_externs is not None
                     and value in builtin_externs, 'unresolved compiler extern')
                library = builtin_externs[value]
                need(library.name.startswith('libproc_macro-') and library.suffix == '.rlib',
                     'wrong builtin proc_macro library')
                consumed(str(library))
        else:
            consumed(value.split('=', 1)[-1], directory=True)
        cursor += 1
    fields = dict(invocation.fields())
    if 'environment' in fields:
        environment = dict(fields['environment'].fields())
        if 'OUT_DIR' in environment:
            consumed(environment['OUT_DIR'].text(), directory=True)
        if 'CARGO_MANIFEST_DIR' in environment:
            package = resolved(environment['CARGO_MANIFEST_DIR'].text())
            # BASE packages are already covered by the exact full source manifest.
            if not any(Path(name).is_relative_to(package) for name in sources):
                consumed(str(package), directory=True)


def validate_manifest(root: Path, path: Path) -> Node:
    data = Node.load(path); keys(data, MANIFEST)
    need(data.get('schema_version').integer() == 1
         and data.get('kind').text() == 'm6a_base72_observer_adapter_v1'
         and data.get('base').text() == BASE
         and data.get('base_library_delta').text() == ''
         and data.get('entry_delta').text() == 'validated_observer_horizon_48_to_72',
         'wrong BASE adapter protocol')
    for name in ('lock', 'original_entry', 'entry', 'original_probe', 'binary'):
        ref(root, data.get(name))
    need(digest(ref(root, data.get('lock'))) == inventory.LOCK, 'authentic BASE lock differs')
    need(digest(ref(root, data.get('original_probe'))) ==
         '685ba630cef709cb4895873433aba553118c2d43ebc7cce55a07938bd5b1ff87', 'original BASE probe differs')
    need(ref(root, data.get('original_entry')).read_bytes() == blob(root,
        'rust/crates/sim-test/src/bin/worldsim_probe.rs'), 'original BASE entry differs')
    need(ref(root, data.get('entry')).read_bytes() == (root / 'tools/v8/m6a_base72_entry.rs').read_bytes(),
         'adapter entry differs from reviewed source')
    need(data.get('binary') != data.get('original_probe'), 'adapter must have distinct binary identity')
    build = Node.load(ref(root, data.get('build'))); keys(build, BUILD)
    need(build.get('profile').text() == 'release' and not build.get('features').rows()
         and build.get('exit_code').integer() == 0
         and build.get('ended_epoch').integer() >= build.get('started_epoch').integer(), 'incomplete adapter build')
    for name in ('compiler', 'compiler_version', 'linker', 'cargo_messages', 'log'):
        need(ref(root, build.get(name)).stat().st_size > 0, 'empty build evidence')
    need(bool(build.get('target').text()) and bool(build.get('sysroot').text()), 'missing target/sysroot')
    cargo = [x.text() for x in build.get('cargo_argv').rows()]
    rustc = [x.text() for x in build.get('rustc_argv').rows()]
    need(all(arg in cargo for arg in ('build', '--locked', '--release', '--lib', 'sim-test')),
         'wrong BASE library command')
    need('--extern' in rustc and any(arg.startswith('sim_test=') for arg in rustc)
         and str(ref(root, data.get('entry'))) in rustc, 'adapter linkage command missing')
    source_inventory = Node.load(ref(root, build.get('source_inventory')))
    expected: dict[str, str] = {}
    for scope in INPUT_ROOTS:
        expected.update(inventory.committed(root, BASE, scope))
    actual: dict[str, str] = {}
    for item in source_inventory.rows():
        keys(item, {'relative', 'file'}); relative = item.get('relative').text()
        need(relative in expected and relative not in actual, 'unexpected/duplicate BASE source')
        frozen = ref(root, item.get('file'))
        need(digest(frozen) == expected[relative], 'BASE source byte differs: ' + relative)
        actual[relative] = digest(frozen)
    need(set(actual) == set(expected), 'incomplete BASE source inventory')
    seal = hashlib.sha256()
    for name, checksum in sorted(actual.items()):
        seal.update(name.encode() + b'\0' + checksum.encode() + b'\n')
    need(all(data.get(name).text() == seal.hexdigest() for name in
             ('library_source_sha256', 'source_before', 'source_after')), 'BASE source build identity differs')
    inputs = build.get('inputs').rows()
    need(bool(inputs), 'missing build inputs')
    paths = [ref(root, item) for item in inputs]
    need(len(paths) == len(set(paths)), 'duplicate build inputs')
    need(Node.load(ref(root, build.get('input_inventory'))).value == build.get('inputs').value,
         'build input inventory differs')
    wrappers = Node.load(ref(root, build.get('wrapper_records'))).rows()
    need(bool(wrappers), 'missing actual compiler invocations')
    library_terminal = Node.load(ref(root, build.get('library_terminal')))
    entry_terminal = Node.load(ref(root, build.get('entry_terminal')))
    for terminal, command in ((library_terminal, cargo), (entry_terminal, rustc)):
        need([x.text() for x in terminal.get('argv').rows()] == command
             and terminal.get('exit_code').integer() == 0
             and terminal.get('ended_epoch').integer() >= terminal.get('started_epoch').integer(),
             'actual build terminal differs')
    need(library_terminal.get('source_unchanged').boolean()
         and library_terminal.get('lock_sha256').text() == inventory.LOCK, 'BASE build source/lock changed')
    need(entry_terminal.get('source') == data.get('entry'), 'compiled entry differs')
    compiled_root = Path(library_terminal.get('cwd').text()).parent
    found: set[str] = set()
    for scope in INPUT_ROOTS:
        start = compiled_root / scope
        paths = start.rglob('*') if start.is_dir() else [start]
        for candidate in paths:
            need(not candidate.is_symlink(), 'symlink compiled source')
            if candidate.is_file():
                found.add(candidate.relative_to(compiled_root).as_posix())
    need(found == set(expected) | {'rust/Cargo.lock'}, 'extra/missing actual BASE build source')
    need(digest(compiled_root / 'rust/Cargo.lock') == inventory.LOCK, 'actual compiled lock differs')
    for relative, checksum in expected.items():
        need(digest(compiled_root / relative) == checksum, 'actual compiled BASE source differs')
    for field in ('source_before_record', 'source_after_record'):
        record = Node.load(ref(root, build.get(field)))
        keys(record, {'base', 'digest', 'files'})
        need(record.get('base').text() == BASE and record.get('digest').text() == seal.hexdigest()
             and record.get('files').value == expected, 'actual before/after source inventory differs')
    frozen_inputs = {(Reference.parse(item).sha256, Reference.parse(item).size) for item in inputs}
    def consumed(path: Path) -> None:
        captured = Reference.capture(path)
        need((captured.sha256, captured.size) in frozen_inputs, 'consumed artifact absent: ' + str(path))
    source_paths = {str((compiled_root / name).resolve()): checksum for name, checksum in expected.items()}
    entry_path = ref(root, data.get('entry'))
    source_paths[str(entry_path)] = digest(entry_path)
    checked: dict[Path, tuple[str, int]] = {}
    compiler_sha = digest(ref(root, build.get('compiler')))
    sysroot_lib = Path(build.get('sysroot').text()) / 'lib/rustlib' / build.get('target').text() / 'lib'
    proc_macro = sorted(sysroot_lib.glob('libproc_macro-*.rlib'))
    need(len(proc_macro) == 1, 'ambiguous/missing sysroot proc_macro')
    builtin_externs = {'proc_macro': proc_macro[0]}
    consumed(proc_macro[0])
    for record in wrappers:
        validate_invocation(Node.load(ref(root, record)), compiler_sha, source_paths, frozen_inputs,
                            checked, builtin_externs)
    validate_invocation(entry_terminal, compiler_sha, source_paths, frozen_inputs, checked, builtin_externs)
    compiler = Path(rustc[0]).resolve(strict=True)
    need(digest(compiler) == digest(ref(root, build.get('compiler'))), 'executed compiler differs')
    need('-o' in rustc and digest(Path(rustc[rustc.index('-o') + 1]))
         == digest(ref(root, data.get('binary'))), 'executed output binary differs')
    linked = Path(next(arg.split('=', 1)[1] for arg in rustc if arg.startswith('sim_test=')))
    consumed(linked)
    for i, arg in enumerate(rustc):
        if arg == '-L':
            folder = Path(rustc[i + 1].split('=', 1)[-1])
            need(folder.is_dir(), 'missing link search directory')
            for artifact in folder.rglob('*'):
                if artifact.is_file(): consumed(artifact)
    messages = [Node.parse(line) for line in ref(root, build.get('cargo_messages')).read_text().splitlines()]
    for message in messages:
        if message.get('reason').text() == 'build-script-executed':
            output = Path(message.get('out_dir').text())
            need(output.is_dir(), 'Cargo generated output missing')
            for generated in output.rglob('*'):
                if generated.is_file(): consumed(generated)
    artifacts = [row for row in messages if row.get('reason').text() == 'compiler-artifact']
    need(any(row.get('reason').text() == 'build-finished' and row.get('success').boolean()
             for row in messages), 'cargo build completion missing')
    linked_seen = False
    for artifact in artifacts:
        for name in artifact.get('filenames').rows():
            artifact_path = Path(name.text()); consumed(artifact_path)
            if artifact.get('target').get('name').text() == 'sim_test' and artifact_path.suffix == '.rlib':
                linked_seen |= digest(artifact_path) == digest(linked)
    need(linked_seen, 'linked sim_test not in Cargo artifact output')
    for days in (48, 72):
        q = Node.load(ref(root, data.get(f'qualification{days}'))); keys(q, QUALIFICATION)
        need(q.get('schema_version').integer() == 1 and q.get('days').integer() == days
             and q.get('seed').integer() == 0 and q.get('original_exit').integer() == 0
             and q.get('adapter_exit').integer() == 0, 'failed qualification')
        for arm, binding in [('original', 'original_probe'), ('adapter', 'binary')]:
            need(q.get(arm + '_binary') == data.get(binding), 'qualification binary changed')
            ref(root, q.get(arm + '_log'))
        a, b = [ref(root, q.get(arm + '_summary')) for arm in ('original', 'adapter')]
        validate_world(Node.load(b), 0, days)
        compare_qualification(Node.load(a), Node.load(b), days)
        if days == 48:
            need(a.read_bytes() == b.read_bytes(), 'qualification48 not byte exact')
    horizon = Node.load(ref(root, data.get('horizon_checks')))
    keys(horizon, {'schema_version', 'source', 'log', 'exit_code', 'checks'})
    need(horizon.get('schema_version').integer() == 1 and horizon.get('exit_code').integer() == 0
         and [x.text() for x in horizon.get('checks').rows()] == list(HORIZON), 'horizon checks incomplete')
    need(digest(ref(root, horizon.get('source'))) == '2d2fc9b755e47ed218897aec184565b3b01b2798e0180600775ce3a6c82f659d',
         'horizon test source differs from reviewed BASE observer tests')
    log = ref(root, horizon.get('log')).read_text()
    need(all(name in log for name in HORIZON) and 'test result: ok.' in log, 'horizon test log incomplete')
    return data


def command(binary: Path, seed: int, target: Path) -> list[str]:
    return [str(binary), '--scene', 'colony', '--seeds', f'{seed}..{seed+1}', '--days', '72',
        '--workers', '1', '--out', str(target), '--steward', 'on', '--defence', 'on',
        '--story', 'on', '--neighbours', 'on', '--mux2-observe', '--mux2-policy-sha256', POLICY]


def validate_cohort(root: Path, manifest_path: Path, cohort_path: Path) -> None:
    from m6a_score import receipt, files
    manifest = validate_manifest(root, manifest_path)
    data = Node.load(cohort_path)
    need(data.get('base_build').value == reference_json(manifest_path)
         and data.get('binary') == manifest.get('binary')
         and data.get('observer').value == reference_json(Path(__file__)), 'BASE cohort adapter binding differs')
    execution = data.get('source_head').text(); current = git(root, 'rev-parse', 'HEAD')
    git(root, 'merge-base', '--is-ancestor', execution, current)
    for scope in INPUT_ROOTS:
        need(inventory.committed(root, execution, scope) == inventory.committed(root, current, scope),
             'BASE runner execution inputs differ')
    need(data.get('source_hash').text() == source_hash(root, current), 'BASE runner source differs')
    for row in receipt(cohort_path, 'base', 'heldout'):
        seed = row.get('seed').integer()
        target = cohort_path.parent / f'seed-{seed}'
        need([x.text() for x in row.get('command').rows()] ==
             command(ref(root, manifest.get('binary')), seed, target), 'BASE execution argv differs')
        need(row.get('observer_error').value is None, 'BASE observation failed')
        refs = files(row)
        need(all(Path(item.path).is_relative_to(target) for item in refs.values()), 'foreign world artifact')
        validate_world(refs['summary.json'].read(), seed, 72)


def run(root: Path, manifest_path: Path, freeze_path: Path, out: Path, deadline: int,
        selected_protocol: str = 'legacy') -> int:
    from m6a_run import probe_environment
    spec = protocol(selected_protocol)
    need(time.time() < deadline <= spec.deadline, 'BASE72 deadline expired/extended')
    manifest_reference = reference_json(manifest_path)
    freeze_reference = reference_json(freeze_path)
    manifest = validate_manifest(root, manifest_path)
    frozen = Node.load(freeze_path)
    check_freeze(frozen, spec)
    need(frozen.get('kind').text() == spec.freeze_kind and recorded(frozen).name == spec.name
         and frozen.get('scorer_sha256').text() == digest(root / 'tools/v8/m6a_score.py'), 'frozen scorer differs')
    head = git(root, 'rev-parse', 'HEAD'); source = source_hash(root, head)
    binary = ref(root, manifest.get('binary')); out = out.resolve()
    need(not out.is_relative_to(root), 'BASE heldout output must be external')
    out.mkdir(exist_ok=False)
    env = probe_environment(os.environ); env['WORLDSIM_GIT_SHA'] = BASE
    rows = []; failed = False
    for seed in spec.heldout:
        if time.time() >= deadline:
            failed = True; break
        target = out / f'seed-{seed}'; log = out / f'seed-{seed}.log'
        argv = command(binary, seed, target)
        with log.open('x') as stream:
            try:
                completed = subprocess.run(argv, cwd=root, env=env, stdout=stream,
                    stderr=subprocess.STDOUT, timeout=max(0.01, deadline-time.time()), check=False)
                code = completed.returncode
            except subprocess.TimeoutExpired:
                code = 124
        evidence = [reference_json(p) for p in sorted(target.rglob('*')) if p.is_file()]
        error = None
        if code == 0:
            try: validate_world(Node.load(target / 'summary.json'), seed, 72)
            except (EvidenceError, OSError) as problem: error = str(problem)
        row = {'seed': seed, 'command': argv, 'exit_code': code, 'files': evidence,
               'log': reference_json(log), 'observer_error': error}
        rows.append(row); write_json(out / f'seed-{seed}-receipt.json', row)
        failed |= code != 0 or error is not None
    Reference.parse(Node(manifest_reference)); Reference.parse(Node(freeze_reference))
    unchanged = source_hash(root, head) == source and Reference.parse(manifest.get('binary')).path == str(binary)
    failed |= not unchanged or len(rows) != 32
    write_json(out / 'receipt.json', {'schema_version': 1, 'kind': 'm6a_cohort_v1', 'evaluation_protocol': spec.name, 'arm': 'base',
        'purpose': 'heldout', 'source_root': str(root), 'source_head': head, 'source_hash': source,
        'probe_environment': {'content_selection': 'bundled', 'WORLDSIM_GIT_SHA': BASE},
        'binary': manifest.get('binary').value, 'base_build': manifest_reference,
        'observer': reference_json(Path(__file__)), 'scorer': reference_json(root/'tools/v8/m6a_score.py'),
        'freeze': freeze_reference, 'days': 72, 'seeds': [seed for seed in spec.heldout],
        'round': None, 'worlds': rows, 'source_unchanged': unchanged, 'exit_code': int(failed)})
    return int(failed)


class Arguments(argparse.Namespace):
    manifest: Path = Path()
    freeze: Path = Path()
    out: Path = Path()
    protocol: str = 'legacy'
    deadline: int | None = None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--freeze', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    _ = parser.add_argument('--protocol', choices=('legacy', 'repair_r1', 'repair_r2'), default='legacy')
    parser.add_argument('--deadline', type=int)
    args = parser.parse_args(namespace=Arguments())
    return run(Path(__file__).resolve().parents[2], args.manifest, args.freeze, args.out, args.deadline if args.deadline is not None else protocol(args.protocol).deadline, args.protocol)


if __name__ == '__main__':
    raise SystemExit(main())
