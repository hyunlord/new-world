"""Original Cargo aggregate accounting, separate from ordinary-world admission."""
import re
from pathlib import Path

from gate_test_summary import parse_log
from generic_gate import boundary, integer, require, text
from generic_process import file_hash
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from generic_whole_gate_workspace_classification import (
    classified_target,
    classify_routes,
)
from generic_whole_gate_workspace_plan import MANIFESTS, TARGETS
from generic_workspace_cases import (
    Listing,
    NestedChild,
    digest,
    nested_policy,
    reconcile_native,
    reconcile_nested_records,
)
from mux2_gate_inputs import Json, Node

COMPLETE_FAMILIES = frozenset({'colony_commands', 'colony_world', 'save_load', 'colony_steward', 'steward_defence', 'engine_appearance_ffi'})

HEADER = re.compile(r'^\s*(?:Running (?:unittests )?(\S+) \(([^\n]+)\)|Doc-tests (\S+))\s*$', re.MULTILINE)


def source_contract(root: Path, inputs: dict[str, str], environment: Node, package: str,
                    name: str, kind: str) -> tuple[tuple[NestedChild, ...], tuple[tuple[str, str], ...], tuple[tuple[str, str], ...]]:
    rules, constants, forbidden = nested_policy(package, name, kind)
    require(not set(forbidden) & environment.mapping().keys(), '/nested', 'outer environment selects nested child')
    files: list[tuple[str, str]] = []
    blocks: list[tuple[str, str]] = []
    for relative, parent, child, _, parent_hash, child_hash in rules:
        path = root / relative
        require(inputs.get(relative) == file_hash(path), relative, 'unbound nested source')
        raw = path.read_bytes()
        files.append((relative, digest(raw)))
        for constant in constants:
            require(raw.decode().splitlines().count(constant) == 1, relative, 'nested environment constant changed')
            blocks.append((constant, digest(constant.encode())))
        for test, expected in ((parent, parent_hash), (child, child_hash)):
            marker = ('#[test]\nfn ' + test.rsplit('::', 1)[-1] + '() {').encode()
            require(raw.count(marker) == 1, relative, 'nested test block absent or ambiguous')
            start = raw.index(marker)
            end = raw.find(b'\n#[test]', start + len(marker))
            block = raw[start:end if end >= 0 else len(raw)].rstrip() + b'\n'
            require(digest(block) == expected, relative, 'nested source contract changed')
            blocks.append((test, expected))
    return tuple(NestedChild(row[1], row[2], row[3]) for row in rules), tuple(files), tuple(blocks)


def account_payload(raw: str, children: tuple[NestedChild, ...] = (),
                    files: tuple[tuple[str, str], ...] = (), blocks: tuple[tuple[str, str], ...] = ()) -> dict[str, Json]:
    """Names are actual observed outcomes, not an independent --list result."""
    summary = parse_log(raw)
    names = tuple(sorted({row.name for row in summary.outcomes}))
    ignored = tuple(sorted({row.name for row in summary.outcomes if row.status == 'ignored'}))
    listing = Listing(names, (), digest(raw.encode()))
    excluded = Listing(ignored, (), digest(raw.encode()))
    data = ('\n' + raw.strip() + '\n\n').encode()
    if children:
        result = reconcile_nested_records(data, listing, excluded, children, files, blocks)
    else:
        result = reconcile_native(data, listing, excluded)
        running = re.findall(r'^running ([0-9]+) tests?$', raw, re.MULTILINE)
        require(running == [str(len(names))], '/running', 'missing/duplicate running inventory')
    require(result.failed == 0, '/outcome', 'original test failure')
    return {'cases': [{'name': row.name, 'status': row.status, 'mode': row.mode} for row in result.outcomes],
            'passed': sum(row.status == 'ok' for row in result.outcomes), 'ignored': len(ignored),
            'nested_passed': 0 if result.nested is None else result.nested.nested_passed,
            'source_files': dict(files), 'source_blocks': dict(blocks), 'raw_sha256': digest(data)}


def identify(label: str, binary: str, payload: str, step: int) -> tuple[str, str, str, str]:
    path = Path(binary)
    profile = 'debug' if step == 4 else 'release'
    require(path.parent.name == 'deps' and path.parent.parent.name == profile, '/binary', 'wrong original profile/location label')
    choices = [row for row in TARGETS if label == ('src/lib.rs' if row[2] == 'lib' else
               'src/bin/' + row[1] + '.rs' if row[2] == 'bin' else 'tests/' + row[1] + '.rs')
               and re.fullmatch(re.escape(row[1]) + r'-[0-9a-f]+', path.name)]
    if len(choices) > 1:
        # The two actual `tests/integration.rs` targets have disjoint named contracts.
        names = {row.name for row in parse_log(payload).outcomes}
        choices = [row for row in choices if
                   (row[0] == 'sim-engine' and 'test_zero_systems_baseline' in names) or
                   (row[0] == 'sim-systems' and 'harness_influence_update_name' in names)]
    require(len(choices) == 1, '/target', 'unknown or ambiguous original target')
    return choices[0]


@boundary
def collect_workspace(session: Node, stage: Node) -> dict[str, Json]:
    step = integer(stage.field('step'))
    require(step in (4, 5), '/step', 'only original workspace commands')
    command = 'cd rust && cargo test --workspace ' + ('--release ' if step == 5 else '') + '--no-fail-fast -- --test-threads=8'
    stage.field('argv').equals(['bash', '-c', command])
    raw, aggregate_pid = aggregate_output(session, stage)
    product = session.field('arms').field(text(stage.field('arm')))
    root = Path(text(product.field('root')))
    inputs = {name: text(Node(value)) for name, value in product.field('source_inputs').mapping().items()}
    for name, sha in MANIFESTS.items():
        require(inputs.get(name) == sha and file_hash(root/name) == sha, name, 'unregistered Cargo topology')
    for _, _, _, source in TARGETS:
        require(inputs.get(source) == file_hash(root/source), source, 'unbound original target source')
    expected_tests = {source for _, _, kind, source in TARGETS if kind == 'test'}
    actual_tests = {p.relative_to(root).as_posix() for member in MANIFESTS if member != 'rust/Cargo.toml'
                    for p in (root/member).parent.joinpath('tests').glob('*.rs')}
    require(actual_tests == expected_tests, '/topology', 'added or missing original test target')
    rows = account_workspace(raw, step, root, inputs, stage.field('environment'))
    classifications = classify_routes(root, inputs, rows)
    return {'status': 'PASS', 'reviewed_mechanism_routes': classifications, 'accounting': 'COMPLETE_SOURCE_QUALIFIED_AGGREGATE',
            'authority': 'ORIGINAL_UNFILTERED_CARGO_COMMAND_AND_ACTUAL_AGGREGATE_OUTCOMES',
            'independent_listing': False, 'native_child_wait': False, 'aggregate_pid': aggregate_pid,
            'qualification': LIMIT, 'world_safety': 'SEPARATE_SOURCE_FAMILY_JOINS_REQUIRED', 'sections': rows,
            'remaining_world_routes': [
                {'package': package, 'target': name, 'kind': kind, 'source': source,
                 'reason': 'partial source-selected replay worlds only' if (package, name, kind) == ('sim-test', 'sim_test', 'lib')
                 else 'gifted hut only; remaining source cases unclassified' if name == 'colony_jobs'
                 else 'source case classification required; no ordinary-world inference from test count'}
                for package, name, kind, source in TARGETS
                if not (package == 'sim-test' and kind == 'test' and name in COMPLETE_FAMILIES)
                and not classified_target(Node({'package': package, 'target': name, 'kind': kind}), classifications)],
            'doctest_world_routes': [{'package': package, 'reason': 'source snippet classification required; actual outcomes retained'}
                                    for package in sorted({row[0] for row in TARGETS})]}


def account_workspace(raw: str, step: int, root: Path, inputs: dict[str, str], environment: Node) -> list[Json]:
    """Pure output/source join; does not authenticate process or confer world admission."""
    headers = list(HEADER.finditer(raw))
    require(bool(headers), '/workspace', 'no actual Cargo target output')
    header_lines = [line.strip() for line in raw.splitlines() if line.strip().startswith(('Running ', 'Doc-tests '))]
    require(header_lines == [item.group().strip() for item in headers], '/headers', 'unknown Cargo header')
    require(not re.search(r'^(?:test |running |\s*(?:Running|Doc-tests) )', raw[:headers[0].start()], re.MULTILINE),
            '/workspace', 'unattributed output before original target')
    seen: set[tuple[str, str, str]] = set()
    docs: set[str] = set()
    rows: list[Json] = []
    for index, header in enumerate(headers):
        payload = raw[header.end():headers[index+1].start() if index+1 < len(headers) else len(raw)]
        if header[3] is not None:
            package = header[3].replace('_', '-')
            require(header[3] == package.replace('-', '_'), '/doctest', 'foreign crate label')
            require(package in {row[0] for row in TARGETS} and package not in docs, '/doctest', 'unknown or duplicate doctest target')
            docs.add(package)
            doc_rows: list[Json] = []
            original_names: list[str] = []
            def encode_doc(match: re.Match[str], names: list[str] = original_names,
                           cases: list[Json] = doc_rows) -> str:
                name, status = match[1], match[2]
                names.append(name)
                cases.append({'name': name, 'status': status})
                return f'test doctest_{len(names)} ... {status}'
            canonical = re.sub(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:, [^\n]*)?$', encode_doc, payload, flags=re.MULTILINE)
            summary = parse_log(canonical)
            require(summary.failed == summary.measured == summary.filtered_out == 0 and summary.summaries == 1,
                    '/doctest', 'unsettled or filtered doctest outcome')
            require(len(original_names) == len(set(original_names)), '/doctest', 'duplicate doctest outcome')
            require(re.findall(r'^running ([0-9]+) tests?$', payload, re.MULTILINE) == [str(len(original_names))], '/doctest', 'running count differs')
            rows.append({'package': package, 'kind': 'doctest', 'cases': doc_rows})
            continue
        require(not docs, '/workspace', 'native section follows doctest phase')
        package, name, kind, source = identify(header[1], header[2], payload, step)
        key = (package, name, kind)
        require(key not in seen, '/target', 'duplicate original target')
        seen.add(key)
        children, files, blocks = source_contract(root, inputs, environment, package, name, kind)
        row = account_payload(payload, children, files, blocks)
        row.update(package=package, target=name, kind=kind, source=source, binary_label=header[2])
        rows.append(row)
    require(seen == {(package, name, kind) for package, name, kind, _ in TARGETS}, '/targets', 'missing original workspace targets')
    require(docs == {row[0] for row in TARGETS}, '/doctests', 'missing original doctest targets')
    return rows


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    candidate.field('step').equals(base.field('step').value)
    left, right = collect_workspace(session, base), collect_workspace(session, candidate)
    return {'status': 'PASS', 'scope': 'WORKSPACE_AGGREGATE_ACCOUNTING_ONLY',
            'world_safety': 'SEPARATE_SOURCE_FAMILY_JOINS_REQUIRED', 'safety': [],
            'base': left, 'candidate': right, 'qualification': LIMIT,
            'publication_authority': False}
