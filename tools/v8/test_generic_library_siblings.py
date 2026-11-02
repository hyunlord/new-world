"""Synthetic parser/process receipts only; no Cargo, native libtest, or build qualification."""
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_library_siblings as library
from gate_test_summary import SummaryError
from generic_gate import GateContractError
from generic_process import ProcessSpec, environment_for, file_hash
from generic_workspace_cases import (
    ListingProcess,
    parse_listing,
    process_output,
    reconcile_native,
)
from test_generic_workspace_cases import listing_process


def listing(names: tuple[str, ...]) -> bytes:
    return (''.join(name+': test\n' for name in names)+ ('\n' if names else '')
            + f'{len(names)} '+('test' if len(names) == 1 else 'tests')+', 0 benchmarks\n').encode()


def native(names: tuple[str, ...], filtered: int, *, failed: int = 0, noise: str = '') -> bytes:
    rows=''.join('test '+name+' ... '+('FAILED' if index < failed else 'ok')+'\n' for index,name in enumerate(names))
    return (f'\nrunning {len(names)} '+('test' if len(names)==1 else 'tests')+'\n'+noise+rows+'\n'
            +'test result: '+('FAILED' if failed else 'ok')+f'. {len(names)-failed} passed; {failed} failed; 0 ignored; 0 measured; {filtered} filtered out; finished in 0.01s\n\n').encode()


def evidence(step: library.LibraryStep) -> library.SelectedEvidence:
    names=library.recipe(step).expected
    return library.SelectedEvidence(step,listing((*names,'unrelated::case','unrelated::ignored')),
                                    listing(('unrelated::ignored',)),native(names,2),0)


class RecipeTests(unittest.TestCase):
    def test_exact_closed_commands_and_profiles(self) -> None:
        root=Path('/product');log=Path('/evidence/run.log')
        for step in library.LibraryStep:
            with self.subTest(step=step):
                prepared=library.prepare(step,root,log)
                self.assertEqual(prepared.dispatch,'NOT_RUN');self.assertEqual(prepared.source_build_admission,'PENDING')
                self.assertEqual(prepared.wrapper_admission,'PENDING')
                self.assertNotIn('--test-threads', ' '.join(prepared.cargo_argv))
                self.assertEqual(prepared.outer_cwd,root)
                self.assertEqual(prepared.recipe.target_kind,'lib')
        first=library.prepare(library.LibraryStep.BASELINE_ISOLATION,root,log)
        self.assertEqual(first.outer_argv,('bash','tools/v8/core_gate_checks.sh','baseline_isolation',str(log)))
        self.assertEqual(first.cargo_argv,('cargo','test','--manifest-path','rust/Cargo.toml','-p','sim-systems','--lib','registration_tests','--','--nocapture'))
        self.assertEqual(len(first.recipe.expected),4);self.assertEqual(first.recipe.profile,'debug')
        mind=library.prepare(library.LibraryStep.MIND_DATA_ONLY,root,log)
        self.assertEqual(mind.cargo_argv,('cargo','test','--manifest-path','rust/Cargo.toml','-p','sim-systems','--lib',library.MIND,'--','--exact'))
        self.assertEqual(mind.outer_argv[2],'data_only')
        diagnosis=library.prepare(library.LibraryStep.READ_DATA_ONLY,root,log)
        self.assertEqual(diagnosis.cargo_cwd,root/'rust');self.assertEqual(diagnosis.recipe.profile,'release')
        self.assertEqual(diagnosis.cargo_argv,('cargo','test','--release','-p','sim-core','--lib',library.DIAGNOSIS,'--','--exact'))
        self.assertEqual(diagnosis.outer_argv,('bash','-c','cd rust && cargo test --release -p sim-core --lib "'+library.DIAGNOSIS+'" -- --exact'))

    def test43_is_one_case_two_source_branches_without_world_admission(self) -> None:
        row=library.reconcile_selected(evidence(library.LibraryStep.MIND_DATA_ONLY))
        self.assertEqual(row.selected,(library.MIND,));self.assertEqual(row.recipe.source_described_lit_branches,(False,True))
        self.assertEqual(row.recipe.source_classification,'CONTROLLED_COLONY_MECHANISM')
        self.assertEqual(row.branch_observation,'NOT_RECORDED');self.assertEqual(row.world_admission,'INCOMPLETE')
        self.assertFalse(row.settlement_authority)


class AccountingTests(unittest.TestCase):
    def test_full_independent_census_derives_filtered_count(self) -> None:
        for step in library.LibraryStep:
            with self.subTest(step=step):
                row=library.reconcile_selected(evidence(step))
                self.assertEqual(row.filtered_out,2);self.assertEqual(row.accounting_status,'PASS')
                self.assertEqual(row.dispatch,'NOT_RUN');self.assertEqual(row.runtime_admission,'PENDING')
                self.assertEqual(row.ignored_inventory,('unrelated::ignored',))
                self.assertEqual(row.summary.passed,len(library.recipe(step).expected))

    def test_wrong_zero_missing_extra_duplicate_ignored_and_benchmark_reject(self) -> None:
        original=evidence(library.LibraryStep.READ_DATA_ONLY)
        variants=(replace(original,full_listing=listing(('wrong',))),
            replace(original,full_listing=listing((library.DIAGNOSIS,library.DIAGNOSIS))),
            replace(original,ignored_listing=listing((library.DIAGNOSIS,))),
            replace(original,ignored_listing=listing(('foreign',))),
            replace(original,native_output=native((),491)),
            replace(original,native_output=native(('wrong',),2)),
            replace(original,native_output=native((library.DIAGNOSIS,library.DIAGNOSIS),2)),
            replace(original,native_output=native((library.DIAGNOSIS,),491)),
            replace(original,full_listing=b'a: benchmark\n\n0 tests, 1 benchmark\n'))
        for changed in variants:
            with self.subTest(changed=changed),self.assertRaises((GateContractError,SummaryError)):
                _=library.reconcile_selected(changed)
        extra=evidence(library.LibraryStep.BASELINE_ISOLATION)
        with self.assertRaises(GateContractError):
            _=library.reconcile_selected(replace(extra,full_listing=listing((*library.REGISTRATION,'prefix_registration_tests::new'))))

    def test_summary_running_measured_ignored_and_extra_sections_reject(self) -> None:
        original=evidence(library.LibraryStep.READ_DATA_ONLY)
        for raw in (original.native_output.replace(b'running 1 test',b'running 0 tests'),
                    original.native_output.replace(b'0 measured',b'1 measured'),
                    original.native_output.replace(b'... ok',b'... ignored').replace(b'1 passed',b'0 passed').replace(b'0 ignored',b'1 ignored'),
                    original.native_output+original.native_output,
                    original.native_output+b'running 0 tests\n',
                    original.native_output[:-2],
                    original.native_output.replace(b'test result: ok',b'test result: FAILED')):
            with self.subTest(raw=raw),self.assertRaises((GateContractError,SummaryError)):
                _=library.reconcile_selected(replace(original,native_output=raw))

    def test42_nocapture_tolerated_but_forged_summary_rejected(self) -> None:
        original=evidence(library.LibraryStep.BASELINE_ISOLATION)
        raw=native(library.REGISTRATION,2,noise='registered systems diagnostic\n')
        self.assertEqual(library.reconcile_selected(replace(original,native_output=raw)).accounting_status,'PASS')
        with self.assertRaises((GateContractError,SummaryError)):
            _=library.reconcile_selected(replace(original,native_output=raw+b'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0s\n'))

    def test_normal_failure_retained_but_unknown_exit_never_settles(self) -> None:
        original=evidence(library.LibraryStep.READ_DATA_ONLY)
        failed=replace(original,native_output=native((library.DIAGNOSIS,),2,failed=1),native_exit=101)
        row=library.reconcile_selected(failed)
        self.assertEqual(row.accounting_status,'FAIL');self.assertEqual(row.summary.failed,1)
        self.assertEqual(row.native_exit,101);self.assertFalse(row.settlement_authority)
        self.assertEqual(library.reconcile_selected(replace(original,native_exit=7)).accounting_status,'FAIL')
        for code in (None,-9,True):
            with self.subTest(code=code),self.assertRaises(GateContractError):_=library.reconcile_selected(replace(original,native_exit=code))
        with self.assertRaises(GateContractError):_=library.reconcile_selected(replace(failed,native_exit=0))

    def test_old_workspace_zero_filtered_contract_unchanged(self) -> None:
        original=evidence(library.LibraryStep.READ_DATA_ONLY)
        with self.assertRaises(GateContractError):
            _=reconcile_native(original.native_output,parse_listing(listing((library.DIAGNOSIS,))),parse_listing(listing(())))


class ListingAuthenticationTests(unittest.TestCase):
    def test_synthetic_receipts_reuse_existing_auth_and_reject_late_mutation(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();binary=root/'binary';_=binary.write_text('synthetic not executable')
            source=root/'source';source.mkdir();inputfile=source/'input';_=inputfile.write_text('bound')
            spec=ProcessSpec((str(binary),'--list'),source,binary,file_hash(binary),source,(('input',file_hash(inputfile)),),root/'full',())
            full=listing_process(root/'full',spec,listing((library.DIAGNOSIS,'other')))
            ignored=listing_process(root/'ignored',replace(spec,argv=(str(binary),'--list','--ignored')),listing(()))
            scope=library.ListingScope(binary,file_hash(binary),source,tuple(environment_for(spec).items()),((inputfile,file_hash(inputfile)),))
            pair=library.IndependentListings(full,ignored)
            result=library.authenticate_listings(scope,pair)
            self.assertEqual(result.full,listing((library.DIAGNOSIS,'other')));self.assertEqual(result.source_build_admission,'PENDING')
            with self.assertRaises(GateContractError):_=library.authenticate_listings(scope,library.IndependentListings(full,full))
            with self.assertRaises(GateContractError):_=library.authenticate_listings(scope,library.IndependentListings(full,replace(ignored,spec=replace(ignored.spec,argv=(str(binary),'--list','--exact')))))
            _=(root/'full/stdout.log').write_bytes(listing(()))
            with self.assertRaises(ValueError):_=library.authenticate_listings(scope,pair)

    def test_source_mutation_between_independent_reads_rejects(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();binary=root/'binary';_=binary.write_text('synthetic')
            source=root/'source';source.mkdir()
            spec=ProcessSpec((str(binary),'--list'),source,binary,file_hash(binary),source,(),root/'full',())
            full=listing_process(root/'full',spec,listing((library.DIAGNOSIS,)))
            ignored=listing_process(root/'ignored',replace(spec,argv=(str(binary),'--list','--ignored')),listing(()))
            scope=library.ListingScope(binary,file_hash(binary),source,tuple(environment_for(spec).items()),())
            original=process_output;calls=0
            def changing(binding: ListingProcess, argv: tuple[str,...], cwd: Path, environment: dict[str,str], required: dict[str,str]) -> bytes:
                nonlocal calls
                raw=original(binding,argv,cwd,environment,required);calls+=1
                if calls==2:_=binary.write_text('changed')
                return raw
            with patch.object(library,'process_output',side_effect=changing),self.assertRaises(GateContractError):
                _=library.authenticate_listings(scope,library.IndependentListings(full,ignored))
