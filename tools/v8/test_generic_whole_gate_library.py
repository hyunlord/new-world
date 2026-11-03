"""Synthetic outcome fixtures exercise attribution validation, not native product tests."""
import sys
import tempfile
import unittest
from pathlib import Path

import generic_process
from generic_gate import GateContractError
from generic_library_siblings import LibraryStep, recipe
from generic_process import ProcessSpec, environment_for, execute, file_hash
from generic_whole_gate_library import SOURCE_FILES, SOURCE_PINS, selected_output
from generic_whole_gate_native import evaluate_family
from mux2_gate_inputs import Json, Node


def output(step: int) -> str:
    names = recipe(LibraryStep(step)).expected
    return ('BASELINE_ISOLATION source PASS\n' if step == 42 else '') + '\n'.join((
        'Running unittests src/lib.rs (synthetic-fixture)',
        f'running {len(names)} '+('test' if len(names)==1 else 'tests'),
        *(f'test {name} ... ok' for name in names),
        f'test result: ok. {len(names)} passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.01s', ''))


class LibraryTests(unittest.TestCase):
    def test_original_selection_is_positive_exact_and_no_extra(self) -> None:
        for step in (42,43,49):
            self.assertEqual(selected_output(output(step),step),recipe(LibraryStep(step)).expected)
            for raw in (output(step)+output(step),output(step).replace(' ... ok',' ... FAILED',1),
                        'running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.01s\n'):
                with self.assertRaises((GateContractError,ValueError)):
                    selected_output(raw,step)

    def test_real_bash_receipt_source_mechanism_and_no_child_wait_claim(self) -> None:
        original=Path(generic_process.__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve(); harness=root/'h'; (harness/'tools/v8').mkdir(parents=True)
            gate=harness/'tools/v8/gate.sh'
            python=Path(sys.executable).resolve();bash=Path('/bin/bash')
            gate.write_text(''.join(f'step {step} lib {python} emitter.py {step}\n' for step in (42,43,49)))
            destination=root/'output';destination.mkdir()
            arms:dict[str,Json]={}
            stages:dict[tuple[int,str],Node]={}
            for arm in ('base','candidate'):
                product=root/arm;product.mkdir();inputs:dict[str,str]={}
                for source in SOURCE_PINS:
                    dest=product/source;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes((original/source).read_bytes());inputs[source]=file_hash(dest)
                emitter=product/'emitter.py';emitter.write_text('import sys\nlogs='+repr({step:output(step) for step in (42,43,49)})+'\nprint(logs[int(sys.argv[1])],end="")\n');inputs['emitter.py']=file_hash(emitter)
                arms[arm]={'root':str(product),'source_inputs':inputs}
                for step in (42,43,49):
                    argv=(str(python),'emitter.py',str(step))
                    spec=ProcessSpec((str(bash),'-c','exec "$@" 2>&1','whole-step',*argv),product,bash,file_hash(bash),product,tuple(sorted(inputs.items())),destination/arm/str(step)/'process',())
                    spec.output_root.parent.mkdir(parents=True)
                    result=execute(spec);self.assertEqual(result.native_exit,0)
                    stages[step,arm]=Node({'step':step,'arm':arm,'state':'RETURNED','native_exit':0,'pid':result.pid,'argv':list(argv),'cwd':str(product),'environment':environment_for(spec),'process_receipt':str(result.receipt),'process_receipt_sha256':file_hash(result.receipt)})
            session=Node({'output':str(destination),'harness':{'root':str(harness),'inputs':{'tools/v8/gate.sh':file_hash(gate)}},'bash':{'path':str(bash),'sha256':file_hash(bash)},'arms':arms})
            for step in (42,43,49):
                result=evaluate_family(session,stages[step,'base'],stages[step,'candidate'])
                self.assertEqual(result['status'],'PASS')
                self.assertEqual(result['safety'],[])
                evidence=Node(result['coverage']).array()[0].field('attribution').field('candidate')
                self.assertIsNone(evidence.field('native_child_pid').value)
                self.assertEqual(evidence.field('native_child_wait').text(),'NOT_INDEPENDENTLY_RECORDED')
            source=root/'candidate'/SOURCE_FILES[43][1];source.write_text('unregistered')
            with self.assertRaises(GateContractError):evaluate_family(session,stages[43,'base'],stages[43,'candidate'])

if __name__=='__main__':unittest.main()
