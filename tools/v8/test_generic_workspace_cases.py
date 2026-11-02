"""Offline listing/section fixtures, never native listing or test execution."""
import sys
import unittest

from generic_gate import GateContractError
from generic_workspace_cases import parse_listing, parse_section


class ParserTests(unittest.TestCase):
    def test_listing_names_and_explicit_zero(self) -> None:
        listed = parse_listing('src/a.rs - item::한글 (line 4): test\n\n1 test, 0 benchmarks\n'.encode())
        self.assertEqual(listed.names, ('src/a.rs - item::한글 (line 4)',))
        self.assertEqual(parse_listing(b'0 tests, 0 benchmarks\n').names, ())

    def test_modes_are_not_part_of_expected_names(self) -> None:
        name = 'src/a.rs - item (line 4)'
        raw = f'\nrunning 1 test\ntest {name} - compile fail ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n\n'.encode()
        section = parse_section(raw, (name,), ())
        self.assertEqual(section.outcomes[0].mode, 'compile fail')
        self.assertEqual(section.outcomes[0].name, name)

    def test_truncated_and_crlf_listing_reject(self) -> None:
        for raw in (b'a: test\n', b'0 tests, 0 benchmarks\r\n', b'a: test\na: test\n\n2 tests, 0 benchmarks\n'):
            with self.subTest(raw=raw), self.assertRaises(GateContractError):
                _ = parse_listing(raw)

    def test_bad_plural_extra_and_opaque_colon_names(self) -> None:
        self.assertEqual(parse_listing(b'a: b: test\n\n1 test, 0 benchmarks\n').names, ('a: b',))
        for raw in (b'0 test, 0 benchmarks\n', b'0 tests, 0 benchmarks\n\n', b'a: what\n\n1 test, 0 benchmarks\n'):
            with self.subTest(raw=raw), self.assertRaises(GateContractError): _ = parse_listing(raw)

    def test_ignored_set_missing_case_and_no_run_mode(self) -> None:
        raw = b'\nrunning 1 test\ntest a - compile ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0s\n\n'
        self.assertEqual(parse_section(raw, ('a',), ()).outcomes[0].mode, 'compile')
        with self.assertRaises(GateContractError): _ = parse_section(raw, ('a',), ('a',))
        with self.assertRaises(GateContractError): _ = parse_section(raw, ('b',), ())

    def test_failed_section_retains_outcome_but_unsupported_diagnostics_reject(self) -> None:
        raw = b'\nrunning 1 test\ntest a ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0s\n\n'
        self.assertEqual(parse_section(raw, ('a',), ()).failed, 1)
        with self.assertRaises(GateContractError): _ = parse_section(raw.replace(b'\ntest result:', b'\nfailures:\ninjected\ntest result:'), ('a',), ())


# These imported builders write synthetic receipts only; no tools/processes run.
from dataclasses import replace
from pathlib import Path

from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from generic_workspace_capture import read
from generic_workspace_cases import (
    CaseLists,
    ListingProcess,
    flattened_test_args,
    join_standalone_sections,
)
from mux2_gate_inputs import Json
from test_generic_workspace_capture import (
    exec_record,
    policy_fixture,
    policy_plan,
    policy_static,
    write,
)


def listing_process(root: Path, spec: ProcessSpec, raw: bytes) -> ListingProcess:
    root.mkdir()
    spec = replace(spec, output_root=root)
    for name, content in (('stdout', raw), ('stderr', b'')): _ = (root/(name+'.log')).write_bytes(content)
    row: dict[str, Json] = {'schema_version': 1, 'status': 'PASS', 'native_exit': 0, 'pid': 600, 'errors': [],
        'argv': list(spec.argv), 'cwd': str(spec.cwd), 'environment': dict(environment_for(spec)),
        'environment_policy': 'fixed_defaults_plus_explicit_only_no_inheritance', 'executable': str(spec.executable),
        'input_root': str(spec.input_root), 'output_root': str(root), 'fresh_directories': [], 'started_at': '2026-11-03T00:00:00+00:00',
        'ended_at': '2026-11-03T00:00:02+00:00', 'duration_ns': 2000000000, 'build_provenance_verified': False}
    for prefix in ('expected_', 'before_', 'after_'):
        row[prefix+'executable_sha256'] = spec.executable_sha256
        row[prefix+'inputs'] = {k: v for k, v in spec.inputs}
        row[prefix+'external_inputs'] = {str(k): v for k, v in spec.external_inputs}
    for name in ('stdout', 'stderr'): row[name+'_sha256'] = file_hash(root/(name+'.log'))
    write(root/'process.json', row)
    start = {key: value for key, value in row.items() if key not in ('ended_at', 'duration_ns', 'stdout_sha256', 'stderr_sha256', 'after_executable_sha256', 'after_inputs')}
    start.update({'pid': None, 'native_exit': None, 'status': 'FAIL', 'after_external_inputs': {}})
    write(root/'process-start.json', start)
    write(root/'process-spawn.json', {'pid': 600, 'spawned_at': '2026-11-03T00:00:01+00:00'})
    return ListingProcess(spec, ProcessResult(root/'process.json', 'PASS', 0, 600), file_hash(root/'process.json'))


class OrdinalTests(unittest.TestCase):
    def fixture(self, count: int = 1, documents: int = 1):
        f = policy_fixture(self)
        manifest = read(f.manifest).mapping(); policy = read(f.manifest).field('doctest_policy').mapping()
        source = read(f.manifest).field('doctest_policy').field('invocations').array()[0].mapping()
        argv = read(f.manifest).field('doctest_policy').field('invocations').array()[0].field('argv').array()
        source['argv'] = [*(arg.text() for arg in argv), '--test-args', '--test-threads=8']
        sources: list[Json] = [source]
        if documents == 2:
            target = f.executables[1].target
            second_args = [arg.text() for arg in argv]
            second_args[second_args.index('--test')+1] = str(target.source)
            second_args[second_args.index('--crate-name')+1] = 'beta'
            second_args[second_args.index('--test-run-directory')+1] = str(target.manifest.parent)
            sources.append({**source, 'id': 'beta-doc', 'package_id': 'beta', 'source': str(target.source),
                'source_argument': str(target.source), 'package_manifest': str(target.manifest), 'crate_name': 'beta',
                'run_cwd': str(target.manifest.parent), 'argv': [*second_args, '--test-args', '--test-threads=8']})
            inputs = read(f.manifest).field('inputs').mapping()
            inputs.update({str(path): file_hash(path) for path in (target.source, target.manifest)})
            manifest['inputs'] = inputs
        policy['invocations'] = sources; manifest['doctest_policy'] = policy
        write(f.manifest, manifest)
        f.spec = replace(f.spec, environment=(('CARGO_CAPTURE_MANIFEST', str(f.manifest)), ('CARGO_CAPTURE_MANIFEST_SHA256', file_hash(f.manifest)),
                         ('CARGO_TARGET_SYNTHETIC_RUNNER', str(Path(sys.executable).resolve())+' '+str(f.helper))),
                         external_inputs=tuple((p, file_hash(p)) for p, _ in f.spec.external_inputs))
        f.parent(0)
        one = policy_static(f, 1); two = policy_static(f, 2); intent = exec_record(f)
        policy_node = read(f.manifest).field('doctest_policy')
        args = tuple(arg.text() for arg in policy_node.field('invocations').array()[0].field('argv').array())
        import shlex
        stderr = b''
        for executable in f.executables:
            command = (str(Path(sys.executable).resolve()), str(f.helper), str(executable.binary), '--test-threads=8')
            stderr += ('     Running `'+command[0]+' '+' '.join(shlex.quote(arg) for arg in command[1:])+'`\n').encode()
        intents = [intent]
        invocation_rows = policy_node.field('invocations').array()
        if documents == 2:
            second_intent = exec_record(f, 'exec-two')
            second_args = [arg.text() for arg in invocation_rows[1].field('argv').array()]
            for filename in ('exec-entry.json', 'exec-intent.json'):
                data = read(second_intent.with_name(filename)).mapping()
                data['rustdoc_argv'] = [policy_node.field('rustdoc').text(), *second_args]
                old_wrapper = read(second_intent.with_name(filename)).field('wrapper_argv').array()[0].text()
                data['wrapper_argv'] = [old_wrapper, *second_args]
                if filename == 'exec-intent.json': data['source_invocation'] = 'beta-doc'
                write(second_intent.with_name(filename), data)
            intents.append(second_intent)
        for invocation in invocation_rows:
            invocation_args = tuple(arg.text() for arg in invocation.field('argv').array())
            stderr += ('     Running `'+policy_node.field('entrypoint').text()+' '+' '.join(shlex.quote(arg) for arg in invocation_args)+'`\n').encode()
        section = b'\nrunning 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n'
        stdout = (one/'stdout.log').read_bytes()+(two/'stdout.log').read_bytes()+section*count
        for name, raw in (('stdout', stdout), ('stderr', stderr)):
            _ = (f.output/(name+'.log')).write_bytes(raw); f.parent_data[name+'_sha256'] = file_hash(f.output/(name+'.log'))
        write(f.output/'process.json', f.parent_data)
        environment = tuple((k, read(intent).field('environment').field(k).text()) for k in read(intent).field('environment').mapping())
        bound = {k: read(f.manifest).field('inputs').field(k).text() for k in read(f.manifest).field('inputs').mapping()}
        bound.update({str(f.manifest): file_hash(f.manifest), **{str(path): file_hash(path) for path in intents}})
        bound.update({row.field('path').text(): row.field('sha256').text() for row in read(f.manifest).field('artifacts').array()})
        tool = Path(policy_node.field('rustdoc').text())
        spec = ProcessSpec((str(tool), *args), f.source/'rust', tool, file_hash(tool), f.source, (), f.base/'unused', environment,
                           tuple((Path(k), v) for k, v in bound.items()))
        pairs: list[CaseLists] = []
        for index, invocation in enumerate(invocation_rows):
            invocation_args = tuple(arg.text() for arg in invocation.field('argv').array())
            local = replace(spec, argv=(str(tool), *invocation_args))
            all_cases = listing_process(f.base/f'all{index}', replace(local, argv=(*local.argv, '--test-args', '--list')), b'0 tests, 0 benchmarks\n')
            ignored = listing_process(f.base/f'ignored{index}', replace(local, argv=(*local.argv, '--test-args', '--list', '--test-args', '--ignored')), b'0 tests, 0 benchmarks\n')
            pairs.append(CaseLists(invocation.field('id').text(), all_cases, ignored))
        return f, tuple(pairs)

    def test_actual_source_and_process_join_zero_section(self) -> None:
        f, lists = self.fixture()
        result = join_standalone_sections(policy_plan(f), f.result, lists)
        self.assertEqual(result.errors, ())
        self.assertEqual(result.status, 'COMPLETE_OBSERVED_AND_LISTED_DOCTEST_CASES')
        self.assertEqual(result.sections[0].invocation, 'alpha-doc')
        self.assertEqual(result.world_safety, 'INCOMPLETE')

    def test_missing_or_extra_section_never_shifts(self) -> None:
        for count in (0, 2):
            with self.subTest(count=count):
                f, lists = self.fixture(count)
                result = join_standalone_sections(policy_plan(f), f.result, lists)
                self.assertEqual(result.status, 'INCOMPLETE')
                self.assertLessEqual(len(result.sections), 1)

    def test_listing_changed_argv_environment_or_exit_rejects(self) -> None:
        for kind in ('argv', 'environment', 'exit'):
            with self.subTest(kind=kind):
                f, lists = self.fixture()
                item = lists[0].all_cases
                if kind == 'argv': item = replace(item, spec=replace(item.spec, argv=(*item.spec.argv, '--ignored')))
                if kind == 'environment': item = replace(item, spec=replace(item.spec, environment=(*item.spec.environment, ('UNBOUND', '1'))))
                if kind == 'exit': item = replace(item, result=replace(item.result, native_exit=1, status='FAIL'))
                result = join_standalone_sections(policy_plan(f), f.result, (replace(lists[0], all_cases=item),))
                self.assertEqual(result.status, 'INCOMPLETE')
                self.assertEqual(result.sections, ())

    def test_flattened_original_args_are_closed(self) -> None:
        self.assertEqual(flattened_test_args(('--test-args=--test-threads=8',)), ('--test-threads=8',))
        for args in (('--test-args',), ('--test-args', '--list'), ('--test-args', '--test-threads=8 --ignored')):
            with self.subTest(args=args), self.assertRaises(GateContractError): _ = flattened_test_args(args)

    def test_two_identical_zero_sections_use_declaration_order(self) -> None:
        from generic_workspace_tests import Inventory

        f, lists = self.fixture(count=2, documents=2)
        plan = replace(policy_plan(f), inventory=Inventory(tuple(f.executables), tuple(row.target for row in f.executables)))
        result = join_standalone_sections(plan, f.result, lists)
        self.assertEqual(result.errors, ())
        self.assertEqual([row.invocation for row in result.sections], ['alpha-doc', 'beta-doc'])
        self.assertEqual(result.sections[0].end, result.sections[1].start)
        self.assertEqual(result.sections[0].section.sha256, result.sections[1].section.sha256)

    def test_two_obligations_one_section_cannot_assign_which_target(self) -> None:
        from generic_workspace_tests import Inventory

        f, lists = self.fixture(count=1, documents=2)
        plan = replace(policy_plan(f), inventory=Inventory(tuple(f.executables), tuple(row.target for row in f.executables)))
        result = join_standalone_sections(plan, f.result, lists)
        self.assertEqual(result.status, 'INCOMPLETE')
        self.assertEqual(result.sections, ())

    def test_source_or_listing_mutation_cannot_be_frozen_as_success(self) -> None:
        f, lists = self.fixture()
        _ = lists[0].all_cases.spec.output_root.joinpath('stdout.log').write_bytes(b'1 test, 0 benchmarks\n')
        result = join_standalone_sections(policy_plan(f), f.result, lists)
        self.assertEqual(result.status, 'INCOMPLETE')

    def test_edition_2024_rejected_not_forced_standalone(self) -> None:
        f, lists = self.fixture()
        # A changed policy cannot keep the authenticated manifest/process bindings.
        raw = f.manifest.read_bytes().replace(b'2021', b'2024')
        _ = f.manifest.write_bytes(raw)
        self.assertEqual(join_standalone_sections(policy_plan(f), f.result, lists).status, 'INCOMPLETE')

    def test_reverse_declarations_attach_identical_sections_by_ordinal(self) -> None:
        from generic_workspace_tests import Inventory

        f, lists = self.fixture(count=2, documents=2)
        path = f.output/'stderr.log'
        chunks = path.read_bytes().splitlines(keepends=True)
        _ = path.write_bytes(b''.join([*chunks[:2], *reversed(chunks[2:])]))
        f.parent_data['stderr_sha256'] = file_hash(path); write(f.result.receipt, f.parent_data)
        plan = replace(policy_plan(f), inventory=Inventory(tuple(f.executables), tuple(row.target for row in f.executables)))
        result = join_standalone_sections(plan, f.result, lists)
        self.assertEqual(result.errors, ())
        self.assertEqual([row.invocation for row in result.sections], ['beta-doc', 'alpha-doc'])

    def test_static_prefix_mismatch_not_recovered_by_suffix_search(self) -> None:
        f, lists = self.fixture()
        path = f.output/'stdout.log'; _ = path.write_bytes(b'unowned\n'+path.read_bytes())
        f.parent_data['stdout_sha256'] = file_hash(path); write(f.result.receipt, f.parent_data)
        result = join_standalone_sections(policy_plan(f), f.result, lists)
        self.assertEqual(result.status, 'INCOMPLETE')
        self.assertEqual(result.sections, ())

    def test_mutation_after_listing_parse_retained_at_final(self) -> None:
        from unittest.mock import patch

        import generic_workspace_cases as cases

        f, lists = self.fixture()
        original = cases.listing_output

        def mutate(binding: ListingProcess, argv: tuple[str, ...], cwd: Path,
                   environment: dict[str, str], required: dict[str, str]) -> bytes:
            raw = original(binding, argv, cwd, environment, required)
            if binding == lists[0].all_cases:
                _ = binding.spec.output_root.joinpath('stdout.log').write_bytes(raw+b'extra\n')
            return raw

        with patch.object(cases, 'listing_output', side_effect=mutate):
            result = cases.join_standalone_sections(policy_plan(f), f.result, lists)
        self.assertEqual(result.status, 'INCOMPLETE')
        self.assertTrue(any('membership/bytes changed' in error for error in result.errors))

    def test_unrecognized_declaration_and_parent_nonzero_preserved(self) -> None:
        f, lists = self.fixture()
        path = f.output/'stderr.log'; _ = path.write_bytes(path.read_bytes()+b'     Running `unknown`\n')
        f.parent_data['stderr_sha256'] = file_hash(path)
        f.parent_data.update({'status': 'FAIL', 'native_exit': 101, 'errors': ['native exit 101']})
        write(f.result.receipt, f.parent_data)
        f.result = replace(f.result, status='FAIL', native_exit=101)
        result = join_standalone_sections(policy_plan(f), f.result, lists)
        self.assertEqual(result.native_outcome, 'FAILED')
        self.assertEqual(result.status, 'INCOMPLETE')


    def test_missing_first_or_last_identical_zero_with_cargo_failure_unassigned(self) -> None:
        from generic_workspace_tests import Inventory

        f, lists = self.fixture(count=1, documents=2)
        f.parent_data.update({'status': 'FAIL', 'native_exit': 101, 'errors': ['native exit 101']})
        write(f.result.receipt, f.parent_data)
        f.result = replace(f.result, status='FAIL', native_exit=101)
        plan = replace(policy_plan(f), inventory=Inventory(tuple(f.executables), tuple(row.target for row in f.executables)))
        result = join_standalone_sections(plan, f.result, lists)
        self.assertEqual(result.sections, ())
        self.assertEqual(result.native_outcome, 'FAILED')

    def test_missing_listing_completion_fields_rejected(self) -> None:
        f, lists = self.fixture()
        item = lists[0].all_cases
        row = read(item.result.receipt).mapping(); del row['ended_at']; write(item.result.receipt, row)
        lists = (replace(lists[0], all_cases=replace(item, receipt_sha256=file_hash(item.result.receipt))),)
        result = join_standalone_sections(policy_plan(f), f.result, lists)
        self.assertEqual(result.status, 'INCOMPLETE')

    def test_resealed_lifecycle_omissions_invalid_and_reversed_fields_reject(self) -> None:
        for defect in ('duration_missing', 'duration_bool', 'duration_zero', 'end_missing', 'end_naive', 'end_reversed',
                       'spawn_missing', 'spawn_naive', 'spawn_reversed', 'start_changed'):
            with self.subTest(defect=defect):
                f, lists = self.fixture()
                item = lists[0].all_cases; root = item.spec.output_root
                row = read(item.result.receipt).mapping()
                spawn = read(root/'process-spawn.json').mapping()
                start = read(root/'process-start.json').mapping()
                if defect == 'duration_missing': del row['duration_ns']
                elif defect == 'duration_bool': row['duration_ns'] = True
                elif defect == 'duration_zero': row['duration_ns'] = 0
                elif defect == 'end_missing': del row['ended_at']
                elif defect == 'end_naive': row['ended_at'] = '2026-11-03T00:00:02'
                elif defect == 'end_reversed': row['ended_at'] = '2026-11-02T00:00:02+00:00'
                elif defect == 'spawn_missing': del spawn['spawned_at']
                elif defect == 'spawn_naive': spawn['spawned_at'] = '2026-11-03T00:00:01'
                elif defect == 'spawn_reversed': spawn['spawned_at'] = '2026-11-04T00:00:01+00:00'
                else: start['expected_inputs'] = {'unbound': 'changed'}
                write(item.result.receipt, row); write(root/'process-spawn.json', spawn); write(root/'process-start.json', start)
                changed = replace(lists[0], all_cases=replace(item, receipt_sha256=file_hash(item.result.receipt)))
                result = join_standalone_sections(policy_plan(f), f.result, (changed,))
                self.assertEqual(result.status, 'INCOMPLETE')
                self.assertEqual(result.sections, ())

    def test_ambiguous_short_stream_retains_only_raw_ranges(self) -> None:
        from generic_workspace_tests import Inventory

        f, lists = self.fixture(count=1, documents=2)
        plan = replace(policy_plan(f), inventory=Inventory(tuple(f.executables), tuple(row.target for row in f.executables)))
        result = join_standalone_sections(plan, f.result, lists)
        self.assertEqual(result.sections, ())
        self.assertEqual(len(result.observed_ranges), 1)
        self.assertEqual(len(result.observed_ranges[0][2]), 64)
