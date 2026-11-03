"""Actual retained V9 bytes; synthetic source binding, no native execution."""
import shutil
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

import generic_workspace_cases as cases
from generic_gate import GateContractError
from generic_process import ProcessSpec, file_hash
from generic_workspace_tests import Executable, Target


class NestedCasesTests(unittest.TestCase):
    def fixture(self, root: Path) -> tuple[ProcessSpec, Executable, bytes, cases.Listing, cases.Listing]:
        data=Path(__file__).parent/'fixtures/workspace_nested_sim_core'
        names=('content/tests.rs','content/ecology_tests.rs','knowledge/registry_tests.rs')
        for name in names:
            target=root/'rust/crates/sim-core/src/colony'/name;target.parent.mkdir(parents=True,exist_ok=True)
            _=shutil.copy2(data/name,target)
        source=root/'rust/crates/sim-core/src/lib.rs';_=source.write_text('// synthetic source binding, not a native binary proof\n')
        manifest=source.parent.parent/'Cargo.toml';_=manifest.write_text('[package]\nname="sim-core"\n')
        binary=root/'rust/target/debug/deps/sim_core-fixture';binary.parent.mkdir(parents=True);_=binary.write_bytes(b'inert')
        cargo=root/'cargo';_=cargo.write_bytes(b'inert compiler identity')
        target=Target('fixture:sim-core','sim-core','sim_core',('lib',),source,manifest,True,True)
        executable=Executable(target,binary,4,'debug')
        inputs=tuple((str(path.relative_to(root)),file_hash(path)) for path in sorted(root.rglob('*')) if path.is_file())
        spec=ProcessSpec((str(cargo),'test','--workspace','--no-fail-fast','--','--test-threads=8'),root/'rust',cargo,file_hash(cargo),root,inputs,root/'out')
        return spec,executable,(data/'stdout.log').read_bytes(),cases.parse_listing((data/'all.list').read_bytes()),cases.parse_listing((data/'ignored.list').read_bytes())

    def test_actual_raw_requires_policy_and_retains_outer_and_nested_counts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            spec,target,raw,all_cases,ignored=self.fixture(Path(temporary).resolve())
            with self.assertRaises(GateContractError):_=cases.reconcile_native(raw,all_cases,ignored)
            source=cases.nested_native_source(spec,target)
            with self.assertRaises(ValueError):
                _=cases.nested_native_source(replace(spec,environment=(('WORLDSIM_CONTENT_TEST_CASE','bundled'),)),target)
            self.assertIsNotNone(source)
            result=cases.reconcile_native(raw,all_cases,ignored,source)
            self.assertEqual(len(result.outcomes),491);self.assertEqual(result.failed,0)
            self.assertEqual(result.sha256,cases.digest(raw))
            self.assertIsNotNone(result.nested)
            assert result.nested is not None
            self.assertEqual(result.nested.outer_cases,491);self.assertEqual(result.nested.nested_passed,7)
            self.assertEqual([row.count for row in result.nested.children],[4,2,1])
            self.assertEqual(result.nested.pid_tree,'NOT_OBSERVED')

    def test_source_and_command_drift_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            spec,target,raw,all_cases,ignored=self.fixture(Path(temporary).resolve())
            source=cases.nested_native_source(spec,target)
            changed=spec.input_root/'rust/crates/sim-core/src/colony/content/tests.rs'
            _=changed.write_text(changed.read_text().replace('["uninstalled", "failed", "bundled", "extension"]','["bundled"]'))
            with self.assertRaises(GateContractError):_=cases.reconcile_native(raw,all_cases,ignored,source)
            resealed=replace(spec,inputs=tuple((name,file_hash(spec.input_root/name)) for name,_ in spec.inputs))
            with self.assertRaises(GateContractError):_=cases.nested_native_source(resealed,target)
            with self.assertRaises(GateContractError):_=cases.nested_native_source(replace(spec,argv=(*spec.argv,'--ignored')),target)

    def test_nested_missing_extra_wrong_counts_and_ambiguous_status_reject(self) -> None:
        from gate_test_summary import SummaryError
        with tempfile.TemporaryDirectory() as temporary:
            spec,target,raw,all_cases,ignored=self.fixture(Path(temporary).resolve())
            source=cases.nested_native_source(spec,target)
            child=b'test colony::content::tests::bootstrap_child ... ok\n'
            summary=b'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 490 filtered out; finished in 0.00s\n'
            variants=(raw.replace(child,b'',1),raw.replace(child,child+child,1),
                      raw.replace(b'490 filtered out',b'489 filtered out',1),
                      raw.replace(summary,summary+summary,1),
                      raw.replace(child,child.replace(b' ... ok',b' ... FAILED'),1),
                      raw.replace(child,b'test invented::child ... ok\n',1),
                      raw.replace(b'running 1 test',b'running 2 tests',1),
                      raw[:-1],raw.replace(b'\n',b'\r\n'),raw+b'extra terminal text\n')
            for malformed in variants:
                with self.subTest(sha=cases.digest(malformed)),self.assertRaises((GateContractError,SummaryError)):
                    _=cases.reconcile_native(malformed,all_cases,ignored,source)
            child_ignored=replace(ignored,names=('colony::content::tests::bootstrap_child',))
            with self.assertRaises(GateContractError):_=cases.reconcile_native(raw,all_cases,child_ignored,source)

    def test_ordinary_outer_failure_is_not_promoted(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            spec,target,raw,all_cases,ignored=self.fixture(Path(temporary).resolve())
            raw=raw.replace(b'test causal::event::tests::accessors_round_trip ... ok',
                            b'test causal::event::tests::accessors_round_trip ... FAILED',1)
            raw=raw.replace(b'test result: ok. 491 passed; 0 failed;',b'test result: FAILED. 490 passed; 1 failed;')
            result=cases.reconcile_native(raw,all_cases,ignored,cases.nested_native_source(spec,target))
            self.assertEqual(result.failed,1)
            self.assertEqual(result.outcomes[0].status,'FAILED')

    def test_unrelated_source_edit_requires_new_binding_but_not_new_test_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            spec,target,raw,all_cases,ignored=self.fixture(Path(temporary).resolve())
            path=spec.input_root/'rust/crates/sim-core/src/colony/content/tests.rs'
            _=path.write_bytes(b'// unrelated source comment\n'+path.read_bytes())
            with self.assertRaises(GateContractError):_=cases.nested_native_source(spec,target)
            bound=replace(spec,inputs=tuple((name,file_hash(spec.input_root/name)) for name,_ in spec.inputs))
            result=cases.reconcile_native(raw,all_cases,ignored,cases.nested_native_source(bound,target))
            self.assertEqual(len(result.outcomes),491)
            with self.assertRaises(GateContractError):_=cases.nested_native_source(replace(bound,inputs=()),target)
            with self.assertRaises(GateContractError):_=cases.nested_native_source(bound,replace(target,threads=1))
            with self.assertRaises(GateContractError):_=cases.nested_native_source(bound,replace(target,filter='child'))
            with self.assertRaises(GateContractError):_=cases.nested_native_source(bound,replace(target,profile='release'))

    def test_single_summary_default_and_foreign_target_remain_strict(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            spec,target,_,_,_=self.fixture(Path(temporary).resolve())
            self.assertIsNone(cases.nested_native_source(spec,replace(target,target=replace(target.target,name='other'))))
            listing=cases.parse_listing(b'a: test\n\n1 test, 0 benchmarks\n')
            ignored=cases.parse_listing(b'0 tests, 0 benchmarks\n')
            raw=b'\nrunning 1 test\ntest a ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n\n'
            self.assertIsNone(cases.reconcile_native(raw,listing,ignored).nested)
            with self.assertRaises(GateContractError):_=cases.reconcile_native(raw+raw,listing,ignored)

    def test_source_bound_direct_partition_and_malformed_suffix(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            spec,target,wrapped,all_cases,ignored=self.fixture(Path(temporary).resolve())
            source=cases.nested_native_source(spec,target)
            raw=(Path(__file__).parent/'fixtures/workspace_nested_sim_core/direct-sim-core.log').read_bytes()
            tail=b'\nrunning 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n'
            ranges=cases.direct_section_ranges(raw+tail,((all_cases,ignored,source),),1)
            self.assertEqual(ranges,((0,len(raw),cases.digest(raw)),(len(raw),len(raw+tail),cases.digest(tail))))
            left=cases.qualified_native_section(raw,all_cases,ignored,source)
            right=cases.qualified_native_section(wrapped,all_cases,ignored,source)
            self.assertEqual(left.outcomes,right.outcomes)
            self.assertEqual(left.nested,right.nested)
            for bad,rows,docs in ((raw+tail,((all_cases,ignored,None),),1),
                                  (raw+tail,((all_cases,ignored,source),),0),
                                  (raw+tail+tail,((all_cases,ignored,source),),1),
                                  (raw[:-1]+tail,((all_cases,ignored,source),),1),
                                  (raw+tail+b'late',((all_cases,ignored,source),),1)):
                with self.subTest(sha=cases.digest(bad)),self.assertRaises(ValueError):
                    _=cases.direct_section_ranges(bad,rows,docs)

    def test_six_closed_targets_use_source_derived_synthetic_listings(self) -> None:
        # These are parser fixtures, not retained independent native listing admissions.
        import re
        targets=(('sim-core','recipe_registry',1,None),('sim-test','b5_flora',1,None),
                 ('sim-test','blueprint_completion_identity',1,None),('sim-test','data_only_ext',9,'WORLDSIM_DATA_ONLY_CASE'),
                 ('sim-test','predator_registry_runtime',1,'WORLDSIM_PREDATOR_REGISTRY_CHILD'),
                 ('sim-test','recipe_registry_runtime',1,None))
        for package,name,count,environment_key in targets:
            with self.subTest(target=name),tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary).resolve()
                data=Path(__file__).parent/'fixtures/workspace_nested_extra'/name
                source=root/f'rust/crates/{package}/tests/{name}.rs';source.parent.mkdir(parents=True)
                _=source.write_bytes((data/'source.rs').read_bytes())
                manifest=root/'rust/crates'/package/'Cargo.toml';_=manifest.write_text('[package]\nname="'+package+'"\n')
                binary=root/'rust/target/debug/deps'/(name+'-fixture');binary.parent.mkdir(parents=True);_=binary.write_bytes(b'inert')
                cargo=root/'cargo';_=cargo.write_bytes(b'inert')
                inputs=tuple((str(p.relative_to(root)),file_hash(p)) for p in sorted(root.rglob('*')) if p.is_file())
                spec=ProcessSpec((str(cargo),'test','--workspace','--no-fail-fast','--','--test-threads=8'),root/'rust',cargo,file_hash(cargo),root,inputs,root/'out')
                target=Executable(Target('fixture:'+package,package,name,('test',),source,manifest,True,False),binary,4,'debug')
                tests=tuple((match[1],match[2]) for match in re.finditer(r'#\[test\]\n((?:#\[[^\n]+\]\n)*)fn (\w+)\(',source.read_text()))
                def listing(names: tuple[str,...]) -> cases.Listing:
                    raw=''.join(item+': test\n' for item in names)+ ('\n' if names else '')+f'{len(names)} '+('test' if len(names)==1 else 'tests')+', 0 benchmarks\n'
                    return cases.parse_listing(raw.encode())
                all_cases=listing(tuple(sorted(item[1] for item in tests)))
                ignored=listing(tuple(sorted(item[1] for item in tests if '#[ignore' in item[0])))
                raw=(data/'stdout.log').read_bytes();policy=cases.nested_native_source(spec,target)
                parsed=cases.reconcile_native(raw,all_cases,ignored,policy)
                self.assertIsNotNone(parsed.nested)
                assert parsed.nested is not None
                self.assertEqual(parsed.nested.nested_passed,count);self.assertEqual(len(parsed.outcomes),len(tests))
                self.assertEqual(cases.direct_section_ranges(raw,((all_cases,ignored,policy),),0),((0,len(raw),cases.digest(raw)),))
                with self.assertRaises(ValueError):_=cases.reconcile_native(raw,all_cases,ignored)
                with self.assertRaises(ValueError):_=cases.reconcile_native(raw,replace(all_cases,names=all_cases.names[:-1]),ignored,policy)
                with self.assertRaises(ValueError):_=cases.reconcile_native(raw.replace(b'running 1 test',b'running 2 tests',1),all_cases,ignored,policy)
                # Resealed source cannot alter a guarded launch/recursion contract.
                _=source.write_text(source.read_text().replace('--nocapture','--show-output'))
                resealed=replace(spec,inputs=tuple((path,file_hash(root/path)) for path,_ in spec.inputs))
                with self.assertRaises(ValueError):_=cases.nested_native_source(resealed,target)
                if environment_key is not None:
                    _=source.write_bytes((data/'source.rs').read_bytes().replace(environment_key.encode(),b'WRONG_ENVIRONMENT_GUARD'))
                    resealed=replace(spec,inputs=tuple((path,file_hash(root/path)) for path,_ in spec.inputs))
                    with self.assertRaises(ValueError):_=cases.nested_native_source(resealed,target)

    def test_unlisted_progress_and_nonignored_suffix_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            spec,target,raw,all_cases,ignored=self.fixture(Path(temporary).resolve())
            policy=cases.nested_native_source(spec,target)
            unknown=b'test nonexistent::unlisted has been running for over 60 seconds\n'
            ordinary=b'test causal::event::tests::accessors_round_trip ... ok\n'
            for malformed in (raw.replace(b'running 491 tests\n',b'running 491 tests\n'+unknown,1),
                              raw.replace(ordinary,ordinary.replace(b' ... ok',b' ... ok, invented arbitrary diagnostic'),1),
                              raw.replace(ordinary,ordinary.replace(b' ... ok',b' ... FAILED, invented arbitrary diagnostic'),1)):
                with self.subTest(sha=cases.digest(malformed)),self.assertRaises(ValueError):
                    _=cases.reconcile_native(malformed,all_cases,ignored,policy)
            progress=b'test causal::event::tests::accessors_round_trip has been running for over 60 seconds\n'
            result=cases.reconcile_native(raw.replace(ordinary,progress+ordinary,1),all_cases,ignored,policy)
            self.assertEqual(len(result.outcomes),491)
