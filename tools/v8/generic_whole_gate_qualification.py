# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_qualification
"""Read separately executed O qualification; never infer it from gate success.

The initial matrix deliberately retains unavailable comparison routes. No report
can qualify those routes until H gains a reviewed comparator and state witness.
"""
import shlex
from pathlib import Path

from generic_gate import document, require
from generic_process import file_hash
from generic_sources import safe_path
from mux2_gate_inputs import Json, Node

MATRIX = 'tools/v8/whole_gate_observer_matrix.json'
MODES = ('original_off', 'derived_off', 'derived_on')


def reference(node: Node) -> tuple[Path, Node]:
    """Reopen an immutable original reference, including its original file mode."""
    require(set(node.mapping()) == {'path', 'sha256', 'mode'}, '/reference', 'closed original reference required')
    path = Path(node.field('path').text())
    require(file_hash(path) == node.field('sha256').text(), str(path), 'evidence bytes changed')
    require(path.lstat().st_mode == node.field('mode').number(), str(path), 'evidence mode changed')
    return path, document(path.read_text())


def matrix(session: Node) -> Node:
    harness = session.field('harness')
    path = safe_path(Path(harness.field('root').text()), MATRIX)
    require(file_hash(path) == harness.field('inputs').field(MATRIX).text(), MATRIX, 'matrix is not admitted H')
    value = document(path.read_text())
    value.field('schema').equals('whole_gate_observer_matrix.v1')
    value.field('modes').equals(list(MODES))
    rows = value.field('rows').array()
    ids = [row.field('id').text() for row in rows]
    require(bool(ids) and len(ids) == len(set(ids)), '/matrix', 'empty or duplicate matrix')
    covered = {p.text() for row in rows for p in row.field('observer_paths').array()}
    require(covered == set(value.field('observer_postimages').mapping()), '/matrix', 'incomplete O coverage')
    for row in rows:
        for name, digest in row.field('source_refs').mapping().items():
            harness.field('inputs').field(name).equals(digest)
    return value


def identity(session: Node, report: Node, required: Node) -> None:
    report.field('schema').equals('whole_gate_observer_qualification.v1')
    report.field('harness_sha').equals(session.field('harness').field('source_sha').text())
    report.field('harness_inputs').equals(session.field('harness').field('inputs').value)
    report.field('observer_postimages').equals(required.field('observer_postimages').value)
    require(set(report.field('arms').mapping()) == {'base', 'candidate'}, '/arms', 'both selected products required')
    for label in ('base', 'candidate'):
        arm = session.field('arms').field(label)
        actual = report.field('arms').field(label)
        actual.field('source_sha').equals(arm.field('source_sha').text())
        actual.field('original_source').equals(arm.field('source_before').field('tracked_manifest').value)
        actual.field('effective_source').equals(arm.field('effective_source').value)
        roles = document(Path(arm.field('layer_manifest_path').text()).read_text())
        selected = required.field('observer_postimages')
        for row in roles.field('files').array():
            if row.field('role').text() == 'O':
                selected.field(row.field('path').text()).array()[1].equals(row.field('post_sha256').text())
        for name, raw in selected.mapping().items():
            arm.field('effective_source').field('manifest').field(name).equals(raw)


def authenticated_build(mode: str, evidence: Node, arm: Node, product: str, session: Node) -> Node:
    """Authenticate real build/process receipts before reading comparison artifacts."""
    build_path, build = reference(evidence.field('build'))
    require(build_path.name == 'build.json', '/build', 'not original generic build receipt')
    build.field('status').equals('PASS')
    build.field('exit_code').equals(0)
    build.field('source_sha').equals(arm.field('source_sha').text())
    build.field('source_before').field('tracked_manifest').equals(arm.field('original_source').value)
    build.field('effective_source_after').equals(build.field('effective_source_before').value)
    expected: dict[str, Json] = ({name: [Node(raw).array()[0].value, Node(raw).array()[2].value]
                 for name, raw in arm.field('original_source').mapping().items()}
                if mode == 'original_off' else arm.field('effective_source').field('manifest').mapping())
    build.field('effective_source_before').field('manifest').equals(expected)
    build_process_path = Path(build.field('process_receipt').text())
    require(build_process_path == build_path.parent/'cargo-build/process.json', '/build', 'foreign build process location')
    require(file_hash(build_process_path) == build.field('process_receipt_sha256').text(),
            '/build', 'original build process changed')
    build_process = document(build_process_path.read_text())
    build_process.field('status').equals('PASS')
    build_process.field('native_exit').equals(0)
    build_process.field('argv').equals(build.field('argv').value)
    require(build_process.field('pid').number() > 0, '/build', 'missing actual build child')
    for phase in ('before', 'after'):
        build_process.field(phase+'_inputs').equals(build_process.field('expected_inputs').value)
        build_process.field(phase+'_external_inputs').equals(build_process.field('expected_external_inputs').value)
        build_process.field(phase+'_executable_sha256').equals(build_process.field('expected_executable_sha256').value)
    for label in ('stdout', 'stderr'):
        require(file_hash(build_process_path.parent/(label+'.log')) == build_process.field(label+'_sha256').text(),
                '/build', 'original build log changed')
    from generic_whole_gate_qualification_build import authenticate
    authenticate(session, build, build_process, product, mode)
    return build


def producer(mode: str, evidence: Node, arm: Node, row: Node, session: Node) -> tuple[Path, Node]:
    """Authenticate real build/process receipts before reading comparison artifacts."""
    build = authenticated_build(mode, evidence, arm, row.field('product').text() if 'product' in row.mapping() else 'worldsim_probe', session)
    expected = build.field('effective_source_before').field('manifest').mapping()
    process_path, process = reference(evidence.field('process'))
    process.field('status').equals('PASS')
    process.field('native_exit').equals(0)
    process.field('errors').equals([])
    require(process.field('pid').number() > 0, '/process', 'missing actual child')
    process.field('input_root').equals(build.field('root').text())
    output = Path(process.field('output_root').text())
    require(output == process_path.parent, '/process', 'foreign output root')
    binary = build.field('artifacts').field(row.field('product').text() if 'product' in row.mapping() else 'worldsim_probe')
    process.field('executable').equals(binary.field('path').text())
    require(file_hash(Path(binary.field('path').text())) == binary.field('sha256').text(), '/binary', 'qualifying binary changed')
    for key in ('expected_executable_sha256', 'before_executable_sha256', 'after_executable_sha256'):
        process.field(key).equals(binary.field('sha256').text())
    for label in ('inputs', 'external_inputs'):
        process.field('before_'+label).equals(process.field('expected_'+label).value)
        process.field('after_'+label).equals(process.field('expected_'+label).value)
    for name, raw in expected.items():
        entry = Node(raw).array()
        if entry[0].text() != '120000':
            process.field('expected_inputs').field(name).equals(entry[1].text())
    full_inputs: dict[str, Json] = {name: Node(raw).array()[1].value for name, raw in expected.items() if Node(raw).array()[0].text() != '120000'}
    full_inputs.update(build.field('effective_inputs_before').mapping())
    process.field('expected_inputs').equals(full_inputs)
    for name, digest in build.field('cargo_configs').mapping().items():
        process.field('expected_external_inputs').field(name).equals(digest)
    for label in ('stdout', 'stderr'):
        require(file_hash(output/(label+'.log')) == process.field(label+'_sha256').text(), '/logs', 'producer log changed')
    argv = shlex.split(row.field('command').text().replace('{out}', str(output/'artifacts')))
    argv[0] = binary.field('path').text()
    env = process.field('environment').mapping()
    if mode == 'derived_on':
        if row.field('id').text() == 'probe_ascii' or ('observer_switch' in row.mapping() and row.field('observer_switch').text() == 'environment'):
            process.field('environment').field('WORLDSIM_GATE_OBSERVER').equals('1')
            if row.field('id').text() == 'probe_ascii':
                process.field('environment').field('WORLDSIM_GATE_TEST_OUT').equals(str(output/'observer'))
        else:
            argv.append('--gate-observations')
    else:
        require(not any(key.startswith('WORLDSIM_GATE_') for key in env), '/environment', 'observer enabled in off arm')
    process.field('argv').equals([arg for arg in argv])
    process.field('cwd').equals(build.field('root').text())
    allowed = {'PATH', 'LANG', 'LC_ALL', 'HOME', 'TMPDIR', 'WORLDSIM_GATE_OBSERVER', 'WORLDSIM_GATE_TEST_OUT', 'WORLDSIM_CONTENT_PACK'}
    if row.field('id').text() == 'm6a-data-only':
        process.field('environment').field('WORLDSIM_CONTENT_PACK').equals(str(Path(build.field('root').text())/'rust/crates/sim-test/data/m6a_apple/pack.ron'))
    else:
        require('WORLDSIM_CONTENT_PACK' not in env, '/environment', 'foreign content override')
    require(set(env) <= allowed, '/environment', 'unreviewed qualification override')
    for name, digest in process.field('expected_inputs').mapping().items():
        require(file_hash(safe_path(Path(process.field('input_root').text()), name)) == Node(digest).text(),
                name, 'qualifying source/input changed')
    for name, digest in process.field('expected_external_inputs').mapping().items():
        require(file_hash(Path(name)) == Node(digest).text(), name, 'qualifying external input changed')
    return output, process


def compare_artifacts(row: Node, outputs: tuple[Path, Path, Path], evidence: Node) -> None:
    """Compare original raw producer bytes, never a supplied equality boolean."""
    from generic_whole_gate_qualification_compare import compare, markdown_simulation
    kind = row.field('comparator').text()
    require(kind in {'bytes','baseline','m6a','m6a-data-only','nav','perf'}, '/comparator', 'unavailable comparator')
    names = [item.text() for item in row.field('artifacts').array()]
    if kind == 'perf':
        index = document((outputs[0]/'artifacts/runs.json').read_text())
        runs = index.field('runs').array()
        require(len(runs) == 1, '/runs', 'one actual run per fixed invocation required')
        run = runs[0]
        run.field('status').equals('ok')
        for field in ('seed','ticks'):
            run.field(field).equals(row.field(field).value)
        run.field('mode').equals(row.field('run_mode').text())
        raw_name = run.field('file').text()
        run.field('file').equals(run.field('stem').text()+'.json')
        names.extend((raw_name, run.field('stem').text()+'.tick_us.bin'))
    require(bool(names) and len(names) == len(set(names)), '/artifacts', 'nonempty fixed artifacts required')
    for mode, output in zip(MODES, outputs, strict=True):
        artifacts = evidence.field(mode).field('artifacts')
        require(set(artifacts.mapping()) == set(names), '/artifacts', 'incomplete fixed artifact set')
        for name in names:
            ref = artifacts.field(name)
            path = safe_path(output/'artifacts', name)
            ref.field('path').equals(str(path))
            require(file_hash(path) == ref.field('sha256').text(), name, 'raw artifact changed')
            require(path.lstat().st_mode == ref.field('mode').number(), name, 'raw artifact mode changed')
    for name in names:
        content = [safe_path(root/'artifacts', name).read_bytes() for root in outputs]
        allowed_empty = [item.text() for item in row.field('allow_empty').array()] if 'allow_empty' in row.mapping() else []
        require(bool(content[0]) or name in allowed_empty, name, 'empty original artifact')
        if kind == 'perf' and name.endswith('.tick_us.bin'):
            require(all(len(raw) == row.field('ticks').number()*4 for raw in content), name, 'raw timing series length differs')
            continue
        if kind == 'perf' and name != 'runs.json':
            for raw in content:
                report = document(raw.decode())
                for field in ('seed','ticks'):
                    report.field(field).equals(row.field(field).value)
                report.field('mode').equals(row.field('run_mode').text())
        if kind == 'm6a' and name == 'summary.json':
            for raw in content:
                document(raw.decode()).field('seed').equals(row.field('seed').value)
        schema = 'perf-index' if kind == 'perf' and name == 'runs.json' else kind
        if kind == 'baseline' and name == 'summary.md':
            stable = [markdown_simulation(raw, (root/'artifacts/summary.json').read_text()) for raw, root in zip(content, outputs, strict=True)]
            require(stable[0] == stable[1] == stable[2], name, 'baseline Markdown state differs')
        elif kind == 'bytes' or (kind == 'm6a' and name != 'summary.json') or (kind == 'baseline' and name.endswith('.jsonl')):
            require(content[0] == content[1] == content[2], name, 'off/on product bytes differ')
        else:
            for other in content[1:]:
                _ = compare(schema, content[0].decode(), other.decode())


def qualification(session: Node) -> str:
    """Absent evidence stays unqualified; supplied invalid/partial evidence rejects."""
    if 'observer_qualification' not in session.mapping():
        return 'PRODUCT_UNQUALIFIED'
    _, report = reference(session.field('observer_qualification'))
    required = matrix(session)
    from generic_whole_gate_qualification_compare import imported_sources
    roots = {Path(session.field('harness').field('root').text()), *(Path(Node(arm).field('root').text()) for arm in session.field('arms').mapping().values())}
    for name, loaded in imported_sources():
        require(loaded in {root/name for root in roots}, name, 'comparator dependency outside admitted roots')
        require(file_hash(loaded) == session.field('harness').field('inputs').field(name).text(), name, 'loaded comparator dependency differs from H')
    identity(session, report, required)
    rows = required.field('rows').array()
    ids = {row.field('id').text() for row in rows}
    for label in ('base', 'candidate'):
        arm = report.field('arms').field(label)
        require(set(arm.field('rows').mapping()) == ids, '/rows', 'missing or extra required route')
        paths: set[Path] = set()
        for row in rows:
            require(row.field('comparator').text() in {'bytes','baseline','m6a','m6a-data-only','nav','perf'}, row.field('id').text(),
                    'required route has no reviewed state comparator: '+row.field('gap').text())
            evidence = arm.field('rows').field(row.field('id').text())
            require(set(evidence.mapping()) == set(MODES), '/modes', 'three independently executed modes required')
            values = tuple(producer(mode, evidence.field(mode), arm, row, session) for mode in MODES)
            environments = [{key: (Node(value).text().replace(process.field('input_root').text()+'/', '{product}/') if key == 'WORLDSIM_CONTENT_PACK' else value) for key, value in process.field('environment').mapping().items()
                             if not key.startswith('WORLDSIM_GATE_')} for _, process in values]
            require(environments[0] == environments[1] == environments[2], '/environment', 'off/on environment differs')
            outputs = (values[0][0], values[1][0], values[2][0])
            require(len(set(outputs)) == 3 and not paths.intersection(outputs), '/process', 'reused execution output')
            paths.update(outputs)
            compare_artifacts(row, outputs, evidence)
    return 'PRODUCT_QUALIFIED'
