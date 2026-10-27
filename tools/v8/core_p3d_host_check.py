"""Verify a reconstructed legacy host and raw execution of scheduling adapters.

This checker never builds. Evidence must describe actual compiled executions;
labels declaring parity are not evidence. Writes only a new explicit output.
"""
from __future__ import annotations

import argparse
from datetime import datetime
from itertools import zip_longest
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tarfile

import core_shadow_freeze as freeze
import core_shadow_successor as prior
import core_perf_check as perf

ARCHIVE_SHA = '029f058369396efc1f33fa2577706b8a664fd7e51905c5349de965b2c8cce1d2'
BUILDER_SHA = '0e4efd1306a1efb4a18981918110d168b9312f0d0827bc924c9bbeb4f6b76450'
EXTERNAL_INPUTS = ('assets/ui/content_presentation.json', 'docs/v8/core/SYSTEM_WORLDS.md')
OVERLAY_DIRS = ('rust/crates/sim-core/src/', 'rust/crates/sim-core/data/', 'rust/crates/sim-engine/src/',
                'rust/crates/sim-systems/src/runtime/')
EXCLUDED_DIRS = ('rust/crates/sim-systems/src/runtime/decision/', 'rust/crates/sim-systems/src/runtime/baseline/')
EXCLUDED_FILES = ('rust/crates/sim-engine/src/registration.rs', 'rust/crates/sim-systems/src/runtime/mod.rs',
                  'rust/crates/sim-systems/src/runtime/jobs/mod.rs', 'rust/crates/sim-systems/src/runtime/jobs/seam.rs')
OVERLAY_FILES = ('rust/Cargo.lock', 'rust/Cargo.toml', 'rust/rust-toolchain.toml',
 'rust/crates/sim-core/Cargo.toml', 'rust/crates/sim-engine/Cargo.toml', 'rust/crates/sim-systems/Cargo.toml',
 'rust/crates/sim-test/Cargo.toml', 'rust/crates/sim-systems/src/commands.rs',
 'rust/crates/sim-systems/src/mind_observer.rs', 'rust/crates/sim-systems/src/subprofile.rs', 'rust/crates/sim-systems/src/perf_arms.rs')
need = prior.require
fields = prior.fields


def overlay(name: str) -> bool:
    return name in OVERLAY_FILES or (name.startswith(OVERLAY_DIRS)
           and not name.startswith(EXCLUDED_DIRS) and name not in EXCLUDED_FILES)


def archive_inventory(path: Path) -> dict[str, str]:
    freeze.pin(path, ARCHIVE_SHA)
    result = {}
    with tarfile.open(path, 'r:gz') as archive:
        seen = set()
        for member in archive.getmembers():
            name = member.name
            parts = PurePosixPath(name).parts
            need(name not in seen and '..' not in parts and not name.startswith('/'), 'unsafe/duplicate frozen member')
            seen.add(name)
            if any(part.startswith('._') for part in parts) or member.isdir():
                continue
            need(member.isfile() and name.startswith('rust/'), 'unexpected frozen member')
            stream = archive.extractfile(member)
            need(stream is not None, 'missing archive contents')
            with stream:
                result[name] = hashlib.sha256(stream.read()).hexdigest()
    return result


def live_inventory(root: Path) -> dict[str, str]:
    result = {}
    for path in (root / 'rust').rglob('*'):
        need(not path.is_symlink(), 'symlink in host/source closure')
        if path.is_file():
            result[path.relative_to(root).as_posix()] = freeze.digest(path)
    for name in EXTERNAL_INPUTS:
        result[name] = freeze.digest(root / name)
    return result


def external_inventory(repo: Path, commit: str) -> dict[str, str]:
    result = {}
    for name in EXTERNAL_INPUTS:
        digest = freeze.digest(repo / name)
        need(digest == hashlib.sha256(prior.git(repo, 'show', commit + ':' + name)).hexdigest(),
             'external input differs from candidate commit: ' + name)
        result[name] = digest
    return result


def external_binding(receipt: freeze.Document, expected: dict[str, str]) -> None:
    need(receipt.get('external_inputs') == expected, 'receipt external inputs differ')


def current_overlay(repo: Path) -> dict[str, str]:
    result = {}
    for directory in OVERLAY_DIRS:
        for path in (repo / directory).rglob('*'):
            need(not path.is_symlink(), 'symlink in candidate overlay')
            name = path.relative_to(repo).as_posix()
            if path.is_file() and overlay(name):
                result[name] = freeze.digest(path)
    for name in OVERLAY_FILES:
        result[name] = freeze.digest(repo / name)
    need(result[prior.POLICY] == prior.POLICY_SHA, 'policy changed')
    return result


def apply_adapters(repo: Path, files: dict[str, str], contract: freeze.Document) -> tuple[dict[str, str], set[str]]:
    fields(contract, 'schema_version candidate_source_sha256 shared_files adapters parity_tests review')
    freeze.version(contract)
    need(bool(contract['review']) and bool(contract['parity_tests']), 'missing adapter review/test obligations')
    shared = contract['shared_files']
    need(bool(shared) and any('/continuation' in name for name in shared), 'shared core continuation module missing')
    for name, digest in shared.items():
        need(name.startswith('rust/crates/sim-core/src/colony/mind/') and files.get(name) == digest
             == freeze.digest(repo / name), 'shared core gate diverges')
    adapted = set()
    result = files.copy()
    for item in contract['adapters']:
        fields(item, 'path before_sha256 after_sha256 replacement reason')
        name = item['path']
        need(name not in adapted and '..' not in PurePosixPath(name).parts and name.endswith('.rs'), 'unsafe/duplicate adapter path')
        need(name.startswith('rust/crates/sim-systems/src/runtime/mind/shadow/')
             or name in ('rust/crates/sim-systems/src/runtime/mod.rs', 'rust/crates/sim-test/src/bin/worldsim_probe.rs'), 'adapter escapes observer boundary')
        need(result.get(name) == item['before_sha256'] and bool(item['reason']), 'adapter before/reason differs')
        replacement = freeze.external(repo, Path(item['replacement']))
        freeze.pin(replacement, item['after_sha256'])
        result[name] = item['after_sha256']
        adapted.add(name)
    need('rust/crates/sim-systems/src/runtime/mind/shadow/mod.rs' in adapted, 'actual shadow adapter absent')
    return result, adapted


def reconstruction(repo: Path, contract: freeze.Document) -> tuple[Path, freeze.Document, freeze.Document]:
    original = archive_inventory(freeze.read_reference(repo, contract['frozen_archive']))
    frozen_path = freeze.read_reference(repo, contract['frozen_manifest'])
    freeze.pin(frozen_path, prior.FROZEN_SHA)
    frozen = freeze.load(frozen_path)
    need(all(original.get(name) == digest for name, digest in frozen['frozen_files'].items()), 'original closure differs')
    builder = freeze.read_reference(repo, contract['builder'])
    freeze.pin(builder, BUILDER_SHA)
    prepared_path = freeze.read_reference(repo, contract['prepare_manifest'])
    prepared = freeze.load(prepared_path)
    host = freeze.external(repo, prepared_path.parent / 'host')
    adapter_path = freeze.read_reference(repo, contract['adapter_contract'])
    adapters = freeze.load(adapter_path)
    source = perf.source_hash()
    need(source == contract['candidate_source_sha256'] == prepared['candidate_source_sha256']
         == adapters['candidate_source_sha256'], 'candidate/adapter source binding differs')
    candidate = current_overlay(repo)
    files = {name: digest for name, digest in original.items() if not overlay(name) or name in candidate}
    files.update(candidate)
    external = external_inventory(repo, contract['candidate_commit'])
    external_binding(prepared, external)
    files.update(external)
    result, adapted = apply_adapters(repo, files, adapters)
    need(candidate['rust/Cargo.lock'] == frozen['current_files']['rust/Cargo.lock'], 'host lock differs from independently frozen lock')
    need(prepared['old_inventory'] == original and prepared['candidate_overlay'] == candidate
         and prepared['host_inventory'] == result == live_inventory(host), 'host reconstruction differs')
    need(prepared['removed_from_overlay'] == sorted(name for name in original if overlay(name) and name not in candidate), 'overlay removal differs')
    need(prepared['preserved_host_files'] == {name: digest for name, digest in result.items() if name not in candidate and name not in adapted and name not in external}, 'preserved old host inventory differs')
    need(prepared['excluded_directories'] == list(EXCLUDED_DIRS) and prepared['excluded_files'] == list(EXCLUDED_FILES), 'closure exclusions differ')
    need(prepared['shared_gate_files'] == list(adapters['shared_files'])
         and prepared['adapter_contract'] == {'path': str(adapter_path), 'sha256': contract['adapter_contract']['sha256'], 'contract': adapters}, 'prepared adapter contract differs')
    need(prepared['candidate_head'] == contract['candidate_commit'] and prepared['policy_sha256'] == prior.POLICY_SHA
         and prepared['builder_sha256'] == BUILDER_SHA and prepared['frozen_archive_sha256'] == ARCHIVE_SHA
         and prepared['frozen_manifest_sha256'] == prior.FROZEN_SHA, 'prepared provenance differs')
    build_path = freeze.read_reference(repo, contract['host_build'])
    build = freeze.load(build_path)
    need(build['manifest_sha256'] == contract['prepare_manifest']['sha256']
         and build['host_inventory'] == result and build['candidate_source_sha256'] == source
         and build['policy_sha256'] == prior.POLICY_SHA, 'host build source differs')
    external_binding(build, external)
    manifest_path = freeze.read_reference(repo, contract['host_manifest'])
    manifest = freeze.load(manifest_path)
    fields(manifest, 'schema_version kind prepare_manifest host_build shadow control')
    freeze.version(manifest)
    need(manifest['kind'] == 'p3d_reconstructed_legacy_host' and manifest['prepare_manifest'] == contract['prepare_manifest']
         and manifest['host_build'] == contract['host_build'], 'gate host manifest differs')
    need([row['role'] for row in build['builds']] == ['control', 'shadow'], 'host build roles missing/duplicated')
    for row in build['builds']:
        role = row['role']
        features = ['mind_shadow'] if role == 'shadow' else []
        argv = ['cargo', 'build', '--locked', '--release', '--manifest-path', 'rust/Cargo.toml', '-p', 'sim-test', '--bin', 'worldsim_probe']
        if features:
            argv += ['--features', 'mind_shadow']
        need(row['features'] == features and row['command'] == argv and type(row['exit_code']) is int and row['exit_code'] == 0, 'host build command/features/exit differs')
        freeze.pin(build_path.parent / (role + '.log'), row['log_sha256'])
        need(freeze.load(build_path.parent / (role + '-exit.json')) == {key: value for key, value in row.items() if key != 'binary_sha256'}, 'observed host build exit differs')
        need(freeze.digest(freeze.read_reference(repo, manifest[role])) == row['binary_sha256'], 'host binary differs')
    return host, manifest, prepared

CASES = {'retained', 'cadence', 'critical', 'finish', 'unsupported_fact', 'missing_history',
         'owner_mismatch', 'invitation', 'replay', 'no_running', 'finished', 'idle_owner', 'social_owner'}
RETAIN_CASES = {'retained', 'idle_owner', 'social_owner'}


def integer(value: freeze.JSON, name: str) -> int:
    need(type(value) is int and 0 <= value < 2**64, 'invalid integer: ' + name)
    return value


def canonical(value: freeze.JSON) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def json_line(line: str) -> freeze.Document:
    need(bool(line.strip()), 'blank/missing parity record')
    value = json.loads(line, object_pairs_hook=freeze.unique_pairs)
    need(type(value) is dict, 'parity record must be object')
    canonical(value)  # rejects NaN and infinities, including nested input values
    return value


def actual(value: freeze.Document) -> None:
    fields(value, 'decision node reason reads')
    need(type(value['reads']) is list, 'missing actual provider read trace')
    if value['decision'] == 'continue':
        integer(value['node'], 'retained node')
        need(value['reason'] is None, 'retained outcome has full-evaluation reason')
    else:
        need(value['decision'] in ('evaluate', 'error') and value['node'] is None
             and type(value['reason']) is str and bool(value['reason']), 'invalid guard decision/error reason')


def live_rows(raw: Path, replay: Path, mode: str, seed: int, observation: freeze.Document) -> tuple[int, int]:
    """Stream every actual evaluation; retain all finalized port ticks in denominator."""
    last_key = None
    last_row = None
    evaluations = persons = ports = retained = full = 0

    def finalize(row: freeze.Document) -> None:
        nonlocal persons, ports, retained, full
        need(row['finalized'] is True, 'person-tick lacks final observation')
        persons += 1
        if row['port']:
            ports += 1
            if row['dispatch'] == 'retained':
                retained += 1
            else:
                full += 1

    with raw.open() as left, replay.open() as right:
        for a, b in zip_longest(left, right):
            need(a is not None and b is not None, 'replay record count differs')
            row, replayed = json_line(a), json_line(b)
            need(canonical(row) == canonical(replayed), 'compiled guard replay differs from actual execution')
            fields(row, 'schema_version mode seed tick agent evaluation finalized port input actual dispatch full_terminal')
            freeze.version(row)
            need(row['mode'] == mode and integer(row['seed'], 'seed') == seed, 'wrong trace world')
            tick, agent, evaluation = (integer(row[key], key) for key in ('tick', 'agent', 'evaluation'))
            need(tick < 20 * 1440 and type(row['finalized']) is bool and type(row['port']) is bool, 'trace coverage/type differs')
            need(type(row['input']) is dict and bool(row['input']), 'typed guard replay input absent')
            actual(row['actual'])
            need(row['dispatch'] == ('retained' if row['actual']['decision'] == 'continue' else 'full'),
                 'observation-only guard prediction is not executed scheduling')
            terminal = row['full_terminal']
            need(terminal is None if row['dispatch'] == 'retained' else terminal == {'selection': 'success', 'settlement': 'success'},
                 'full-selection/final legacy settlement is missing or failed')
            key = (tick, agent)
            if key != last_key:
                if last_row is not None:
                    need(key > last_key, 'duplicate/out-of-order person-tick')
                    finalize(last_row)
                need(evaluation == 0, 'first evaluation index is not zero')
            else:
                need(last_row['finalized'] is False and evaluation == last_row['evaluation'] + 1,
                     'duplicate/missing evaluation or multiple final observations')
            last_key, last_row = key, row
            evaluations += 1
    need(last_row is not None, 'empty live parity trace')
    finalize(last_row)
    counts = observation['counts']
    need(evaluations == integer(counts['evaluations'], 'old evaluations')
         == integer(observation['prediction_evaluations'], 'prediction evaluations'), 'evaluation trace incomplete')
    need(persons == integer(counts['person_ticks'], 'all person ticks')
         and ports == integer(counts['port_person_ticks'], 'port person ticks'), 'finalized denominator omitted ticks')
    need(integer(observation['error_count'], 'error count') == 0 and integer(observation['unmatched_predictions'], 'unmatched predictions') == 0
         and all(integer(counts[name], name) == 0 for name in ('missing_prediction', 'pending_settlement', 'uncovered')),
         'shadow has missing/error/uncovered ticks')
    return retained, full


def adapter_rows(raw: Path, replay: Path) -> None:
    """Fixed actual adapter fixtures are separate from live read-trace replay."""
    seen = set()
    with raw.open() as left, replay.open() as right:
        for a, b in zip_longest(left, right):
            need(a is not None and b is not None, 'adapter replay count differs')
            row, result = json_line(a), json_line(b)
            fields(row, 'schema_version case_id input production host')
            freeze.version(row)
            case = row['case_id']
            need(case in CASES and case not in seen, 'unknown/duplicate adapter fixture')
            seen.add(case)
            need(type(row['input']) is dict and bool(row['input']), 'primitive adapter fixture inputs missing')
            actual(row['production'])
            actual(row['host'])
            need(canonical(row) == canonical(result), 'compiled actual adapter reconstruction differs')
            need(canonical(row['production']) == canonical(row['host']), 'production/host adapters disagree')
            need(row['host']['decision'] == ('continue' if case in RETAIN_CASES else 'evaluate'), 'fixture does not cover named admission outcome')
    need(seen == CASES, 'required primitive adapter scenarios missing')


def observed_command(repo: Path, row: freeze.Document, argv: list[str]) -> None:
    fields(row, 'argv exit_code started_utc finished_utc log')
    need(row['argv'] == argv and type(row['exit_code']) is int and row['exit_code'] == 0,
         'actual capture command/exit mismatch')
    start, finish = (datetime.fromisoformat(row[key]) for key in ('started_utc', 'finished_utc'))
    need(start.tzinfo is not None and finish.tzinfo is not None and start <= finish, 'capture timestamps invalid')
    freeze.read_reference(repo, row['log'])


def captures(repo: Path, contract: freeze.Document, manifest: freeze.Document, evidence: Path) -> dict[str, freeze.Document]:
    capture_path = freeze.read_reference(repo, contract['capture_receipt'])
    need(capture_path.is_relative_to(evidence), 'capture receipt outside pinned evidence tree')
    receipt = freeze.load(capture_path)
    fields(receipt, 'schema_version candidate_source_sha256 pre_source_sha256 post_source_sha256 pre_host_manifest_sha256 post_host_manifest_sha256 runs adapter_fixtures')
    freeze.version(receipt)
    need(receipt['candidate_source_sha256'] == receipt['pre_source_sha256'] == receipt['post_source_sha256']
         == contract['candidate_source_sha256'] and receipt['pre_host_manifest_sha256']
         == receipt['post_host_manifest_sha256'] == contract['host_manifest']['sha256'], 'capture source/host stamps differ')
    need(set(receipt['runs']) == {'off', 'on'} and set(contract['calibration']) == {'off', 'on'}, 'capture modes missing')
    reports = {}
    for mode in ('off', 'on'):
        need(set(receipt['runs'][mode]) == {'shadow', 'control', 'feature_off'}, 'missing observed/control/off capture')
        references = contract['calibration'][mode]
        fields(references, 'shadow control feature_off review')
        observed_path = freeze.read_reference(repo, references['shadow'])
        report = freeze.load(observed_path)
        reports[mode] = report
        need(report['source_sha'] == contract['candidate_commit'] and report['observe'] is True
             and [integer(value, 'calibration seed') for value in report['seeds']] == list(range(8))
             and integer(report['days'], 'calibration days') == 20 and report['coverage_complete'] is True,
             'raw calibration source/cohort differs')
        inputs = report['inputs']
        need(inputs['steward'] is (mode == 'on') and inputs['player_orders'] is False, 'calibration scripted/mode mismatch')
        workers = integer(report['workers'], 'workers')
        need(1 <= workers <= 4, 'unexpected calibration concurrency')
        flags = ['--scene', 'colony', '--seeds', '0..8', '--days', '20', '--workers', str(workers), '--steward', mode]
        if inputs['defence'] is not None:
            need(type(inputs['defence']) is bool, 'invalid defence flag')
            flags += ['--defence', 'on' if inputs['defence'] else 'off']
        need(inputs['story'] in ('on', 'off', 'no-beasts') and inputs['neighbours'] in ('on', 'off'), 'invalid world inputs')
        flags += ['--story', inputs['story'], '--neighbours', inputs['neighbours']]
        for role in ('shadow', 'control', 'feature_off'):
            binary = freeze.read_reference(repo, manifest['control' if role == 'feature_off' else 'shadow'])
            report_path = freeze.read_reference(repo, references[role])
            need(report_path.is_relative_to(evidence), 'calibration raw data outside pinned evidence tree')
            prefix = [] if role == 'feature_off' else ['mind-shadow', '--observe', 'on' if role == 'shadow' else 'off']
            process = receipt['runs'][mode][role]
            need(freeze.read_reference(repo, process['log']).is_relative_to(evidence), 'capture log outside raw tree')
            observed_command(repo, process, [str(binary), *prefix, *flags, '--out', str(report_path.parent)])
    fixture_path = freeze.read_reference(repo, contract['adapter_fixtures'])
    need(fixture_path.is_relative_to(evidence), 'adapter fixture raw data outside pinned tree')
    need(freeze.read_reference(repo, receipt['adapter_fixtures']['log']).is_relative_to(evidence), 'fixture capture log outside raw tree')
    observed_command(repo, receipt['adapter_fixtures'], [str(freeze.read_reference(repo, manifest['shadow'])),
                     'mind-adapter-fixtures', '--out', str(fixture_path)])
    return reports


def replay_command(binary: Path, args: list[str], log: Path) -> None:
    with log.open('x') as stream:
        subprocess.run([str(binary), *args], stdout=stream, stderr=subprocess.STDOUT, check=True)


def check(repo: Path, contract_path: Path, evidence: Path, output: Path) -> None:
    repo = freeze.canonical(repo)
    contract_path = freeze.external(repo, contract_path)
    contract_sha = freeze.digest(contract_path)
    contract = freeze.load(contract_path)
    fields(contract, 'schema_version kind candidate_commit candidate_source_sha256 frozen_archive frozen_manifest builder prepare_manifest adapter_contract host_build host_manifest calibration evidence_manifest capture_receipt adapter_fixtures')
    freeze.version(contract)
    need(contract['kind'] == 'p3d_actual_scheduling_host_parity', 'wrong host parity contract')
    commit = contract['candidate_commit']
    prior.run(repo, 'git', 'merge-base', '--is-ancestor', commit, 'HEAD')
    need(prior.git(repo, 'rev-parse', 'HEAD:rust') == prior.git(repo, 'rev-parse', commit + ':rust'), 'host checker publication Rust differs')
    names = prior.git(repo, 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', 'rust').split(b'\0')
    inventory = {os.fsdecode(name): freeze.digest(repo / os.fsdecode(name)) for name in set(names) if name}
    prior.committed_inventory(repo, commit, inventory)
    need(prior.source_digest(inventory) == contract['candidate_source_sha256'], 'host checker source not committed candidate')
    from core_p3d_successor import committed_tools
    expected_tools = committed_tools(repo, commit)
    tool_names = prior.git(repo, 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', 'tools/v8').split(b'\0')
    live_tools = {os.fsdecode(name): freeze.digest(repo / os.fsdecode(name)) for name in set(tool_names) if name}
    need(live_tools == expected_tools, 'host verifier tools differ from committed candidate')
    evidence = freeze.external(repo, evidence)
    need(prior.pinned_tree(repo, contract['evidence_manifest']) == evidence, 'raw evidence tree differs')
    output = freeze.external(repo, output)
    need(not output.exists() and not output.is_relative_to(evidence), 'new output outside raw evidence required')
    work = output.with_name(output.name + '.verification')
    work.mkdir(parents=True, exist_ok=False)
    try:
        _, manifest, _ = reconstruction(repo, contract)
        reports = captures(repo, contract, manifest, evidence)
        binary = freeze.read_reference(repo, manifest['shadow'])
        retained = full = 0
        journal = {}
        for mode in ('off', 'on'):
            worlds = reports[mode]['worlds']
            need([integer(row['seed'], 'world seed') for row in worlds] == list(range(8)), 'raw report worlds missing/duplicated')
            observed = freeze.read_reference(repo, contract['calibration'][mode]['shadow']).parent
            need({p.name for p in observed.glob('guard_s*.bin')} == {f'guard_s{seed}.bin' for seed in range(8)}, 'missing/extra live binary traces')
            for world in worlds:
                seed = world['seed']
                need(integer(world['end_tick'], 'world end') == integer(world['expected_ticks'], 'world expected ticks') == 20 * 1440, 'world truncated before expected tick')
                capture = observed / f'guard_s{seed}.bin'
                freeze.canonical(capture)
                raw, replay = (work / f'{mode}-{seed}-{suffix}.jsonl' for suffix in ('raw', 'replay'))
                before = freeze.digest(capture)
                replay_command(binary, ['mind-guard-replay', '--input', str(capture), '--raw', str(raw), '--out', str(replay)], work / f'{mode}-{seed}.log')
                a, b = live_rows(raw, replay, mode, seed, world['observation'])
                retained += a
                full += b
                need(freeze.digest(capture) == before, 'raw binary trace changed during replay')
                journal[f'{mode}:{seed}'] = {'binary_trace_sha256': before, 'raw_sha256': freeze.digest(raw),
                                            'replay_sha256': freeze.digest(replay), 'retained_port_ticks': a, 'full_port_ticks': b,
                                            'evaluations': world['observation']['counts']['evaluations'],
                                            'person_ticks': world['observation']['counts']['person_ticks'],
                                            'port_person_ticks': world['observation']['counts']['port_person_ticks']}
                # Original binary trace is durable; these successful exports are reproducible scratch.
                raw.unlink()
                replay.unlink()
        fixtures = freeze.read_reference(repo, contract['adapter_fixtures'])
        regenerated = work / 'adapter-regenerated.jsonl'
        replay_command(binary, ['mind-adapter-fixtures', '--out', str(regenerated)], work / 'adapter-regenerated.log')
        adapter_rows(fixtures, regenerated)
        replay = work / 'adapter-replay.jsonl'
        replay_command(binary, ['mind-adapter-replay', '--input', str(fixtures), '--out', str(replay)], work / 'adapter-replay.log')
        adapter_rows(fixtures, replay)
        need(retained > 0 and full > 0, 'no executed retained/full scheduling coverage')
        reconstruction(repo, contract)
        captures(repo, contract, manifest, evidence)
        prior.pinned_tree(repo, contract['evidence_manifest'])
        freeze.pin(contract_path, contract_sha)
        final_tool_names = prior.git(repo, 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', 'tools/v8').split(b'\0')
        need({os.fsdecode(name): freeze.digest(repo / os.fsdecode(name)) for name in set(final_tool_names) if name} == expected_tools, 'verifier tools changed during replay')
        with (work / 'replay-digests.json').open('x') as stream:
            json.dump(journal, stream, indent=2, sort_keys=True)
        result = {'schema_version': 1, 'kind': 'executed_scheduling_adapter_parity',
                  'candidate_source_sha256': contract['candidate_source_sha256'],
                  'host_manifest_sha256': contract['host_manifest']['sha256'], 'contract_sha256': contract_sha,
                  'raw_evidence_manifest_sha256': contract['evidence_manifest']['sha256'],
                  'calibration': contract['calibration'], 'compared_person_ticks': retained + full,
                  'retained_person_ticks': retained, 'full_person_ticks': full, 'mismatches': 0}
        with output.open('x') as stream:
            json.dump(result, stream, indent=2, sort_keys=True, allow_nan=False)
            stream.write('\n')
    except (OSError, ValueError, KeyError, TypeError, freeze.FreezeError, subprocess.SubprocessError) as error:
        with (work / 'failure.json').open('x') as stream:
            json.dump({'status': 'INVALID', 'error': str(error)}, stream)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--contract', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    check(Path(__file__).resolve().parents[2], args.contract, args.evidence, args.output)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
