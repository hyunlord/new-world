# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Four full native integration-test binaries; broader gate17/37 stays incomplete.

Consumes generic_build integration_tests artifacts, never development receipts.
No native builds occur here. Source/observer semantic qualification remains a
separate gate obligation; endpoint provenance alone does not grant publication.
"""
import json
import os
import re
from dataclasses import asdict
from pathlib import Path

from generic_bench import AuditError, Product
from generic_build import derived_snapshot
from generic_gate import document, integer, require
from generic_process import ProcessSpec, atomic_json, execute, file_hash
from generic_sources import git, inventory
from generic_test_history import (
    FORMAT_SOURCE,
    identity,
    resolve,
)
from generic_test_history_graph import HistoryEvidence
from generic_test_invocation import invocations, verify_artifact
from generic_test_jobs_plans import jobs_plan
from generic_test_observations import (
    Binding,
    bound_bytes,
    process_receipt,
)
from generic_test_plans import JOBS_INPUTS as JOBS_INPUTS
from generic_test_plans import JOBS_RULES as JOBS_RULES
from generic_test_plans import SOURCE_PINS as SOURCE_PINS
from generic_test_plans import TESTS as TESTS
from generic_test_plans import Planned as Planned
from generic_test_plans import fixed_plan as fixed_plan
from generic_test_plans import hut_evidence as shared_hut_evidence
from generic_test_plans import save_plan as shared_save_plan
from generic_test_safety import pair
from mux2_gate_inputs import Json, Node
from mux2_gate_lock import lane


def _history_evidence(binding: Binding) -> HistoryEvidence:
    return HistoryEvidence(binding.process.output_root, binding.pid, binding.output_hashes,
                           binding.process.input_root, binding.process.inputs)


def save_plan(binding: Binding) -> tuple[Planned, ...]:
    return shared_save_plan(_history_evidence(binding))


def hut_evidence(binding: Binding) -> tuple[Planned, dict[str,Json]]:
    return shared_hut_evidence(_history_evidence(binding))


def _process_from_receipt(path: Path, digest: str) -> Binding:
    receipt = document(bound_bytes(path, digest).decode())
    spec = ProcessSpec(tuple(row.text() for row in receipt.field('argv').array()),
        Path(receipt.field('cwd').text()), Path(receipt.field('executable').text()),
        receipt.field('expected_executable_sha256').text(), Path(receipt.field('input_root').text()),
        tuple((name, Node(value).text()) for name,value in receipt.field('expected_inputs').mapping().items()),
        Path(receipt.field('output_root').text()),
        tuple((name, Node(value).text()) for name,value in receipt.field('environment').mapping().items()),
        tuple((Path(name), Node(value).text()) for name,value in receipt.field('expected_external_inputs').mapping().items()),
        tuple(row.text() for row in receipt.field('fresh_directories').array()))
    require(path == spec.output_root/'process.json', str(path), 'foreign compiler process receipt')
    binding = Binding(spec, digest, integer(receipt.field('pid')), ())
    node = process_receipt(binding)
    node.field('status').equals('PASS')
    node.field('native_exit').equals(0)
    return binding


def verify_test_product(product: Product, target: str, profile: str = 'release') -> None:
    """Follow artifact->build->actual compiler process; reject dirty development receipts."""
    require(target in TESTS, target, 'unregistered native family')
    verify_artifact(product, profile)
    artifact = document(bound_bytes(product.build_receipt, product.build_receipt_sha256).decode())
    artifact.field('binary').equals(str(product.binary))
    build_path = Path(artifact.field('build_receipt').text())
    build = document(bound_bytes(build_path, artifact.field('build_receipt_sha256').text()).decode())
    for key,value in (('status','PASS'), ('build_kind','integration_tests'), ('source_sha',product.source_sha),
                      ('root',str(product.root)), ('profile',profile), ('exit_code',0)):
        build.field(key).equals(value)
    build.field('packages').equals(['sim-test'])
    build.field('features').equals([])
    targets = tuple(row.text() for row in build.field('test_targets').array())
    require(target in targets and len(set(targets)) == len(targets), target, 'test target absent/duplicate')
    built = build.field('artifacts').field(target)
    built.field('path').equals(str(product.binary))
    built.field('sha256').equals(product.binary_sha256)
    require(product.binary.is_relative_to(product.root/f'rust/target/{profile}/deps'), target, 'test binary outside independent target')
    inputs = {name: Node(value).text() for name,value in build.field('effective_inputs_before').mapping().items()}
    require(dict(product.input_files) == inputs and len(product.input_files) == len(inputs), target, 'not full build input inventory')
    require(inventory(product.root, list(inputs)) == inputs, target, 'effective build input drift')
    for name in (FORMAT_SOURCE, f'rust/crates/sim-test/tests/{target}.rs'):
        require(name in inputs, name, 'missing source-planned input')
    require(inputs[f'rust/crates/sim-test/tests/{target}.rs'] == SOURCE_PINS[target], target, 'reviewed test source changed')
    source = build.field('source_before').mapping()
    source_kind = build.field('source_kind').text()
    artifact.field('source_kind').equals(source_kind)
    changed: dict[str, tuple[str | None, str]] = {}
    if source_kind == 'DERIVED_OBSERVER':
        overlay = build.field('overlay')
        _ = bound_bytes(Path(overlay.field('patch').text()), overlay.field('patch_sha256').text())
        manifest = document(bound_bytes(Path(overlay.field('manifest').text()), overlay.field('manifest_sha256').text()).decode())
        manifest.field('product_base_sha').equals(product.source_sha)
        manifest.field('observer_patch_sha256').equals(overlay.field('patch_sha256').value)
        for row in manifest.field('files').array():
            before = row.field('base0_sha256')
            changed[row.field('path').text()] = (None if before.value is None else before.text(), row.field('observer_sha256').text())
    else:
        require(source_kind == 'PRISTINE_COMMIT', target, 'unqualified source kind')
    _ = git(product.root, 'diff', '--cached', '--exit-code', 'HEAD')
    current = derived_snapshot(product.root, source, changed, git(product.root,'ls-files','--stage','-z'))
    build.field('effective_source_before').equals(current)
    build.field('effective_source_after').equals(current)
    compiler = _process_from_receipt(Path(build.field('process_receipt').text()), build.field('process_receipt_sha256').text())
    build.field('cargo_sha256').equals(compiler.process.executable_sha256)
    argv = compiler.process.argv
    expected_argv = [argv[0], 'test', '--locked', '--message-format=json-render-diagnostics', '--no-run']
    for name in targets:
        expected_argv.extend(('--test', name))
    if profile == 'release':
        expected_argv.append('--release')
    for package in build.field('packages').array():
        expected_argv.extend(('-p', package.text()))
    features = [feature.text() for feature in build.field('features').array()]
    if features:
        expected_argv.extend(('--features', ','.join(features)))
    require(list(argv) == expected_argv and compiler.process.cwd == product.root/'rust',
            target, 'not the source-bound original integration-test compilation')
    require(compiler.process.input_root == product.root and dict(compiler.process.inputs) == inputs, target, 'foreign compiler inputs')
    build.field('argv').equals(list(argv))
    messages = [document(line) for line in (compiler.process.output_root/'stdout.log').read_bytes().decode().splitlines()]
    terminals = [row for row in messages if row.field('reason').value == 'build-finished']
    require(len(terminals) == 1 and messages[-1] is terminals[0], target, 'missing/duplicate compiler terminal')
    terminals[0].field('success').equals(True)
    matches = [row for row in messages if row.field('reason').value == 'compiler-artifact'
               and row.field('target').field('name').value == target
               and row.field('executable').value == str(product.binary)]
    require(len(matches) == 1, target, 'missing/duplicate native test artifact')
    row = matches[0]
    row.field('target').field('kind').equals(['test'])
    row.field('profile').field('test').equals(True)
    row.field('fresh').equals(False)
    require(str(product.binary) in [value.text() for value in row.field('filenames').array()], target, 'artifact filename absent')


def freeze(root: Path) -> dict[str, str]:
    result: dict[str,str] = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), str(path), 'forbidden output symlink')
        if not path.is_dir():
            result[path.relative_to(root).as_posix()] = file_hash(path)
    return result


def authenticate_family(target: str, binding: Binding) -> tuple[Planned, ...]:
    root = binding.process.output_root
    require({path.name for path in root.iterdir()} ==
            {'process-start.json','process-spawn.json','process.json','stdout.log','stderr.log','tmp',
             f'save_load_pid{binding.pid}'}, target, 'unregistered native process output or extra world producer')
    stdout = bound_bytes(binding.process.output_root/'stdout.log', dict(binding.output_hashes)['stdout.log']).decode()
    outcomes = [(match.group(1),match.group(2)) for match in re.finditer(
        r'^test ([A-Za-z0-9_:]+) \.\.\. (ok|FAILED|ignored)\s*$', stdout, re.MULTILINE)]
    require(len(outcomes) == len(TESTS[target]) and {name for name,_ in outcomes} == set(TESTS[target]),
            target, 'missing/extra/duplicate original native test outcomes')
    require(all(status == 'ok' for _,status in outcomes), target, 'original native test failed or ignored')
    planned = (save_plan(binding) if target == 'save_load' else
               jobs_plan(_history_evidence(binding))[0] if target == 'colony_jobs' else fixed_plan(target))
    resolved = resolve(binding, tuple(row.occurrence for row in planned))
    for plan, observed in zip(planned, resolved, strict=True):
        require(observed.observation.stop_reason == plan.stop, identity(plan.occurrence), 'wrong native stop condition')
        require(observed.history_complete and not observed.pending, identity(plan.occurrence), 'unresolved native history')
    return planned


def native_summary(binding: Binding, target: str) -> None:
    stdout = bound_bytes(binding.process.output_root/'stdout.log',dict(binding.output_hashes)['stdout.log']).decode()
    summaries = re.findall(r'^test result: ok\. ([1-9][0-9]*) passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in .+$',stdout,re.MULTILINE)
    require(summaries == [str(len(TESTS[target]))], target, 'original positive native summary missing or altered')


def authenticate_invocation(path: Path, product: Product, plan: str) -> Binding:
    """Recheck a recorded identity against compiler and actual native process evidence."""
    receipt = document(path.read_bytes().decode())
    receipt.field('schema').equals('native_invocation_v1')
    receipt.field('plan').equals(plan)
    target = receipt.field('identity').field('target').text()
    selected = [row for row in invocations(plan) if row.target == target]
    require(len(selected) == 1, target, 'target absent from named invocation')
    row = selected[0]
    receipt.field('identity').equals(document(json.dumps(asdict(row))).value)
    require(receipt.field('arm').text() in {'base','candidate'}, str(path), 'unknown arm')
    receipt.field('artifact_receipt').equals(str(product.build_receipt))
    receipt.field('artifact_sha256').equals(product.build_receipt_sha256)
    verify_test_product(product,target,row.profile)
    binding = _process_from_receipt(Path(receipt.field('process_receipt').text()),receipt.field('process_sha256').text())
    spec = binding.process
    expected_cwd = product.root/('rust' if plan == 'development' else 'rust/crates/sim-test')
    require(spec.argv == (str(product.binary),*row.argv) and spec.cwd == expected_cwd, target, 'invocation argv/cwd mismatch')
    require(spec.executable == product.binary and spec.executable_sha256 == product.binary_sha256, target, 'foreign executable')
    require(spec.input_root == product.root and dict(spec.inputs) == dict(product.input_files) and len(spec.inputs) == len(product.input_files), target, 'foreign invocation inputs')
    expected_environment = {'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','LANG':'C','LC_ALL':'C','WORLDSIM_GATE_OBSERVER':'1','WORLDSIM_GATE_TEST_OUT':str(spec.output_root),
                                'TMPDIR':str(spec.output_root/'tmp')}
    if plan != 'development':
        expected_environment['WORLDSIM_GATE_TEST_INVOCATION'] = plan
    require(dict(spec.environment) == expected_environment and spec.fresh_directories == ('tmp',)
            and spec.external_inputs == (), target, 'foreign observer environment/inputs')
    require(spec.output_root == path.parent/f"{receipt.field('arm').text()}-{target}", target, 'foreign invocation output')
    receipt.field('pid').equals(binding.pid)
    receipt.field('argv').equals(list(spec.argv))
    receipt.field('cwd').equals(str(spec.cwd))
    hashes = freeze(spec.output_root)
    receipt.field('output_sha256').equals({name:digest for name,digest in hashes.items()})
    binding = Binding(spec,binding.receipt_sha256,binding.pid,tuple(hashes.items()))
    _ = authenticate_family(target,binding)
    if plan != 'development':
        native_summary(binding,target)
    return binding


def prepare(base: dict[str, Product], candidate: dict[str, Product], output: Path, *, invocation: str = 'development') -> None:
    selected = invocations(invocation)
    targets = {row.target for row in selected}
    require(set(base) == set(candidate) == targets, '/families', 'exact named invocation families required')
    require(output.is_absolute() and output == output.resolve() and not output.exists(), '/output', 'fresh canonical output required')
    for row in selected:
        target = row.target
        require(not base[target].root.is_relative_to(candidate[target].root) and
                not candidate[target].root.is_relative_to(base[target].root), target, 'independent roots required')
        require(not os.path.samefile(base[target].binary,candidate[target].binary), target, 'shared binary inode')
        for products in (base, candidate):
            product = products[target]
            require(not output.is_relative_to(product.root) and not product.root.is_relative_to(output), target, 'overlapping evidence/source roots')
            verify_test_product(product, target, row.profile)
    for products in (base, candidate):
        require(len({row.source_sha for row in products.values()}) == 1 and len({row.root for row in products.values()}) == 1,
                '/products', 'one independent source/build root per arm required')
        require(len({os.stat(row.binary).st_ino for row in products.values()}) == len(targets), '/products', 'test binaries reused')


def run_pair(base: dict[str, Product], candidate: dict[str, Product], output: Path, *, invocation: str = 'development') -> Path:
    selected = invocations(invocation)
    prepare(base, candidate, output, invocation=invocation)
    output.mkdir(parents=True)
    families: list[Json] = []
    report: dict[str,Json] = {'status':'PENDING', 'broader_gate17_37':'INCOMPLETE', 'publication_authorized':False,
        'invocation_plan':invocation, 'invocations':[document(json.dumps(asdict(row))).value for row in selected],
        'workspace_enumeration':'INCOMPLETE', 'step_coverage':'PARTIAL',
        'cargo_working_directory_equivalence':('NOT_ORIGINAL_CARGO_QUALIFICATION' if invocation == 'development' else 'PACKAGE_ROOT'),
        'products':{arm:{target:{'source_sha':product.source_sha, 'binary':str(product.binary),
                    'binary_sha256':product.binary_sha256, 'artifact_receipt':str(product.build_receipt),
                    'artifact_receipt_sha256':product.build_receipt_sha256}
                    for target,product in products.items()} for arm,products in (('base',base),('candidate',candidate))},
        'coverage':('save_load46 + colony_world20 + colony_commands3 + colony_jobs10 planned ordinary occurrences per arm'
                    if invocation == 'development' else 'Named invocation subset; see invocation_coverage.'),
        'invocation_coverage':{row.target:{'save_load':46,'colony_world':20,'colony_commands':3,'colony_jobs':10}[row.target] for row in selected},
        'limits':['Other world occurrences and native suite families remain unregistered.',
                  'Observer semantic qualification is a separate gate obligation.'], 'families':families, 'errors':[]}
    frozen: dict[Path,dict[str,str]] = {}
    frozen_receipts: dict[Path,str] = {}
    try:
        with lane():
            bindings: dict[tuple[str,str],Binding] = {}
            plans: dict[tuple[str,str],tuple[Planned,...]] = {}
            unresolved = False
            for invocation_row in selected:
                target = invocation_row.target
                declared_setup: Json = None
                for arm, products in (('base',base),('candidate',candidate)):
                    require(all(file_hash(path) == digest for path,digest in frozen_receipts.items()), '/invocations', 'prior invocation drift')
                    for directory, hashes in frozen.items():
                        require(freeze(directory) == hashes, str(directory), 'completed producer evidence changed')
                    product = products[target]
                    directory = output/f'{arm}-{target}'
                    cwd = product.root/('rust' if invocation == 'development' else 'rust/crates/sim-test')
                    spec = ProcessSpec((str(product.binary),*invocation_row.argv), cwd,
                        product.binary, product.binary_sha256, product.root, product.input_files, directory,
                        (('WORLDSIM_GATE_OBSERVER','1'), ('WORLDSIM_GATE_TEST_OUT',str(directory)),
                         ('TMPDIR',str(directory/'tmp'))) + (() if invocation == 'development' else
                         (('WORLDSIM_GATE_TEST_INVOCATION',invocation),)), fresh_directories=('tmp',))
                    result = execute(spec)
                    hashes = freeze(directory)
                    frozen[directory] = hashes
                    require(result.status == 'PASS' and result.native_exit == 0 and result.pid is not None,
                            target, 'native full test binary failed')
                    assert result.pid is not None
                    binding = Binding(spec, hashes['process.json'], result.pid, tuple(hashes.items()))
                    bindings[arm,target] = binding
                    plans[arm,target] = authenticate_family(target, binding)
                    if invocation != 'development':
                        native_summary(binding,target)
                    receipt: dict[str,Json] = {'schema':'native_invocation_v1', 'plan':invocation,
                        'identity':document(json.dumps(asdict(invocation_row))).value, 'arm':arm,
                        'process_receipt':str(directory/'process.json'), 'process_sha256':hashes['process.json'],
                        'pid':result.pid, 'argv':list(spec.argv), 'cwd':str(spec.cwd), 'output_sha256':{name:digest for name,digest in hashes.items()},
                        'artifact_receipt':str(product.build_receipt), 'artifact_sha256':product.build_receipt_sha256}
                    atomic_json(output/f'{arm}-{target}-invocation.json',receipt)
                    frozen_receipts[output/f'{arm}-{target}-invocation.json'] = file_hash(output/f'{arm}-{target}-invocation.json')
                if target == 'colony_jobs':
                    left, left_setup = jobs_plan(_history_evidence(bindings['base',target]))
                    right, right_setup = jobs_plan(_history_evidence(bindings['candidate',target]))
                    declared_setup = left_setup
                    if left_setup != right_setup or left != right:
                        unresolved = True
                        families.append({'target':target,'status':'UNRESOLVED','reason':'declared setup or actual horizon differs; matched companion required',
                            'base_setup':left_setup,'candidate_setup':right_setup,
                            'base_end_ticks':[row.occurrence.end_tick for row in left],
                            'candidate_end_ticks':[row.occurrence.end_tick for row in right]})
                        continue
                paired = pair(bindings['base',target], tuple(row.occurrence for row in plans['base',target]),
                              bindings['candidate',target], tuple(row.occurrence for row in plans['candidate',target]))
                families.append({'target':target, 'status':'PAIRED_SAFETY_PASS', 'declared_setup':declared_setup,
                    'occurrences':[document(json.dumps({'base':asdict(row.base), 'candidate':asdict(row.candidate),
                                                       'safety':row.safety})).value for row in paired.occurrences]})
            for products in (base,candidate):
                for row in selected:
                    verify_test_product(products[row.target],row.target,row.profile)
            require(all(file_hash(path) == digest for path,digest in frozen_receipts.items()), '/invocations', 'invocation receipt drift')
            require(all(freeze(directory) == hashes for directory,hashes in frozen.items()), '/outputs', 'terminal evidence drift')
            report['status'] = 'UNRESOLVED' if unresolved else 'COMPONENT_PASS'
    except (AuditError, ValueError, OSError, KeyError) as error:
        report['status'] = 'FAIL'
        report['errors'] = [str(error)]
    try:
        report['output_sha256'] = {name:digest for name,digest in freeze(output).items()}
        report['output_inventory_complete'] = True
        require(all(file_hash(path) == digest for path,digest in frozen_receipts.items()), '/invocations', 'final invocation drift')
        require(all(freeze(directory) == hashes for directory,hashes in frozen.items()), '/outputs', 'final evidence drift')
    except (ValueError, OSError) as error:
        report['status'] = 'FAIL'
        report['output_inventory_complete'] = False
        errors = report['errors']
        report['errors'] = [*(errors if isinstance(errors,list) else []),str(error)]
    terminal = output/'result.json'
    atomic_json(terminal,report)
    return terminal
