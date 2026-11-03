"""Pure standalone case accounting; no source/build, runtime, or world admission."""
import hashlib
import json
import re
import shlex
import tomllib
from collections import Counter
from dataclasses import dataclass
from datetime import datetime
from itertools import pairwise
from pathlib import Path
from typing import Literal

from gate_rustdoc_exec import match_invocation, policy_for
from gate_test_summary import parse_log
from generic_gate import document, integer, require
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from generic_workspace_capture import CapturePlan, read, reconcile_capture, snapshot
from generic_workspace_tests import Executable, Target
from mux2_gate_inputs import Json, Node


@dataclass(frozen=True, slots=True)
class Listing:
    names: tuple[str, ...]
    benchmarks: tuple[str, ...]
    sha256: str


@dataclass(frozen=True, slots=True)
class Outcome:
    name: str
    status: str
    mode: str | None


@dataclass(frozen=True, slots=True)
class NestedChild:
    parent: str
    name: str
    count: int


@dataclass(frozen=True, slots=True)
class NestedAccounting:
    outer_cases: int
    nested_passed: int
    children: tuple[NestedChild, ...]
    source_files: tuple[tuple[str, str], ...]
    source_blocks: tuple[tuple[str, str], ...]
    pid_tree: Literal['NOT_OBSERVED'] = 'NOT_OBSERVED'
    attribution: Literal['SOURCE_BOUND_AGGREGATE_ONLY'] = 'SOURCE_BOUND_AGGREGATE_ONLY'


@dataclass(frozen=True, slots=True)
class NestedSource:
    spec: ProcessSpec
    executable: Executable
    files: tuple[tuple[str, str], ...]
    blocks: tuple[tuple[str, str], ...]
    children: tuple[NestedChild, ...]


@dataclass(frozen=True, slots=True)
class Section:
    outcomes: tuple[Outcome, ...]
    sha256: str
    failed: int
    nested: NestedAccounting | None = None


@dataclass(frozen=True, slots=True)
class ListingProcess:
    spec: ProcessSpec
    result: ProcessResult
    receipt_sha256: str


@dataclass(frozen=True, slots=True)
class CaseLists:
    invocation: str
    all_cases: ListingProcess
    ignored_cases: ListingProcess


@dataclass(frozen=True, slots=True)
class JoinedSection:
    invocation: str
    package_id: str
    source: str
    start: int
    end: int
    section: Section


@dataclass(frozen=True, slots=True)
class CaseAccounting:
    sections: tuple[JoinedSection, ...]
    errors: tuple[str, ...]
    status: str
    native_outcome: str
    stdout_sha256: str
    stderr_sha256: str
    unresolved_start: int
    source_build_admission: str = 'CALLER_AUTHENTICATED_BUILD_REQUIRED'
    world_safety: str = 'INCOMPLETE'
    observed_ranges: tuple[tuple[int, int, str], ...] = ()


def digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def lines(raw: bytes) -> list[str]:
    text = raw.decode('utf-8')
    require(bool(raw) and raw.endswith(b'\n'), '/text', 'empty or truncated output')
    require(all(char == '\n' or ord(char) >= 32 and ord(char) != 127 for char in text), '/text', 'control/CRLF output unsupported')
    return text.splitlines()


def parse_listing(raw: bytes) -> Listing:
    records = lines(raw)
    match = re.fullmatch(r'(\d+) (test|tests), (\d+) (benchmark|benchmarks)', records[-1])
    require(match is not None, '/listing', 'missing exact summary')
    assert match is not None
    count, benches = int(match[1]), int(match[3])
    require(match[2] == ('test' if count == 1 else 'tests') and match[4] == ('benchmark' if benches == 1 else 'benchmarks'), '/listing', 'wrong plural')
    body = records[:-1]
    if count + benches:
        require(bool(body) and body[-1] == '', '/listing', 'missing summary separator')
        body = body[:-1]
    names: list[str] = []; benchmarks: list[str] = []
    for row in body:
        name, separator, kind = row.rpartition(': ')
        require(bool(name) and bool(separator) and kind in ('test', 'benchmark'), '/listing', 'invalid name/type')
        (names if kind == 'test' else benchmarks).append(name)
    require(len(set(names + benchmarks)) == len(names + benchmarks) and (len(names), len(benchmarks)) == (count, benches), '/listing', 'duplicate or mismatched count')
    return Listing(tuple(names), tuple(benchmarks), digest(raw))


def parse_section(raw: bytes, names: tuple[str, ...], ignored: tuple[str, ...]) -> Section:
    records = lines(raw)
    require(len(set(names)) == len(names) and len(set(ignored)) == len(ignored) and set(ignored) <= set(names), '/cases', 'invalid independent inventory')
    require(len(records) >= 4 and records[0] == '' and records[-1] == '', '/section', 'missing exact boundaries')
    require(records[1] == f'running {len(names)} '+('test' if len(names) == 1 else 'tests'), '/section', 'running count differs')
    summary = re.fullmatch(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in (\d+(?:\.\d+)?)s', records[-2])
    require(summary is not None, '/section', 'missing exact summary')
    assert summary is not None
    outcomes: list[Outcome] = []
    body = records[2:-2]
    require(bool(body) and body[-1] == '', '/section', 'missing summary separator')
    for row in body[:-1]:
        candidates: list[Outcome] = []
        for name in names:
            prefix = 'test '+name
            if row.startswith(prefix+' has been running for over ') and re.fullmatch(r'\d+ seconds', row[len(prefix+' has been running for over '):]):
                candidates.append(Outcome(name, 'progress', None))
            for mode in (None, 'compile fail', 'compile', 'should panic'):
                start = prefix + ('' if mode is None else ' - '+mode) + ' ... '
                if row.startswith(start):
                    status = row[len(start):]
                    if status in ('ok', 'FAILED', 'ignored') or status.startswith('ignored, '):
                        candidates.append(Outcome(name, status.split(',', 1)[0], mode))
        require(len(candidates) == 1, '/section', 'unknown, ambiguous or unsupported output record')
        if candidates[0].status != 'progress': outcomes.append(candidates[0])
    actual = [row.name for row in outcomes]
    require(len(actual) == len(set(actual)) and set(actual) == set(names), '/section', 'missing/extra/duplicate case')
    require({row.name for row in outcomes if row.status == 'ignored'} == set(ignored), '/section', 'ignored set differs')
    counts = tuple(sum(row.status == status for row in outcomes) for status in ('ok', 'FAILED', 'ignored'))
    require(tuple(int(summary[i]) for i in (2, 3, 4)) == counts and (summary[5], summary[6]) == ('0', '0'), '/section', 'numeric accounting differs')
    require(summary[1] == ('FAILED' if counts[1] else 'ok'), '/section', 'status differs')
    return Section(tuple(outcomes), digest(raw), counts[1])


# Source-owned parent/child test contracts. Other gameplay source is not version-pinned.
_NESTED_RULES: tuple[tuple[str,str,str,int,str,str], ...] = (
    ('content/tests.rs','colony::content::tests::bootstrap_negative_cases_run_in_fresh_processes',
     'colony::content::tests::bootstrap_child',4,
     'b70113a4ff4deb454fb183e88ce0293d7bb8c5ce1e90f5eb398c6f1d5fe5da7a',
     '1f08871e132cc6ec3e1127a3f6e1cdff0f17700009581d1f1e70584c30dddac2'),
    ('content/ecology_tests.rs','colony::content::ecology_tests::ecology_id_bootstrap_boundaries_are_isolated',
     'colony::content::ecology_tests::ecology_id_child',2,
     '3a8714172edcba97cf35a4324914aa3a9e9b2e02bbb146bfb4187a36c3852780',
     '98c299cca4535300dca3d001e826796904319161367101e0caaa6d32e8898765'),
    ('knowledge/registry_tests.rs','colony::knowledge::registry_tests::local_catalog_and_knowledge_decode_are_bootstrap_isolated',
     'colony::knowledge::registry_tests::uninstalled_child',1,
     'cd2c481a31df94b120a309aa3a53715cea1774d5c7639e8dd2aa060550c0c8f4',
     '7a0d227cb05cb8764e77041a863a53c68b29f5cf777cdd19c9a7dfe637be0aa3'),
)
_NESTED_TARGETS: dict[tuple[str, str], tuple[str,str,str,int,str,str,tuple[str,...]]] = {
    ('sim-core', 'recipe_registry'): ('rust/crates/sim-core/tests/recipe_registry.rs', 'steward_rule_parsing_requires_boot_without_poisoning_later_rules', 'steward_rule_boot_child', 1,
        '14bb0633145b88f4653c15f264fc062d785ef40d399a02ba6c189861c112bc45', 'eb8a1c6ed8b1e79c24ce07ff55bb082392ee09fdc79d5c8a19e1351f5b62de17', ()),
    ('sim-test', 'b5_flora'): ('rust/crates/sim-test/tests/b5_flora.rs', 'actual_host_distribution_replacement_runs_in_fresh_process', 'host_pine_child', 1,
        '7598c83a43049721c906e3f440fa2a15f347d5be16bde35d223899ba0eecf812', 'cc18506f30a98a6dd7b5ed75c835b9e8bd2b638ee86bf7f003c07764168f7c8e', ()),
    ('sim-test', 'blueprint_completion_identity'): ('rust/crates/sim-test/tests/blueprint_completion_identity.rs', 'novel_same_art_layers_survive_save_and_upper_layer_demolition', 'novel_same_art_layers_survive_save_and_upper_layer_demolition', 1,
        'a1a384eaa234715c231e3a498b2d0d656d91e20efba9f384287a88cfbe4fc23c', 'a1a384eaa234715c231e3a498b2d0d656d91e20efba9f384287a88cfbe4fc23c', ()),
    ('sim-test', 'data_only_ext'): ('rust/crates/sim-test/tests/data_only_ext.rs', 'selected_pack_runs_all_simulation_families_and_rejects_bad_boundaries', 'selected_pack_child', 9,
        'b5dc52c185bff0aae8ddbce84089f2510c077ff4441c31d880345c37785b8ea2', '93511563c114a2671d3fe735cd25628248daff314ce50401d6717d3374cbdfa2', ('const CHILD: &str = "WORLDSIM_DATA_ONLY_CASE";',)),
    ('sim-test', 'predator_registry_runtime'): ('rust/crates/sim-test/tests/predator_registry_runtime.rs', 'novel_predator_pack_runs_in_an_isolated_catalogue_process', 'novel_predator_pack_child', 1,
        'a844472f9451377022e8ea3e75c954b74025a9e49c4379ef89b52c0cdc9fdde8', 'da0bf9f4989e6552e075ad1f9bdf52746d11aa1d6f11b5f6c5e37a4f807e41e8', ('const CHILD: &str = "WORLDSIM_PREDATOR_REGISTRY_CHILD";',)),
    ('sim-test', 'recipe_registry_runtime'): ('rust/crates/sim-test/tests/recipe_registry_runtime.rs', 'alternate_cutter_runs_in_an_isolated_catalogue_process', 'alternate_cutter_child', 1,
        'af88d93b81cb083a9ca8ee15e4d5697a7f7e2c2b2429d2cbeb899f89bf11ce6e', 'c08d3c8f85ea5b008defa81f9102c2bd6d4c5b7114eec0b9fdc47b4a646e280d', ()),
}
_NESTED_ENV: dict[tuple[str,str],tuple[str,...]] = {
    ('sim-core','sim_core'): ('WORLDSIM_CONTENT_TEST_CASE','WORLDSIM_ECOLOGY_TEST_CASE','WORLDSIM_KNOWLEDGE_TEST_CHILD'),
    ('sim-core','recipe_registry'): ('WORLDSIM_STEWARD_BOOT_CHILD',),
    ('sim-test','b5_flora'): ('WORLDSIM_B5_HOST_CHILD',),
    ('sim-test','blueprint_completion_identity'): ('WORLDSIM_BLUEPRINT_LAYER_TEST_CHILD',),
    ('sim-test','data_only_ext'): ('WORLDSIM_DATA_ONLY_CASE',),
    ('sim-test','predator_registry_runtime'): ('WORLDSIM_PREDATOR_REGISTRY_CHILD',),
    ('sim-test','recipe_registry_runtime'): ('WORLDSIM_RECIPE_CUTTER_CHILD',),
}
_NATIVE_SUMMARY = re.compile(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in (\d+(?:\.\d+)?)s')


def nested_native_source(spec: ProcessSpec, executable: Executable) -> NestedSource | None:
    """Admit fixed test blocks from actual upstream-authenticated process inputs."""
    target=executable.target
    root=spec.input_root
    constants: tuple[str,...]=()
    if (target.package,target.name,target.kinds)==('sim-core','sim_core',('lib',)):
        rules=tuple(('rust/crates/sim-core/src/colony/'+row[0],*row[1:]) for row in _NESTED_RULES)
        source=root/'rust/crates/sim-core/src/lib.rs'
    else:
        admitted=_NESTED_TARGETS.get((target.package,target.name))
        if admitted is None or target.kinds!=('test',):return None
        rules=(admitted[:6],)
        constants=admitted[6]
        source=root/admitted[0]
    paths=tuple(root/rule[0] for rule in rules)
    require(not any(key in environment_for(spec) for key in _NESTED_ENV[(target.package,target.name)]),
            '/nested/environment','outer invocation already selects child branch')
    # Synthetic/other source without this test family keeps strict single-summary accounting.
    if not any(path.exists() for path in paths):return None
    require(root==root.resolve() and target.source==source
            and target.manifest==root/'rust/crates'/target.package/'Cargo.toml', '/nested/source','native source identity differs')
    require(executable.step in (4,5) and executable.profile==('debug' if executable.step==4 else 'release')
            and executable.filter is None and executable.ignored=='exclude' and executable.threads==8,
            '/nested/source','original native selection differs')
    expected=(str(spec.executable),'test','--workspace',*(('--release',) if executable.step==5 else ()),
              '--no-fail-fast','--','--test-threads=8')
    require(spec.argv==expected and spec.cwd==root/'rust' and spec.timeout_seconds is None,
            '/nested/source','original parent invocation differs')
    require(executable.binary.parent==root/'rust/target'/executable.profile/'deps',
            '/nested/source','native binary location differs')
    bound={str(root/name):value for name,value in spec.inputs}
    require(len(bound)==len(spec.inputs),'/nested/source','duplicate process input')
    for path,value in spec.external_inputs:
        require(str(path) not in bound,'/nested/source','duplicate external binding')
        bound[str(path)]=value
    files: list[tuple[str,str]]=[];blocks: list[tuple[str,str]]=[]
    for path in (*paths,target.source,target.manifest,executable.binary):
        require(path==path.resolve() and bound.get(str(path))==file_hash(path),'/nested/source','unbound or changed source/binary')
    for path,rule in zip(paths,rules,strict=True):
        raw=path.read_bytes();files.append((str(path),digest(raw)))
        for declaration in constants:
            require(raw.decode('utf-8').splitlines().count(declaration)==1, '/nested/source', 'nested environment constant changed')
            blocks.append((declaration,digest(declaration.encode())))
        for name,expected_hash in ((rule[1],rule[4]),(rule[2],rule[5])):
            marker=('#[test]\nfn '+name.rsplit('::',1)[-1]+'() {').encode()
            require(raw.count(marker)==1,'/nested/source','missing/ambiguous test block')
            start=raw.index(marker);end=raw.find(b'\n#[test]',start+len(marker))
            block=raw[start:end if end>=0 else len(raw)].rstrip()+b'\n'
            require(digest(block)==expected_hash,'/nested/source','nested test command/body contract changed')
            blocks.append((name,expected_hash))
        require(file_hash(path)==bound[str(path)],'/nested/source','source changed during admission')
    return NestedSource(spec,executable,tuple(files),tuple(blocks),tuple(NestedChild(row[1],row[2],row[3]) for row in rules))


def reconcile_nested_native(raw: bytes, all_cases: Listing, ignored: Listing, source: NestedSource) -> Section:
    """Reconcile known aggregate multiplicities; interleaving supplies no child PID tree."""
    require(nested_native_source(source.spec,source.executable)==source,'/nested/source','source policy changed')
    records=lines(raw);count=len(all_cases.names)
    children=source.children; nested_count=sum(child.count for child in children)
    require(len(records)>=4 and records[0]==records[-1]=='','/nested','truncated outer framing')
    for row in records:
        if not row or row.startswith(('running ','test result:')): continue
        outcome=re.fullmatch(r'test (\S+) \.\.\. (?:ok|FAILED|ignored(?:, [^\n]*)?)',row)
        progress=re.fullmatch(r'test (\S+) has been running for over \d+ seconds',row)
        matched=outcome if outcome is not None else progress
        require(matched is not None and matched[1] in all_cases.names,
                '/nested','unsupported, unlisted or ambiguous native record')
    require(count>1 and len(set(all_cases.names))==count and len(set(ignored.names))==len(ignored.names),
            '/nested','invalid independent listing')
    headers=[row for row in records if row.startswith('running ')]
    require(headers==[f'running {count} tests',*(['running 1 test']*nested_count)],'/nested','outer/nested running counts differ')
    nonempty=[row for row in records if row]
    summary_rows=[row for row in records if row.startswith('test result:')]
    summaries=[_NATIVE_SUMMARY.fullmatch(row) for row in summary_rows]
    require(len(summaries)==nested_count+1 and all(row is not None for row in summaries),'/nested','summary framing/count differs')
    outer=_NATIVE_SUMMARY.fullmatch(nonempty[-1])
    require(outer is not None and summary_rows[-1]==nonempty[-1],'/nested','missing terminal outer summary')
    assert outer is not None
    for child in summaries[:-1]:
        assert child is not None
        require(tuple(child[i] for i in range(1,7))==('ok','1','0','0','0',str(count-1)),
                '/nested','nested summary is not one successful exact selected test')
    summary=parse_log(raw.decode('utf-8'))
    counts=Counter(row.name for row in summary.outcomes)
    expected=Counter({name:1 for name in all_cases.names})
    for child in children:
        require(child.parent in expected and child.name in expected and child.parent not in ignored.names
                and child.name not in ignored.names,'/nested','parent/child listing missing or ignored')
        expected[child.name]+=child.count
    require(counts==expected,'/nested','unknown/missing/extra repeated outcomes')
    statuses: dict[str,str]={}
    protected={name for row in children for name in (row.parent,row.name)}
    for outcome in summary.outcomes:
        require(outcome.name not in statuses or statuses[outcome.name]==outcome.status,'/nested','ambiguous duplicate status')
        require(outcome.name not in protected or outcome.status=='ok','/nested','nested parent/child failed')
        statuses[outcome.name]=outcome.status
    outcomes=tuple(Outcome(name,statuses[name],None) for name in all_cases.names)
    require({row.name for row in outcomes if row.status=='ignored'}==set(ignored.names),'/nested','ignored inventory differs')
    values=tuple(sum(row.status==status for row in outcomes) for status in ('ok','FAILED','ignored'))
    require(tuple(int(outer[i]) for i in (2,3,4))==values and (outer[5],outer[6])==('0','0')
            and outer[1]==('FAILED' if values[1] else 'ok'),'/nested','outer numeric accounting differs')
    require(summary.summaries==nested_count+1 and summary.measured==0 and summary.filtered_out==nested_count*(count-1),
            '/nested','aggregate numeric accounting differs')
    require(nested_native_source(source.spec,source.executable)==source,'/nested/source','source changed during accounting')
    return Section(outcomes,digest(raw),values[1],NestedAccounting(count,nested_count,children,source.files,source.blocks))


def reconcile_native(raw: bytes, all_cases: Listing, ignored_cases: Listing, nested: NestedSource | None = None) -> Section:
    require(not all_cases.benchmarks and not ignored_cases.benchmarks and set(ignored_cases.names) <= set(all_cases.names), '/native', 'unsupported benchmark or ignored inventory')
    if nested is not None:return reconcile_nested_native(raw,all_cases,ignored_cases,nested)
    summary = parse_log(raw.decode('utf-8'))
    names = [row.name for row in summary.outcomes]
    require(summary.summaries == 1 and len(names) == len(set(names)) and set(names) == set(all_cases.names), '/native', 'case inventory differs')
    require({row.name for row in summary.outcomes if row.status == 'ignored'} == set(ignored_cases.names) and
            summary.measured == 0 and summary.filtered_out == 0, '/native', 'selection differs')
    return Section(tuple(Outcome(row.name, row.status, None) for row in summary.outcomes), digest(raw), summary.failed)


def flattened_test_args(argv: tuple[str, ...]) -> tuple[str, ...]:
    result: list[str] = []; index = 0
    while index < len(argv):
        arg = argv[index]
        if arg == '--test-args':
            index += 1
            require(index < len(argv), '/argv', 'missing test-args value')
            result.extend(argv[index].split())
        elif arg.startswith('--test-args='):
            result.extend(arg.split('=', 1)[1].split())
        index += 1
    require(tuple(result) == ('--test-threads=8',), '/argv', 'original test selection differs')
    return tuple(result)


def listing_output(binding: ListingProcess, argv: tuple[str, ...], cwd: Path,
                   environment: dict[str, str], required: dict[str, str]) -> bytes:
    require(binding.result.status == 'PASS' and binding.result.native_exit == 0,
            '/listing-process', 'listing did not succeed')
    return process_output(binding, argv, cwd, environment, required)


def process_output(binding: ListingProcess, argv: tuple[str, ...], cwd: Path,
                   environment: dict[str, str], required: dict[str, str]) -> bytes:
    spec, result = binding.spec, binding.result
    require(spec.argv == argv and spec.cwd == cwd and spec.timeout_seconds is None and environment_for(spec) == environment,
            '/listing-process', 'argv/cwd/environment/timeout differs')
    require(result.receipt == spec.output_root/'process.json' and file_hash(result.receipt) == binding.receipt_sha256,
            '/listing-process', 'receipt differs')
    row = read(result.receipt)
    require(type(result.native_exit) is int and type(result.pid) is int and result.pid > 0, '/process', 'actual outcome/PID required')
    status = 'PASS' if result.native_exit == 0 else 'FAIL'
    require(result.status == status, '/process', 'native outcome/status differs')
    expected: dict[str, Json] = {'schema_version': 1, 'status': status, 'native_exit': result.native_exit, 'pid': result.pid,
        'errors': [] if result.native_exit == 0 else [f'native exit {result.native_exit}'],
        'argv': list(argv), 'cwd': str(cwd), 'environment': {k: v for k, v in environment.items()},
        'environment_policy': 'fixed_defaults_plus_explicit_only_no_inheritance', 'executable': str(spec.executable),
        'input_root': str(spec.input_root), 'output_root': str(spec.output_root), 'fresh_directories': list(spec.fresh_directories)}
    require(required.get(str(spec.executable)) == spec.executable_sha256, '/listing-process', 'wrong required tool hash')
    require(argv[0] == str(spec.executable), '/listing-process', 'entry executable differs')
    for key, value in expected.items(): row.field(key).equals(value)
    require('timed_out' not in row.mapping() and 'timeout_seconds' not in row.mapping(), '/listing-process', 'changed timeout policy')
    for prefix in ('expected_', 'before_', 'after_'): row.field(prefix+'executable_sha256').equals(spec.executable_sha256)
    bound: dict[str, str] = {str(spec.executable): spec.executable_sha256}
    for suffix, pairs in (('inputs', spec.inputs), ('external_inputs', tuple((str(p), h) for p, h in spec.external_inputs))):
        values = dict(pairs)
        require(len(values) == len(pairs), '/listing-process', 'duplicate binding')
        for prefix in ('expected_', 'before_', 'after_'): row.field(prefix+suffix).equals({k: v for k, v in values.items()})
        for name, value in values.items(): bound[str(spec.input_root/name) if suffix == 'inputs' else name] = value
    require(all(bound.get(name) == value for name, value in required.items()), '/listing-process', 'required source/exec/tool binding missing')
    for name, value in bound.items(): require(file_hash(Path(name)) == value, '/listing-process', 'input changed')
    spawn = read(spec.output_root/'process-spawn.json')
    spawn.field('pid').equals(result.pid)
    times = tuple(datetime.fromisoformat(value) for value in (row.field('started_at').text(), spawn.field('spawned_at').text(), row.field('ended_at').text()))
    require(all(value.tzinfo is not None and value.utcoffset() is not None for value in times), '/listing-process', 'timezone missing')
    require(times[0] <= times[1] <= times[2], '/listing-process', 'start/spawn/end chronology differs')
    duration = row.field('duration_ns').value
    require(type(duration) is int and duration > 0, '/listing-process', 'positive nonboolean duration required')
    row.field('build_provenance_verified').equals(False)
    start = read(spec.output_root/'process-start.json')
    for key in (*[key for key in expected if key not in ('status', 'native_exit', 'pid', 'errors')], 'started_at',
                'expected_executable_sha256', 'before_executable_sha256', 'expected_inputs', 'before_inputs',
                'expected_external_inputs', 'before_external_inputs', 'build_provenance_verified'):
        start.field(key).equals(row.field(key).value)
    start_values: dict[str, Json] = {'pid': None, 'native_exit': None, 'status': 'FAIL', 'after_external_inputs': {}, 'errors': []}
    for key, value in start_values.items():
        start.field(key).equals(value)
    require(not {'ended_at', 'duration_ns'} & start.mapping().keys(), '/listing-process', 'start contains terminal completion')
    for name in ('stdout', 'stderr'): row.field(name+'_sha256').equals(file_hash(spec.output_root/(name+'.log')))
    return (spec.output_root/'stdout.log').read_bytes()


def declarations(raw: bytes, executable: str, static_commands: tuple[tuple[str, ...], ...] = ()) -> tuple[tuple[str, ...], ...]:
    prefixes = {executable, *(row[0] for row in static_commands)}
    require(all(re.fullmatch(r'/[A-Za-z0-9_./+@-]+', prefix) is not None for prefix in prefixes), '/declarations', 'ambiguous executable path')
    result: list[tuple[str, ...]] = []; observed_static: list[tuple[str, ...]] = []
    for raw_line in raw.splitlines(keepends=True):
        if b'Running `' not in raw_line:
            require(executable.encode() not in raw_line, '/declarations', 'malformed executable display')
            continue
        line = raw_line.decode('utf-8').removesuffix('\n')
        require(all(ord(c) >= 32 and ord(c) != 127 for c in line) and re.fullmatch(r' +Running `.*`', line) is not None,
                '/declarations', 'unsupported display framing')
        command = line.lstrip()[9:-1]
        matches = [prefix for prefix in prefixes if command.startswith(prefix+' ')]
        require(len(matches) == 1, '/declarations', 'unknown/ambiguous executable prefix')
        prefix = matches[0]
        args = tuple(shlex.split(command[len(prefix)+1:]))
        if prefix == executable:
            require('--test' in args and args not in result, '/declarations', 'duplicate or non-test invocation')
            result.append(args)
        else:
            require(not result, '/declarations', 'static command after doctests')
            observed_static.append((prefix, *args))
    require(tuple(observed_static) == static_commands, '/declarations', 'static execution declarations differ')
    return tuple(result)


def join_standalone_sections(plan: CapturePlan, parent: ProcessResult, listings: tuple[CaseLists, ...]) -> CaseAccounting:
    joined: list[JoinedSection] = []; errors: list[str] = []; offset = 0
    raw_ranges: list[tuple[int, int, str]] = []
    stdout = b''; stderr = b''; frozen: dict[Path, dict[str, str]] = {}
    try:
        capture = reconcile_capture(plan, parent)
        require(capture.integrity == 'PASS' and capture.completeness == 'COMPLETE', '/capture', 'capture incomplete or invalid')
        stdout = (plan.parent_spec.output_root/'stdout.log').read_bytes()
        stderr = (plan.parent_spec.output_root/'stderr.log').read_bytes()
        manifest = read(plan.manifest); policy = policy_for(manifest)
        require(integer(policy.field('schema_version')) in (2, 3, 4), '/policy', 'schema2/3/4 required')
        for root in (plan.parent_spec.output_root, plan.capture_root, Path(policy.field('exec_root').text()),
                     *(item.spec.output_root for pair in listings for item in (pair.all_cases, pair.ignored_cases))):
            frozen[root] = snapshot(root)
        # Exact serial native bytes precede doctests; no timestamp or suffix search.
        prefix = b''.join(row.receipt.with_name('stdout.log').read_bytes() for row in sorted(capture.targets, key=lambda row: row.sequence))
        require(stdout.startswith(prefix), '/prefix', 'static stdout prefix differs')
        offset = len(prefix)
        environment = environment_for(plan.parent_spec)
        runners = [value for key, value in environment.items() if key.startswith('CARGO_TARGET_') and key.endswith('_RUNNER')]
        require(len(runners) == 1, '/runner', 'exactly one bound host runner required')
        runner = tuple(shlex.split(runners[0]))
        require(len(runner) == 2 and file_hash(Path(runner[0])) == manifest.field('python_sha256').text() and
                file_hash(Path(runner[1])) == manifest.field('launcher_sha256').text(), '/runner', 'runner bindings differ')
        static_commands = tuple((*runner, str(item.executable.binary), '--test-threads=8') for item in sorted(capture.targets, key=lambda item: item.sequence))
        ordered = declarations(stderr, policy.field('entrypoint').text(), static_commands)
        rows = tuple(match_invocation(manifest, policy, args, plan.parent_spec.cwd) for args in ordered)
        ids = tuple(row.field('id').text() for row in rows)
        require(len(ids) == len(set(ids)) and set(ids) == {row.field('id').text() for row in policy.field('invocations').array()}, '/order', 'declarations differ from obligations')
        expected = {item.invocation: item for item in listings}
        require(len(expected) == len(listings) and set(expected) == set(ids), '/listings', 'missing/duplicate/extra expected inventory')
        boundary = offset
        for match in re.finditer(rb'\ntest result: [^\n]*\n\n', stdout[offset:]):
            end = offset+match.end()
            raw_ranges.append((boundary, end, digest(stdout[boundary:end])))
            boundary = end
        require(len(raw_ranges) == len(rows) and boundary == len(stdout), '/sections', 'ordinal cardinality/framing incomplete; all ranges unassigned')
        for row, actual_args in zip(rows, ordered, strict=True):
            identity = row.field('id').text()
            row.field('edition').equals('2021')
            args = actual_args if integer(policy.field('schema_version')) == 4 else tuple(arg.text() for arg in row.field('argv').array())
            _ = flattened_test_args(args)
            require(not any(arg in ('--nocapture', '--no-capture', '--show-output') for arg in args), '/mode', 'changed standalone mode')
            intents = [item for item in capture.exec_intents if item.source_invocation == identity]
            require(len(intents) == 1, '/exec', 'missing unique actual compiler invocation')
            intent = read(intents[0].receipt)
            if integer(policy.field('schema_version')) == 4:
                require(file_hash(intents[0].receipt) == intents[0].receipt_sha256, '/exec', 'intent changed before accounting')
                intent.field('rustdoc_argv').equals([policy.field('rustdoc').text(), *args])
                intent.field('cwd').equals(str(plan.parent_spec.cwd))
            environment = {key: intent.field('environment').field(key).text() for key in intent.field('environment').mapping()}
            required = {key: manifest.field('inputs').field(key).text() for key in manifest.field('inputs').mapping()}
            required.update({str(plan.manifest): plan.manifest_sha256, str(intents[0].receipt): intents[0].receipt_sha256})
            required.update({item.field('path').text(): item.field('sha256').text() for item in manifest.field('artifacts').array()})
            real_argv = (policy.field('rustdoc').text(), *args)
            pair = expected[identity]
            all_raw = listing_output(pair.all_cases, (*real_argv, '--test-args', '--list'), Path(row.field('compiler_cwd').text()), environment, required)
            ignored_raw = listing_output(pair.ignored_cases, (*real_argv, '--test-args', '--list', '--test-args', '--ignored'), Path(row.field('compiler_cwd').text()), environment, required)
            all_cases, ignored_cases = parse_listing(all_raw), parse_listing(ignored_raw)
            require(not all_cases.benchmarks and not ignored_cases.benchmarks, '/doctests', 'benchmark listing unsupported')
            # One standalone summary per serial invocation. Ambiguous diagnostics fail parsing.
            terminal = re.search(rb'\ntest result: [^\n]*\n\n', stdout[offset:])
            require(terminal is not None, '/section', 'missing terminal section; later records not shifted')
            assert terminal is not None
            end = offset+terminal.end()
            section = parse_section(stdout[offset:end], all_cases.names, ignored_cases.names)
            joined.append(JoinedSection(identity, row.field('package_id').text(), row.field('source').text(), offset, end, section))
            offset = end
        require(offset == len(stdout), '/stdout', 'unconsumed output or extra section')
        _ = policy_for(manifest)
        require(file_hash(plan.manifest) == plan.manifest_sha256, '/manifest', 'manifest changed')
    except (OSError, ValueError, IndexError) as error:
        errors.append(str(error))
    finally:
        final_capture = reconcile_capture(plan, parent)
        if final_capture.integrity != 'PASS' or final_capture.completeness != 'COMPLETE':
            errors.append('capture-final: source/receipt/closure no longer complete and valid')
        for root, before in frozen.items():
            try: require(snapshot(root) == before, '/freeze', 'evidence membership/bytes changed')
            except (OSError, ValueError) as error: errors.append(str(error))
    if errors: joined.clear()  # Incomplete framing may shift even an apparently matching prefix.
    outcome = 'FAILED' if parent.native_exit != 0 or any(row.section.failed for row in joined) else 'SUCCEEDED'
    return CaseAccounting(tuple(joined), tuple(errors), 'INCOMPLETE' if errors else 'COMPLETE_OBSERVED_AND_LISTED_DOCTEST_CASES',
                          outcome, digest(stdout), digest(stderr), offset, observed_ranges=tuple(raw_ranges))


@dataclass(frozen=True, slots=True)
class NativeCaseLists:
    executable: Executable
    manifest_sha256: str
    all_cases: ListingProcess
    ignored_cases: ListingProcess


@dataclass(frozen=True, slots=True)
class PairedCases:
    direct: tuple[JoinedSection, ...]
    wrapped: tuple[JoinedSection, ...]
    required: tuple[str, ...]
    errors: tuple[str, ...]
    case_parity: str
    direct_exit: int | None
    wrapped_exit: int | None
    native_outcome: str
    direct_ranges: tuple[tuple[int, int, str], ...]
    evidence: tuple[tuple[str, str], ...]
    limitations: tuple[str, ...] = ('CALLER_AUTHENTICATED_SOURCE_BUILD_REQUIRED',
        'DIRECT_CHILD_ENVIRONMENT_NOT_OBSERVED', 'DIRECT_CHILD_PID_AND_EXIT_NOT_OBSERVED',
        'POPULATED_JOBSERVER_FDS_NOT_QUALIFIED', 'WORLD_SAFETY_SEPARATE_INCOMPLETE')


def standard_harness(target: Target, manifest_sha256: str) -> None:
    """Source bytes plus upstream compiler/metadata identity, never a boolean claim."""
    raw = target.manifest.read_bytes()
    require(digest(raw) == manifest_sha256, '/harness', 'package manifest changed')
    node = document(json.dumps(tomllib.loads(raw.decode('utf-8'))))
    node.field('package').field('name').equals(target.package)
    require(target.test and target.source.is_relative_to(target.manifest.parent), '/harness', 'foreign or unselected target')
    library = set(target.kinds) <= {'lib', 'rlib', 'staticlib', 'cdylib', 'dylib', 'proc-macro'} and bool(target.kinds)
    require(library or target.kinds in (('bin',), ('test',)), '/harness', 'unsupported custom/target kind')
    kind = 'lib' if library else target.kinds[0]
    entries = ([node.field('lib')] if 'lib' in node.mapping() else []) if library else (
        node.field(kind).array() if kind in node.mapping() else [])
    matches: list[Node] = []
    for entry in entries:
        name = entry.field('name').text() if 'name' in entry.mapping() else target.package.replace('-', '_') if library else target.package
        if name == target.name: matches.append(entry)
    require(len(matches) <= 1 and (not library or not entries or len(matches) == 1), '/harness', 'ambiguous or mismatched target declaration')
    defaults = (target.manifest.parent/'src/lib.rs',) if library else (
        (target.manifest.parent/'src/main.rs',) if kind == 'bin' and target.name == target.package else ()) + (
        target.manifest.parent/('src/bin' if kind == 'bin' else 'tests')/(target.name+'.rs'),
        target.manifest.parent/('src/bin' if kind == 'bin' else 'tests')/target.name/'main.rs')
    if matches:
        entry = matches[0]
        if 'harness' in entry.mapping(): entry.field('harness').equals(True)
        if 'test' in entry.mapping(): entry.field('test').equals(True)
        if 'path' in entry.mapping():
            path = target.manifest.parent/entry.field('path').text()
            require(path.resolve(strict=True) == target.source and '..' not in path.parts, '/harness', 'target path differs')
            return
    else:
        auto = 'autobins' if kind == 'bin' else 'autotests'
        if not library and auto in node.field('package').mapping(): node.field('package').field(auto).equals(True)
        require(not library or target.name == target.package.replace('-', '_'), '/harness', 'unknown inferred library name')
    existing = tuple(path for path in defaults if path.is_file())
    require(existing == (target.source,), '/harness', 'ambiguous or nondefault inferred source')


def host_args(args: tuple[str, ...], runner: tuple[str, ...]) -> tuple[str, ...]:
    require(len(runner) == 2 and args.count('--test-run-directory') == 1 and
            not any(arg.startswith(('--test-run-directory=', '--test-runtool', '--target')) for arg in args),
            '/host-transform', 'ambiguous host runtool insertion')
    index = args.index('--test-run-directory')+2
    require(index <= len(args), '/host-transform', 'missing package directory')
    return (*args[:index], '--test-runtool', runner[0], '--test-runtool-arg', runner[1], *args[index:])


def section_ranges(raw: bytes, count: int) -> tuple[tuple[int, int, str], ...]:
    result: list[tuple[int, int, str]] = []; start = 0
    for match in re.finditer(rb'\ntest result: [^\n]*\n\n', raw):
        result.append((start, match.end(), digest(raw[start:match.end()])))
        start = match.end()
    require(len(result) == count and start == len(raw), '/sections', 'total cardinality/framing differs; no ordinal identity admitted')
    return tuple(result)


def direct_commands(raw: bytes, binaries: tuple[str, ...], rustdoc: str) -> tuple[tuple[str, ...], tuple[tuple[str, ...], ...]]:
    """Observe order, then validate the complete exact declaration sequence."""
    commands: list[tuple[str, ...]] = []
    for line in raw.splitlines():
        for binary in binaries:
            prefix = b'Running `'+binary.encode()+b' '
            if prefix in line:
                commands.append((binary, '--test-threads=8'))
    require(len(commands) == len(binaries) and {row[0] for row in commands} == set(binaries), '/direct', 'missing/duplicate native declaration')
    docs = declarations(raw, rustdoc, tuple(commands))
    return tuple(row[0] for row in commands), docs


def after_parent(binding: ListingProcess, parent: ProcessResult) -> None:
    started = datetime.fromisoformat(read(binding.result.receipt).field('started_at').text())
    ended = datetime.fromisoformat(read(parent.receipt).field('ended_at').text())
    require(started.tzinfo is not None and ended.tzinfo is not None and started >= ended,
            '/listing', 'listing did not follow wrapped execution')


def native_section(raw: bytes, all_cases: Listing, ignored: Listing) -> Section:
    # parse_log supplies native numeric accounting; exact pretty framing disallows dropped junk.
    parsed = reconcile_native(raw, all_cases, ignored)
    strict = parse_section(raw, all_cases.names, ignored.names)
    require(all(row.mode is None for row in strict.outcomes) and parsed == strict, '/native', 'non-native or unsupported framing')
    return parsed


def qualified_native_section(raw: bytes, all_cases: Listing, ignored: Listing, source: NestedSource | None) -> Section:
    return native_section(raw, all_cases, ignored) if source is None else reconcile_native(raw, all_cases, ignored, source)


def direct_section_ranges(raw: bytes, native: tuple[tuple[Listing, Listing, NestedSource | None], ...],
                          docs: int) -> tuple[tuple[int, int, str], ...]:
    """Source-qualified serial Cargo sections, with exact aggregate validation."""
    ends = tuple(match.end() for match in re.finditer(rb'\ntest result: [^\n]*\n\n', raw))
    result: list[tuple[int, int, str]] = []
    cursor = 0; start = 0
    for all_cases, ignored, source in native:
        # Source policy is rederived by native_section before any range is admitted.
        summaries = 1 + (sum(child.count for child in source.children) if source is not None else 0)
        require(cursor + summaries <= len(ends), '/sections', 'missing source-required native summaries')
        end = ends[cursor + summaries - 1]
        _ = qualified_native_section(raw[start:end], all_cases, ignored, source)
        result.append((start, end, digest(raw[start:end])))
        start = end; cursor += summaries
    require(docs >= 0 and len(ends) - cursor == docs, '/sections', 'unresolved native/doctest summary cardinality')
    for end in ends[cursor:]:
        require(raw[start:end].startswith(b'\nrunning '), '/sections', 'missing standalone section boundary')
        result.append((start, end, digest(raw[start:end])))
        start = end
    require(start == len(raw), '/sections', 'unconsumed direct output')
    return tuple(result)


def pair_cases(direct: ListingProcess, plan: CapturePlan, parent: ProcessResult,
               native: tuple[NativeCaseLists, ...], docs: tuple[CaseLists, ...]) -> PairedCases:
    """Full-case equality is separate from upstream build/runtime and world admission."""
    errors: list[str] = []; left: list[JoinedSection] = []; right: list[JoinedSection] = []
    required: list[str] = ['native:'+str(row.binary) for row in plan.inventory.executables]; ranges: tuple[tuple[int, int, str], ...] = ()
    frozen: dict[Path, dict[str, str]] = {}; evidence: dict[str, str] = {}
    direct_required: dict[str, str] = {}
    try:
        manifest = read(plan.manifest); policy = policy_for(manifest)
        required.extend('doctest:'+row.field('id').text() for row in policy.field('invocations').array())
        capture = reconcile_capture(plan, parent)
        require(capture.integrity == 'PASS' and capture.completeness == 'COMPLETE', '/capture', 'incomplete capture')
        roots = (direct.spec.output_root, plan.parent_spec.output_root, plan.capture_root, Path(policy.field('exec_root').text()),
                 *(binding.spec.output_root for pair in (*native, *docs) for binding in (pair.all_cases, pair.ignored_cases)))
        frozen = {root: snapshot(root) for root in roots}
        evidence = {str(root/name): value for root, rows in frozen.items() for name, value in rows.items()}
        bound = {key: manifest.field('inputs').field(key).text() for key in manifest.field('inputs').mapping()}
        artifacts = {row.field('path').text(): row.field('sha256').text() for row in manifest.field('artifacts').array()}
        bound.update(artifacts)
        # Both ProcessSpecs independently bind the same immutable source/tool/artifact closure.
        original = (str(direct.spec.executable), 'test', '--workspace', *(('--release',) if plan.step == 5 else ()), '--no-fail-fast', '--', '--test-threads=8')
        require(plan.step in (4, 5) and direct.spec.executable == plan.parent_spec.executable and
                direct.spec.executable_sha256 == plan.parent_spec.executable_sha256 and direct.spec.input_root == plan.parent_spec.input_root,
                '/direct', 'tool/source differs')
        direct_required = {**bound, str(direct.spec.executable): direct.spec.executable_sha256}
        raw = process_output(direct, original, direct.spec.input_root/'rust', environment_for(direct.spec), direct_required)
        stderr = (direct.spec.output_root/'stderr.log').read_bytes()
        env = environment_for(plan.parent_spec)
        runners = [value for key, value in env.items() if key.startswith('CARGO_TARGET_') and key.endswith('_RUNNER')]
        require(len(runners) == 1, '/runner', 'unique wrapped runner required')
        runner = tuple(shlex.split(runners[0]))
        direct_env = environment_for(direct.spec)
        require(not any(key.startswith('CARGO_CAPTURE_') or key.startswith('CARGO_TARGET_') and key.endswith('_RUNNER') for key in direct_env),
                '/direct', 'direct unexpectedly instrumented')
        require(direct_env.get('RUSTDOC') == policy.field('rustdoc').text() and env.get('RUSTDOC') == policy.field('entrypoint').text(),
                '/environment', 'direct real rustdoc/wrapped shim not selected')
        runner_keys = {key for key in env if key.startswith('CARGO_TARGET_') and key.endswith('_RUNNER')}
        observer_keys = {'CARGO_CAPTURE_MANIFEST', 'CARGO_CAPTURE_MANIFEST_SHA256', 'RUSTDOC', *runner_keys}
        require({key: value for key, value in env.items() if key not in observer_keys} ==
                {key: value for key, value in direct_env.items() if key != 'RUSTDOC'}, '/environment', 'outer delta exceeds reviewed observer additions')
        expected = {str(row.executable.binary): row for row in native}
        require(len(expected) == len(native) and set(expected) == {str(row.binary) for row in plan.inventory.executables}, '/native', 'full native inventory differs')
        ordered, doc_args = direct_commands(stderr, tuple(expected), policy.field('rustdoc').text())
        doc_rows = tuple(match_invocation(manifest, policy, host_args(args, runner), direct.spec.cwd) for args in doc_args)
        ids = tuple(row.field('id').text() for row in doc_rows)
        require(len(ids) == len(set(ids)) and set(ids) == {row.field('id').text() for row in policy.field('invocations').array()}, '/direct-docs', 'source obligations differ')
        ends = (0, *(match.end() for match in re.finditer(rb'\ntest result: [^\n]*\n\n', raw)))
        ranges = tuple((start, end, digest(raw[start:end])) for start, end in pairwise(ends))
        for pair in docs:
            for listing in (pair.all_cases, pair.ignored_cases): after_parent(listing, parent)
        wrapped_docs = join_standalone_sections(plan, parent, docs)
        require(not wrapped_docs.errors, '/wrapped-docs', '; '.join(wrapped_docs.errors))
        wrapped_doc_rows = {row.invocation: row for row in wrapped_docs.sections}
        native_listings: dict[str, tuple[Listing, Listing]] = {}
        for binary in ordered:
            item = expected[binary]
            targets = [row for row in capture.targets if row.executable == item.executable]
            require(len(targets) == 1 and item.executable in plan.inventory.executables, '/native', 'actual captured identity differs')
            actual = targets[0]; target = item.executable.target
            require(bound.get(str(target.manifest)) == item.manifest_sha256 and bound.get(str(target.source)) == file_hash(target.source), '/harness', 'manifest/source absent from source closure')
            standard_harness(target, item.manifest_sha256)
            listing_required = {**bound, str(plan.manifest): plan.manifest_sha256, str(actual.receipt): actual.receipt_sha256}
            parsed: list[Listing] = []
            for listing, suffix in ((item.all_cases, ('--list',)), (item.ignored_cases, ('--list', '--ignored'))):
                output = listing_output(listing, (binary, *suffix), item.executable.cwd, dict(actual.original_environment), listing_required)
                after_parent(listing, parent)
                parsed.append(parse_listing(output))
            native_listings[binary] = (parsed[0], parsed[1])
            wrapped_raw = actual.receipt.with_name('stdout.log').read_bytes()
            section = qualified_native_section(wrapped_raw, *native_listings[binary], nested_native_source(plan.parent_spec, item.executable))
            require(actual.native_exit == (101 if section.failed else 0), '/native', 'libtest section/native exit differs')
            right.append(JoinedSection('native:'+binary, target.package_id, str(target.source), 0, len(wrapped_raw), section))
        direct_sources = {binary: nested_native_source(direct.spec, expected[binary].executable) for binary in ordered}
        ranges = direct_section_ranges(raw, tuple((*native_listings[binary], direct_sources[binary]) for binary in ordered), len(ids))
        for index, binary in enumerate(ordered):
            start, end, _ = ranges[index]; target = expected[binary].executable.target
            left.append(JoinedSection('native:'+binary, target.package_id, str(target.source), start, end,
                                      qualified_native_section(raw[start:end], *native_listings[binary], direct_sources[binary])))
        for index, row in enumerate(doc_rows, len(ordered)):
            identity = row.field('id').text(); actual = wrapped_doc_rows[identity]
            start, end, _ = ranges[index]
            names = tuple(outcome.name for outcome in actual.section.outcomes)
            ignored = tuple(outcome.name for outcome in actual.section.outcomes if outcome.status == 'ignored')
            section = parse_section(raw[start:end], names, ignored)
            left.append(JoinedSection('doctest:'+identity, actual.package_id, actual.source, start, end, section))
            right.append(JoinedSection('doctest:'+identity, actual.package_id, actual.source, actual.start, actual.end, actual.section))
        require(direct.result.native_exit == parent.native_exit, '/parity', 'outer native exit differs')
        def outcomes(rows: list[JoinedSection]) -> dict[str, tuple[Outcome, ...]]:
            return {row.invocation: tuple(sorted(row.section.outcomes, key=lambda item: item.name)) for row in rows}
        require(outcomes(left) == outcomes(right), '/parity', 'per-identity case outcomes differ')
    except (OSError, ValueError, IndexError) as error:
        errors.append(str(error))
    finally:
        try: _ = policy_for(read(plan.manifest))
        except (OSError, ValueError, IndexError) as error: errors.append('final-policy: '+str(error))
        try:
            final = reconcile_capture(plan, parent)
            require(final.integrity == 'PASS' and final.completeness == 'COMPLETE', '/final', 'capture no longer complete/valid')
        except (OSError, ValueError, IndexError) as error: errors.append('final-capture: '+str(error))
        for root, before in frozen.items():
            try: require(snapshot(root) == before, '/freeze', 'membership/bytes changed')
            except (OSError, ValueError, IndexError) as error: errors.append('final-freeze: '+str(error))
        try:
            _ = process_output(direct, direct.spec.argv, direct.spec.cwd, environment_for(direct.spec), direct_required)
        except (OSError, ValueError, IndexError) as error: errors.append('final-process: '+str(error))
    if errors: left.clear(); right.clear()
    outcome = 'FAILED' if direct.result.native_exit != 0 or parent.native_exit != 0 or any(row.section.failed for row in (*left, *right)) else 'SUCCEEDED'
    return PairedCases(tuple(left), tuple(right), tuple(required), tuple(errors), 'INCOMPLETE' if errors else 'EQUAL',
                       direct.result.native_exit, parent.native_exit, outcome, ranges, tuple(sorted(evidence.items())))
